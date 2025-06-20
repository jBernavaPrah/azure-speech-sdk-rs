mod common;

use azure_speech::{synthesizer, Message};
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio_websockets::{ClientBuilder, WebSocketStream};

fn synthesizer_server() -> impl Fn(WebSocketStream<TcpStream>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Clone {
    |mut ws: WebSocketStream<TcpStream>| Box::pin(async move {
        let request_id = match ws.next().await {
            Some(Ok(msg)) => Message::try_from(msg).unwrap().id,
            _ => return,
        };

        // speech.context and ssml
        ws.next().await;
        ws.next().await;

        use crate::common::{make_binary_payload, make_text_payload};

        let start = make_text_payload(
            vec![
                ("X-RequestId".to_string(), request_id.clone()),
                ("Path".to_string(), "turn.start".to_string()),
                (
                    "Content-Type".to_string(),
                    "application/json; charset=utf-8".to_string(),
                ),
            ],
            Some("{\"webrtc\":{\"connectionString\":\"abc\"}}"),
        );
        let response = make_text_payload(
            vec![
                ("X-RequestId".to_string(), request_id.clone()),
                ("Path".to_string(), "response".to_string()),
                (
                    "Content-Type".to_string(),
                    "application/json; charset=utf-8".to_string(),
                ),
            ],
            Some("{\"audio\":{\"streamId\":\"stream\",\"type\":\"main\"}}"),
        );
        let audio1 = make_binary_payload(
            vec![
                ("X-RequestId".to_string(), request_id.clone()),
                ("Path".to_string(), "audio".to_string()),
                ("X-StreamId".to_string(), "stream".to_string()),
            ],
            Some(&[1, 2, 3]),
        );
        let audio2 = make_binary_payload(
            vec![
                ("X-RequestId".to_string(), request_id.clone()),
                ("Path".to_string(), "audio".to_string()),
                ("X-StreamId".to_string(), "stream".to_string()),
            ],
            None,
        );
        let end = make_text_payload(
            vec![
                ("X-RequestId".to_string(), request_id.clone()),
                ("Path".to_string(), "turn.end".to_string()),
            ],
            None,
        );

        ws.send(tokio_websockets::Message::text(start)).await.unwrap();
        ws.send(tokio_websockets::Message::text(response)).await.unwrap();
        ws.send(tokio_websockets::Message::binary(audio1)).await.unwrap();
        ws.send(tokio_websockets::Message::binary(audio2)).await.unwrap();
        ws.send(tokio_websockets::Message::text(end)).await.unwrap();

        let _ = ws.close().await;
    })
}

#[tokio::test]
async fn functional_disconnect_reconnect_synthesizer() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let address = "127.0.0.1:4567";

    common::start_server(
        address,
        VecDeque::from_iter(vec![synthesizer_server(), synthesizer_server()]),
    )
    .await;

    let uri = Uri::from_str(&format!("ws://{}", address)).unwrap();
    let client = azure_speech::connector::Client::connect(ClientBuilder::from_uri(uri.clone()))
        .await
        .unwrap();

    let synthesizer = synthesizer::Client::new(client, synthesizer::Config::default());

    synthesizer
        .synthesize("hello")
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;

    synthesizer.disconnect().await.unwrap();

    let client = azure_speech::connector::Client::connect(ClientBuilder::from_uri(uri))
        .await
        .unwrap();

    let synthesizer = synthesizer::Client::new(client, synthesizer::Config::default());

    synthesizer
        .synthesize("hello")
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;
}
