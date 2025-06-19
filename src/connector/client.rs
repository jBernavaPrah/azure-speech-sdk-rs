use futures_util::SinkExt;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tokio_websockets::{self, ClientBuilder, MaybeTlsStream, WebSocketStream};

#[async_trait::async_trait]
trait Connector {
    async fn connect_stream(&self) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, tokio_websockets::Error>;
}

#[async_trait::async_trait]
impl Connector for ClientBuilder<'static> {
    async fn connect_stream(&self) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, tokio_websockets::Error> {
        Ok(self.connect().await?.0)
    }
}

async fn reconnect_with_attempts<C: Connector>(
    client: &C,
    attempts: usize,
) -> crate::Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
    let mut last_error = None;
    for i in 0..attempts {
        tracing::debug!("Reconnecting ({}/{})", i + 1, attempts);
        match client.connect_stream().await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                tracing::error!("Failed to reconnect ({}/{}): {}", i + 1, attempts, e);
                last_error.replace(e);
            }
        }
    }

    Err(crate::Error::ConnectionError(
        last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "reconnect failed".to_string()),
    ))
}

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
                Err(_e) => Some(Err(crate::Error::Timeout)),
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
                                    match reconnect_with_attempts(&client, 3).await {
                                        Ok(new_stream) => {
                                            connected = true;
                                            stream = new_stream;
                                        }
                                        Err(err) => {
                                            let _ = c.send(Err(err));
                                            continue;
                                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockConnector {
        fail_times: usize,
        calls: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl Connector for MockConnector {
        async fn connect_stream(
            &self,
        ) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, tokio_websockets::Error>
        {
            let attempt = self.calls.fetch_add(1, Ordering::SeqCst);
            if attempt < self.fail_times {
                Err(tokio_websockets::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "fail",
                )))
            } else {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();
                tokio::spawn(async move { let _ = listener.accept().await; });
                let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
                Ok(ClientBuilder::new().take_over(MaybeTlsStream::Plain(stream)))
            }
        }
    }

    #[tokio::test]
    async fn reconnect_helper_succeeds_after_retries() {
        let builder = MockConnector { fail_times: 2, calls: AtomicUsize::new(0) };
        let _ = reconnect_with_attempts(&builder, 3).await.expect("should connect");
        assert_eq!(builder.calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn reconnect_helper_fails_after_max_attempts() {
        let builder = MockConnector { fail_times: 5, calls: AtomicUsize::new(0) };
        let res = reconnect_with_attempts(&builder, 3).await;
        assert!(res.is_err());
        assert_eq!(builder.calls.load(Ordering::SeqCst), 3);
    }
}
