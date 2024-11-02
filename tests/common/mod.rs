use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use tokio::net::{TcpListener, TcpStream};
use tokio_websockets::{Error, Message, ServerBuilder, WebSocketStream};

pub async fn start_server<F>(address: impl Into<String>, mut connections_to_test: VecDeque<F>)
where
    F: Fn(WebSocketStream<TcpStream>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        + Send
        + 'static,
{
    let listener = TcpListener::bind(address.into()).await.unwrap();

    tokio::spawn(async move {
        while let Ok((s, _)) = listener.accept().await {
            let ws_stream = ServerBuilder::new().accept(s).await?;
            let next = connections_to_test
                .pop_front()
                .expect("Unexpected connection!");

            tokio::spawn(next(ws_stream));

            // let (mut write, mut read) = ws_stream.split();
            //
            // for (recv, send) in messages.to_owned() {
            //     if let Some(msg) = recv {
            //         let next_msg = tokio::time::timeout(Duration::from_millis(100), read.next())
            //             .await
            //             .expect(format!("Server Timeout! awaiting for {:?}", msg).as_str())
            //             .expect("Server Stream ended unexpectedly");
            //
            //         match next_msg {
            //             Ok(next_msg) => {
            //                 println!("Server: {:?} - {:?}", msg, next_msg);
            //                 assert!(messages_match(msg, next_msg), "Message does not match");
            //             }
            //             Err(e) => {
            //                 println!("Server Error: {}", e);
            //             }
            //         }
            //     }
            //
            //     if let Some(msg) = send {
            //         write.send(msg).await.expect("Couldn't send message");
            //     }
            // }
            // write.close().await.expect("Couldn't send message");
        }

        Ok::<_, Error>(())
    });
}
#[allow(dead_code)]
pub fn messages_match(msg1: Message, msg2: Message) -> bool {
    if !msg2.is_text() || !msg1.is_binary() {
        return true;
    }

    (msg1.is_text() && msg2.is_text() && msg1.as_text() == msg2.as_text())
        || (msg1.is_binary()
            && msg2.is_binary()
            && msg1.as_payload().to_vec() == msg2.as_payload().to_vec())
}
