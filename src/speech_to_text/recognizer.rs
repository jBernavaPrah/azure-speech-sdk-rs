use std::io;
use std::path::Path;
use cpal::{BuildStreamError, Device, InputCallbackInfo, Sample, SampleFormat, Stream, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{debug, error, trace};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_util::StreamExt;
use crate::speech_to_text::utils::{generate_uri_for_stt_speech_azure};
use crate::errors::Result;
use crate::speech_to_text::AudioFormat;
use crate::speech_to_text::utils::AudioHeaders;
use crate::speech_to_text::config::{AdvancedConfig, LanguageDetectMode, Os, OutputFormat, Profanity, RecognitionConfig, RecognitionMode, Source, System};
use crate::speech_to_text::connector::Connector;
use crate::speech_to_text::request::Message as UpMessage;
use crate::speech_to_text::request::speech_config::SpeechConfig;
use crate::speech_to_text::request::speech_context::SpeechContext;
use crate::speech_to_text::response::Message as DownMessage;


#[derive(Debug)]
pub struct Recognizer {
    pub(crate) config: RecognitionConfig,
}

impl Recognizer {
    pub fn new(
        region: impl Into<String>,
        subscription: impl Into<String>,
    ) -> Self {
        Recognizer {
            config: RecognitionConfig::new(region, subscription),
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


    async fn connect_to_azure(self, headers: &AudioHeaders) -> Result<Connector> {
        let mut connector = Connector::connect(generate_uri_for_stt_speech_azure(&self.config), Uuid::new_v4().to_string()).await?;

        let _ = connector.send(UpMessage::SpeechConfig(SpeechConfig::from_config(&self.config, headers))).await;
        let _ = connector.send(UpMessage::SpeechContext(SpeechContext::from_config(self.config.clone()))).await;

        let _ = connector.send(UpMessage::AudioHeaders {
            content_type: "audio/x-wav".to_string(),
            data: headers.to_vec(),
        }).await;

        Ok(connector)
    }

    async fn _stream(self, mut stream: Receiver<Vec<u8>>, headers: AudioHeaders) -> Result<Receiver<DownMessage>> {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        let connector = self.connect_to_azure(&headers).await?;

        let (mut sender, mut receiver) = connector.split();

        // Send the audio stream to the connector
        tokio::spawn(async move {
            while let Some(s) = stream.recv().await {
                let _ = sender.send(UpMessage::Audio { data: s }).await;
            }

            let _ = sender.send(UpMessage::EndAudio).await;
        });

        // Receive Events from the connector
        tokio::spawn(async move {
            while let Some(r) = receiver.next().await {
                match r {
                    Ok(message) => {
                        match tx.send(message).await {
                            Ok(_) => (),
                            Err(e) => {
                                trace!("Failed to send response: {:?}", e);
                                break;
                            }
                        }
                    }
                    Err(_) => break
                }
            }

            debug!("Azure dropped stream.");

            drop(tx)
        });

        Ok(rx)
    }

    async fn _microphone(mut self, device: Device) -> Result<Receiver<DownMessage>> {
        self.set_source(Source::microphone());

        let (tx, rx) = tokio::sync::mpsc::channel(5000);

        debug!("Input device: {}", device.name().unwrap_or("Unknown".to_string()));

        let config = device.default_input_config().expect("Failed to get default input config");

        debug!("Default input config: {:?}", config);

        // Let's get the default configuration from the audio driver.
        let sample_rate = config.sample_rate().0.clone();
        let output_channels = config.channels().clone();
        let bit_per_sec = (config.sample_format().sample_size() * 8) as u16;

        debug!("Sample rate: {}", sample_rate);
        debug!("Output channels: {}", output_channels);
        debug!("Bit per sec: {}", bit_per_sec);


        let config2 = config.clone();

        tokio::task::spawn_blocking(move || {
            let stream = match config.sample_format() {
                SampleFormat::F32 => send_microphone_to_sender::<f32>(&device, &config, tx.clone()),
                SampleFormat::I16 => send_microphone_to_sender::<i16>(&device, &config, tx.clone()),
                SampleFormat::U16 => send_microphone_to_sender::<u16>(&device, &config, tx.clone()),
                SampleFormat::I8 => send_microphone_to_sender::<i8>(&device, &config, tx.clone()),
                SampleFormat::I32 => send_microphone_to_sender::<i32>(&device, &config, tx.clone()),
                SampleFormat::I64 => send_microphone_to_sender::<i64>(&device, &config, tx.clone()),
                SampleFormat::U8 => send_microphone_to_sender::<u8>(&device, &config, tx.clone()),
                SampleFormat::U32 => send_microphone_to_sender::<u32>(&device, &config, tx.clone()),
                SampleFormat::U64 => send_microphone_to_sender::<u64>(&device, &config, tx.clone()),
                SampleFormat::F64 => send_microphone_to_sender::<f64>(&device, &config, tx.clone()),
                _ => panic!("Unsupported sample format"),
            }.unwrap();
            stream.play().unwrap();

            loop {
                // Sleep for a while
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });


        self._stream(rx, AudioHeaders::new(match config2.sample_format().is_float() {
            true => AudioFormat::IEEE,
            false => AudioFormat::PCM,
        }, sample_rate, bit_per_sec, output_channels)).await
    }

    /// Recognize audio from the default microphone.
    pub async fn microphone(self) -> Result<Receiver<DownMessage>> {
        let host = cpal::default_host();
        self._microphone(host.default_input_device().expect("Default device not found.")).await
    }

    pub async fn microphone_by_name(self, name: impl Into<String>) -> Result<Receiver<DownMessage>> {
        let host = cpal::default_host();
        let name: String = name.into().to_lowercase();
        let device = host.devices()
            .expect("Impossible to load the list of devices")
            .find(|d| d.name().unwrap_or("unknown".to_string())
                .to_lowercase().contains(&name)).expect("Device not found");
        self._microphone(device).await
    }

    /// Recognize audio from a stream of bytes.
    pub async fn stream(mut self, stream: Receiver<Vec<u8>>, headers: AudioHeaders) -> Result<Receiver<DownMessage>> {
        self.set_source(Source::stream());
        self._stream(stream, headers).await
    }

    /// Recognize audio from a wav file path.
    pub async fn file(mut self, file_path: impl Into<String> + AsRef<Path> + AsRef<std::ffi::OsStr> + Send + 'static) -> Result<Receiver<DownMessage>> {
        self.set_source(Source::file());

        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        let mut reader = hound::WavReader::open(file_path).expect("Failed to open file");
        let spec = reader.spec();

        tokio::spawn(async move {
            match reader.spec().sample_format {
                hound::SampleFormat::Float => send_buffer_to_sender::<f32, _>(&mut reader, tx).await,
                hound::SampleFormat::Int => send_buffer_to_sender::<i32, _>(&mut reader, tx).await,
            }
        });

        self._stream(rx, AudioHeaders::from_hound_spec(spec)).await
    }
}


pub trait ReceiverAsEventExt<T>
{
    fn events(self) -> Receiver<T>;
}


impl<T, U> ReceiverAsEventExt<U> for Receiver<T>
    where U: From<T>,
          T: Send + 'static,
          U: Send + 'static
{
    fn events(mut self) -> Receiver<U> {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        tokio::spawn(async move {
            while let Some(response) = self.recv().await {
                if tx.send(response.into()).await.is_err() {
                    break;
                }
            }
        });

        rx
    }
}


async fn send_buffer_to_sender<S, R>(reader: &mut hound::WavReader<R>, tx: Sender<Vec<u8>>)
    where
        S: hound::Sample + funty::Numeric,
        <S as funty::Numeric>::Bytes: AsRef<[u8]>,
        R: io::Read, {
    let mut buffer = Vec::new();
    for sample in reader.samples::<S>() {
        match sample {
            Ok(s) => {
                let s = S::from(s);
                buffer.extend_from_slice(s.to_le_bytes().as_ref());

                if buffer.len() >= 4096 {
                    if let Err(_) = tx.send(buffer.clone()).await {
                        error!("Failed to send buffer");
                    }
                    buffer.clear();
                }
            }
            Err(e) => error!("Failed to read sample: {}", e),
        }
    }

    // Send any remaining data in the buffer
    if !buffer.is_empty() && tx.send(buffer).await.is_err() {
        error!("Failed to send buffer");
    }

    drop(tx);
}


fn send_microphone_to_sender<T>(device: &Device, config: &SupportedStreamConfig, tx: Sender<Vec<u8>>) -> std::result::Result<Stream, BuildStreamError>
    where
        T: Sample + cpal::SizedSample + funty::Numeric, <T as funty::Numeric>::Bytes: AsRef<[u8]>,
{
    device.build_input_stream(
        &config.clone().into(),
        move |data: &[T], _: &InputCallbackInfo| {
            let mut v = Vec::new();
            for &x in data {
                v.extend_from_slice(x.to_le_bytes().as_ref());
            }
            tx.blocking_send(v.clone()).unwrap();
        },
        |err| panic!("{err}"),
        None,
    )
}
