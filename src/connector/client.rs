use async_trait::async_trait;
use ezsockets::{ClientConfig, CloseCode, CloseFrame, Error};
use async_broadcast::{Receiver, Sender};
use ezsockets::client::ClientCloseMode;
use crate::message::Message;

pub type Client = ezsockets::Client<BaseClient>;

pub struct BaseClient {
    handle: ezsockets::Client<Self>,
    sender: Sender<crate::Result<Message>>,
    ready: Option<tokio::sync::oneshot::Sender<()>>,
}

impl BaseClient {
    pub fn new(handle: ezsockets::Client<Self>,
               sender: Sender<crate::Result<Message>>,
               ready: tokio::sync::oneshot::Sender<()>) -> Self {
        Self { handle, sender, ready: Some(ready) }
    }
}

pub enum Call {
    Text(String),
    Binary(Vec<u8>),
}

#[async_trait]
impl ezsockets::ClientExt for BaseClient {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), Error> {
        return match text.try_into() {
            Ok(value) => {
                self.sender.broadcast(Ok(value)).await?;
                Ok(())
            }
            _ => Err(Error::from("Error parsing text".to_string())),
        };
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        return match bytes.try_into() {
            Ok(value) => {
                self.sender.broadcast(Ok(value)).await?;
                Ok(())
            }
            _ => Err(Error::from("Error parsing bytes".to_string())),
        };
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        match call {
            Call::Text(text) => self.handle.text(text)?,
            Call::Binary(bytes) => self.handle.binary(bytes)?,
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
                    CloseCode::Restart | CloseCode::Again => {
                        tracing::debug!("Reconnecting...");
                        ClientCloseMode::Reconnect
                    }
                    _ => {
                        tracing::debug!("Sending server error message...");
                        self.sender.broadcast(Err(crate::Error::ServerDisconnect(reason.clone().to_string()))).await?;
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

pub(crate) async fn connect(config: ClientConfig) -> (Client, Receiver<crate::Result<Message>>) {
    let (sender, receiver) = async_broadcast::broadcast(100);
    let (await_connection_tx, await_connection_rx) = tokio::sync::oneshot::channel::<()>();

    let (client, future) = ezsockets::connect(|client| BaseClient::new(client, sender, await_connection_tx), config).await;

    tokio::spawn(async move {
        tracing::debug!("Inside the connection task");
        let _ = future.await;

        tracing::debug!("Connection closed");
    });

    // todo: await the connection and handle any error could arise. 
    let _ = await_connection_rx.await;

    (client, receiver)
}