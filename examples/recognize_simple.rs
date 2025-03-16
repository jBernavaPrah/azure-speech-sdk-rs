use azure_speech::recognizer;
use azure_speech::stream::StreamExt;
use azure_speech::Auth;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "INFO");
    }
    tracing_subscriber::fmt::init();

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
        // Here is your input audio stream. The audio headers needs to be present if required by the content type used.
        // In this example, the content type is Wav, so the headers are required at the start of the file.
        // Generally you read a file, the headers are already present.
        // If you are creating a stream from a microphone, you need to add the headers.
        // Check the relative example for more details.
        .recognize_file(
            "tests/audios/examples_sample_files_turn_on_the_lamp.wav", // Try also the mp3 version of the file.
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
