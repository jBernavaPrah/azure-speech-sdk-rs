use std::ffi::OsStr;

/// The Audio Format of the audio data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    /// The audio is in WAV format. 
    /// 
    /// Header needs to be present in the beginning of the audio data.
    Wav,
    /// The audio is in FLAC format.
    Flac,
    /// The audio is in Opus format.
    Opus,
    /// The audio is in MP3 format.
    Mp3,
    /// The audio is in WebM Opus format.
    WebmOpus,
    /// The audio is in Ogg Opus format.
    OggOpus,
    /// The audio is in MPEG format.
    Mpeg,
}

impl TryFrom<&OsStr> for AudioFormat {
    type Error = crate::Error;
    fn try_from(extension: &OsStr) -> Result<Self, crate::Error> {
        match extension.to_str().unwrap().to_lowercase().as_str() {
            "wav" | "wave" => Ok(AudioFormat::Wav),
            "flac" => Ok(AudioFormat::Flac),
            "opus" => Ok(AudioFormat::Opus),
            "mp3" => Ok(AudioFormat::Mp3),
            "webm" => Ok(AudioFormat::WebmOpus),
            "ogg" | "oga" => Ok(AudioFormat::OggOpus),
            "mpeg" | "mpg" => Ok(AudioFormat::Mpeg),
            ext => Err(Self::Error::IOError(format!(
                "Unsupported audio format. ({})",
                ext
            ))),
        }
    }
}

impl AudioFormat {
    pub(crate) fn as_content_type(&self) -> &str {
        match self {
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Opus => "audio/ogg; codecs=opus",
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::WebmOpus => "audio/webm; codecs=opus",
            AudioFormat::OggOpus => "audio/ogg",
            AudioFormat::Mpeg => "audio/mpeg",
        }
    }
}
