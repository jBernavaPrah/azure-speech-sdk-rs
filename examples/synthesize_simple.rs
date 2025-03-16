use azure_speech::synthesizer::AudioFormat;
use azure_speech::{synthesizer, Auth};
use std::env;
use std::error::Error;
use tokio_stream::StreamExt;

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

    // Set the configuration for the synthesizer.
    //
    // The default configuration will create a Riff16Khz16BitMonoPcm audio chunks,
    // but you can change it using the .with_output_format(AudioFormat) method.
    //
    // It will understand the en-US language and will use the EnUsJennyNeural voice.
    // You can change it by using the Config struct and its methods.
    let config = synthesizer::Config::default()
        .enable_word_boundary()
        .enable_sentence_boundary()
        .enable_punctuation_boundary()
        .enable_viseme()
        .enable_bookmark()
        .enable_session_end()
        .with_language(synthesizer::Language::EnGb)
        .with_voice(synthesizer::Voice::EnGbLibbyNeural)
        .with_audio_format(AudioFormat::Audio48Khz192KBitRateMonoMp3);

    let client = synthesizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    let mut stream = client
        // here you put your text to synthesize.
        .synthesize("Hello World!")
        .await
        .expect("to synthesize");

    // Here we are streaming the events from the synthesizer.
    // But you can also use the callbacks (see: examples/synthesize_callbacks.rs) if you prefer.
    while let Some(event) = stream.next().await {
        // Each event is a part of the synthesis process.
        match event {
            Ok(synthesizer::Event::Synthesising(request_id, audio)) => {
                // here you can use the audio to create your output.
                // the audio is a Vec<u8> that contains the audio chunk.
                // you can use it to create a file, to play it or to send it to a speaker.
                tracing::info!(
                    "Synthesizer: Synthesising {:?} len: {:?}",
                    request_id,
                    audio.len()
                );

                tracing::info!("audio header: {:x?}", &audio[0..44]);
            }
            // this will print a lot of events to the console.
            _ => tracing::info!("Synthesizer: Event {:?}", event),
        }
    }

    Ok(())
}
