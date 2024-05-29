use std::{cmp, env};
use std::io::{Read, Seek, SeekFrom, Write};
use log::{debug, info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use symphonia::core::audio::{SampleBuffer};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokio::sync::mpsc::Receiver;
use tokio_util::bytes::Buf;
use azure_speech::{Auth, recognizer};
use azure_speech::recognizer::{Details, Event, EventBase, Sample, SampleFormat, Source, WavSpec};
use azure_speech::recognizer::config::{LanguageDetectMode, ResolverConfig};
use azure_speech::recognizer::speech::EventSpeech;

#[tokio::main]
async fn main() {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let radio_source = create_stream_audio("https://stream.live.vc.bbcmedia.co.uk/bbc_world_service").await;
    let source = create_source(radio_source).await;

    let mut config = ResolverConfig::new(
        Auth::from_subscription(
            env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
            env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
        )
    );
    config.set_detect_languages(vec!["it-it", "en-gb"], LanguageDetectMode::Continuous);

    let mut stream = recognizer::speech(config, source).await.expect("Failed to create recognizer stream");

    info!("Control-c to stop the service!");
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
                info!("Recognized: {}\n\n{:?}", text,raw );
            }
            Event::Specific(EventSpeech::Recognizing { text,.. }) => {
                info!("Recognizing: {:?}", text);
            }
            r => info!("Received: {:?}", r)
        }
    }
    info!("Done main service!");
}

async fn create_source(audio_source: Receiver<Vec<u8>>) -> Source {
    let test_buff = MyMediaSource::new(audio_source);
    let mss = MediaSourceStream::new(Box::new(test_buff), Default::default());
    let mut hint = Hint::new();
    hint.mime_type("audio/mpeg");

    let probed = tokio::task::spawn_blocking(move || {
        symphonia::default::get_probe().format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default()).unwrap()
    }).await.unwrap();

    let mut format = probed.format;
    let track = format.default_track().unwrap();
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default()).unwrap();
    let track_id = track.id;

    let (source, sender) = Source::new(WavSpec {
        sample_rate: track.codec_params.sample_rate.unwrap(),
        channels: track.codec_params.channels.unwrap().count() as u16,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    }, Details::unknown());

    tokio::task::spawn_blocking(move || {
        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            if let Ok(audio_buf) = decoder.decode(&packet) {
                let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.capacity() as u64, *audio_buf.spec());
                sample_buf.copy_interleaved_ref(audio_buf);
                sample_buf.samples().iter().for_each(|s| {
                    let _ = sender.blocking_send(Sample::from(*s));
                });
                debug!("Decoded {} samples", sample_buf.samples().len());
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
        while let Some(chunk) = response.chunk().await.unwrap() {
            request_data_tx.send(chunk.chunk().to_vec()).await.unwrap();
        }
        info!("Done from sender")
    });
    request_data_rx
}

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
        while self.buff.len() < length {
            if let Some(data) = self.response.blocking_recv() {
                self.buff.extend_from_slice(&data);
            } else {
                break;
            }
        }
        self.buff.drain(..cmp::min(length, self.buff.len())).collect()
    }
}

impl Read for MyMediaSource {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.read_from_response(buf.len());
        buf.write_all(&data)?;
        Ok(data.len())
    }
}

impl Seek for MyMediaSource {
    fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
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