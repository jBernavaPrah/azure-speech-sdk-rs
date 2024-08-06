
/// The content type of the audio data.
/// 
/// The audio data needs to have the headers (if present) set accordingly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Wav,
    Flac,
    Opus,
    Pcm,
    Mp3,
    Webm,
    Ogg,
    Mpeg,
    Specific(&'static str),
}

impl ContentType {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            ContentType::Wav => "audio/wav",
            ContentType::Flac => "audio/flac",
            ContentType::Opus => "audio/ogg; codecs=opus",
            ContentType::Pcm => "audio/wav",
            ContentType::Mp3 => "audio/mpeg",
            ContentType::Webm => "audio/webm",
            ContentType::Ogg => "audio/ogg",
            ContentType::Mpeg => "audio/mpeg",
            ContentType::Specific(s) => s,
        }
    }
}
