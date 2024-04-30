
use log::{error, info};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;
use crate::connector::connect;
use crate::connector::message::Message;
use crate::speech_to_text::event::cancelled::{CancelCode, CancelReason};
use crate::speech_to_text::event::{Event, TryFromMessage};
use crate::speech_to_text::config::ResolverConfig;
use crate::speech_to_text::utils::{create_speech_audio_headers_message, create_speech_audio_message, create_speech_config_message, create_speech_context_message, generate_uri_for_stt_speech_azure};

pub struct RecognizerClient<T>
{
    pub uuid: Uuid,
    pub event_tx: Sender<Event<T>>,
    pub event_rx: Receiver<Event<T>>,
}


impl<T> RecognizerClient<T>
    where T: Send + TryFromMessage<T> + 'static{
    pub async fn connect(config: ResolverConfig) -> crate::errors::Result<Self> {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(1024);

        let uuid = Uuid::new_v4();

        let (upstream_sender, upstream_receiver) = connect(generate_uri_for_stt_speech_azure(&config)).await?;

        tokio::spawn(upstream_audio(uuid, config, upstream_sender, event_tx.clone()));
        tokio::spawn(upstream_listen(uuid, upstream_receiver, event_tx.clone()));

        Ok(RecognizerClient {
            uuid,
            event_tx,
            event_rx,
        })
    }
}

async fn upstream_audio<T>(uuid: Uuid, mut config: ResolverConfig, upstream_sender: Sender<Message>, event_tx: Sender<Event<T>>) -> ()
    where T: Send + TryFromMessage<T> {
    event_tx.send(Event::SessionStarted { session_id: uuid }).await.unwrap();

    // send config
    if upstream_sender.send(create_speech_config_message(uuid, &config)).await.is_err() {
        event_tx.send(Event::Cancelled { code: CancelCode::ConnectionFailure, reason: CancelReason::Error }).await.unwrap();
    };

    // send context
    if upstream_sender.send(create_speech_context_message(uuid, &config)).await.is_err() {
        event_tx.send(Event::Cancelled { code: CancelCode::RuntimeError, reason: CancelReason::Error }).await.unwrap();
    }

    // send audio headers
    if upstream_sender.send(create_speech_audio_headers_message(uuid, "audio/x-wav".to_string(), config.source.headers)).await.is_err() {
        event_tx.send(Event::Cancelled { code: CancelCode::RuntimeError, reason: CancelReason::Error }).await.unwrap();
    }

    // load buffer and send audio data.
    // buffer length is 4kb
    let mut buffer: Vec<u8> = Vec::with_capacity(4096);

    while let Some(data) = config.source.source.recv().await {
        buffer.extend_from_slice(data.as_slice());

        if buffer.len() < 4096 {
            continue;
        }

        // remove from the buffer the first 4096 bytes. Leave in the buffer the rest of the data
        let first_part = buffer.drain(..4096).collect();

        // send audio data
        if upstream_sender.send(create_speech_audio_message(uuid, Some(first_part))).await.is_err() {
            event_tx.send(Event::Cancelled { code: CancelCode::RuntimeError, reason: CancelReason::Error }).await.unwrap();
            break;
        }
    }

    if buffer.len() > 0 {
        if upstream_sender.send(create_speech_audio_message(uuid, Some(buffer))).await.is_err() {
            event_tx.send(Event::Cancelled { code: CancelCode::RuntimeError, reason: CancelReason::Error }).await.unwrap();
        }
    }

    // end of audio data
    if upstream_sender.send(create_speech_audio_message(uuid, None)).await.is_err() {
        event_tx.send(Event::Cancelled { code: CancelCode::RuntimeError, reason: CancelReason::Error }).await.unwrap();
    }

    info!("Exiting sending loop");
}

async fn upstream_listen<T>(uuid: Uuid, mut upstream_receiver: Receiver<Message>, event_tx: Sender<Event<T>>) -> ()
    where T: Send + TryFromMessage<T> {
    while let Some(message) = upstream_receiver.recv().await {

        // minimum requirements to continue
        if message.path().is_none() {
            error!("Received message without a path");
            event_tx.send(Event::Cancelled { code: CancelCode::RuntimeError, reason: CancelReason::Error }).await.unwrap();

            break;
        }

        if message.is_text() && message.path() == Some("turn.start".to_string()) {
            continue;
        }

        if message.is_text() && message.path() == Some("turn.end".to_string()) {
            event_tx.send(Event::SessionStopped { session_id: uuid }).await.unwrap();
            break;
        }

        // convert the message to an event
        let event = T::try_from_message(&message).unwrap_or_else(|_| {
            error!("Error converting message to event. {:?}", message);
            Some(Event::Cancelled { code: CancelCode::RuntimeError, reason: CancelReason::Error })
        });

        if let Some(Event::Cancelled { .. }) = event {
            event_tx.send(event.unwrap()).await.unwrap();
            break;
        }

        // if there is some event, send it
        if event.is_some() {
            event_tx.send(event.unwrap()).await.unwrap()
        }

        // otherwise go to the next message
    }
    drop(event_tx);
    info!("Exiting listen loop");
}