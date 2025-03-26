use crate::connector::Client as BaseClient;
use crate::recognizer::audio_format::AudioFormat;
use crate::recognizer::session::Session;
use crate::recognizer::utils::{
    create_audio_header_message, create_audio_message, create_speech_config_message,
    create_speech_context_message,
};
use crate::recognizer::{
    AudioDevice, Confidence, Config, Event, OutputFormat, PrimaryLanguage, Recognized,
};
use crate::utils::get_azure_hostname_from_region;
use crate::{stream_ext::StreamExt, Auth, Data, Message};
use std::cmp::min;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt as _};
use tracing::warn;
use url::Url;

const BUFFER_SIZE: usize = 4096;

#[derive(Clone)]
pub struct Client {
    pub client: BaseClient,
    pub config: Config,
}

impl Client {
    pub fn new(client: BaseClient, config: Config) -> Self {
        Self { client, config }
    }

    pub async fn connect(auth: Auth, config: Config) -> crate::Result<Self> {
        let base_url = format!(
            "wss://{}.stt.speech{}/speech/recognition/{}/cognitiveservices/v1",
            auth.region,
            get_azure_hostname_from_region(&auth.region),
            config.mode.as_str()
        );
        let mut url = Url::parse(&base_url)?;

        let language = config
            .languages
            .first()
            .ok_or_else(|| crate::Error::IOError("No language specified.".to_string()))?;
        url.query_pairs_mut()
            .append_pair("language", language.to_string().as_str())
            .append_pair("format", config.output_format.as_str())
            .append_pair("profanity", config.profanity.as_str())
            .append_pair("storeAudio", &config.store_audio.to_string());
        if config.output_format == OutputFormat::Detailed {
            url.query_pairs_mut()
                .append_pair("wordLevelTimestamps", "true");
        }
        if config.languages.len() > 1 {
            url.query_pairs_mut().append_pair("lidEnabled", "true");
        }
        if let Some(ref connection_id) = config.connection_id {
            url.query_pairs_mut()
                .append_pair("X-ConnectionId", connection_id);
        }

        let ws_client = tokio_websockets::ClientBuilder::new()
            .uri(url.as_str())
            .unwrap()
            .add_header(
                "Ocp-Apim-Subscription-Key".try_into().unwrap(),
                auth.subscription.to_string().as_str().try_into().unwrap(),
            )?
            .add_header(
                "X-ConnectionId".try_into().unwrap(),
                uuid::Uuid::new_v4().to_string().try_into().unwrap(),
            )?;

        let client = BaseClient::connect(ws_client).await?;
        Ok(Self::new(client, config))
    }

    pub async fn disconnect(&self) -> crate::Result<()> {
        self.client.disconnect().await
    }

    pub async fn recognize_file(
        &self,
        path: impl Into<std::path::PathBuf>,
    ) -> crate::Result<impl Stream<Item = crate::Result<Event>>> {
        let path = path.into();
        let file = tokio::fs::File::open(&path).await?;
        let ext = path
            .extension()
            .ok_or_else(|| crate::Error::IOError("Missing file extension.".to_string()))?;

        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(file);
            loop {
                let mut chunk = vec![0; BUFFER_SIZE];
                match reader.read(&mut chunk).await {
                    Ok(0) => break,
                    Ok(n) => {
                        chunk.truncate(n);
                        if let Err(e) = tx.send(chunk).await {
                            warn!("Failed to send chunk: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read chunk: {}", e);
                        break;
                    }
                }
            }
        });

        self.recognize(
            ReceiverStream::new(rx),
            ext.try_into()?,
            AudioDevice::file(),
        )
        .await
    }
    pub async fn recognize<A>(
        &self,
        mut audio: A,
        audio_format: AudioFormat,
        audio_device: AudioDevice,
    ) -> crate::Result<impl Stream<Item = crate::Result<Event>>>
    where
        A: Stream<Item = Vec<u8>> + Sync + Send + Unpin + 'static,
    {
        let messages = self.client.stream().await?;
        let session = Session::new();
        let config = self.config.clone();
        let client = self.client.clone();
        let (restart_tx, mut restart_rx) = tokio::sync::mpsc::channel(1);

        // Send the initial speech configuration.
        client
            .send(create_speech_config_message(
                session.request_id().to_string(),
                &config,
                &audio_device,
            ))
            .await?;

        // Send the initial context and audio header messages.
        client
            .send(create_speech_context_message(
                session.request_id().to_string(),
                &config,
            ))
            .await?;

        // For WAV audio, extract the header and extra data.
        let (audio_header, extra) = match audio_format {
            AudioFormat::Wav => {
                let (header, extra) = extract_header_from_wav(&mut audio).await?;
                (Some(header), extra)
            }
            _ => (None, vec![]),
        };

        // Create the audio data buffer and seed it with any extra bytes.
        let mut buffer = Vec::with_capacity(BUFFER_SIZE);
        buffer.extend(extra);

        client
            .send(create_audio_header_message(
                session.request_id().to_string(),
                audio_format.clone(),
                audio_header.as_deref(),
            ))
            .await?;

        let _session = session.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle any restart signal.
                    _ = restart_rx.recv() => {
                        tracing::info!("Refreshing audio header");
                        _session.refresh();

                        if client.send(create_audio_header_message(
                            _session.request_id().to_string(),
                            audio_format.clone(),
                            audio_header.as_deref(),
                        )).await.is_err() {
                            warn!("Failed to refresh audio header");
                            break;
                        }
                    },
                    // Process the next chunk from the audio stream.
                    maybe_chunk = audio.next() => {
                        match maybe_chunk {
                            Some(chunk) => {
                                // Append the new data to the buffer.
                                buffer.extend(chunk);
                                // While there is enough data, send it in fixed-size chunks.
                                while buffer.len() >= BUFFER_SIZE {
                                    let data: Vec<u8> = buffer.drain(..BUFFER_SIZE).collect();
                                    if client.send(create_audio_message(_session.request_id().to_string(), Some(&data))).await.is_err() {
                                        warn!("Failed to send audio message");
                                        break;
                                    }
                                }
                            }
                            None => {
                                // No more audio: flush remaining bytes in the buffer.
                                while !buffer.is_empty() {
                                    let data: Vec<u8> = buffer.drain(..min(buffer.len(), BUFFER_SIZE)).collect();
                                    if client.send(create_audio_message(_session.request_id().to_string(), Some(&data))).await.is_err() {
                                        warn!("Failed to send final audio chunk");
                                        break;
                                    }
                                }
                                // Signal the end of audio.
                                let _ = client.send(create_audio_message(_session.request_id().to_string(), None)).await;
                                _session.set_audio_completed(true);
                                break;
                            }
                        }
                    }
                }
            }
        });

        // Build the output stream that filters and converts messages into events.
        let session_clone = session.clone();
        let output_stream = messages
            .filter(move |msg| match msg {
                Ok(m) => m.id == session.request_id().to_string(),
                Err(_) => true,
            })
            .filter_map(move |msg| {
                let session_ref = session_clone.clone();
                match msg {
                    Ok(m) => convert_message_to_event(m, &session_ref),
                    Err(e) => Some(Err(e)),
                }
            })
            .map(move |event| {
                if let Ok(Event::SessionEnded(_)) = event {
                    let _ = restart_tx.try_send(());
                }
                event
            })
            .stop_after(|event| event.is_err());

        Ok(output_stream)
    }
}

fn convert_message_to_event(message: Message, session: &Session) -> Option<crate::Result<Event>> {
    match (message.path.as_str(), message.data, message.headers) {
        ("turn.start", _, _) => Some(Ok(Event::SessionStarted(session.request_id()))),
        ("speech.startdetected", Data::Text(Some(data)), _) => {
            serde_json::from_str::<crate::recognizer::message::SpeechStartDetected>(&data)
                .map(|v| Event::StartDetected(session.request_id(), v.offset))
                .map(Ok)
                .ok()
        }
        ("speech.enddetected", Data::Text(Some(data)), _) => {
            let value =
                serde_json::from_str::<crate::recognizer::message::SpeechEndDetected>(&data)
                    .unwrap_or_default();
            Some(Ok(Event::EndDetected(session.request_id(), value.offset)))
        }
        ("speech.hypothesis", Data::Text(Some(data)), _)
        | ("speech.fragment", Data::Text(Some(data)), _) => {
            match serde_json::from_str::<crate::recognizer::message::SpeechHypothesis>(&data) {
                Ok(value) => {
                    let offset = value.offset + session.audio_offset();
                    session.on_hypothesis_received(offset);
                    Some(Ok(Event::Recognizing(
                        session.request_id(),
                        Recognized {
                            text: value.text,
                            primary_language: value.primary_language.map(|l| {
                                PrimaryLanguage::new(
                                    l.language.into(),
                                    l.confidence.map_or(Confidence::Unknown, |c| c.into()),
                                )
                            }),
                            speaker_id: value.speaker_id,
                        },
                        offset,
                        value.duration,
                        data,
                    )))
                }
                Err(e) => Some(Err(crate::Error::ParseError(e.to_string()))),
            }
        }
        ("speech.phrase", Data::Text(Some(data)), _) => {
            match serde_json::from_str::<crate::recognizer::message::SpeechPhrase>(&data) {
                Ok(value) => {
                    let offset = value.offset.unwrap_or(0) + session.audio_offset();
                    let duration = value.duration.unwrap_or(0);
                    if value.recognition_status.is_end_of_dictation() {
                        return None;
                    }
                    if value.recognition_status.is_no_match() {
                        return Some(Ok(Event::UnMatch(
                            session.request_id(),
                            offset,
                            duration,
                            data,
                        )));
                    }
                    match serde_json::from_str::<crate::recognizer::message::SimpleSpeechPhrase>(
                        &data,
                    ) {
                        Ok(simple) => Some(Ok(Event::Recognized(
                            session.request_id(),
                            Recognized {
                                text: simple.display_text,
                                primary_language: simple.primary_language.map(|l| {
                                    PrimaryLanguage::new(
                                        l.language.into(),
                                        l.confidence.map_or(Confidence::Unknown, |c| c.into()),
                                    )
                                }),
                                speaker_id: simple.speaker_id,
                            },
                            offset,
                            duration,
                            data,
                        ))),
                        Err(e) => Some(Err(crate::Error::ParseError(e.to_string()))),
                    }
                }
                Err(e) => Some(Err(crate::Error::ParseError(e.to_string()))),
            }
        }
        ("turn.end", _, _) => Some(Ok(Event::SessionEnded(session.request_id()))),
        _ => None,
    }
}

async fn extract_header_from_wav(
    reader: &mut (impl Stream<Item = Vec<u8>> + Unpin + Send + Sync + 'static),
) -> Result<(Vec<u8>, Vec<u8>), crate::Error> {
    let mut header = Vec::new();

    // Loop until the stream is exhausted.
    while let Some(chunk) = reader.next().await {
        header.extend(chunk);

        // We need at least 12 bytes to check the RIFF and WAVE identifiers.
        if header.len() < 12 {
            continue;
        }

        // Check for a valid WAV header: bytes 0..4 must be "RIFF" and 8..12 must be "WAVE".
        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return Err(crate::Error::ParseError("Invalid wav header".to_string()));
        }

        // Look for the "data" descriptor.
        if let Some(pos) = header.windows(4).position(|w| w == b"data") {
            // Ensure we have read the 4 bytes following "data" (i.e. the length field).
            if header.len() < pos + 8 {
                // Not enough bytes yet; continue reading.
                continue;
            }
            // Split the header at the end of the "data" chunk descriptor and its length field.
            let header_end = pos + 8;
            let remainder = header.split_off(header_end);
            return Ok((header, remainder));
        }
    }

    Err(crate::Error::ParseError(
        "Reached end of stream without finding 'data' chunk".to_string(),
    ))
}
