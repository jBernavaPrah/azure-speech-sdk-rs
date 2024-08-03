use tokio_stream::{Stream, StreamExt as _};
use crate::{StreamExt};
use url::Url;
use crate::auth::Auth;
use crate::connector::Client as BaseClient;
use crate::connector::{Data, Message, STREAM_ID_HEADER};
use crate::synthesizer::{Config, message, ToSSML};
use crate::synthesizer::event::Event;
use crate::synthesizer::utils::{create_speech_config_message, create_ssml_message, create_synthesis_context_message};
use crate::utils::get_azure_hostname_from_region;

#[derive(Clone)]
pub struct Client
{
    /// The client to send and receive messages.
    pub client: BaseClient,
    config: Config,
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
        let mut url = Url::parse(&url_str).unwrap();
        url.query_pairs_mut()
            .append_pair("Ocp-Apim-Subscription-Key", &auth.subscription)
            .append_pair("X-ConnectionId", &uuid::Uuid::new_v4().to_string());

        let client_config = ezsockets::ClientConfig::new(url.as_str())
            .max_initial_connect_attempts(3);

        let client = BaseClient::connect(client_config).await?;
        Ok(Self::new(client, config))
    }

    pub async fn disconnect(self) -> crate::Result<()> {
        self.client.disconnect().await
    }
}

impl Client {
    /// Stops the synthesizing.
    pub fn stop_synthesize(&self) -> crate::Result<()> {
        //self.client.send_text(create_stop_speaking_message(self.config.uuid))
        unimplemented!("stop")
    }

    pub async fn synthesize(&self, text: impl ToSSML) -> crate::Result<impl Stream<Item=crate::Result<Event>>> {
        let xml = text.to_ssml(self.config.language.clone(), self.config.voice.clone().unwrap_or(self.config.language.default_voice()))?;
        tracing::debug!("Sending ssml message: {:?}", xml);

        let uuid = uuid::Uuid::new_v4();
        let mut session = SynthesizerSession::new(uuid);

        // create first the stream.
        // This is necessary to not lost any message after the sending.
        // The stream will be used to filter out messages that are not from the current session.
        let stream = self.client.stream().await?;

        self.client.send_text(create_speech_config_message(uuid, &self.config))?;
        self.client.send_text(create_synthesis_context_message(uuid, &self.config))?;
        self.client.send_text(create_ssml_message(uuid, xml))?;


        let config = self.config.clone();
        Ok(stream
            // Filter out errors.
            .map(move |message| match message {
                Ok(message) => message,
                Err(e) => Err(crate::Error::InternalError(e.to_string()))
            })

            // Filter out messages that are not from the current session.
            .filter(move |message| match message {
                Ok(message) => message.id == session.uuid.to_string(),
                Err(_) => true
            })
            // Convert the message to an event.
            .filter_map(move |message| match message {
                Ok(message) => convert_message_to_event(message, &mut session),
                Err(e) => Some(Err(e))
            })
            // Handle the events and call the callbacks.
            .map(move |event| match event {
                Ok(Event::SessionEnded) => {
                    tracing::debug!("Session ended");
                    config.on_session_ended.as_ref().map(|f| f());
                    Ok(Event::SessionEnded)
                }
                Ok(Event::SessionStarted) => {
                    tracing::debug!("Session started");
                    config.on_session_started.as_ref().map(|f| f());
                    Ok(Event::SessionStarted)
                }

                Ok(Event::Synthesising(audio)) => {
                    tracing::debug!("Synthesising audio: {:?}", audio.len());
                    config.on_synthesising.as_ref().map(|f| f(audio.clone()));
                    Ok(Event::Synthesising(audio))
                }

                Ok(Event::Synthesised) => {
                    tracing::debug!("Synthesised");
                    config.on_synthesised.as_ref().map(|f| f());
                    Ok(Event::Synthesised)
                }

                Ok(Event::AudioMetadata(metadata)) => {
                    tracing::debug!("Audio metadata: {:?}", metadata);
                    config.on_audio_metadata.as_ref().map(|f| f(metadata.clone()));
                    Ok(Event::AudioMetadata(metadata))
                }

                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                    config.on_error.as_ref().map(|f| f(e.clone()));
                    Err(e)
                }
            })
            // Stop the stream if there is an error or the session ended.
            .stop_after(|event| event.is_err() || event.eq(&Ok(Event::SessionEnded))))
    }
}

#[derive(Debug, Default)]
struct SynthesizerSession {
    uuid: uuid::Uuid,
    stream_id: Option<String>,
    webrtc_connection_string: Option<String>,
}

impl SynthesizerSession {
    fn new(uuid: uuid::Uuid) -> Self {
        Self { uuid, ..Default::default() }
    }
}

fn convert_message_to_event(message: Message, session: &mut SynthesizerSession) -> Option<crate::Result<Event>> {
    match (message.path.as_str(), message.data.clone(), message.headers.clone()) {
        ("turn.start", Data::Text(Some(data)), _) => {
            let value = match serde_json::from_str::<message::TurnStart>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };

            if let Some(webrtc) = value.webrtc {
                session.webrtc_connection_string = Some(webrtc.connection_string);
            }

            Some(Ok(Event::SessionStarted))
        }
        ("response", Data::Text(Some(data)), _) => {
            let value = match serde_json::from_str::<message::Response>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };

            session.stream_id = Some(value.audio.stream_id);
            None
        }
        ("audio", Data::Binary(audio), headers) => {
            if audio.is_none() {
                return Some(Ok(Event::Synthesised));
            }


            let stream_id = session.stream_id.clone().unwrap_or_default();


            if headers.contains(&(STREAM_ID_HEADER.to_string(), stream_id)) {
                return Some(Ok(Event::Synthesising(audio.unwrap())));
            }

            None
        }
        ("audio.metadata", Data::Text(Some(string)), _) => {
            Some(Ok(Event::AudioMetadata(string)))
        }
        ("turn.end", _, _) => {
            Some(Ok(Event::SessionEnded))
        }
        _ => {
            tracing::warn!("Unknown message: {:?}", message);
            None
        }
    }
}
