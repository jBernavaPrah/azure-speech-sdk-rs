use azure_speech::{recognizer, Auth};
use std::env;
use tokio_stream::{Stream, StreamExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let client = recognizer::Client::connect(
        auth,
        recognizer::Config::default().set_detect_languages(
            vec![recognizer::Language::EnGb],
            recognizer::LanguageDetectMode::Continuous,
        ),
    )
    .await
    .expect("Failed to connect to Azure");

    let radio_stream =
        create_audio_stream("https://stream.live.vc.bbcmedia.co.uk/bbc_world_service").await;
    let mut events = client
        .recognize(
            radio_stream,
            recognizer::ContentType::Mpeg,
            recognizer::Details::stream("mac", "stream"),
        )
        .await
        .expect("Failed to recognize");

    while let Some(event) = events.next().await {
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
