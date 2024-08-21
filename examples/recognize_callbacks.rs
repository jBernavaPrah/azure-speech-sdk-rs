use azure_speech::stream::Stream;
use azure_speech::Auth;
use azure_speech::{recognizer, StreamExt};
use std::env;
use std::error::Error;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Check on the example recognize_simple.rs for more details on how to set the recognizer.
    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );
    let config = recognizer::Config::default();

    let client = recognizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    // Create a callbacks for the recognizer.
    // The callbacks are used to get information about the recognition process.
    let callbacks = recognizer::Callback::default()
        .on_start_detected(|id, offset| async move {
            tracing::info!("Start detected: {:?} - {:?}", id, offset);
        })
        .on_recognized(|id, result, _offset, _duration, _raw | async move {
            tracing::info!("Recognized: {:?} - {:?}", id, result);
        })
        .on_session_end(|id| async move {
            tracing::info!("Session end: {:?}", id);
        });
        //.on_... // check the other callbacks available.

    client
        .recognize(
            create_audio_stream("tests/audios/examples_sample_files_turn_on_the_lamp.wav").await, // Try also the mp3 version of the file.
            recognizer::ContentType::Wav, // Be sure to set it correctly.
            recognizer::Details::file(),
        )
        .await
        .expect("to recognize")
        // When you set the callbacks, the events will be sent to the callbacks and not to the stream.
        .use_callbacks(callbacks)
        .await; // it's important to await here.

    tracing::info!("Completed!");

    Ok(())
}

async fn create_audio_stream(path: impl AsRef<Path>) -> impl Stream<Item = Vec<u8>> {
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
