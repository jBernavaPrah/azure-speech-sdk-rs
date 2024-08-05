use std::{env};
use std::io::Read;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use azure_speech::Auth;
use azure_speech::recognizer;
use azure_speech::recognizer::Details;

#[tokio::main]
async fn main() -> azure_speech::Result<()> {

    // Initialize the logger
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    

    let file = match recognizer::wav::WavReader::open("tests/audios/whatstheweatherlike.wav") {
        Ok(file) => file,
        Err(recognizer::wav::Error::IoError(e)) => return Err(azure_speech::Error::IOError(e.to_string())),
        Err(_) => return Err(azure_speech::Error::InternalError("Error opening the file".to_string())),
    };

    let spec = file.spec();

    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let client = recognizer::Client::connect(auth, recognizer::Config::default()).await?;

    let (tx, rx) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(async move {
        let mut inner = file.into_inner();
        let mut chunk = vec![0; 4096];
        while let Ok(n) = inner.read(&mut chunk) {
            if n == 0 {
                break;
            }
        
            if tx.send(chunk.clone()).await.is_err() {
                tracing::error!("Error sending data");
                break;
            }
        }
        drop(tx);
    });

    let mut stream = client.recognize(ReceiverStream::new(rx), spec, Details::file()).await?;

    while let Some(event) = stream.next().await {
        tracing::info!("Event: {:?}", event);
    }


    tracing::info!("Completed!");

    Ok(())
}

// pub async fn recognize_from_file<P: AsRef<Path>>(&self, filename: P) -> crate::Result<mpsc::Receiver<Event<EventSpeech>>> {
//         let file = match WavReader::open(filename) {
//             Ok(file) => file,
//             Err(Error::IoError(e)) => return Err(crate::Error::IOError(e)),
//             Err(_) => return Err(crate::Error::InternalError("Error opening the file".to_string())),
//         };
// 
//         let spec = file.spec();
//         let (tx, rx) = mpsc::channel(1024);
// 
//         tokio::spawn(async move {
//             let mut inner = file.into_inner();
//             loop {
//                 let mut chunk = vec![0; 4096];
//                 match inner.read(&mut chunk) {
//                     Ok(0) => {
//                         break;
//                     }
//                     Ok(n) => {
//                         chunk.truncate(n);
//                         match tx.send(chunk).await {
//                             Ok(_) => {}
//                             Err(e) => {
//                                 error!("Error sending data: {:?}", e);
//                                 break;
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         error!("Error reading data: {:?}", e);
//                         break;
//                     }
//                 }
//             }
// 
//             drop(tx);
//         });
// 
//         self.recognize(rx, spec, Details::file()).await
//     }
// 
//     pub async fn recognize_from_default_microphone(&self) -> crate::Result<(mpsc::Receiver<Event<EventSpeech>>, Stream)> {
//         let host = cpal::default_host();
//         let device: CPALDevice = host.default_input_device()
//             .ok_or(crate::Error::InternalError("Failed to get default input device".to_string()))?;
// 
//         self.recognize_from_input_device(device).await
//     }
// 
//     // todo: add #![cfg(feature = "cpal")] to the Cargo.toml
//     pub async fn recognize_from_input_device(&self, device: CPALDevice) -> crate::Result<(mpsc::Receiver<Event<EventSpeech>>, Stream)> {
//         let (tx, rx) = mpsc::channel(1024);
// 
//         // Get the default input configuration
//         let audio_config = device.default_input_config()
//             .map_err(|e| crate::Error::InternalError(format!("Failed to get default input config: {:?}", e)))?;
// 
//         // Error handler
//         let err = |err| warn!("Trying to stream input: {err}");
//         // Send the audio data to the channel.
// 
//         let config = audio_config.clone().into();
// 
//         let stream = match audio_config.sample_format() {
//             CPALSampleFormat::I8 => device.build_input_stream(&config, move |data: &[i8], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::U8 => device.build_input_stream(&config, move |data: &[u8], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::I16 => device.build_input_stream(&config, move |data: &[i16], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::U16 => device.build_input_stream(&config, move |data: &[u16], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::I32 => device.build_input_stream(&config, move |data: &[i32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::U32 => device.build_input_stream(&config, move |data: &[u32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::F32 => device.build_input_stream(&config, move |data: &[f32], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::I64 => device.build_input_stream(&config, move |data: &[i64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::U64 => device.build_input_stream(&config, move |data: &[u64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             CPALSampleFormat::F64 => device.build_input_stream(&config, move |data: &[f64], _| data.iter().for_each(|d| tx.try_send(d.to_le_bytes().to_vec()).unwrap_or(())), err, None),
//             _ => return Err(crate::Error::InternalError("Unsupported sample format".to_string())),
//         }.expect("Failed to build input stream");
// 
//         Ok((self.recognize(rx, Spec {
//             sample_rate: audio_config.sample_rate().0,
//             channels: audio_config.channels(),
//             bits_per_sample: (audio_config.sample_format().sample_size() * 8) as u16,
//             sample_format: match audio_config.sample_format().is_float() {
//                 true => SampleFormat::Float,
//                 false => SampleFormat::Int,
//             },
//         }, Details::microphone("CPAL", "CPAL")).await?, stream))
//     }