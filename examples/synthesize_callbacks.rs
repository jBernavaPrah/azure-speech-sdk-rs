use azure_speech::{synthesizer, Auth, StreamExt};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Check the examples/synthesize_simple.rs file for the full code.

    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let config = synthesizer::Config::default();
    let client = synthesizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    // Create the callbacks for the synthesizer.
    let callbacks = synthesizer::Callback::default()
        .on_synthesising(|request_id, audio| async move {
            tracing::info!(
                "Callback - request: {:?}: Synthesising bytes {:?} ",
                request_id,
                audio.len()
            );
        })
        .on_synthesised(|request_id| async move {
            tracing::info!("Callback - request: {:?}: Synthesised", request_id);
        })
        .on_audio_metadata(|request_id, metadata| async move {
            tracing::info!(
                "Callback - request: {:?}: Audio metadata {:?}",
                request_id,
                metadata
            );
        })
        .on_session_start(|request_id| async move {
            tracing::info!("Callback - request: {:?}: Session started", request_id);
        })
        .on_session_end(|request_id| async move {
            tracing::info!("Callback - request: {:?}: Session ended", request_id);
        })
        .on_error(|request_id, error| async move {
            tracing::info!("Callback - request: {:?}: Error {:?}", request_id, error);
        });

    // you can use both the stream and callback in the same functions.
    client
        // here you put your text to synthesize.
        .synthesize("Hello World!")
        .await
        .expect("to synthesize")
        .use_callbacks(callbacks)
        .await;

    Ok(())
}
