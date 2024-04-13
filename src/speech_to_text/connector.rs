use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, trace};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message as TMessage;
use crate::errors::Result;
use crate::speech_to_text::request::Message as UpMessage;
use crate::speech_to_text::response::Message as DownMessage;

type WS = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub(crate) struct SenderConnector {
    pub(crate) sender: SplitSink<WS, TMessage>,
    pub(crate) session_id: String,
}

impl SenderConnector {
    pub(crate) fn new(sender: SplitSink<WS, TMessage>, session_id: String) -> Self {
        SenderConnector {
            sender,
            session_id,
        }
    }
    pub(crate) async fn send(&mut self, request: UpMessage) -> Result<()> {
        let request_clone = request.clone();
        match request_clone {
            UpMessage::Audio { data } => {
                debug!("Sending audio data: {:?}", data.len());
                trace!("Sending audio data: {:X?}", data);
            }
            _ => debug!("Sending request: {:?}", request_clone),
        }

        self.sender.send(request.into_message(self.session_id.clone())).await?;
        debug!("Message sent successfully");
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ReceiverConnector {
    pub(crate) receiver: SplitStream<WS>,
    pub(crate) session_id: String,
}

// impl Stream for ReceiverConnector {
//     type Item = Result<Response>;
//
//     fn poll_next(self: Pin<&mut ReceiverConnector>, cx: &mut Context<'_>) -> Poll<Option<Result<Response>>> {
//         ready!(self.receiver.poll_next(cx))
//     }
// }

impl ReceiverConnector {
    pub(crate) fn new(receiver: SplitStream<WS>, session_id: String) -> Self {
        ReceiverConnector {
            receiver,
            session_id,
        }
    }

    pub(crate) async fn next(&mut self) -> Option<Result<DownMessage>> {
        let message = self.receiver.next().await;

        return match message {
            Some(Ok(message)) => {
                debug!("Received message: {:?}", message);

                if message.is_ping() || message.is_pong() || message.is_binary() {
                    return Box::pin(self.next()).await;
                }

                if message.is_close() {
                    return None;
                }

                Some(Ok(message.into()))
            }
            Some(Err(e)) => {
                error!("Error reading message: {:?}", e);
                Some(Err(e.into()))
            }
            None => {
                None
            }
        };
    }
}


#[derive(Debug)]
pub(crate) struct Connector {
    sender: SenderConnector,
    receiver: ReceiverConnector,
}

impl Connector {
    pub(crate) fn new(sender: SenderConnector, receiver: ReceiverConnector) -> Self {
        Connector {
            sender,
            receiver,
        }
    }
    pub(crate) async fn connect(url: String, session_id: String) -> Result<Connector> {
        let (ws, res) = connect_async(url).await?;
        info!("Connected to Azure Websocket (Session: {:?}): {:?}", session_id, res);

        let (sender, receiver) = ws.split();

        Ok(Self::new(SenderConnector::new(sender, session_id.clone()), ReceiverConnector::new(receiver, session_id)))
    }

    pub(crate) fn split(self) -> (SenderConnector, ReceiverConnector) {
        let sender_connector = SenderConnector::new(self.sender.sender, self.sender.session_id.clone());
        let receiver_connector = ReceiverConnector::new(self.receiver.receiver, self.receiver.session_id.clone());

        (sender_connector, receiver_connector)
    }

    /// Sends a request to the websocket
    pub(crate) async fn send(&mut self, request: UpMessage) -> Result<()> {
        self.sender.send(request).await
    }

    /// Returns the next response from the websocket
    pub(crate) async fn next(&mut self) -> Option<Result<DownMessage>> {
        self.receiver.next().await
    }
}


