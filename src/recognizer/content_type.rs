/// The content type of the audio data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// The audio data is in WAV format.
    Wav,
    /// The audio data is in WAV format without the header.
    /// Pass the header.
    Raw(Vec<u8>),
    Flac,
    Opus,
    Mp3,
    Webm,
    Ogg,
    Mpeg,
    Specific(&'static str, Option<Vec<u8>>),
}

impl ContentType {
    pub(crate) fn as_header(&self) -> Option<Vec<u8>> {
        match self {
            ContentType::Raw(header) => Some(header.clone()),
            ContentType::Specific(_, header) => header.clone(),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            ContentType::Wav => "audio/x-wav",
            ContentType::Raw(_) => "audio/x-wav",
            ContentType::Flac => "audio/flac",
            ContentType::Opus => "audio/ogg; codecs=opus",
            ContentType::Mp3 => "audio/mpeg",
            ContentType::Webm => "audio/webm",
            ContentType::Ogg => "audio/ogg",
            ContentType::Mpeg => "audio/mpeg",
            ContentType::Specific(s, _) => s,
        }
    }
}
