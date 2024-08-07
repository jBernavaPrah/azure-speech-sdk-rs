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

    let config = synthesizer::Config::default()
        .on_synthesising(|request_id, audio| {
            tracing::info!(
                "Callback - request: {:?}: Synthesising bytes {:?} ",
                request_id,
                audio.len()
            );
        })
        .on_synthesised(|request_id| {
            tracing::info!("Callback - request: {:?}: Synthesised", request_id);
        })
        .on_error(|request_id, error| {
            tracing::info!("Callback - request: {:?}: Error {:?}", request_id, error);
        })
        .on_audio_metadata(|request_id, metadata| {
            tracing::info!(
                "Callback - request: {:?}: Audio metadata {:?}",
                request_id,
                metadata
            );
        })
        .on_session_start(|request_id| {
            tracing::info!("Callback - request: {:?}: Session started", request_id);
        })
        .on_session_end(|request_id| {
            tracing::info!("Callback - request: {:?}: Session ended", request_id);
        });

    let client = synthesizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    // you can use both the stream and callback in the same functions.
    let mut stream = client
        // here you put your text to synthesize.
        .synthesize("Hello World!")
        .await
        .expect("to synthesize");

    while let Some(event) = stream.next().await {
        tracing::info!("Synthesizer Event: {:?}", event);
    }

    Ok(())
}
