use std::env;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat as CPALSampleFormat;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use azure_speech::Auth;
use azure_speech::recognizer;

#[tokio::main]
async fn main() -> azure_speech::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();


    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let (rx, stream) = listen_from_default_input().await;

    let client = recognizer::Client::connect(auth, recognizer::Config::default()
        .set_detect_languages(vec![recognizer::Language::ItIt], recognizer::LanguageDetectMode::Continuous),
    ).await?;

    let mut events = client.recognize(ReceiverStream::new(rx), recognizer::ContentType::Wav, recognizer::Details::stream("mac", "stream")).await?;

    stream.play().expect("play failed");

    tracing::info!("Starting to listen...");

    while let Some(event) = events.next().await {
        if let Ok(recognizer::Event::Recognized(_, result, _,_, _)) = event {
            tracing::info!("recognized: {:?}", result.text);
        }
    }

    tracing::info!("Completed!");


    Ok(())
}

async fn listen_from_default_input() -> (tokio::sync::mpsc::Receiver<Vec<u8>>, cpal::Stream) {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("Failed to get default input device");
    let device_config = device.default_input_config().expect("Failed to get default input config");

    let config = device_config.clone().into();

    let (tx, rx) = tokio::sync::mpsc::channel(1024);

    tx.send(hound::WavSpec {
        sample_rate: device_config.sample_rate().0,
        channels: device_config.channels(),
        bits_per_sample: (device_config.sample_format().sample_size() * 8) as u16,
        sample_format: match device_config.sample_format().is_float() {
            true => hound::SampleFormat::Float,
            false => hound::SampleFormat::Int,
        },
    }.into_header_for_infinite_file()).await.expect("Failed to send wav header");


    let err = |err| tracing::error!("Trying to stream input: {err}");

    let stream = match device_config.sample_format() {
        CPALSampleFormat::I8 => device.build_input_stream(&config, move |data: &[i8], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::U8 => device.build_input_stream(&config, move |data: &[u8], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::I16 => device.build_input_stream(&config, move |data: &[i16], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::U16 => device.build_input_stream(&config, move |data: &[u16], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::I32 => device.build_input_stream(&config, move |data: &[i32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::U32 => device.build_input_stream(&config, move |data: &[u32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::F32 => device.build_input_stream(&config, move |data: &[f32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::I64 => device.build_input_stream(&config, move |data: &[i64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::U64 => device.build_input_stream(&config, move |data: &[u64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        CPALSampleFormat::F64 => device.build_input_stream(&config, move |data: &[f64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
        _ => panic!("Unsupported sample format"),
    }.expect("Failed to build input stream");

    (rx, stream)
}