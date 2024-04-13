use serde::{Deserialize, Serialize};
use crate::speech_to_text::AudioHeaders;
use crate::speech_to_text::config::{ Os, RecognitionConfig, RecognitionMode, System};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SpeechConfig {
    pub(crate) context: Context,
    pub(crate) recognition: RecognitionMode,
}

impl SpeechConfig {
    pub(crate) fn from_config(config: &RecognitionConfig, audio_headers: &AudioHeaders) -> Self {
        SpeechConfig {
            recognition: config.mode,
            context: Context {
                audio: Audio {
                    source: Source {
                        connectivity: config.source.connectivity.clone(),
                        name: config.source.name.clone(),
                        model: config.source.model.clone(),
                        manufacturer: config.source.manufacturer.clone(),
                        bits_per_sample: audio_headers.bits_per_sample,
                        sample_rate: audio_headers.sample_rate,
                        channel_count: audio_headers.channels,
                    }
                },
                system: config.system.clone(),
                os: config.os.clone(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Context {
    pub(crate) system: System,
    pub(crate) os: Os,
    pub(crate) audio: Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Audio {
    pub(crate) source: Source,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Source {
    pub connectivity: String,
    pub manufacturer: String,
    pub model: String,
    #[serde(rename = "type")]
    pub name: String,

    #[serde(rename = "samplerate")]
    pub sample_rate: u32,
    #[serde(rename = "bitspersample")]
    pub bits_per_sample: u16,
    #[serde(rename = "channelcount")]
    pub channel_count: u16,
}