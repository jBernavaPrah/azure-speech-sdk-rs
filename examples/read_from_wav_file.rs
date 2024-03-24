use std::env;
use std::time::{Duration, Instant};
use log::{debug, error, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger};
use uuid::Uuid;
use azure_speech::errors::Error;
use azure_speech::speech_to_text::{
    recognizer::Recognizer,
    response::Response,
    config::{LanguageDetectMode, OutputFormat, Source},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

    let _start = Instant::now();

    let mut recognizer = Recognizer::new(env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"), env::var("AZURE_SUBSCRIPTION").expect("Subscription set on AZURE_SUBSCRIPTION env"));
    recognizer.set_detect_languages(vec!["it-it", "en-us"], LanguageDetectMode::Continuous);
    recognizer.set_source(Source::default());
    recognizer.set_output_format(OutputFormat::Detailed);


    let mut stream = recognizer.recognize_file_wav("examples/audios/whatstheweatherlike.wav").await.unwrap();
    while let Some(r) = stream.recv().await {
        match r {
            Response::TurnStart(t) => {
                debug!("Turn: {:?}", t);
            }
            Response::SpeechStart(s) => {
                debug!("Start: {:?}", s);
            }
            Response::SpeechHypothesis(h) => {
                debug!("Hypothesis: {:?}", h);
            }
            Response::SpeechPhrase(p) => {
                debug!("Phrase: {}, original data: {:?}", p.clone().display_text.unwrap_or("".to_string() ), p);
            }
            Response::SpeechEnd(e) => {
                debug!("End: {:?}", e);
            }
            Response::TurnEnd => {
                debug!("End", );
                break;
            }
            Response::ErrorDecoding {
                path,
                json
            } => {
                error!("Unknown response from azure. Please, open an issue on github I will investigate ;) Path: {} Json: {}", path, json);
            }
            Response::UnknownPath {
                path,
                json
            } => {
                error!("Unknown response from azure. Please, open an issue on github I will investigate ;)");
            }
        }
    }

    tokio::time::sleep(Duration::from_secs(5)).await;


    Ok(())
}

