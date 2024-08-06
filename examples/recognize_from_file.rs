use std::{env};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use tokio_stream::{Stream, StreamExt};
use tokio_stream::wrappers::ReceiverStream;
use azure_speech::Auth;
use azure_speech::recognizer;
use azure_speech::recognizer::Details;

#[tokio::main]
async fn main() -> azure_speech::Result<()> {

    // Initialize the logger
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();


    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let client = recognizer::Client::connect(auth, recognizer::Config::default()).await?;
    
    // if you change the path, make sure to change the content-type accordingly in the next line
    let audio_stream = create_audio_stream("tests/audios/examples_sample_files_turn_on_the_lamp.mp3").await;
    let mut stream = client.recognize(audio_stream, recognizer::ContentType::Mp3, Details::file()).await?;

    while let Some(event) = stream.next().await {
        tracing::info!("Event: {:?}", event);
    }

    tracing::info!("Completed!");

    Ok(())
}

async fn create_audio_stream(path: impl AsRef<Path>) -> impl Stream<Item=Vec<u8>> {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    let file = File::open(path).await.expect("Failed to open file");
    let mut reader = BufReader::new(file);
    
    tokio::spawn(async move {
        let mut chunk = vec![0; 4096];
        while let Ok(n) = reader.read(&mut chunk).await {
            if n == 0 {
                break;
            }
            if tx.send(chunk.clone()).await.is_err() {
                tracing::error!("Error sending data");
                break;
            }
        }
        drop(tx);
    });

    ReceiverStream::new(rx)
}