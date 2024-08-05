use async_trait::async_trait;
use ezsockets::{ClientConfig, CloseCode, CloseFrame, Error};
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use ezsockets::client::ClientCloseMode;
use tokio_stream::wrappers::BroadcastStream;
use crate::connector::message::Message;


#[derive(Clone)]
pub struct Client {
    handle: ezsockets::Client<BaseClient>,
}

impl Client {
    /// Create a new client.
    pub(crate) fn new(handle: ezsockets::Client<BaseClient>) -> Self {
        Self { handle }
    }
}
impl Client {
    /// Send a text message to the server.
    pub fn send_text(&self, text: impl Into<String>) -> crate::Result<()> {
        self.handle.text(text)?;
        Ok(())
    }

    /// Send a binary message to the server.
    pub fn send_binary(&self, bytes: impl Into<Vec<u8>>) -> crate::Result<()> {
        self.handle.binary(bytes)?;
        Ok(())
    }

    /// Stream messages from the server.
    pub async fn stream(&self) -> crate::Result<BroadcastStream<crate::Result<Message>>> {
        self.handle.call_with(Call::Subscribe).await
            .map_or(Err(crate::Error::InternalError("Failed to subscribe to messages".to_string())), |rx| Ok(BroadcastStream::new(rx)))
    }
}

impl Client {
    pub(crate) async fn connect(config: ClientConfig) -> crate::Result<Self> {
        let (await_connection_tx, await_connection_rx) = oneshot::channel::<()>();
        let (client, future) = ezsockets::connect(|_| BaseClient::new(await_connection_tx), config).await;

        tokio::select! {
            _ = await_connection_rx => {
                tracing::debug!("Client is ready to send messages");
                Ok(Client::new(client))
            }
            _ = future => {
                tracing::error!("Connection closed before the client was ready to send messages");
                Err(crate::Error::ServerDisconnect("Connection closed before the client was ready to send messages".to_string()))
            }
        }
    }

    /// Disconnect the client.
    pub(crate) async fn disconnect(self) -> crate::Result<()> {
        let _ = self.handle.close(None)?;
        Ok(())
    }
}

pub(crate) enum Call {
    Subscribe(async_channel::Sender<broadcast::Receiver<crate::Result<Message>>>),
}

pub(crate) struct BaseClient {
    messages: broadcast::Sender<crate::Result<Message>>,
    ready: Option<oneshot::Sender<()>>,
}

impl BaseClient {
    pub(crate) fn new(ready: oneshot::Sender<()>) -> Self {
        let (sender, _) = broadcast::channel(1024*5);
        Self { messages: sender, ready: Some(ready) }
    }
}


#[async_trait]
impl ezsockets::ClientExt for BaseClient {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), Error> {
        tracing::debug!("Received text: {:?}", text);

        return match text.try_into() {
            Ok(value) => {
                self.messages.send(Ok(value))?;
                Ok(())
            }
            _ => Err(Error::from("Error parsing text".to_string())),
        };
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        tracing::debug!("Received binary: {:?}", bytes.len());
        tracing::trace!("Received Binary data: {:?}", bytes);
        return match bytes.try_into() {
            Ok(value) => {
                self.messages.send(Ok(value))?;
                Ok(())
            }
            _ => Err(Error::from("Error parsing bytes".to_string())),
        };
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        match call {
            Call::Subscribe(respond_to) => {
                let _ = respond_to.send(self.messages.subscribe()).await?;
            }
        };
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), Error> {
        // send the ready signal.
        // This is used to notify the connector that the client is ready to send messages.
        if let Some(ready) = self.ready.take() {
            let _ = ready.send(());
        }

        Ok(())
    }

    async fn on_close(&mut self, frame: Option<CloseFrame>) -> Result<ClientCloseMode, Error> {
        tracing::debug!("Server close the connection...{:?}", frame);

        match frame {
            Some(CloseFrame { code, reason }) => {
                let mode = match code {
                    CloseCode::Restart | CloseCode::Again | CloseCode::Normal => {
                        tracing::debug!("Reconnecting...");
                        ClientCloseMode::Reconnect
                    }
                    _ => {
                        tracing::debug!("Sending server error message...");
                        self.messages.send(Err(crate::Error::ServerDisconnect(reason.clone().to_string())))?;
                        tracing::debug!("Closing...");
                        ClientCloseMode::Close
                    }
                };
                tracing::debug!("Close mode: {:?}", mode);
                Ok(mode)
            }
            None => {
                tracing::debug!("Reconnecting...");
                Ok(ClientCloseMode::Reconnect)
            }
        }
    }
}