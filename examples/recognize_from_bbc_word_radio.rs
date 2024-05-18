use std::{cmp, env};
use std::io::{Read, Seek, SeekFrom, Write};
use log::{debug, error, info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use symphonia::core::audio::{SampleBuffer};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokio::sync::mpsc::Receiver;
use tokio_util::bytes::Buf;
use azure_speech::{Auth, recognizer};
use azure_speech::recognizer::{Details, Event, EventBase, SampleFormat, Source, WavSpec};
use azure_speech::recognizer::config::{LanguageDetectMode, ResolverConfig};
use azure_speech::recognizer::speech::EventSpeech;

#[tokio::main]
async fn main() {

    // Initialize the logger
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let radio_source = create_stream_audio("https://stream.live.vc.bbcmedia.co.uk/bbc_world_service").await;

    let source = create_source(radio_source).await;

    let mut config = ResolverConfig::new(
        Auth::from_subscription(

            // Add your Azure region and subscription key here.
            // Create a free account at https://azure.microsoft.com/en-us/try/cognitive-services/ to get the subscription key
            // and the region where the subscription is created.

            env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
            env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
        )
    );
    config.set_detect_languages(vec!["it-it", "en-gb"], LanguageDetectMode::Continuous);
    // config.set_output_format(OutputFormat::Detailed);

    let mut stream = recognizer::speech(config, source).await.expect("Failed to create recognizer stream");

    info!("Control-c to stop the service!");
    loop {
        tokio::select! {
            biased; 
            _ = tokio::signal::ctrl_c() => break,
            Some(r) = stream.recv() =>  {
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
                        info!("Recognized: {}\n\n{:?}", text,raw );
                    }
        
                    Event::Specific(EventSpeech::Recognizing { text, .. }) => {
                        //info!("Recognizing: {:?}", text);
                    }
        
                    r => info!("Received: {:?}", r)
                }
            }
        }
    }


    info!("Done main service!");
}

async fn create_source(audio_source: Receiver<Vec<u8>>) -> Source<f32> {
    let test_buff = MyMediaSource::new(audio_source);

    // Create the media source stream using the boxed media source from above.

    let mss = MediaSourceStream::new(Box::new(test_buff), Default::default());

    // Create a hint to help the format registry guess what format reader is appropriate. In this
    // example we'll leave it empty.
    let mut hint = Hint::new();
    hint.mime_type("audio/mpeg");

    // Use the default options when reading and decoding.
    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    let decoder_opts: DecoderOptions = Default::default();


    let probed = tokio::task::spawn_blocking(move || {
        // Probe the media source stream for a format.
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts).unwrap()
    }).await.unwrap();

    // Get the format reader yielded by the probe operation.
    let mut format = probed.format;

    // Get the default track.
    let track = format.default_track().unwrap();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts).unwrap();

    // Store the track identifier, we'll use it to filter packets.
    let track_id = track.id;

    let mut sample_count = 0;
    let mut sample_buf = None;

    let (source, sender) = Source::new(WavSpec {
        sample_rate: track.codec_params.sample_rate.unwrap(),
        channels: track.codec_params.channels.unwrap().count() as u16,
        bits_per_sample: 32, // the audio is float, so only 32 bits could be used.
        sample_format: SampleFormat::Float,
    }, Details::unknown());

    tokio::task::spawn_blocking(move || {
        loop {
            // Get the next packet from the format reader.
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(err) => {
                    // could fail due to EOF or other errors.
                    error!("Error reading packet: {:?}", err);
                    break;
                }
            };

            // If the packet does not belong to the selected track, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples, ignoring any decode errors.
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // The decoded audio samples may now be accessed via the audio buffer if per-channel
                    // slices of samples in their native decoded format is desired. Use-cases where
                    // the samples need to be accessed in an interleaved order or converted into
                    // another sample format, or a byte buffer is required, are covered by copying the
                    // audio buffer into a sample buffer or raw sample buffer, respectively. In the
                    // example below, we will copy the audio buffer into a sample buffer in an
                    // interleaved order while also converting to a f32 sample format.


                    // If this is the *first* decoded packet, create a sample buffer matching the
                    // decoded audio buffer format.
                    if sample_buf.is_none() {
                        // Get the audio buffer specification.
                        let spec = *audio_buf.spec();

                        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                        let duration = audio_buf.capacity() as u64;

                        // Create the f32 sample buffer.
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }


                    // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);

                        let _ = sender.blocking_send(buf.samples().to_vec());

                        // The samples may now be access via the `samples()` function.
                        sample_count += buf.samples().len();
                        debug!("Decoded {} samples", sample_count);
                    }
                }
                Err(Error::DecodeError(_)) => (),
                Err(_) => break,
            }
        }

        drop(sender)
    });

    source
}

async fn create_stream_audio(endpoint: impl Into<String>) -> Receiver<Vec<u8>> {
    let (request_data_tx, request_data_rx) = tokio::sync::mpsc::channel(1024);

    let mut response = reqwest::get(endpoint.into()).await.unwrap();
    info!("Response: {:?}", response);
    tokio::spawn(async move {
        // Stream the response body and write it to the file chunk by chunk
        while let Some(chunk) = response.chunk().await.unwrap() {
            request_data_tx.send(chunk.chunk().to_vec()).await.unwrap();
        }
        info!("Done from sender")
    });

    request_data_rx
}

// Media Source implementation
struct MyMediaSource {
    response: Receiver<Vec<u8>>,
    buff: Vec<u8>,
}

impl MyMediaSource {
    fn new(response: Receiver<Vec<u8>>) -> Self {
        Self {
            response,
            buff: Vec::with_capacity(4096),
        }
    }

    fn read_from_response(&mut self, length: usize) -> Vec<u8> {

        // if the buffer has enough data, return the required length.
        while self.buff.len() < length {
            match self.response.blocking_recv() {
                Some(data) => {
                    self.buff.extend_from_slice(&data);
                }
                // if no data is available, break the loop.
                None => break,
            }
        }

        // drain the buffer and return the data.
        self.buff.drain(..cmp::min(length, self.buff.len())).collect()
    }
}

impl Read for MyMediaSource {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {

        // read the required length of data from the buffer.
        // continue to read from buffer until the required length is met.
        // if no data is available, return the length of the copied data.
        let data = self.read_from_response(buf.len());
        buf.write_all(&data)?;
        Ok(data.len())
    }
}

impl Seek for MyMediaSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}

impl MediaSource for MyMediaSource {
    fn is_seekable(&self) -> bool {
        false
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}
