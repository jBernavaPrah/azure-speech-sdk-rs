use std::env;
use tokio_stream::StreamExt;
use azure_speech::Auth;
use azure_speech::Error;
use azure_speech::synthesizer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the logger
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();


    let client = synthesizer::Client::connect(
        Auth::from_subscription(
            env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
            env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
        ),
        synthesizer::Config::default()
            .with_language(synthesizer::Language::AutoDetect),
    ).await?;

    let mut stream = client.synthesize("Hello Word").await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => {
                match data {
                    synthesizer::Event::Synthesising(data) => {
                        tracing::info!("Audio data len {:?}", data.len());
                    }
                    synthesizer::Event::SessionEnded => {
                        tracing::info!("Completed");
                        break;
                    }
                    _ => {
                        tracing::info!("Event {:?}", data);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error {:?}", e);
            }
        }
    }

    tracing::info!("Synthesized");

    client.disconnect().await?;

    Ok(())
}