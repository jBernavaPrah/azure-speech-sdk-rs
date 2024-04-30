
use cpal::SupportedStreamConfig;
use hound::WavSpec;
use tokio::sync::mpsc::Receiver;


#[derive(Debug)]
pub struct Source {
    pub(crate) headers: Headers,
    pub(crate) details: Details,
    pub(crate) source: Receiver<Vec<u8>>,
}

impl Source {
    pub fn new(source: Receiver<Vec<u8>>, headers: Headers, details: Details) -> Self {
        Self {
            headers,
            details,
            source,
        }
    }

    pub fn microphone(source: Receiver<Vec<u8>>, headers: Headers) -> Self {
        Self {
            headers,
            details: Details {
                name: "Microphone".to_string(),
                model: "Microphone".to_string(),
                manufacturer: "Unknown".to_string(),
                connectivity: "Unknown".to_string(),
            },
            source,
        }
    }

    pub fn file(source: Receiver<Vec<u8>>, headers: Headers) -> Self {
        Self {
            headers,
            details: Details {
                name: "File".to_string(),
                model: "File".to_string(),
                manufacturer: "Unknown".to_string(),
                connectivity: "Unknown".to_string(),
            },
            source,
        }
    }
    
    pub fn stream(source: Receiver<Vec<u8>>, headers: Headers) -> Self {
        Self {
            headers,
            details: Details {
                name: "Stream".to_string(),
                model: "Stream".to_string(),
                manufacturer: "Unknown".to_string(),
                connectivity: "Unknown".to_string(),
            },
            source,
        }
    
    }
}


#[derive(Debug, Clone)]
pub enum AudioFormat {
    PCM,
    IEEE,
}

#[derive(Debug, Clone)]
pub struct Headers {
    pub bits_per_sample: u16,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
}

impl Headers {
    pub fn new(format: AudioFormat, sample_rate: u32, bits_per_sample: u16, channels: u16) -> Self {
        Self {
            bits_per_sample,
            sample_rate,
            channels,
            format,
        }
    }
}

impl From<SupportedStreamConfig> for Headers {
    fn from(config: SupportedStreamConfig) -> Headers {
        let sample_rate = config.sample_rate().0;
        let output_channels = config.channels();
        let bit_per_sec = (config.sample_format().sample_size() * 8) as u16;

        Headers::new(match config.sample_format().is_float() {
            true => AudioFormat::IEEE,
            false => AudioFormat::PCM,
        }, sample_rate, bit_per_sec, output_channels)
    }
}

impl From<WavSpec> for Headers {
    fn from(spec: WavSpec) -> Headers {
        Headers {
            bits_per_sample: spec.bits_per_sample,
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            format: match spec.sample_format {
                hound::SampleFormat::Int => AudioFormat::PCM,
                hound::SampleFormat::Float => AudioFormat::IEEE,
            },
        }
    }
}

impl From<Headers> for WavSpec {
    fn from(headers: Headers) -> WavSpec {
        WavSpec {
            channels: headers.channels,
            sample_rate: headers.sample_rate,
            bits_per_sample: headers.bits_per_sample,
            sample_format: match headers.format {
                AudioFormat::PCM => hound::SampleFormat::Int,
                AudioFormat::IEEE => hound::SampleFormat::Float,
            },
        }
    }
}

impl From<Headers> for Vec<u8> {
    fn from(headers: Headers) -> Vec<u8> {
        let wav_spec: WavSpec = headers.into();
        wav_spec.into_header_for_infinite_file()
    }
}


#[derive(Debug)]
pub struct Details {
    pub name: String,
    pub model: String,
    pub connectivity: String,
    pub manufacturer: String,
}

impl Details {
    pub fn unknown() -> Self {
        Details {
            name: "Unknown".to_string(),
            model: "Unknown".to_string(),
            manufacturer: "Unknown".to_string(),
            connectivity: "Unknown".to_string(),
        }
    }
}
