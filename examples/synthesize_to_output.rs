use std::env;
use std::error::Error;
use std::io::SeekFrom;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::Receiver;
use tokio_stream::StreamExt;
use azure_speech::{Auth, synthesizer};
use azure_speech::synthesizer::Event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let client = synthesizer::Client::connect(
        Auth::from_subscription(
            env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
            env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
        ),
        synthesizer::Config::default()
            .with_output_format(synthesizer::AudioFormat::Audio16Khz128KBitRateMonoMp3)
            .with_language(synthesizer::Language::AutoDetect),
    ).await.expect("to connect to azure");

    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let mut buffer = String::new();
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());

    tokio::spawn(async move {
        loop {
            buffer.clear();
            if reader.read_line(&mut buffer).await.expect("Failed to read line") <= 0 {
                break;
            }

            if buffer.trim() == "exit" || buffer.len() == 0 {
                break;
            }

            let mut stream = client.synthesize(buffer.trim().to_string()).await.expect("to synthesize");
            while let Some(data) = stream.next().await {
                
                match data {
                    Ok(Event::Audio(audio)) => {
                        println!("Event Data: {:?}", audio.len());
                        tx.send(Some(audio)).await.expect("send audio chunk");
                    }
                    Ok(Event::Completed) | Ok(Event::Cancelled(_)) => {
                        println!("Event: {:?}", data);
                        tx.send(None).await.expect("send audio chunk");
                    }
                    Ok(_) => {
                        println!("Event: {:?}", data);
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        //tx.send(None).await.expect("send audio chunk");
                        break;
                    }
                }
            }
        }
        
        client.disconnect().await.expect("to disconnect");
        tracing::info!("exit");

        drop(tx);
    });

    tokio::task::spawn_blocking(move || {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        // while let Some(data) = rx.blocking_recv() {
        //     sink.append(rodio::Decoder::new(std::io::Cursor::new(data)).unwrap());
        // }
        sink.append(rodio::Decoder::new(StreamMediaSource::new(rx)).unwrap());
        sink.sleep_until_end();
    }).await.expect("to run blocking task");
    
    
    Ok(())
    // pub async fn synthesize_to_output_device(&self, speaks_rx: Receiver<Speak>, _device: cpal::Device) -> crate::Result<()> {
    //     let stream = self.synthesize(speaks_rx).await?;
    // 
    //     tokio::task::spawn_blocking::<_, crate::Result<()>>(move || {
    //         let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    // 
    //         let sink = rodio::Sink::try_new(&handle).unwrap();
    // 
    //         //let file = std::fs::File::open("tests/whatstheweatherlike.wav").unwrap();
    // 
    //         sink.append(rodio::Decoder::new(StreamMediaSource::new(stream.audio())).unwrap());
    // 
    //         sink.sleep_until_end();
    // 
    //         Ok(())
    //     }).await.expect("to run blocking task")?;
    // 
    //     Ok(())
    // }
    // 
    // pub async fn synthesize_to_default_output_device(&self, speaks_rx: Receiver<Speak>) -> crate::Result<()> {
    //     let host = cpal::default_host();
    //     let device = host.default_output_device()
    //         .ok_or(crate::Error::InternalError("Failed to get default input device".to_string()))?;
    // 
    //     self.synthesize_to_output_device(speaks_rx, device).await
    // }
}


pub(crate) struct StreamMediaSource
{
    inner: Receiver<Option<Vec<u8>>>,
    buffer: Vec<u8>,
}

impl StreamMediaSource
{
    pub fn new(inner: Receiver<Option<Vec<u8>>>) -> Self

    {
        Self {
            inner,
            buffer: Vec::with_capacity(1024),
        }
    }

    fn read_inner(&mut self, len: usize) -> Vec<u8> {
        
        tracing::info!("Messages left: {}", self.inner.len());
        
        while self.buffer.len() < len {
            match self.inner.blocking_recv() {
                Some(Some(data)) => {
                    self.buffer.extend(data);
                }
                Some(None) => {
                    break;
                }
                None => {
                    break;
                }
            }
        }
        let len = std::cmp::min(len, self.buffer.len());
        self.buffer.drain(..len).collect()
    }
}


impl std::io::Read for StreamMediaSource
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.read_inner(buf.len());
        let len = std::cmp::min(buf.len(), data.len());
        buf[..len].copy_from_slice(&data[..len]);

        Ok(len)
    }
}

impl std::io::Seek for StreamMediaSource
{
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        unreachable!("StreamMediaSource does not support seeking")
    }
}


#[cfg(test)]
mod tests {
    use std::io::Read;

    #[test]
    fn test_stream_media_source() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let mut source = super::StreamMediaSource::new(rx);
        drop(tx);

        assert_eq!(source.read(&mut [0u8; 10]).unwrap(), 0);
    }

    #[test]
    fn test_stream_media_source_with_data() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let mut source = super::StreamMediaSource::new(rx);

        tx.blocking_send(Some(vec![1, 2, 3, 4, 5])).unwrap();
        drop(tx);

        let mut buffer = [0u8; 10];
        assert_eq!(source.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer[..5], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_stream_media_source_with_data_larger_than_buffer() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let mut source = super::StreamMediaSource::new(rx);

        tx.blocking_send(Some(vec![1, 2, 3, 4, 5, 6, 7])).unwrap();
        tx.blocking_send(Some(vec![8, 9, 10])).unwrap();
        tx.blocking_send(Some(vec![])).unwrap();
        tx.blocking_send(None).unwrap();
        drop(tx);

        let mut buffer = [0u8; 5];
        assert_eq!(source.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer, &[1, 2, 3, 4, 5]);

        let mut buffer = [0u8; 5];
        assert_eq!(source.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer, &[6, 7, 8, 9, 10]);

        let mut buffer = [0u8; 5];
        assert_eq!(source.read(&mut buffer).unwrap(), 0);
        assert_eq!(&buffer, &[0, 0, 0, 0, 0]);
    }
}
