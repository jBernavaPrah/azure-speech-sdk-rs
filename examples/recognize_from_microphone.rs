use std::env;
use cpal::{Device, InputCallbackInfo, Sample, SampleFormat, Stream, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use tokio::sync::mpsc::Sender;
use azure_speech::Auth;
use azure_speech::errors::Error;
use azure_speech::speech_to_text::config::{LanguageDetectMode, OutputFormat, ResolverConfig};
use azure_speech::speech_to_text::event::Event;
use azure_speech::speech_to_text::{AudioFormat, Headers, Source};
use azure_speech::speech_to_text::speech::event::Event as SpeechEvent;


#[tokio::main]
async fn main() -> Result<(), Error>{
    // Initialize the logger
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel(1024);


    let host = cpal::default_host();
    let device = host.default_input_device().expect("Failed to get default input device");
    let audio_config = device.default_input_config().expect("Failed to get default input config");


    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let source = Source::microphone(rx, audio_headers(audio_config.clone()));

    let mut config = ResolverConfig::new(auth, source);
    config.set_detect_languages(vec!["it-it", "en-us"], LanguageDetectMode::Continuous);
    config.set_output_format(OutputFormat::Detailed);
    
    tokio::spawn(async move {
        let mut stream = azure_speech::speech_to_text::Recognizer::speech(config).await.expect("Failed to create recognizer stream");

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
       
    });
    
    let stream = match audio_config.sample_format() {
        SampleFormat::F32 => build_input_stream::<f32>(device, audio_config, tx),
        SampleFormat::I16 => build_input_stream::<i16>(device, audio_config, tx),
        SampleFormat::U16 => build_input_stream::<u16>(device, audio_config, tx),
        SampleFormat::I8 => build_input_stream::<i8>(device, audio_config, tx),
        SampleFormat::I32 => build_input_stream::<i32>(device, audio_config, tx),
        SampleFormat::I64 => build_input_stream::<i64>(device, audio_config, tx),
        SampleFormat::U8 => build_input_stream::<u8>(device, audio_config, tx),
        SampleFormat::U32 => build_input_stream::<u32>(device, audio_config, tx),
        SampleFormat::U64 => build_input_stream::<u64>(device, audio_config, tx),
        SampleFormat::F64 => build_input_stream::<f64>(device, audio_config, tx),
        _ => panic!("Unsupported sample format"),
    };
    stream.play().expect("Error playing audio");
    //
    info!("Press ctrl-c to stop the stream");
    info!("Listening for audio...");

    tokio::signal::ctrl_c().await.expect("Error waiting for ctrl-c");
    
    drop(stream);

    info!("Stream completed!");

    Ok(())
}


fn build_input_stream<T>(device: Device, config: SupportedStreamConfig, tx: Sender<Vec<u8>>) -> Stream
    where
        T: Sample + cpal::SizedSample + funty::Numeric, <T as funty::Numeric>::Bytes: AsRef<[u8]>,
{
    device.build_input_stream(
        &config.clone().into(),
        move |data: &[T], _: &InputCallbackInfo| {
            let mut vec = Vec::new();
            for &x in data {
                vec.extend_from_slice(x.to_le_bytes().as_ref());
            }
            tx.blocking_send(vec).unwrap();
        },
        |err| panic!("{err}"),
        None,
    ).expect("Failed to build input stream")
}

fn audio_headers(config: SupportedStreamConfig) -> Headers {
    let sample_rate = config.sample_rate().0;
    let output_channels = config.channels();
    let bit_per_sec = (config.sample_format().sample_size() * 8) as u16;

    Headers::new(match config.sample_format().is_float() {
        true => AudioFormat::IEEE,
        false => AudioFormat::PCM,
    }, sample_rate, bit_per_sec, output_channels)
}