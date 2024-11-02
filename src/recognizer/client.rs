use crate::connector::Client as BaseClient;
use crate::recognizer::content_type::ContentType;
use crate::recognizer::session::Session;
use crate::recognizer::utils::{
    create_audio_message, create_speech_config_message, create_speech_context_message,
};
use crate::recognizer::{
    message, Confidence, Config, Details, Event, OutputFormat, PrimaryLanguage, Recognized,
};
use crate::utils::get_azure_hostname_from_region;
use crate::{stream_ext::StreamExt, Auth, Data, Message};
use tokio_stream::{Stream, StreamExt as _};
use url::Url;

static BUFFER_SIZE: usize = 4096;

/// Recognizer Client.
#[derive(Clone)]
pub struct Client {
    /// The client to send and receive messages.
    pub client: BaseClient,
    pub config: Config,
}

impl Client {
    pub fn new(client: BaseClient, config: Config) -> Self {
        Self { client, config }
    }

    pub async fn connect(auth: Auth, config: Config) -> crate::Result<Self> {
        let mut url = Url::parse(&format!(
            "wss://{}.stt.speech{}/speech/recognition/{}/cognitiveservices/v1",
            auth.region,
            get_azure_hostname_from_region(&auth.region),
            config.mode.as_str()
        ))?;

        let lang = config
            .languages
            .first()
            .expect("Select at least one language!");

        url.query_pairs_mut()
            .append_pair("language", lang.to_string().as_str());
        url.query_pairs_mut()
            .append_pair("format", config.output_format.as_str());
        url.query_pairs_mut()
            .append_pair("profanity", config.profanity.as_str());
        url.query_pairs_mut()
            .append_pair("storeAudio", config.store_audio.to_string().as_str());

        if config.output_format == OutputFormat::Detailed {
            url.query_pairs_mut()
                .append_pair("wordLevelTimestamps", "true");
        }

        if config.languages.len() > 1 {
            url.query_pairs_mut()
                .append_pair("lidEnabled", true.to_string().as_str());
        }

        if let Some(ref connection_id) = config.connection_id {
            url.query_pairs_mut()
                .append_pair("X-ConnectionId", connection_id.as_str());
        }

        let client = BaseClient::connect(
            tokio_websockets::ClientBuilder::new()
                .uri(url.as_str())
                .unwrap()
                .add_header(
                    "Ocp-Apim-Subscription-Key".try_into().unwrap(),
                    auth.subscription.to_string().as_str().try_into().unwrap(),
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
    /// Recognize audio from a stream.
    pub async fn recognize<A>(
        &self,
        stream: A,
        content_type: ContentType,
        details: Details,
    ) -> crate::Result<impl Stream<Item = crate::Result<Event>>>
    where
        A: Stream<Item = Vec<u8>> + Send + 'static,
    {
        let mut audio = Box::pin(stream);

        // todo: add on configuration the connection timeout
        let messages = self.client.stream().await?;

        let session = Session::new(uuid::Uuid::new_v4());
        let config = self.config.clone();
        let request_id = session.request_id().to_string();

        self.client
            .send(create_speech_config_message(
                request_id.clone(),
                &config,
                &details,
            ))
            .await?;

        self.client
            .send(create_speech_context_message(request_id.clone(), &config))
            .await?;

        // Here I'm moving away from the original code.
        // I'm not sending anymore in the audio headers but only the content-type of the stream.
        self.client
            .send(create_audio_message(
                request_id.clone(),
                Some(content_type), // <-- this is enough
                None,
            ))
            .await?;

        let client = self.client.clone();
        let session1 = session.clone();

        tokio::spawn(async move {
            // todo: add throttle to the audio stream.
            // src/common.speech/ServiceRecognizerBase.ts:857

            let mut buffer = Vec::with_capacity(BUFFER_SIZE);

            while let Some(chunk) = audio.next().await {
                buffer.extend(chunk);
                while buffer.len() >= BUFFER_SIZE {
                    let data: Vec<u8> = buffer.drain(..BUFFER_SIZE).collect();
                    if let Err(e) = client
                        .send(create_audio_message(
                            session1.request_id().to_string(),
                            None,
                            Some(&data),
                        ))
                        .await
                    {
                        tracing::error!("Error: {:?}", e);
                        return;
                    }
                }
            }

            while !buffer.is_empty() {
                let data: Vec<u8> = buffer.drain(..BUFFER_SIZE).collect();
                let _ = client
                    .send(create_audio_message(
                        session1.request_id().to_string(),
                        None,
                        Some(&data),
                    ))
                    .await;
            }
            // notify that we have finished sending the audio.
            let _ = client
                .send(create_audio_message(
                    session1.request_id().to_string(),
                    None,
                    None,
                ))
                .await;
            session1.set_audio_completed(true);
        });

        let session2 = session.clone();
        let session3 = session.clone();

        Ok(messages
            // Filter out messages that are not from the current session.
            .filter(move |message| match message {
                Ok(message) => message.id == request_id.clone(),
                Err(_) => true, // move the error to next step
            })
            .filter_map(move |message| match message {
                Ok(message) => convert_message_to_event(message, session2.clone()),
                Err(e) => Some(Err(e)),
            })
            // Merge the session started event with the other events.
            // todo: remove start session and use turn-start!
            .merge(tokio_stream::iter(vec![Ok(Event::SessionStarted(
                session3.request_id(),
            ))]))
            // Stop the stream if there is an error or the session ended.
            .stop_after(move |event| event.is_err() || matches!(event, Ok(Event::SessionEnded(_)))))
    }
}

fn convert_message_to_event(message: Message, session: Session) -> Option<crate::Result<Event>> {
    match (message.path.as_str(), message.data, message.headers) {
        // todo: check if another turn has started, before the latest finished?
        ("turn.start", _, _) => None, // todo: enable back this (it's the start session!)
        ("speech.startdetected", Data::Text(Some(data)), _) => {
            let value = match serde_json::from_str::<message::SpeechStartDetected>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };
            Some(Ok(Event::StartDetected(session.request_id(), value.offset)))
        }
        ("speech.enddetected", Data::Text(Some(data)), _) => {
            let value =
                serde_json::from_str::<message::SpeechEndDetected>(&data).unwrap_or_default();
            Some(Ok(Event::EndDetected(session.request_id(), value.offset)))
        }

        // speech recognizer
        ("speech.hypothesis", Data::Text(Some(data)), _)
        | ("speech.fragment", Data::Text(Some(data)), _) => {
            let value = match serde_json::from_str::<message::SpeechHypothesis>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };

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

        ("speech.phrase", Data::Text(Some(data)), _) => {
            // general check
            let value = match serde_json::from_str::<message::SpeechPhrase>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };

            session.on_phrase_recognized(
                value.offset.unwrap_or(0) + value.duration.unwrap_or(0) + session.audio_offset(),
            );

            let recognition_status = &value.recognition_status;
            let _error: Option<crate::Error> = match recognition_status.into() {
                None => None,
                Some(e) => return Some(Err(e)),
            };

            if recognition_status.is_end_of_dictation() {
                // this case is already mapped in the Event::EndDetected
                // if not correct, I will add a separate "Event::EndOfDictation"
                return None;
            }

            //todo: check if was required the simple or detailed recognition.
            // in case the detailed was requested, then get the first NBest, if present, otherwise teh DisplayText.

            let offset = value.offset.unwrap_or(0) + session.audio_offset();
            let duration = value.duration.unwrap_or(0);

            if value.recognition_status.is_no_match() {
                return Some(Ok(Event::UnMatch(
                    session.request_id(),
                    offset,
                    duration,
                    data,
                )));
            }

            // todo: in case of detailed phrase, we need to correct the offset and duration.

            let value = match serde_json::from_str::<message::SimpleSpeechPhrase>(&data) {
                Ok(value) => value,
                Err(e) => return Some(Err(crate::Error::ParseError(e.to_string()))),
            };

            Some(Ok(Event::Recognized(
                session.request_id(),
                Recognized {
                    text: value.display_text,
                    primary_language: value.primary_language.map(|l| {
                        PrimaryLanguage::new(
                            l.language.into(),
                            l.confidence.map_or(Confidence::Unknown, |c| c.into()),
                        )
                    }),
                    speaker_id: value.speaker_id,
                },
                offset,
                duration,
                data,
            )))
        }

        ("turn.end", _, _) => {
            if session.is_audio_completed() {
                return Some(Ok(Event::SessionEnded(session.request_id())));
            };

            None
        }

        _ => None,
    }
}
