use std::env;
use tokio::io::AsyncBufReadExt;
use tokio_stream::StreamExt;
use azure_speech::auth::Auth;
use azure_speech::Error;
use azure_speech::synthesizer;
use azure_speech::synthesizer::Language;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the logger
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let synthesizer = synthesizer::connect(auth, synthesizer::Config::default()).await;

    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut buffer = String::new();

    'outer: loop {
        buffer.clear();
        if reader.read_line(&mut buffer).await.expect("Failed to read line") <= 0 {
            break;
        }

        if buffer.trim() == "exit" || buffer.len() == 0 {
            break;
        }

        let speak = ssml::speak(Some("en-US"), [buffer.as_str()]);
        tracing::debug!("Synthesizing {:?}", speak);

        let mut stream = synthesizer.synthesize_text(buffer.as_str(), Language::ItIt, None).await.expect("Failed to synthesize");

        while let Some(result) = stream.next().await {
            match result {
                Ok(data) => println!("Received from stream {:?}", data),
                Err(azure_speech::Error::ServerDisconnect(reason)) => {
                    tracing::error!("Server disconnected. Reason {:?}", reason);
                    break 'outer;
                }
                Err(e) => {
                    tracing::error!("Error {:?}", e);
                    break 'outer;
                }
            }
        }

        tracing::debug!("Synthesized");
    }

    synthesizer.disconnect();

    Ok(())
}