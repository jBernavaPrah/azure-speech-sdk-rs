use azure_speech::{recognizer, Auth};
use std::env;
use tokio_stream::{Stream, StreamExt};

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "INFO");
    }
    tracing_subscriber::fmt::init();

    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let client = recognizer::Client::connect(
        auth,
        recognizer::Config::default()
            // The BBC World Service stream is in English.
            .set_language(recognizer::Language::EnGb),
    )
    .await
    .expect("Failed to connect to Azure");

    let mut events = client
        .recognize(
            // The BBC World Service stream is a good example to test the recognizer.
            create_audio_stream("https://stream.live.vc.bbcmedia.co.uk/bbc_world_service").await,
            // The content type is MPEG.
            recognizer::AudioFormat::Mpeg,
            recognizer::AudioDevice::stream(),
        )
        .await
        .expect("Failed to recognize");

    while let Some(event) = events.next().await {
        // You will need to wait for some time before the first recognition is successful.
        // The best motivation is because they are talking to fast and
        // the recognition is waiting for a silence pause to wrap-up the sentence.

        // Currently is not possible to configure better the silence times and other parameters.
        // but will be implemented in the future.

        if let Ok(recognizer::Event::Recognized(_, result, ..)) = event {
            tracing::info!("Recognized: {:?}", result.text);
        }
    }
}

async fn create_audio_stream(endpoint: impl Into<String>) -> impl Stream<Item = Vec<u8>> {
    let response = reqwest::get(endpoint.into()).await.unwrap();
    tracing::info!("Response: {:?}", response);

    response.bytes_stream().map(|r| r.unwrap().to_vec())
}
