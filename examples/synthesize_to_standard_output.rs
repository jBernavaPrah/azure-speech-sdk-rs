use azure_speech::{synthesizer, Auth};
use std::env;
use std::error::Error;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "INFO");
    }
    tracing_subscriber::fmt::init();

    // Add your Azure region and subscription key to the environment variables
    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    // Set the configuration for the synthesizer
    // The default configuration will create a Riff16Khz16BitMonoPcm audio chunks.
    // It will understand the en-GB language and will use the EnGbLibbyNeural voice.
    // You can change it by using the Config struct and its methods.
    let config = synthesizer::Config::default();

    let client = synthesizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    let mut stream = client
        .synthesize("Hello World!")
        .await
        .expect("to synthesize");

    let (sender, handler) = sender_for_default_audio_output();
    while let Some(event) = stream.next().await {
        match event {
            Ok(synthesizer::Event::Synthesising(.., audio)) => {
                sender.send(audio).expect("send audio chunk");
            }
            Ok(synthesizer::Event::SessionEnded(..)) => {
                break;
            }
            _ => {}
        }
    }

    tracing::info!("Synthesized.");

    drop(sender);

    let _ = handler.join();

    Ok(())
}

/// Returns a sender that you can use to feed WAV data to the default audio output.
/// The returned thread will run until the sink finishes playing.
pub fn sender_for_default_audio_output() -> (
    std::sync::mpsc::Sender<Vec<u8>>,
    std::thread::JoinHandle<()>,
) {
    // Use a synchronous channel for this blocking thread.
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let handler = std::thread::spawn(move || {
        // Initialize the default audio output stream.
        let (_stream, handle) =
            rodio::OutputStream::try_default().expect("Failed to obtain default output stream");
        let sink = rodio::Sink::try_new(&handle).expect("Failed to create audio sink");

        // Create our custom stream source and pass it to the WAV decoder.
        let source = StreamMediaSource::new(rx);
        let decoder = rodio::Decoder::new_wav(source).expect("Failed to decode WAV stream");
        sink.append(decoder);
        sink.sleep_until_end();
    });
    (tx, handler)
}

pub(crate) struct StreamMediaSource {
    inner: std::sync::Mutex<std::sync::mpsc::Receiver<Vec<u8>>>,
    buffer: Vec<u8>,
}

impl StreamMediaSource {
    pub fn new(inner: std::sync::mpsc::Receiver<Vec<u8>>) -> Self {
        Self {
            inner: std::sync::Mutex::new(inner),
            buffer: Vec::with_capacity(1024),
        }
    }

    /// Reads data until at least `len` bytes are available or until no more data can be received.
    /// This uses a blocking call with a 10ms timeout instead of busy-waiting.
    fn read_inner(&mut self, len: usize) -> Vec<u8> {
        tracing::debug!("Current buffer length: {}", self.buffer.len());

        while self.buffer.len() < len {
            // Lock the receiver for each attempt to receive data.
            let result = {
                let rx = self.inner.lock().unwrap();
                rx.recv_timeout(std::time::Duration::from_millis(1))
            };
            match result {
                Ok(data) => self.buffer.extend(data),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if !self.buffer.is_empty() {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        let read_len = std::cmp::min(len, self.buffer.len());
        self.buffer.drain(..read_len).collect()
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
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        // Return an error instead of panicking when a seek is attempted.
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "StreamMediaSource does not support seeking",
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    #[test]
    fn test_stream_media_source() {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut source = super::StreamMediaSource::new(rx);
        drop(tx);

        assert_eq!(source.read(&mut [0u8; 10]).unwrap(), 0);
    }

    #[test]
    fn test_stream_media_source_with_data() {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut source = super::StreamMediaSource::new(rx);

        tx.send(vec![1_u8, 2_u8, 3_u8, 4_u8, 5u8]).unwrap();
        drop(tx);

        let mut buffer = [0u8; 10];
        assert_eq!(source.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer[..5], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_stream_media_source_with_data_larger_than_buffer() {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut source = super::StreamMediaSource::new(rx);

        tx.send(vec![1, 2, 3, 4, 5, 6, 7]).unwrap();
        tx.send(vec![8, 9, 10]).unwrap();
        tx.send(vec![]).unwrap();
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
