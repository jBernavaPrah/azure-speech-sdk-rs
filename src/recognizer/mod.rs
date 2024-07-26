//! Speech to text module.

pub(crate) mod event;
pub(crate) mod utils;
pub(crate) mod config;
use std::io::Read;
use std::path::Path;

use cpal::{Device as CPALDevice, SampleFormat as CPALSampleFormat, Stream};
use cpal::traits::{DeviceTrait, HostTrait};

use ezsockets::{ClientConfig};
use hound::{WavReader, Error};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use url::Url;
use uuid::Uuid;
use crate::{Event, Spec, Details, SampleFormat, AzureSpeech};
use crate::config::Device;
use crate::connector::{Client};
use crate::connector::message::Message;
use crate::recognizer::config::{OutputFormat, RecognizerConfig};
use crate::recognizer::utils::{create_speech_audio_headers_message, create_speech_audio_message, create_speech_config_message, create_speech_context_message};
use crate::recognizer::event::EventSpeech;
use crate::utils::{get_azure_hostname_from_region, send_message, transform_messages_to_events};

pub struct Connected;
pub struct Disconnected;

pub struct Recognizer<S = Disconnected> {
    inner: AzureSpeech,
    config: RecognizerConfig,
    _phantom: std::marker::PhantomData<S>,
}


impl Recognizer {
    pub fn new(inner: AzureSpeech, config: RecognizerConfig) -> Recognizer<Disconnected> {
        Recognizer {
            inner,
            config,
            _phantom: std::marker::PhantomData,
        }
    }
    
    
}

impl Recognizer<Connected> {
    pub async fn recognize(&self, stream: mpsc::Receiver<Vec<u8>>, spec: Spec, details: Details) -> crate::Result<mpsc::Receiver<Event<EventSpeech>>> {
        let (event_tx, event_rx) = mpsc::channel::<Event<EventSpeech>>(1024);
        let messages = self.recognize_raw(stream, spec, details).await?;

        tokio::spawn(transform_messages_to_events(messages, event_tx));

        Ok(event_rx)
    }


    pub async fn connect(&self,
                               stream: mpsc::Receiver<Vec<u8>>,
                               spec: Spec,
                               details: Details) -> crate::Result<mpsc::Receiver<Message>> {
        let config: ClientConfig = self.into();

        let (connector, mut rx) = Connector::connect(config).await;

        let (copy_1_tx, copy_1_rx) = mpsc::channel(100);
        let (copy_2_tx, copy_2_rx) = mpsc::channel(100);
        let (copy_3_tx, mut copy_3_rx) = mpsc::channel(100);


        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let _ = copy_1_tx.send(data.clone()).await;
                let _ = copy_2_tx.send(data.clone()).await;
                let _ = copy_3_tx.send(data.clone()).await;
            }

            drop(copy_1_tx);
            drop(copy_2_tx);
            drop(copy_3_tx);
        });

        let device = self.inner.device.clone();
        let recognition_config = self.config.clone();

        tokio::spawn(async move {
            match sender(&connector, stream, spec, details, device, recognition_config, copy_2_rx).await {
                Ok(_) => {
                    // todo: limit await of turn.end message to 5 sec, then close the connection.
                    debug!("Awaiting for the end of the recognition.");
                    while let Some(message) = copy_3_rx.recv().await {
                        match message {
                            Message::Text { path, .. } => {
                                if path.as_str() == "turn.end" {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    error!("Error sending data: {:?}", e);
                }
            }

            debug!("disconnect the connector");
            connector.disconnect().await;
            debug!("Connector disconnected");
        });

        Ok(copy_1_rx)
    }

    pub async fn recognize_from_file<P: AsRef<Path>>(&self, filename: P) -> crate::Result<mpsc::Receiver<Event<EventSpeech>>> {
        let file = match WavReader::open(filename) {
            Ok(file) => file,
            Err(Error::IoError(e)) => return Err(crate::Error::IOError(e)),
            Err(_) => return Err(crate::Error::InternalError("Error opening the file".to_string())),
        };

        let spec = file.spec();
        let (tx, rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            let mut inner = file.into_inner();
            loop {
                let mut chunk = vec![0; 4096];
                match inner.read(&mut chunk) {
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => {
                        chunk.truncate(n);
                        match tx.send(chunk).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Error sending data: {:?}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading data: {:?}", e);
                        break;
                    }
                }
            }

            drop(tx);
        });

        self.recognize(rx, spec, Details::file()).await
    }

    pub async fn recognize_from_default_microphone(&self) -> crate::Result<(mpsc::Receiver<Event<EventSpeech>>, Stream)> {
        let host = cpal::default_host();
        let device: CPALDevice = host.default_input_device()
            .ok_or(crate::Error::InternalError("Failed to get default input device".to_string()))?;

        self.recognize_from_input_device(device).await
    }

    // todo: add #![cfg(feature = "cpal")] to the Cargo.toml
    pub async fn recognize_from_input_device(&self, device: CPALDevice) -> crate::Result<(mpsc::Receiver<Event<EventSpeech>>, Stream)> {
        let (tx, rx) = mpsc::channel(1024);

        // Get the default input configuration
        let audio_config = device.default_input_config()
            .map_err(|e| crate::Error::InternalError(format!("Failed to get default input config: {:?}", e)))?;

        // Error handler
        let err = |err| warn!("Trying to stream input: {err}");
        // Send the audio data to the channel.

        let config = audio_config.clone().into();

        let stream = match audio_config.sample_format() {
            CPALSampleFormat::I8 => device.build_input_stream(&config, move |data: &[i8], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::U8 => device.build_input_stream(&config, move |data: &[u8], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::I16 => device.build_input_stream(&config, move |data: &[i16], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::U16 => device.build_input_stream(&config, move |data: &[u16], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::I32 => device.build_input_stream(&config, move |data: &[i32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::U32 => device.build_input_stream(&config, move |data: &[u32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::F32 => device.build_input_stream(&config, move |data: &[f32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::I64 => device.build_input_stream(&config, move |data: &[i64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::U64 => device.build_input_stream(&config, move |data: &[u64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            CPALSampleFormat::F64 => device.build_input_stream(&config, move |data: &[f64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
            _ => return Err(crate::Error::InternalError("Unsupported sample format".to_string())),
        }.expect("Failed to build input stream");

        Ok((self.recognize(rx, Spec {
            sample_rate: audio_config.sample_rate().0,
            channels: audio_config.channels(),
            bits_per_sample: (audio_config.sample_format().sample_size() * 8) as u16,
            sample_format: match audio_config.sample_format().is_float() {
                true => SampleFormat::Float,
                false => SampleFormat::Int,
            },
        }, Details::microphone("CPAL", "CPAL")).await?, stream))
    }
}


impl<T> From<&Recognizer<T>> for ClientConfig {
    fn from(recognizer: &Recognizer<T>) -> Self {
        let mut url = Url::parse(format!("wss://{}.stt.speech{}", recognizer.inner.auth.region, get_azure_hostname_from_region(recognizer.inner.auth.region.as_str())).as_str()).unwrap();

        url.set_path(recognizer.config.mode.to_uri_path());

        let lang = recognizer.config.languages.first().expect("Select at least one language!");

        url.query_pairs_mut().append_pair("Ocp-Apim-Subscription-Key", recognizer.inner.auth.subscription.to_string().as_str());
        url.query_pairs_mut().append_pair("language", lang.as_str());
        url.query_pairs_mut().append_pair("format", &recognizer.config.output_format.as_str());
        url.query_pairs_mut().append_pair("profanity", recognizer.config.profanity.as_str());
        url.query_pairs_mut().append_pair("storeAudio", recognizer.config.store_audio.to_string().as_str());

        if recognizer.config.output_format == OutputFormat::Detailed {
            url.query_pairs_mut().append_pair("wordLevelTimestamps", "true");
        }

        if recognizer.config.languages.len() > 1 {
            url.query_pairs_mut().append_pair("lidEnabled", true.to_string().as_str());
        }

        if let Some(ref connection_id) = recognizer.config.connection_id {
            url.query_pairs_mut().append_pair("X-ConnectionId", connection_id.as_str());
        }

        ClientConfig::new(url)
    }
}


async fn sender(connector: &Connector,
                mut audio_rx: mpsc::Receiver<Vec<u8>>,
                spec: Spec,
                details: Details,
                azure_speech_config: Device,
                config: RecognizerConfig,
                mut message_rx: mpsc::Receiver<Message>,
) -> crate::Result<()> {
    let mut buffer = Vec::with_capacity(4096);

    'outer: loop {
        let uuid = uuid::Uuid::new_v4();
        // send config
        send_message(&connector, create_speech_config_message(uuid, &config, &azure_speech_config, &spec, &details))?;
        // send context
        send_message(&connector, create_speech_context_message(uuid, &config))?;
        // send audio headers
        send_message(&connector, create_speech_audio_headers_message(uuid, "audio/x-wav", &spec))?;

        loop {
            tokio::select! {

                Some(message) = message_rx.recv() => {
                    // if the message is a text message and the path is "turn.end" break the loop,
                    // so the outer loop can be executed again.
                    if let Message::Text {path, ..} = message {
                        if path.as_str() == "turn.end" {
                            break;
                        }
                    }
                }
                data = audio_rx.recv() => {
                    // in case there is some data:
                    if let Some(data) = data {
                        buffer.extend(data);
                        if buffer.len() >= 4096 {
                            send_audio_data(&connector, uuid, &mut buffer)?;
                        }
                        // continue the loop awaiting more data or messages.
                        continue
                    }
                    // in case there is no data, send the remaining buffer
                    while !buffer.is_empty() {
                        send_audio_data(&connector, uuid, &mut buffer)?;
                    }
                    send_message(&connector, create_speech_audio_message(uuid, None))?;
                    // end the sender by exiting the outer loop
                    // in this case I don't care anymore of any message that could be received.
                    break 'outer;
                }
            }
        }
    }

    debug!("Exiting sender");

    Ok(())
}


fn send_audio_data(connector: &Connector, uuid: Uuid, buffer: &mut Vec<u8>) -> crate::Result<()> {
    let data = buffer.drain(..std::cmp::min(buffer.len(), 4096)).collect();
    send_message(connector, create_speech_audio_message(uuid, Some(data)))
}