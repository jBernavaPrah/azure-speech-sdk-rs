use std::{env, io};
use std::fmt::Pointer;
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger};
use tokio::sync::mpsc::Sender;
use azure_speech::errors::Error;
use azure_speech::speech_to_text::{Recognizer, LanguageDetectMode, AudioHeaders, Message};

#[tokio::main]
async fn main() -> Result<(), Error> {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();


    let (tx, rx) = tokio::sync::mpsc::channel(5000);

    let mut reader = hound::WavReader::open("examples/audios/whatstheweatherlike.wav").unwrap();
    let spec = reader.spec();


    tokio::spawn(async move {
        let mut recognizer = Recognizer::new(env::var("AZURE_REGION").expect("Missing AZURE_REGION env"), env::var("AZURE_SUBSCRIPTION").expect("Missing AZURE_SUBSCRIPTION env"));
        recognizer.set_detect_languages(vec!["it-it", "en-us"], LanguageDetectMode::Continuous);

        let mut stream = recognizer.stream(rx, AudioHeaders::from_hound_spec(spec)).await.unwrap();

        while let Some(r) = stream.recv().await {
            match r {
                Message::SpeechPhrase(phrase) => {
                    info!("Received: {:?}", phrase.get("DisplayText"));
                }
                _ => info!("Received: {:?}", r)
            }
        }
    });

    tokio::task::spawn_blocking(move || {
        read_bytes_and_push_to_stream::<i16, _>(&mut reader, tx.clone());
    });

    tokio::signal::ctrl_c().await.expect("failed to listen for event");

    Ok(())
}

/// Read the samples from the wav file, transform them into bytes, buffer to 4kb and then send them to the stream.
fn read_bytes_and_push_to_stream<S, R>(reader: &mut hound::WavReader<R>, sender: Sender<Vec<u8>>)
    where
        f64: From<S>,
        S: hound::Sample,
        R: io::Read, i16: From<S>
{
    let mut buffer = Vec::new();

    for sample in reader.samples::<S>() {
        match sample {
            Ok(s) => {
                let s = i16::from(s);
                buffer.extend_from_slice(&s.to_le_bytes());

                if buffer.len() >= 4096 {
                    if let Err(_) = sender.blocking_send(buffer.clone()) {
                        error!("Failed to send buffer");
                    }
                    buffer.clear();
                }
            }
            Err(e) => error!("Failed to read sample: {}", e),
        }
    }

    // Send any remaining data in the buffer
    if !buffer.is_empty() {
        if let Err(_) = sender.blocking_send(buffer) {
            error!("Failed to send buffer");
        }
    }
}
