use tokio::sync::broadcast;
use tokio_stream::{Stream, StreamExt as _, wrappers::BroadcastStream};
use crate::StreamExt;
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
    client: BaseClient,
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
    pub async fn stream(&self) -> crate::Result<BroadcastStream<crate::Result<Message>>> {
        self.client.stream().await
    }
}

impl Client {
    /// Stops the synthesizer.
    pub fn stop(&self) -> crate::Result<()> {
        unimplemented!("stop_speaking is not implemented yet");
    }

    pub async fn synthesize(&self, text: impl ToSSML) -> crate::Result<impl Stream<Item=crate::Result<Event>>> {
        tracing::debug!("Sending a new ssml speak: {:?}", text);
        let uuid = uuid::Uuid::new_v4();
        let mut session = SynthesizerSession::new(uuid);
        let config = self.config.clone();

        self.client.send_text(create_speech_config_message(uuid, &self.config))?;

        self.client.send_text(create_synthesis_context_message(uuid, &self.config))?;

        let xml = text.to_ssml(self.config.language.clone(), self.config.voice.clone())?;
        tracing::debug!("Sending ssml message: {:?}", xml);

        self.client.send_text(create_ssml_message(uuid, xml))?;

        Ok(self.stream().await?
            .stop_after(|message| message.is_err())
            .filter_map(move |message| match message {
                Ok(message) => Some(message),
                Err(e) => {
                    tracing::error!("Error in synthesizer stream: {:?}", e);
                    None
                }
            })
            .filter_map(move |message| convert_message_to_event(message, &mut session, &config).transpose())
            .merge(tokio_stream::iter(vec![Ok(Event::Started)]))
            .stop_after(|event| event.is_err() || event.eq(&Ok(Event::Completed))))
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

fn convert_message_to_event(message: crate::Result<Message>, session: &mut SynthesizerSession, config: &Config) -> crate::Result<Option<Event>> {
    match message {
        Ok(message) => {
            if message.id != session.uuid.to_string() {
                return Ok(None);
            }

            match (message.path.as_str(), message.data.clone(), message.headers.clone()) {
                ("turn.start", Data::Text(Some(data)), _) => {
                    let value = serde_json::from_str::<message::TurnStart>(&data)
                        .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                    if let Some(webrtc) = value.webrtc {
                        session.webrtc_connection_string = Some(webrtc.connection_string);
                    }

                    Ok(None)
                }
                ("response", Data::Text(Some(data)), _) => {
                    let value = serde_json::from_str::<message::Response>(&data)
                        .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                    session.stream_id = Some(value.audio.stream_id);
                    Ok(None)
                }
                ("audio", Data::Binary(audio), headers) => {
                    if audio.is_none() {
                        return Ok(None);
                    }


                    let stream_id = session.stream_id.clone().unwrap_or_default();

                    // todo: add headers to the audio data.

                    let audio_header = config.output_format.header(audio.as_ref().unwrap());
                    let mut data = audio.unwrap();
                    // todo: append on the front headers to the audio data
                    data.splice(0..0, audio_header);


                    if headers.contains(&(STREAM_ID_HEADER.to_string(), stream_id)) {
                        //config.on_audio_chunk.as_mut().map(|f| f(data.clone()));
                        // TODO: Add callback onSynthesizing
                        return Ok(Some(Event::Audio(data)));
                    }

                    Ok(None)
                }
                ("audio.metadata", Data::Text(Some(string)), _) => {
                    //config.on_audio_metadata.as_mut().map(|f| f(string.clone()));
                    // TODO: Add callback to metadata
                    Ok(Some(Event::AudioMetadata(string)))
                }
                ("turn.end", _, _) => {
                    //config.on_session_end.as_mut().map(|f| f());
                    Ok(Some(Event::Completed))
                }
                _ => {
                    tracing::warn!("Unknown message: {:?}", message);
                    Ok(None)
                }
            }
        }
        Err(e) => Err(e),
    }
}
