use azure_speech::{synthesizer, Auth};
use std::env;
use std::error::Error;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let config = synthesizer::Config::default();

    // Check the synthesize_simple example for more information on client configuration.
    let client = synthesizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    // Create your specific SSML here.

    // Check the (SSML documentation)[https://github.com/pykeio/ssml] for more information.
    let ssml = synthesizer::ssml::speak(
        Some("en-GB").into(),
        [synthesizer::ssml::Voice::new(
            "en-GB-LibbyNeural",
            ["hello world"],
        )],
    );

    // Using a specific SSML, the language and voice specified in the config will be overwritten by the SSML.
    let mut stream = client.synthesize(ssml).await.expect("to synthesize");

    while let Some(event) = stream.next().await {
        // this will print a lot of events to the console.
        // you can use the events to create your own audio output.

        // check examples/synthesize_to_standard_output.rs to see how to create an audio output.
        tracing::info!("Synthesized: {:?}", event);
    }

    Ok(())
}
