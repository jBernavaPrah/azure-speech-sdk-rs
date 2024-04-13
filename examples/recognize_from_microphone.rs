use std::env;
use std::error::Error;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger};
use azure_speech::speech_to_text::{LanguageDetectMode, Message, OutputFormat, Recognizer};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

    // Create a recognizer
    let mut recognizer = Recognizer::new(env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"), env::var("AZURE_SUBSCRIPTION").expect("Subscription set on AZURE_SUBSCRIPTION env"));
    recognizer.set_detect_languages(vec!["it-it", "en-us"], LanguageDetectMode::Continuous);
    recognizer.set_output_format(OutputFormat::Simple);

    let mut stream = recognizer.microphone().await.unwrap();

    while let Some(v) = stream.recv().await {
        match v {
            Message::SpeechPhrase(val) => info!("Received: {:?}", val["DisplayText"]),

            _ => info!("Received: {:?}", v)
        }
    }

    Ok(())
}
