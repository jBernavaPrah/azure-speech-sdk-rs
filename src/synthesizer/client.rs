use ssml::{Flavor, Serialize, SerializeOptions, Speak};
use tokio_stream::{Stream, StreamExt as _};
use tracing::debug;
use url::Url;
use crate::auth::Auth;
use crate::connector;
use crate::connector::Call;
use crate::message::{Data, Message};
use crate::stream_ext::StreamExt;
use crate::synthesizer::{Config, message};
use crate::synthesizer::event::EventSynthesis;
use crate::synthesizer::language::Language;
use crate::synthesizer::utils::{create_speech_config_message, create_ssml_message, create_synthesis_context_message};
use crate::synthesizer::voice::Voice;
use crate::utils::get_azure_hostname_from_region;

#[derive(Clone)]
pub struct SynthesizerClient {
    client: connector::Client,
    receiver: async_broadcast::Receiver<crate::Result<Message>>,
    config: Config,
}


impl SynthesizerClient {
    pub fn new(client: connector::Client, receiver: async_broadcast::Receiver<crate::Result<Message>>, config: Config) -> Self {
        Self {
            client,
            receiver,
            config,
        }
    }

    pub fn receiver(&self) -> async_broadcast::Receiver<crate::Result<Message>> {
        self.receiver.clone()
    }

    pub fn call(&self, message: Call) -> crate::Result<()> {
        self.client.call(message).map_err(|e| crate::Error::InternalError(e.to_string()))
    }
    
    pub fn stop_speaking(&self) -> crate::Result<()> {
        unimplemented!("stop_speaking is not implemented yet");
    }

    pub async fn synthesize_text(&self, text: impl Into<String>, language: Language, voice: Option<Voice>) -> crate::Result<impl Stream<Item=crate::Result<EventSynthesis>>> {
        
        let ssml = Speak::new(Some(language.as_str()), [ssml::voice(voice.unwrap_or(self.config.voice.clone().unwrap_or(Voice::default_for_language(language))).as_str(), [text.into()])]);
        self.synthesize(ssml).await
    }

    pub async fn synthesize(&self, ssml: Speak) -> crate::Result<impl Stream<Item=crate::Result<EventSynthesis>>> {
        
        let config = self.config.clone();

        debug!("Sending a new ssml speak: {:?}", ssml);

        let uuid = uuid::Uuid::new_v4();
        let mut session = SynthesizerSession::new(uuid);

        self.client.text(create_speech_config_message(uuid, &config))
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        self.client.text(create_synthesis_context_message(uuid, &config))
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        let serialize_option = SerializeOptions::default()
            .flavor(Flavor::MicrosoftAzureCognitiveSpeechServices);
        let ssml = ssml.serialize_to_string(&serialize_option)
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;
        
        tracing::debug!("Sending ssml message: {:?}", ssml);
        
        self.client.text(create_ssml_message(uuid, ssml))
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        Ok(self.receiver
            .clone()

            .stop_after(|message| message.is_err())
            .filter_map(move |message| match convert_message_to_event(message, &mut session) {
                Ok(Some(event)) => Some(Ok(event)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .merge(tokio_stream::iter(vec![Ok(EventSynthesis::SynthesisStarted)]))
            .stop_after(|event| event.is_err()))
    }

    pub fn disconnect(self) {
        let _ = self.client.close(None);
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
        Self {
            uuid,
            ..Default::default()
        }
    }
}

fn convert_message_to_event(message: crate::Result<Message>, session: &mut SynthesizerSession) -> crate::Result<Option<EventSynthesis>> {
    match message {
        Ok(message) => {
            if message.id != session.uuid.to_string() {
                return Ok(None);
            }

            match (message.path.as_str(), message.data.clone(), message.headers.clone()) {
                ("turn.start", Data::Text(Some(data)), _) => {
                    let value = serde_json::from_str::<message::TurnStart>(data.as_str())
                        .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                    if let Some(webrtc) = value.webrtc {
                        session.webrtc_connection_string = Some(webrtc.connection_string);
                    }

                    Ok(None)
                }
                ("response", Data::Text(Some(data)), _) => {
                    let value = serde_json::from_str::<message::Response>(data.as_str())
                        .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                    session.stream_id = Some(value.audio.stream_id);

                    Ok(None)
                }
                ("audio", Data::Binary(Some(data)), headers) => {
                    let stream_id = session.stream_id.clone().unwrap_or("".to_string());

                    if headers.contains(&("X-StreamId".to_string(), stream_id)) {
                        // todo: add callback onSynthesizing
                        return Ok(Some(EventSynthesis::Synthesizing(data)));
                    }

                    Ok(None)
                }
                ("audio.metadata", Data::Text(Some(string)), _) => {
                    // todo: add callback to metadata. 
                    // this differ from the original code. I will add this later.
                    Ok(Some(EventSynthesis::AudioMetadata(string)))
                }
                ("turn.end", _, _) => {
                    Ok(Some(EventSynthesis::SynthesisCompleted))
                }

                _ => {
                    tracing::warn!("Unknown message: {:?}", message);
                    Ok(None)
                }
            }

            // match EventSynthesis::try_from(message) {
            //     Ok(event) => Some(Ok(event)),
            //     Err(Error::Skip) => None,
            //     Err(e) => Some(Err(e.into())),
            // }
        }
        Err(e) => Err(e),
    }
}

pub async fn connect(auth: Auth, config: Config) -> SynthesizerClient {
    let mut url = Url::parse(format!("wss://{}.tts.speech{}/cognitiveservices/websocket/v1", auth.region, get_azure_hostname_from_region(auth.region.as_str())).as_str()).unwrap();
    url.query_pairs_mut().append_pair("Ocp-Apim-Subscription-Key", auth.subscription.to_string().as_str());
    url.query_pairs_mut().append_pair("X-ConnectionId", uuid::Uuid::new_v4().to_string().as_str());

    let client_config = ezsockets::ClientConfig::new(url.as_str());

    let (client, receiver) = connector::connect(client_config).await;

    SynthesizerClient::new(client, receiver, config)
}

