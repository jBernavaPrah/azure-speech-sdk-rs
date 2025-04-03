use futures_util::SinkExt;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tokio_websockets::ClientBuilder;

enum InternalMessage {
    SendMessage(tokio_websockets::Message),
    Subscribe(
        oneshot::Sender<
            crate::Result<broadcast::Receiver<crate::Result<tokio_websockets::Message>>>,
        >,
    ),
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
    pub async fn send(&self, message: tokio_websockets::Message) -> crate::Result<()> {
        self.channel
            .send(InternalMessage::SendMessage(message))
            .await?;
        Ok(())
    }

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
    pub async fn stream(&self) -> crate::Result<impl Stream<Item = crate::Result<crate::Message>>> {
        let (sender, receiver) = oneshot::channel();
        self.channel
            .send(InternalMessage::Subscribe(sender))
            .await?;

        let br = BroadcastStream::new(receiver.await.map_err(|_| {
            crate::Error::InternalError("Failed to subscribe to messages".to_string())
        })??)
        .timeout(Duration::from_secs(30));

        let br = Box::pin(br);

        let br = br
            .map(move |m| {
                tracing::trace!("Downstream message: {:?}", m);
                m
            })
            .filter_map(move |message| match message {
                Ok(message) => message.ok(),
                // timeout error
                Err(e) => Some(Err(crate::Error::ConnectionError(e.to_string()))),
            })
            .map(move |message| {
                message.and_then(|msg| {
                    crate::Message::try_from(msg)
                        .map_err(|e| crate::Error::InternalError(e.to_string()))
                })
            })
            .map(move |m| m);

        Ok(br)
    }
}

impl Client {
    pub async fn connect(client: ClientBuilder<'static>) -> crate::Result<Self> {
        let (mut stream, _res) = client.connect().await?;
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
                                tracing::trace!("Upstream message: {:?}", msg.as_text());
                                let _ = stream.send(msg).await;
                            },
                            InternalMessage::Subscribe(c) => {
                                if !connected {
                                    // We got disconnected from the server for whatever reason. Since we are currently
                                    // expecting a stream, now would be a good time to try to reconnect.
                                    let mut last_error = None;
                                    for i in 0..3 {
                                        tracing::debug!("Reconnecting ({i}/3)");

                                        match client.connect().await {

                                            Ok((new_stream, _)) => {
                                                tracing::debug!("Reconnected successfully");
                                                drop(last_error.take());
                                                connected = true;
                                                stream = new_stream;
                                                break;
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to reconnect ({i}/3): {e}");
                                                last_error.replace(e);
                                            }
                                        }
                                    }

                                    // If we still haven't reconnected, send the error to the client.
                                    if let Some(err) = last_error.take() {
                                        let _ = c.send(Err(crate::Error::ConnectionError(err.to_string())));
                                        continue;
                                    }
                                }

                                let _ = c.send(Ok(broadcaster.subscribe()));
                            },
                            InternalMessage::Disconnect => {
                                let _ = stream.close().await;
                                break;
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

                                if msg.is_text() || msg.is_binary() {
                                    let _ = broadcaster.send(Ok(msg.clone()));
                                } else if msg.is_close() {
                                    connected = false;

                                    let close = msg.as_close().unwrap();
                                    let _ = broadcaster.send(Err(crate::Error::ServerDisconnect(format!("{:?}", close))));
                                    tracing::warn!(reason = ?close.0, msg = close.1, "disconnected from server");
                                }
                            },
                            Err(e) => {
                                tracing::warn!(?e, "connection errored");
                                let _ = broadcaster.send(Err(e.into()));
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
    pub(crate) async fn disconnect(&self) -> crate::Result<()> {
        self.channel.send(InternalMessage::Disconnect).await?;
        // await the client to disconnect.
        self.channel.closed().await;
        Ok(())
    }
}
