use log::{error, info};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;
use crate::connector::connect;
use crate::connector::message::Message;
use crate::recognizer::event::{CancelledReason, EventError, FromMessage};
use crate::recognizer::event::{EventBase, Event};
use crate::recognizer::config::ResolverConfig;
use crate::recognizer::Source;
use crate::recognizer::Sample;
use crate::recognizer::utils::{create_speech_audio_headers_message, create_speech_audio_message, create_speech_config_message, create_speech_context_message, generate_uri_for_stt_speech_azure};

pub async fn recognize<T, S: Sample>(config: ResolverConfig, source: Source<S>) -> crate::errors::Result<(Sender<Event<T>>, Receiver<Event<T>>)>
    where T: Send + FromMessage<T> + 'static {
    {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(1024);

        let uuid = Uuid::new_v4();

        let (upstream_sender, upstream_receiver) = connect(generate_uri_for_stt_speech_azure(&config)).await?;

        let audio_event_tx = event_tx.clone();

        tokio::spawn(async move {
            audio_event_tx.send(Event::Base(EventBase::SessionStarted { session_id: uuid })).await.unwrap();

            match upstream_audio(uuid, config, source, upstream_sender).await {
                Ok(_) => info!("Upstream audio finished"),
                Err(e) => {
                    error!("Error in upstream audio: {:?}", e);
                    audio_event_tx.send(Event::Base(EventBase::Cancelled { reason: CancelledReason::RuntimeError })).await.unwrap();
                }
            }
        });

        tokio::spawn(upstream_listen(uuid, upstream_receiver, event_tx.clone()));

        Ok((event_tx, event_rx))
    }
}


async fn upstream_audio<S>(uuid: Uuid, config: ResolverConfig, mut source: Source<S>, upstream_sender: Sender<Message>) -> crate::errors::Result<()>
    where S: Sample
{

    // send config
    upstream_sender.send(create_speech_config_message(uuid, &config, &source)).await?;

    // send context
    upstream_sender.send(create_speech_context_message(uuid, &config)).await?;

    // send audio headers
    upstream_sender.send(create_speech_audio_headers_message(uuid, "audio/x-wav".to_string(), source.spec)).await?;

    // load buffer and send audio data.
    // buffer length is 4kb
    let mut buffer: Vec<u8> = Vec::with_capacity(4096);

    while let Some(data) = source.next().await {

        for d in data {
            buffer.extend_from_slice(&d.to_le_bytes());
        }

        if buffer.len() < 4096 {
            continue;
        }

        // remove from the buffer the first 4096 bytes. Leave in the buffer the rest of the data
        let first_part = buffer.drain(..4096).collect();

        // send audio data
        upstream_sender.send(create_speech_audio_message(uuid, Some(first_part))).await?;
    }

    if buffer.len() > 0 {
        upstream_sender.send(create_speech_audio_message(uuid, Some(buffer))).await?
    }

    // end of audio data
    upstream_sender.send(create_speech_audio_message(uuid, None)).await?;

    info!("Exiting sending loop");

    Ok(())
}

async fn upstream_listen<T>(uuid: Uuid, mut upstream_receiver: Receiver<Message>, event_tx: Sender<Event<T>>) -> ()
    where T: Send + FromMessage<T> + 'static {
    while let Some(message) = upstream_receiver.recv().await {

        // minimum requirements to continue
        if message.path().is_none() {
            error!("Received message without a path");
            event_tx.send(Event::Base(EventBase::Cancelled { reason: CancelledReason::RuntimeError })).await.unwrap();

            break;
        }

        if message.is_text() && message.path() == Some("turn.start".to_string()) {
            continue;
        }

        if message.is_text() && message.path() == Some("turn.end".to_string()) {
            event_tx.send(Event::Base(EventBase::SessionStopped { session_id: uuid })).await.unwrap();
            break;
        }


        let event = match T::from_message(&message) {
            Err(EventError::Unprocessable) | Err(EventError::NoPath) => {
                // try to convert the message to a base event
                EventBase::from_message(&message)
                    .unwrap_or_else(|e| {
                        // if also this is not possible, then cancel the session!
                        error!("Error converting message to event. Error: {:?} - Message: {:?}", e, message);
                        Event::Base(EventBase::Cancelled { reason: CancelledReason::RuntimeError })
                    })
            }
            Err(EventError::Skip) => continue,
            Ok(e) => e,
        };

        // if the event is a cancel event, then exit the loop
        if let Event::Base(EventBase::Cancelled { .. }) = event {
            // notify the client that the session is cancelled
            event_tx.send(event).await.unwrap();
            // exit the loop
            break;
        }

        // send the event to the client
        event_tx.send(event).await.unwrap();
    }
    drop(event_tx);
    info!("Exiting listen loop");
}