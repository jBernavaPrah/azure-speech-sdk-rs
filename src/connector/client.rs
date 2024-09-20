use crate::connector::message::Message;
use futures_util::SinkExt;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tokio_websockets::ClientBuilder;

enum InternalMessage {
    SendMessage(tokio_websockets::Message),
    Subscribe(oneshot::Sender<crate::Result<broadcast::Receiver<crate::Result<Message>>>>),
    Disconnect,
}

#[derive(Clone)]
pub struct Client {
    channel: mpsc::Sender<InternalMessage>,
}

impl Client {
    /// Create a new client.
    fn new(channel: mpsc::Sender<InternalMessage>) -> Self {
        Self { channel }
    }
}
impl Client {
    /// Send a text message to the server.
    pub async fn send_text(&self, text: impl Into<String>) -> crate::Result<()> {
        self.channel
            .send(InternalMessage::SendMessage(
                tokio_websockets::Message::text(text.into()),
            ))
            .await?;
        Ok(())
    }

    /// Send a binary message to the server.
    pub async fn send_binary(&self, bytes: impl Into<Vec<u8>>) -> crate::Result<()> {
        self.channel
            .send(InternalMessage::SendMessage(
                tokio_websockets::Message::binary(bytes.into()),
            ))
            .await?;
        Ok(())
    }

    /// Stream messages from the server.
    pub async fn stream(&self) -> crate::Result<BroadcastStream<crate::Result<Message>>> {
        let (sender, receiver) = oneshot::channel();
        self.channel
            .send(InternalMessage::Subscribe(sender))
            .await?;
        Ok(BroadcastStream::new(receiver.await.map_err(|_| {
            crate::Error::InternalError("Failed to subscribe to messages".to_string())
        })??))
    }
}

impl Client {
    pub(crate) async fn connect(config: ClientBuilder<'static>) -> crate::Result<Self> {
        let (mut stream, _res) = config.connect().await.unwrap();
        let (sender, mut receiver) = mpsc::channel(16);
        tokio::spawn(async move {
            let (broadcaster, _) = broadcast::channel(32);
            let mut connected = true;
            loop {
                tokio::select! {
                    msg = receiver.recv() => {
                        let Some(msg) = msg else {
                            // Receiving `None` here means the client has been dropped, so the task should stop as well.
                            break;
                        };
                        match msg {
                            InternalMessage::SendMessage(msg) => {
                                let _ = stream.send(msg).await;
                            },
                            InternalMessage::Subscribe(c) => {
                                if !connected {
                                    // We got disconnected from the server for whatever reason. Since we are currently
                                    // expecting a stream, now would be a good time to try to reconnect.
                                    let mut last_error = None;
                                    for i in 0..3 {
                                        tracing::debug!("Reconncting ({i}/3)");
                                        match config.connect().await {
                                            Ok((new_stream, _)) => {
                                                tracing::debug!("Reconnected successfully");
                                                drop(last_error.take());
                                                connected = true;
                                                stream = new_stream;
                                                break;
                                            }
                                            Err(e) => {
                                                tracing::warn!("Failed to reconnect ({i}/3): {e}");
                                                last_error.replace(e);
                                            }
                                        }
                                    }

                                    // If we still haven't reconnected, send the error to the client.
                                    if let Some(err) = last_error.take() {
                                        c.send(Err(crate::Error::ConnectionError(err.to_string()))).unwrap();
                                        continue;
                                    }
                                }

                                c.send(Ok(broadcaster.subscribe())).unwrap();
                            },
                            InternalMessage::Disconnect => {
                                connected = false;
                                let _ = stream.close().await;
                            }
                        }
                    }
                    msg = stream.next(), if connected => {
                        let Some(msg) = msg else {
                            // Receiving `None` here means the socket has been disconnected and can no longer receive messages.
                            // We set `connected` to false just to make sure that the stream isn't polled again until we're reconnected.
                            connected = false;
                            continue;
                        };
                        match msg {
                            Ok(msg) => {
                                if msg.is_text() {
                                    let text = msg.as_text().unwrap();
                                    broadcaster.send(Message::try_from(text)).unwrap();
                                } else if msg.is_binary() {
                                    let bin = msg.as_payload();
                                    broadcaster.send(Message::try_from(&**bin)).unwrap();
                                } else if msg.is_close() {
                                    connected = false;

                                    let close = msg.as_close().unwrap();
                                    tracing::info!(reason = ?close.0, msg = close.1, "disconnected from server");
                                }
                            },
                            Err(e) => {
                                tracing::warn!(?e, "connection errored");
                                connected = false;
                            }
                        }
                    }
                }
            }
        });
        Ok(Client::new(sender))
    }

    /// Disconnect the client.
    pub(crate) async fn disconnect(self) -> crate::Result<()> {
        self.channel.send(InternalMessage::Disconnect).await?;
        Ok(())
    }
}
