pub(crate) mod message;

use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use log::{debug, error};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as TMessage;
use crate::connector::message::Message;
use crate::errors::Result;


pub(crate) async fn connect<R>(request: R) -> Result<(Sender<Message>, Receiver<Message>)>
    where R: IntoClientRequest + Unpin {
    let (ws, res) = connect_async(request).await?;

    debug!("Connected to upstream: {:?}", res);

    let (sender, receiver) = ws.split();

    let (tx1, rx1) = tokio::sync::mpsc::channel(1024);
    let (tx2, rx2) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(send_message(sender, rx1));
    tokio::spawn(receive_message(receiver, tx2));

    Ok((tx1, rx2))
}


async fn send_message(mut sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, TMessage>, mut rx: Receiver<Message>) -> Result<()> {
    while let Some(message) = rx.recv().await {
        match message {
            Message::Text { .. } => {
                debug!("Sending message: {:?}", message);
            }
            Message::Binary { .. } => {
                debug!("Sending binary message. Headers {:?} Data: {:?} len", message.headers(), message.data().unwrap_or(vec![]).len());
            }
        }
        match sender.send(message.into()).await {
            Ok(_) => (),
            Err(e) => error!("Error sending message: {:?}", e),
        }
    }

    Ok(())
}

async fn receive_message(mut receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, tx: Sender<Message>) -> Result<()> {
    while let Some(message) = receiver.next().await {
        match message {
            Ok(message) => {
                debug!("Received message: {:?}", message);
                // ping and pongs are already handled by the library.
                if message.is_ping() || message.is_pong() {
                    debug!("Received ping/pong message");
                    continue;
                }

                if message.is_empty() {
                    debug!("Received empty message");
                    continue;
                }

                if message.is_close() {
                    debug!("Received close message");
                    break;
                }

                match tx.send(message.into()).await {
                    Ok(_) => (),
                    Err(e) => error!("Error sending message: {:?}", e),
                }
            }
            // todo: Do I need to break here?
            Err(e) => error!("Error receiving message: {:?}", e),
        }
    }
    Ok(())
}