use std::env;
use cpal::{Sample, SampleFormat, Stream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use tokio::sync::mpsc::Sender;
use azure_speech::{Auth, recognizer};
use azure_speech::errors::Error;
use azure_speech::recognizer::config::{LanguageDetectMode, ResolverConfig};
use azure_speech::recognizer::{Event, EventBase, WavSpec};
use azure_speech::recognizer::{Source, Details, SampleFormat as AudioFormat};
use azure_speech::recognizer::speech::EventSpeech;


#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the logger
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let mut config = ResolverConfig::new(
        Auth::from_subscription(

            // Add your Azure region and subscription key here.
            // Create a free account at https://azure.microsoft.com/en-us/try/cognitive-services/ to get the subscription key
            // and the region where the subscription is created.

            env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
            env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
        )
    );
    config.set_detect_languages(vec!["it-it", "en-us"], LanguageDetectMode::Continuous);
    // config.set_output_format(OutputFormat::Detailed);

    let (source, microphone) = create_source();

    microphone.play().expect("Failed to start microphone");

    let mut stream = recognizer::speech(config, source).await.expect("Failed to create recognizer stream");

    while let Some(r) = stream.recv().await {
        match r {
            Event::Base(EventBase::Cancelled { reason }) => {
                info!("Cancelled reason {:?}", reason);
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
                info!("Recognized: {} raw: {:?}", text,raw );
            }

            Event::Specific(EventSpeech::Recognizing { text, .. }) => {
                info!("Recognizing: {:?}", text);
            }

            r => info!("Received: {:?}", r)
        }
    }

    info!("End of the recognition.");


    Ok(())
}

fn create_source() -> (Source, Stream) {

    // Get the default input device
    let host = cpal::default_host();
    // Get the default input device
    let device = host.default_input_device().expect("Failed to get default input device");
    // Get the default input configuration
    let audio_config = device.default_input_config().expect("Failed to get default input config");

    let (tx, rx) = tokio::sync::mpsc::channel(1024);

    let source = Source::new(rx, WavSpec {
        sample_rate: audio_config.sample_rate().0,
        channels: audio_config.channels(),
        bits_per_sample: (audio_config.sample_format().sample_size() * 8) as u16,
        sample_format: match audio_config.sample_format().is_float() {
            true => AudioFormat::Float,
            false => AudioFormat::Int,
        },
    }, Details::microphone("CPAL", "CPAL"),
    );

    // Error handler
    let err = |err| error!("Error trying to stream input: {err}");
    // Send the audio data to the channel. 
    // This callback will be executed in a height priority thread, managed directly by the OS.
    fn send_to_channel<T>(data: &[T], tx: Sender<Vec<u8>>)
        where T: funty::Numeric, <T as funty::Numeric>::Bytes: IntoIterator<Item=u8>
    {
        let vec = data.iter().flat_map(|x| x.to_le_bytes()).collect();
        // Using try_send to avoid blocking the thread
        let _ = tx.try_send(vec);
    }

    let config = audio_config.clone().into();

    let stream = match audio_config.sample_format() {
        SampleFormat::I8 => device.build_input_stream(&config, move |data, _| send_to_channel::<i8>(data, tx.clone()), err, None),
        SampleFormat::U8 => device.build_input_stream(&config, move |data, _| send_to_channel::<u8>(data, tx.clone()), err, None),
        SampleFormat::I16 => device.build_input_stream(&config, move |data, _| send_to_channel::<i16>(data, tx.clone()), err, None),
        SampleFormat::U16 => device.build_input_stream(&config, move |data, _| send_to_channel::<u16>(data, tx.clone()), err, None),
        SampleFormat::I32 => device.build_input_stream(&config, move |data, _| send_to_channel::<i32>(data, tx.clone()), err, None),
        SampleFormat::U32 => device.build_input_stream(&config, move |data, _| send_to_channel::<u32>(data, tx.clone()), err, None),
        SampleFormat::F32 => device.build_input_stream(&config, move |data, _| send_to_channel::<f32>(data, tx.clone()), err, None),
        SampleFormat::I64 => device.build_input_stream(&config, move |data, _| send_to_channel::<i64>(data, tx.clone()), err, None),
        SampleFormat::U64 => device.build_input_stream(&config, move |data, _| send_to_channel::<u64>(data, tx.clone()), err, None),
        SampleFormat::F64 => device.build_input_stream(&config, move |data, _| send_to_channel::<f64>(data, tx.clone()), err, None),
        _ => panic!("Unsupported sample format"),
    }.expect("Failed to build input stream");


    (source, stream)
}
