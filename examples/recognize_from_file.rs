use std::{env, path};
use hound::{WavReader};
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use azure_speech::{Auth, recognizer};
use azure_speech::errors::Error;
use azure_speech::recognizer::{Details, Event, EventBase, Source};
use azure_speech::recognizer::config::{LanguageDetectMode, ResolverConfig};
use azure_speech::recognizer::speech::EventSpeech;

#[tokio::main]
async fn main() -> Result<(), Error> {

    // Initialize the logger
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    // Configure the resolver 
    let mut config = ResolverConfig::new(Auth::from_subscription(

        // Add your Azure region and subscription key here. 
        // Create a free account at https://azure.microsoft.com/en-us/try/cognitive-services/ to get the subscription key
        // and the region where the subscription is created.

        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    ));
    config.set_detect_languages(vec!["it-it", "en-us"], LanguageDetectMode::Continuous);
    //config.set_output_format(OutputFormat::Simple);
    // ...


    // Create a source for recognizer. This will be used to send the audio data to the recognizer
    let source = create_source_from_file("tests/audios/whatstheweatherlike.wav");
    let mut stream = recognizer::speech(config, source).await?;

    while let Some(r) = stream.recv().await {
        match r {
            // Base Events are associated with Event
            Event::Base(EventBase::Cancelled { reason }) => {
                info!("Cancelled {:?}", reason);
                break;
            }

            Event::Base(EventBase::SessionStarted { session_id }) => {
                info!("SessionStarted: {:?}", session_id);
            }

            Event::Base(EventBase::SessionStopped { session_id }) => {
                info!("SessionStopped: {:?}", session_id);
                break;
            }

            Event::Specific(EventSpeech::UnMatch { raw }) => {
                info!("UnMatch: {:?}", raw);
            }
            Event::Specific(EventSpeech::Recognized { text, raw, .. }) => {
                info!("Recognized: {} raw: {:?}", text, raw );
            }
            Event::Specific(EventSpeech::Recognizing { text, .. }) => {
                info!("Recognizing: {:?}", text);
            }

            _ => info!("Received: {:?}", r)
        }
    }

    info!("End of the recognition.");

    Ok(())
}

fn create_source_from_file<P: AsRef<path::Path>>(filename: P) -> Source
{
    let mut file = WavReader::open(filename).expect("Error opening file");

    let (source, sender) = Source::new(file.spec().into(), Details::file());
    tokio::spawn(async move {
        let bits_per_sample = file.spec().bits_per_sample;
        let sample_format = file.spec().sample_format;

        match (sample_format, bits_per_sample) {
            (hound::SampleFormat::Int, 16) => {
                for s in file.samples::<i16>().filter_map(Result::ok) {
                    sender.send(recognizer::Sample::from(s)).await.unwrap();
                }
            }
            (hound::SampleFormat::Int, 32) => {
                for s in file.samples::<i32>().filter_map(Result::ok) {
                    sender.send(recognizer::Sample::from(s)).await.unwrap();
                }
            }

            (hound::SampleFormat::Float, 32) => {
                for s in file.samples::<f32>().filter_map(Result::ok) {
                    sender.send(recognizer::Sample::from(s)).await.unwrap();
                }
            }
            _ => panic!("Unsupported sample format")
        }
    });
    source
}


