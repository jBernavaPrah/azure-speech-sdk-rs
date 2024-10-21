use crate::auth::Auth;
use crate::connector::Client as BaseClient;
use crate::connector::{Data, Message, STREAM_ID_HEADER};
use crate::stream_ext::StreamExt;
use crate::synthesizer::event::Event;
use crate::synthesizer::session::Session;
use crate::synthesizer::utils::{
    create_speech_config_message, create_ssml_message, create_synthesis_context_message,
};
use crate::synthesizer::{message, ssml::ToSSML, Config};
use crate::utils::get_azure_hostname_from_region;
use tokio_stream::{Stream, StreamExt as _};

#[derive(Clone)]
pub struct Client {
    /// The client to send and receive messages.
    pub client: BaseClient,
    pub config: Config,
}

impl Client {
    pub(crate) fn new(client: BaseClient, config: Config) -> Self {
        Self { client, config }
    }

    pub async fn connect(auth: Auth, config: Config) -> crate::Result<Self> {
        let url_str = format!(
            "wss://{}.tts.speech{}/cognitiveservices/websocket/v1",
            auth.region,
            get_azure_hostname_from_region(auth.region.as_str())
        );

        let client = BaseClient::connect(
            tokio_websockets::ClientBuilder::new()
                .uri(&url_str)
                .unwrap()
                .add_header(
                    "Ocp-Apim-Subscription-Key".try_into().unwrap(),
                    (&auth.subscription).try_into().unwrap(),
                )
                .add_header(
                    "X-ConnectionId".try_into().unwrap(),
                    uuid::Uuid::new_v4().to_string().try_into().unwrap(),
                ),
        )
        .await?;
        Ok(Self::new(client, config))
    }

    pub async fn disconnect(self) -> crate::Result<()> {
        self.client.disconnect().await
    }
}

impl Client {
    pub async fn synthesize(
        &self,
        text: impl ToSSML,
    ) -> crate::Result<impl Stream<Item = crate::Result<Event>>> {
        let xml = text.to_ssml(
            self.config.language.clone(),
            self.config
                .voice
                .clone()
                .unwrap_or(self.config.language.default_voice()),
        )?;
        tracing::debug!("Sending ssml message: {:?}", xml);

        let session = Session::new(uuid::Uuid::new_v4());
        let config = self.config.clone();
        let request_id = session.request_id().to_string();

        // create first the stream.
        // This is necessary to not lost any message after the sending.
        // The stream will filter out messages that are not from the current request.
        let stream = self.client.stream().await?;

        self.client
            .send_text(create_speech_config_message(
                request_id.to_string(),
                &config,
            ))
            .await?;
        self.client
            .send_text(create_synthesis_context_message(
                request_id.to_string(),
                &config,
            ))
            .await?;
        self.client
            .send_text(create_ssml_message(request_id.to_string(), &xml))
            .await?;

        let session2 = session.clone();
        Ok(stream
            // Map errors.
            .map(move |message| match message {
                Ok(message) => message,
                Err(e) => Err(crate::Error::InternalError(e.to_string())),
            })
            // Filter out messages that are not from the current session.
            .filter(move |message| match message {
                Ok(message) => message.id == session.request_id().to_string(),
                Err(_) => true,
            })
            // Convert the message to an event.
            .filter_map(move |message| match message {
                Ok(message) => convert_message_to_event(message, session2.clone()),
                Err(e) => Some(Err(e)),
            })
            // Stop the stream if there is an error or the session ended.
            .stop_after(|event| event.is_err() || matches!(event, Ok(Event::SessionEnded(_)))))
    }
}

fn convert_message_to_event(message: Message, session: Session) -> Option<crate::Result<Event>> {
    match (
        message.path.as_str(),
        message.data.clone(),
        message.headers.clone(),
    ) {
        ("turn.start", Data::Text(Some(data)), _) => {
            let value = match serde_json::from_str::<message::TurnStart>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };

            if let Some(webrtc) = value.webrtc {
                session.set_webrtc_connection_string(webrtc.connection_string);
            }
            Some(Ok(Event::SessionStarted(session.request_id())))
        }
        ("response", Data::Text(Some(data)), _) => {
            let value = match serde_json::from_str::<message::Response>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };

            session.set_stream_id(value.audio.stream_id);
            None
        }
        ("audio", Data::Binary(audio), headers) => {
            if audio.is_none() {
                return Some(Ok(Event::Synthesised(session.request_id())));
            }

            let stream_id = session.stream_id().unwrap_or_default();
            if headers.contains(&(STREAM_ID_HEADER.to_string(), stream_id)) {
                return Some(Ok(Event::Synthesising(
                    session.request_id(),
                    audio.unwrap(),
                )));
            }

            None
        }
        ("audio.metadata", Data::Text(Some(string)), _) => {
            Some(Ok(Event::AudioMetadata(session.request_id(), string)))
        }
        ("turn.end", _, _) => Some(Ok(Event::SessionEnded(session.request_id()))),
        _ => {
            tracing::warn!("Unknown message: {:?}", message);
            None
        }
    }
}
