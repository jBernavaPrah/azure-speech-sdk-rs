use std::path::Path;
use futures_util::{SinkExt, StreamExt};
use log::{debug, trace};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TMessage;
use uuid::Uuid;
use tokio::io::{AsyncReadExt, BufReader};
use crate::speech_to_text::utils::{generate_uri_for_stt_speech_azure};
use crate::errors::Result;
use crate::speech_to_text::config::{AdvancedConfig, AudioHeaders, LanguageDetectMode, Os, OutputFormat, Profanity, RecognitionConfig, RecognitionMode, Source, System};
use crate::speech_to_text::request::Request;
use crate::speech_to_text::request::speech_config::SpeechConfig;
use crate::speech_to_text::request::speech_context::SpeechContext;
use crate::speech_to_text::response::Response;


#[derive(Debug, Clone)]
pub struct Recognizer {
    pub(crate) config: RecognitionConfig,
    pub(crate) region: String,
    pub(crate) subscription: String,
}

impl Recognizer {
    pub fn new(
        region: impl Into<String>,
        subscription: impl Into<String>,
    ) -> Self {
        Recognizer {
            config: RecognitionConfig::new(),
            region: region.into(),
            subscription: subscription.into(),
        }
    }

    pub fn set_mode(&mut self, mode: RecognitionMode) -> &mut Self {
        self.config.mode = mode;
        self
    }

    pub fn set_language(&mut self, language: impl Into<String>) -> &mut Self {
        self.config.languages = vec![language.into()];
        self
    }

    pub fn set_detect_languages(&mut self,
                                languages: Vec<impl Into<String>>,
                                language_detect_mode: LanguageDetectMode,
    ) -> &mut Self {
        self.config.languages = languages.into_iter().map(|l| l.into()).collect();
        self.config.language_detect_mode = Some(language_detect_mode);
        self
    }

    pub fn set_output_format(&mut self, format: OutputFormat) -> &mut Self {
        self.config.output_format = format;
        self
    }

    pub fn set_phrases(&mut self, phrases: Vec<String>) -> &mut Self {
        self.config.phrases = Some(phrases);
        self
    }

    pub fn set_store_audio(&mut self, store: bool) -> &mut Self {
        self.config.store_audio = store;
        self
    }

    pub fn set_profanity(&mut self, profanity: Profanity) -> &mut Self {
        self.config.profanity = profanity;
        self
    }

    pub fn set_source(&mut self, source: Source) -> &mut Self {
        self.config.source = source;
        self
    }

    pub fn set_os(&mut self, os: Os) -> &mut Self {
        self.config.os = os;
        self
    }

    pub fn set_system(&mut self, system: System) -> &mut Self {
        self.config.system = system;
        self
    }

    pub fn set_advanced_config(&mut self, advanced_config: AdvancedConfig) -> &mut Self {
        self.config.advanced_config = Some(advanced_config);
        self
    }

    pub fn set_custom_models(&mut self, custom_models: Vec<(String, String)>) -> &mut Self {
        self.config.custom_models = Some(custom_models);
        self
    }

    pub async fn recognize_stream(self, mut stream: Receiver<Vec<u8>>, headers: AudioHeaders) -> Result<Receiver<Response>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Response>(1024);
        // connect to websocket.
        let (ws, _) = connect_async(generate_uri_for_stt_speech_azure(self.region, self.subscription, self.config.clone())).await?;
        let (mut sender, mut receiver) = ws.split();
        let session_id = Uuid::new_v4().to_string();

        tokio::spawn(async move {
            let speech_config: TMessage = Request::SpeechConfig {
                session_id: session_id.clone(),
                data: SpeechConfig::from_config(self.config.clone(), headers.clone()),
            }.into();
            trace!("Speech Config: {:?}", speech_config);
            let _ = sender.send(speech_config).await;

            let speech_context: TMessage = Request::SpeechContext {
                session_id: session_id.clone(),
                data: SpeechContext::from_config(self.config.clone()),
            }.into();
            trace!("Speech Context: {:?}",speech_context);
            let _ = sender.send(speech_context).await;

            let speech_context: TMessage = Request::StartAudio {
                content_type: "audio/x-wav".to_string(),
                data: headers.into(),
                session_id: session_id.clone(),
            }.into();
            trace!("Audio Headers: {:X?}",speech_context);
            let _ = sender.send(speech_context).await;


            while let Some(s) = stream.recv().await {
                let audio: TMessage = Request::Audio {
                    session_id: session_id.clone(),
                    data: Some(s),
                }.into();
                trace!("Binary send: {:X?}",audio.to_string());
                if let Err(_) = sender.send(audio).await {
                    break;
                };
            }

            // verify if this is correct!
            let final_audio: TMessage = Request::Audio {
                session_id: session_id.clone(),
                data: None,
            }.into();
            trace!("Final audio: {:X?}",final_audio);
            let _ = sender.send(final_audio).await;
        });

        tokio::spawn(async move {
            while let Some(r) = receiver.next().await {
                match r {
                    Ok(message) => {
                        let _ = tx.send(Response::from_message(message)).await;
                    }
                    Err(_) => break
                }
            }

            drop(tx)
        });

        Ok(rx)
    }

    pub async fn recognize_file_wav(self, file_path: impl Into<String> + AsRef<Path> + AsRef<std::ffi::OsStr> + Send + 'static) -> Result<Receiver<Response>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let file = tokio::fs::File::open(Path::new(&file_path).as_os_str()).await.unwrap();
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 44];
        let _ = reader.read(&mut buffer[..]).await;

        let s = tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            while let Ok(size) = reader.read(&mut buffer[..]).await {
                if size == 0 {
                    break;
                }
                let _ = tx.send(buffer[0..size].to_vec()).await;
            }

            drop(tx);
        });

        s.await?;

        self.recognize_stream(rx, buffer.to_vec().into()).await
    }
}



