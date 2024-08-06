use azure_speech::{synthesizer, Auth};
use std::env;
use std::error::Error;
use std::io::{stdin, SeekFrom};
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let client = synthesizer::Client::connect(
        // Add your Azure region and subscription key to the environment variables
        Auth::from_subscription(
            env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
            env::var("AZURE_SUBSCRIPTION_KEY")
                .expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
        ),
        // Set the configuration for the synthesizer
        synthesizer::Config::default()
            .with_output_format(synthesizer::AudioFormat::Audio16Khz128KBitRateMonoMp3)
            .with_language(synthesizer::Language::ItIt)
            .on_session_start(|session| {
                tracing::info!("Callback: Session started {:?}", session);
            })
            .on_session_end(|session| {
                tracing::info!("Callback: Session ended {:?}", session);
            }),
    )
    .await
    .expect("to connect to azure");

    let sender = sender_for_default_audio_output();

    while let Some(line) = recv_from_stdin().next().await {
        if line == "exit" {
            tracing::info!("exiting...");
            break;
        }

        let mut stream = client
            .synthesize(line.clone())
            .await
            .expect("to synthesize");

        while let Some(event) = stream.next().await {
            match event {
                Ok(synthesizer::Event::Synthesising(.., audio)) => {
                    sender.send(Some(audio)).await.expect("send audio chunk");
                }
                Ok(synthesizer::Event::SessionEnded(..)) => {
                    sender.send(None).await.expect("send audio chunk");
                    break;
                }
                _ => {}
            }
        }

        tracing::info!("Synthesized: {:?}", line);
    }

    drop(sender);

    Ok(())
}

pub fn recv_from_stdin() -> impl Stream<Item = String> {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(10);
    std::thread::spawn(move || {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        tx.blocking_send(buffer.trim().to_string()).unwrap();
    });

    ReceiverStream::new(rx)
}

pub fn sender_for_default_audio_output() -> tokio::sync::mpsc::Sender<Option<Vec<u8>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Option<Vec<u8>>>(10);
    std::thread::spawn(move || {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        sink.append(rodio::Decoder::new(StreamMediaSource::new(rx)).unwrap());
        sink.sleep_until_end();
    });
    tx
}

pub(crate) struct StreamMediaSource {
    inner: Receiver<Option<Vec<u8>>>,
    buffer: Vec<u8>,
}

impl StreamMediaSource {
    pub fn new(inner: Receiver<Option<Vec<u8>>>) -> Self {
        Self {
            inner,
            buffer: Vec::with_capacity(1024),
        }
    }

    fn read_inner(&mut self, len: usize) -> Vec<u8> {
        tracing::debug!("Messages left: {}", self.inner.len());

        while self.buffer.len() < len {
            match self.inner.blocking_recv() {
                Some(Some(data)) => {
                    self.buffer.extend(data);
                }
                Some(None) | None => {
                    break;
                }
            }
        }
        let len = std::cmp::min(len, self.buffer.len());
        self.buffer.drain(..len).collect()
    }
}

impl std::io::Read for StreamMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.read_inner(buf.len());
        let len = std::cmp::min(buf.len(), data.len());
        buf[..len].copy_from_slice(&data[..len]);

        Ok(len)
    }
}

impl std::io::Seek for StreamMediaSource {
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
