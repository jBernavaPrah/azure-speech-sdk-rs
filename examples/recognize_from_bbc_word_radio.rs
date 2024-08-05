use std::{env};
use tokio_stream::{Stream, StreamExt};
use azure_speech::{Auth, recognizer};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let client = recognizer::Client::connect(auth, recognizer::Config::default()
        .set_detect_languages(vec![recognizer::Language::ItIt, recognizer::Language::EnGb], recognizer::LanguageDetectMode::Continuous),
    ).await.expect("Failed to connect to Azure");


    let radio_stream = create_stream_audio("https://stream.live.vc.bbcmedia.co.uk/bbc_world_service").await;

    let mut events = client.recognize(radio_stream, recognizer::ContentType::Mpeg, recognizer::Details::stream("mac", "stream")).await.expect("Failed to recognize");

    while let Some(event) = events.next().await {
        match event {
            Ok(recognizer::Event::Recognized(_, result, ..)) => {
                tracing::info!("Recognized: {:?}", result.text);
            }
            _ => {}
        }
    }
}

async fn create_stream_audio(endpoint: impl Into<String>) -> impl Stream<Item=Vec<u8>> {
    let response = reqwest::get(endpoint.into()).await.unwrap();
    tracing::info!("Response: {:?}", response);

    response.bytes_stream().map(|r| r.unwrap().to_vec())
}
