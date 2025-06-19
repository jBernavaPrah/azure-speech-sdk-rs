mod common;

use azure_speech::{recognizer, Data};
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::{panic, process};
use tokio::net::TcpStream;
use tokio_websockets::{ClientBuilder, WebSocketStream};
use tracing::{error, info};

#[tokio::test]
async fn integration_test_reconnect() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        error!("{}", panic_info);
        orig_hook(panic_info);
        process::exit(0);
    }));

    let address = "127.0.0.1:3456";

    common::start_server(
        address,
        VecDeque::from_iter(vec![
            |mut ws: WebSocketStream<TcpStream>| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                Box::pin(async move {
                    // get the first 4, then close the connection.
                    ws.next().await;
                    ws.next().await;
                    ws.next().await;
                    ws.next().await;

                    //let x = ws.chunks_timeout(4, Duration::from_secs(1)).await;
                    //info!("first 4 messages {:?}", x);

                    info!("1st Closing server");
                    tokio::spawn(async move {
                        ws.close().await.unwrap();
                    });
                })
            },
            |ws: WebSocketStream<TcpStream>| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                Box::pin(async move {
                    let (mut sink, stream) = ws.split();

                    info!("Start 2nd server");

                    let x = stream.take(4).collect::<Vec<_>>().await;
                    info!("first 4 messages {:?}", x);

                    info!("2st Closing server");
                    tokio::spawn(async move {
                        sink.close().await.unwrap();
                    });
                })
            },
            |mut ws: WebSocketStream<TcpStream>| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                Box::pin(async move {
                    info!("Receiving anything else...");

                    let mut msgs = vec![];
                    while let Some(msg) = ws.next().await {
                        match msg {
                            Ok(msg) => msgs.push(msg),
                            Err(err) => panic!("{}", err),
                        }
                    }

                    info!("Received {} messages", msgs.len());

                    info!("3st Closing server");
                    tokio::spawn(async move {
                        ws.close().await.unwrap();
                    });
                })
            },
        ]),
    )
    .await;

    info!("server started");

    let uri = Uri::from_str(format!("ws://{}", address).as_str()).unwrap();
    let client = azure_speech::connector::Client::connect(ClientBuilder::from_uri(uri))
        .await
        .unwrap();

    info!("client connected");

    let recognizer = recognizer::Client::new(client, recognizer::Config::default());

    let events = recognizer
        .recognize(
            tokio_stream::iter(vec![]),
            recognizer::AudioFormat::Mp3,
            recognizer::AudioDevice::unknown(),
        )
        .await
        .unwrap();

    let e: Vec<_> = events.collect().await;

    info!("{:?}", e);

    let events = recognizer
        .recognize(
            tokio_stream::iter(vec![]),
            recognizer::AudioFormat::Mp3,
            recognizer::AudioDevice::unknown(),
        )
        .await
        .unwrap();

    let e: Vec<_> = events.collect().await;
    info!("{:?}", e);

    let events = recognizer
        .recognize(
            tokio_stream::iter(vec![]),
            recognizer::AudioFormat::Mp3,
            recognizer::AudioDevice::unknown(),
        )
        .await
        .unwrap();

    let e: Vec<_> = events.collect().await;
    info!("{:?}", e);

    // while let Some(event) = events.next().await {
    //     println!("event: {:?}", event);
    // }
}

#[tokio::test]
async fn integration_test_recognizer() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        error!("{}", panic_info);
        orig_hook(panic_info);
        process::exit(0);
    }));

    let address = "127.0.0.1:3456";

    common::start_server(
        address,
        VecDeque::from_iter(vec![
            |ws: WebSocketStream<TcpStream>| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                Box::pin(async move {
                    let (_sink, mut stream) = ws.split();

                    match tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
                        .await
                    {
                        Ok(Some(Ok(msg))) => {
                            info!("{:?}", msg.as_text());
                            // convert to msg
                            let msg = azure_speech::Message::try_from(msg)
                                .expect("To convert to message");
                            assert_eq!(msg.path, "speech.config");
                            assert_eq!(
                                msg.headers
                                    .iter()
                                    .map(|h| h.0.clone())
                                    .collect::<Vec<String>>(),
                                vec!["Content-Type".to_string(), "X-Timestamp".to_string()]
                            );
                        }
                        e => {
                            panic!("Timed out waiting for messages: {:?}", e);
                        }
                    }

                    match tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
                        .await
                    {
                        Ok(Some(Ok(msg))) => {
                            let msg = azure_speech::Message::try_from(msg)
                                .expect("To convert to message");
                            assert_eq!(msg.path, "speech.context");
                            assert_eq!(
                                msg.headers.iter().map(|h| h.0.clone()).collect::<Vec<_>>(),
                                ["Content-Type", "X-Timestamp"]
                            );
                        }
                        e => {
                            panic!("Timed out waiting for messages: {:?}", e);
                        }
                    }

                    match tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
                        .await
                    {
                        Ok(Some(Ok(msg))) => {
                            let msg = azure_speech::Message::try_from(msg)
                                .expect("To convert to message");
                            assert_eq!(msg.path, "audio");
                            assert!(msg
                                .headers
                                .iter()
                                .find(|h| h.0 == "Content-Type" && h.1 == "audio/mpeg")
                                .is_some());
                            assert_eq!(
                                msg.headers.iter().map(|h| h.0.clone()).collect::<Vec<_>>(),
                                ["X-Timestamp", "Content-Type"]
                            );
                            assert_eq!(msg.data, Data::Binary(None))
                        }
                        e => {
                            panic!("Timed out waiting for messages: {:?}", e);
                        }
                    }

                    match tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
                        .await
                    {
                        Ok(Some(Ok(msg))) => {
                            let msg = azure_speech::Message::try_from(msg)
                                .expect("To convert to message");
                            assert_eq!(msg.path, "audio");
                            assert!(msg.headers.iter().find(|h| h.0 == "Content-Type").is_none());
                            assert_eq!(
                                msg.headers.iter().map(|h| h.0.clone()).collect::<Vec<_>>(),
                                ["X-Timestamp"]
                            );
                            assert_eq!(msg.data, Data::Binary(None))
                        }
                        e => {
                            panic!("Timed out waiting for messages: {:?}", e);
                        }
                    }

                    match tokio::time::timeout(std::time::Duration::from_millis(10), stream.next())
                        .await
                    {
                        Ok(_) => panic!("Not expecting anything new."),
                        _ => {}
                    }
                })
            },
        ]),
    )
    .await;

    info!("server started");

    let uri = Uri::from_str(format!("ws://{}", address).as_str()).unwrap();
    let client = azure_speech::connector::Client::connect(ClientBuilder::from_uri(uri))
        .await
        .unwrap();

    info!("client connected");

    let recognizer = recognizer::Client::new(client, recognizer::Config::default());

    let events = recognizer
        .recognize(
            tokio_stream::iter(vec![]),
            recognizer::AudioFormat::Mp3,
            recognizer::AudioDevice::unknown(),
        )
        .await
        .unwrap();

    events.collect::<Vec<_>>().await;

    //tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    // while let Some(event) = events.next().await {
    //     println!("event: {:?}", event);
    // }
}
