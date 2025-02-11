use azure_speech::recognizer;
use azure_speech::stream::{Stream, StreamExt};
use azure_speech::Auth;
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

    // Add your Azure region and subscription key to the environment variables.
    // In this version only the default subscription key is supported.
    // Other authentication methods are in the roadmap.
    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    // Set the configuration for the recognizer.
    //
    // The default configuration will try to recognize en-US language,
    // will use the Conversation mode and will require the simple output format.
    // You can change it by using the Config struct and its methods.
    let config = recognizer::Config::default();

    let client = recognizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    // Here we are streaming the events from the synthesizer.
    // But you can also use the callbacks (see: examples/recognize_callbacks.rs) if you prefer.
    let mut stream = client
        .recognize(
            // Here is your input audio stream. The audio headers needs to be present if required by the content type used.
            // In this example, the content type is Wav, so the headers are required at the start of the file.
            // Generally you read a file, the headers are already present.
            // If you are creating a stream from a microphone, you need to add the headers.
            // Check the relative example for more details.
            create_audio_stream("tests/audios/examples_sample_files_turn_on_the_lamp.wav").await, // Try also the mp3 version of the file.
            // Here is the content type of the audio stream.
            recognizer::ContentType::Wav, // The headers are empty because they are already present in the file.
            // The typology of the source. You can use unknown, file or stream.
            // More information can be requested by the method.
            recognizer::Details::file(),
        )
        .await
        .expect("to recognize");
    
    // disconnect manually after 5 seconds. 
    // automatically will disconnect after 30 seconds...
    tokio::spawn(async move {
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        tracing::info!("Disconnecting...");
        client.disconnect().await.expect("to disconnect");
        tracing::info!("Disconnected!");
        
    });

    while let Some(event) = stream.next().await {
        // Each event is a part of the recognition process.
        match event {
            // as example the Recognized event will give you the result of the recognition.
            Ok(recognizer::Event::Recognized(
                request_id,
                result,
                offset,
                duration,
                raw_message,
            )) => {
                tracing::info!("Recognized session: {:?}", request_id);
                tracing::info!("Result: {:?}", result);
                tracing::info!("Offset: {:?}", offset);
                tracing::info!("Duration: {:?}", duration);

                // the raw message is the json message received from the service.
                // You can use it to extract more information when needed.
                tracing::info!("Raw message: {:?}", raw_message);
            }
            _ => {
                tracing::info!("Event: {:?}", event);
            }
        }
    }

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
