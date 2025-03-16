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
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tokio::select;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt as _};
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
        path: impl Into<PathBuf>,
    ) -> crate::Result<impl Stream<Item = crate::Result<Event>>> {
        let path = path.into();
        let file = File::open(&path).await?;
        let reader = BufReader::new(file);
        let ext = path
            .extension()
            .ok_or_else(|| crate::Error::IOError("Missing file extension.".to_string()))?;
        let (audio_stream, audio_format) = create_audio_stream_from_reader(reader, ext).await?;
        self.recognize(audio_stream, audio_format, AudioDevice::file())
            .await
    }

    pub async fn recognize<A>(
        &self,
        mut audio: A,
        audio_format: AudioFormat,
        audio_device: AudioDevice,
    ) -> crate::Result<impl Stream<Item = crate::Result<Event>>>
    where
        A: Stream<Item = Vec<u8>> + Send + Unpin + 'static,
    {
        let messages = self.client.stream().await?;
        let session = Session::new();
        let config = self.config.clone();
        let client = self.client.clone();
        let (restart_sender, mut restart_rx) = tokio::sync::mpsc::channel(1);

        // Send initial configuration
        client
            .send(create_speech_config_message(
                session.request_id().to_string(),
                &config,
                &audio_device,
            ))
            .await?;

        // Spawn a task to send audio messages.
        tokio::spawn({
            let client = client.clone();
            let audio_format = audio_format.clone();
            let session = session.clone();
            async move {
                // Send context and header messages.
                if client
                    .send(create_speech_context_message(
                        session.request_id().to_string(),
                        &config,
                    ))
                    .await
                    .is_err()
                {
                    return;
                }
                if client
                    .send(create_audio_header_message(
                        session.request_id().to_string(),
                        audio_format.clone(),
                    ))
                    .await
                    .is_err()
                {
                    return;
                }

                let mut buffer = Vec::with_capacity(BUFFER_SIZE);
                loop {
                    select! {
                        _ = restart_rx.recv() => {
                            session.refresh();
                            break;
                        }
                        maybe_chunk = audio.next() => {
                            match maybe_chunk {
                                Some(chunk) => {
                                    buffer.extend(chunk);
                                    while buffer.len() >= BUFFER_SIZE {
                                        let data: Vec<u8> = buffer.drain(..BUFFER_SIZE).collect();
                                        if client.send(create_audio_message(session.request_id().to_string(), Some(&data))).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }

                // Flush any remaining data.
                while !buffer.is_empty() {
                    let data: Vec<u8> = buffer.drain(..min(buffer.len(), BUFFER_SIZE)).collect();
                    let _ = client
                        .send(create_audio_message(
                            session.request_id().to_string(),
                            Some(&data),
                        ))
                        .await;
                }
                // Signal end of audio stream.
                let _ = client
                    .send(create_audio_message(session.request_id().to_string(), None))
                    .await;
                session.set_audio_completed(true);
            }
        });

        let session_clone = session.clone();
        let output_stream = messages
            .filter(move |msg| match msg {
                Ok(m) => m.id == session.request_id().to_string(),
                Err(_) => true,
            })
            .filter_map(move |msg| {
                let session_ref = session_clone.clone();
                {
                    match msg {
                        Ok(m) => convert_message_to_event(m, &session_ref),
                        Err(e) => Some(Err(e)),
                    }
                }
            })
            .map(move |event| {
                if let Ok(Event::SessionEnded(_)) = event {
                    let _ = restart_sender.try_send(());
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

async fn create_audio_stream_from_reader(
    mut reader: impl AsyncRead + Unpin + Send + Sync + 'static,
    extension: &OsStr,
) -> Result<(impl Stream<Item = Vec<u8>>, AudioFormat), crate::Error> {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    let audio_format = AudioFormat::try_from_reader(&mut reader, extension).await?;

    tokio::spawn(async move {
        let mut buf = vec![0; BUFFER_SIZE];
        while let Ok(n) = reader.read(&mut buf).await {
            if n == 0 {
                break;
            }
            if tx.send(buf[..n].to_vec()).await.is_err() {
                break;
            }
        }
    });

    Ok((ReceiverStream::new(rx), audio_format))
}
