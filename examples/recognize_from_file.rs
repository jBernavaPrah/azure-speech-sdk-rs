use std::{env, io};
use std::io::Read;
use hound::{Sample, WavReader, WavSpec};
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use tokio::sync::mpsc::Sender;
use azure_speech::Auth;
use azure_speech::errors::Error;
use azure_speech::speech_to_text::event::Event;
use azure_speech::speech_to_text::{AudioFormat, Headers, Source};
use azure_speech::speech_to_text::speech::event::Event as SpeechEvent;
use azure_speech::speech_to_text::config::{LanguageDetectMode, OutputFormat, ResolverConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // Initialize the logger
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel(1024);

    let file = hound::WavReader::new(
        std::fs::File::open("examples/audios/whatstheweatherlike.wav").expect("Error opening file")
    ).unwrap();

    let source = Source::file(rx, generate_audio_headers(file.spec()));

    let format = file.spec().sample_format;
    match format {
        hound::SampleFormat::Float => tokio::spawn(read_from_file::<f32, _>(file, tx)),
        hound::SampleFormat::Int => tokio::spawn(read_from_file::<i32, _>(file, tx)),
    };
    
    let mut config = ResolverConfig::new(Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    ), source);
    config.set_detect_languages(vec!["it-it", "en-us"], LanguageDetectMode::Continuous);
    config.set_output_format(OutputFormat::Simple);

    let mut stream = azure_speech::speech_to_text::Recognizer::speech(config).await?;

    while let Some(r) = stream.recv().await {
        match r {
            Event::Cancelled { reason, code } => {
                info!("Cancelled {:?} {:?}", reason, code);
                break;
            }

            Event::SessionStarted { session_id } => {
                info!("SessionStarted: {:?}", session_id);
            }

            Event::SessionStopped { session_id } => {
                info!("SessionStopped: {:?}", session_id);
                break;
            }

            Event::R(SpeechEvent::UnMatch { raw }) => {
                info!("UnMatch: {:?}", raw);
            }

            Event::R(SpeechEvent::Recognized { text, raw, .. }) => {
                info!("Recognized: {} raw: {:?}", text,raw );
            }
            Event::R(SpeechEvent::Recognizing { text, .. }) => {
                info!("Recognizing: {:?}", text);
            }

            _ => info!("Received: {:?}", r)
        }
    }

    info!("End of the recognition.");

    Ok(())
}

async fn read_from_file<S, R>(mut reader: WavReader<R>, sender: Sender<Vec<u8>>)
    where S: Sample + funty::Numeric + Send + 'static,
          R: io::Read,
          <S as funty::Numeric>::Bytes: AsRef<[u8]>, {
    while let Some(sample) = reader.samples::<S>().next() {
        let s = S::from(sample.expect("Error reading sample"));
        sender.send(s.to_le_bytes().as_ref().to_vec()).await.unwrap();
    }
}


fn generate_audio_headers(spec: WavSpec) -> Headers {
    Headers {
        bits_per_sample: spec.bits_per_sample,
        sample_rate: spec.sample_rate,
        channels: spec.channels,
        format: match spec.sample_format {
            hound::SampleFormat::Int => AudioFormat::PCM,
            hound::SampleFormat::Float => AudioFormat::IEEE,
        },
    }
}