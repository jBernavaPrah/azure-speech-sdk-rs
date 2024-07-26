use crate::config::Device;
use crate::synthesizer::voice::Voice;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub(crate) output_format: AudioOutputFormat,

    pub(crate) device: Device,

    pub(crate) voice: Option<Voice>,
    pub(crate) language: String,

    pub(crate) bookmark_enabled: bool,
    pub(crate) word_boundary_enabled: bool,
    pub(crate) punctuation_boundary_enabled: bool,
    pub(crate) sentence_boundary_enabled: bool,
    pub(crate) session_end_enabled: bool,
    pub(crate) viseme_enabled: bool,

    pub(crate) auto_detect_language: bool,
}


impl Config {
    pub fn new() -> Self {
        Self {
            session_end_enabled: true,
            auto_detect_language: true,
            language: "en-US".to_string(),
            ..Default::default()
        }
    }

    pub fn with_output_format(mut self, output_format: AudioOutputFormat) -> Self {
        self.output_format = output_format;
        self
    }

    pub fn enable_bookmark(mut self) -> Self {
        self.bookmark_enabled = true;
        self
    }

    pub fn enable_word_boundary(mut self) -> Self {
        self.word_boundary_enabled = true;
        self
    }

    pub fn enable_punctuation_boundary(mut self) -> Self {
        self.punctuation_boundary_enabled = true;
        self
    }

    pub fn enable_sentence_boundary(mut self) -> Self {
        self.sentence_boundary_enabled = true;
        self
    }

    pub fn enable_session_end(mut self) -> Self {
        self.session_end_enabled = true;
        self
    }

    pub fn enable_viseme(mut self) -> Self {
        self.viseme_enabled = true;
        self
    }

    pub fn disable_auto_detect_language(mut self) -> Self {
        self.auto_detect_language = false;
        self
    }
    
    pub fn set_device(mut self, device: Device) -> Self {
        self.device = device;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Default)]
pub enum AudioOutputFormat {
    #[default]
    Riff16Khz16BitMonoPcm,
    Riff8Khz16BitMonoPcm,
    Riff24Khz16BitMonoPcm,
    Riff8Khz8BitMonoMULaw,
    Raw16Khz16BitMonoPcm,
    Raw24Khz16BitMonoPcm,
    Raw8Khz16BitMonoPcm,
    Raw48Khz16BitMonoPcm,
    Riff48Khz16BitMonoPcm,
    Raw22050Hz16BitMonoPcm,
    Riff22050Hz16BitMonoPcm,
    Raw44100Hz16BitMonoPcm,
    Riff44100Hz16BitMonoPcm,
    Raw8Khz8BitMonoMULaw,

    Riff16Khz16KbpsMonoSiren,
    Audio16Khz16KbpsMonoSiren,

    Audio16Khz32KBitRateMonoMp3,
    Audio16Khz128KBitRateMonoMp3,
    Audio16Khz64KBitRateMonoMp3,
    Audio24Khz48KBitRateMonoMp3,
    Audio24Khz96KBitRateMonoMp3,
    Audio24Khz160KBitRateMonoMp3,
    Raw16Khz16BitMonoTrueSilk,

    Ogg16Khz16BitMonoOpus,
    Ogg24Khz16BitMonoOpus,
    Audio48Khz96KBitRateMonoMp3,
    Audio48Khz192KBitRateMonoMp3,
    Ogg48Khz16BitMonoOpus,
    Webm16Khz16BitMonoOpus,
    Webm24Khz16BitMonoOpus,
    Webm24Khz16Bit24KbpsMonoOpus,
    Raw24Khz16BitMonoTrueSilk,
    Raw8Khz8BitMonoALaw,
    Riff8Khz8BitMonoALaw,
    Audio16Khz16Bit32KbpsMonoOpus,
    Audio24Khz16Bit48KbpsMonoOpus,
    Audio24Khz16Bit24KbpsMonoOpus,
}

impl AudioOutputFormat {
    pub fn as_str(&self) -> &'static str {
        match *self {
            AudioOutputFormat::Raw8Khz8BitMonoMULaw => "raw-8khz-8bit-mono-mulaw",
            AudioOutputFormat::Riff16Khz16KbpsMonoSiren => "riff-16khz-16kbps-mono-siren",
            AudioOutputFormat::Audio16Khz16KbpsMonoSiren => "audio-16khz-16kbps-mono-siren",
            AudioOutputFormat::Audio16Khz32KBitRateMonoMp3 => "audio-16khz-32kbitrate-mono-mp3",
            AudioOutputFormat::Audio16Khz128KBitRateMonoMp3 => "audio-16khz-128kbitrate-mono-mp3",
            AudioOutputFormat::Audio16Khz64KBitRateMonoMp3 => "audio-16khz-64kbitrate-mono-mp3",
            AudioOutputFormat::Audio24Khz48KBitRateMonoMp3 => "audio-24khz-48kbitrate-mono-mp3",
            AudioOutputFormat::Audio24Khz96KBitRateMonoMp3 => "audio-24khz-96kbitrate-mono-mp3",
            AudioOutputFormat::Audio24Khz160KBitRateMonoMp3 => "audio-24khz-160kbitrate-mono-mp3",
            AudioOutputFormat::Raw16Khz16BitMonoTrueSilk => "raw-16khz-16bit-mono-truesilk",
            AudioOutputFormat::Riff16Khz16BitMonoPcm => "riff-16khz-16bit-mono-pcm",
            AudioOutputFormat::Riff8Khz16BitMonoPcm => "riff-8khz-16bit-mono-pcm",
            AudioOutputFormat::Riff24Khz16BitMonoPcm => "riff-24khz-16bit-mono-pcm",
            AudioOutputFormat::Riff8Khz8BitMonoMULaw => "riff-8khz-8bit-mono-mulaw",
            AudioOutputFormat::Raw16Khz16BitMonoPcm => "raw-16khz-16bit-mono-pcm",
            AudioOutputFormat::Raw24Khz16BitMonoPcm => "raw-24khz-16bit-mono-pcm",
            AudioOutputFormat::Raw8Khz16BitMonoPcm => "raw-8khz-16bit-mono-pcm",
            AudioOutputFormat::Ogg16Khz16BitMonoOpus => "ogg-16khz-16bit-mono-opus",
            AudioOutputFormat::Ogg24Khz16BitMonoOpus => "ogg-24khz-16bit-mono-opus",
            AudioOutputFormat::Raw48Khz16BitMonoPcm => "raw-48khz-16bit-mono-pcm",
            AudioOutputFormat::Riff48Khz16BitMonoPcm => "riff-48khz-16bit-mono-pcm",
            AudioOutputFormat::Audio48Khz96KBitRateMonoMp3 => "audio-48khz-96kbitrate-mono-mp3",
            AudioOutputFormat::Audio48Khz192KBitRateMonoMp3 => "audio-48khz-192kbitrate-mono-mp3",
            AudioOutputFormat::Ogg48Khz16BitMonoOpus => "ogg-48khz-16bit-mono-opus",
            AudioOutputFormat::Webm16Khz16BitMonoOpus => "webm-16khz-16bit-mono-opus",
            AudioOutputFormat::Webm24Khz16BitMonoOpus => "webm-24khz-16bit-mono-opus",
            AudioOutputFormat::Webm24Khz16Bit24KbpsMonoOpus => "webm-24khz-16bit-24kbps-mono-opus",
            AudioOutputFormat::Raw24Khz16BitMonoTrueSilk => "raw-24khz-16bit-mono-truesilk",
            AudioOutputFormat::Raw8Khz8BitMonoALaw => "raw-8khz-8bit-mono-alaw",
            AudioOutputFormat::Riff8Khz8BitMonoALaw => "riff-8khz-8bit-mono-alaw",
            AudioOutputFormat::Audio16Khz16Bit32KbpsMonoOpus => "audio-16khz-16bit-32kbps-mono-opus",
            AudioOutputFormat::Audio24Khz16Bit48KbpsMonoOpus => "audio-24khz-16bit-48kbps-mono-opus",
            AudioOutputFormat::Audio24Khz16Bit24KbpsMonoOpus => "audio-24khz-16bit-24kbps-mono-opus",
            AudioOutputFormat::Raw22050Hz16BitMonoPcm => "raw-22050hz-16bit-mono-pcm",
            AudioOutputFormat::Riff22050Hz16BitMonoPcm => "riff-22050hz-16bit-mono-pcm",
            AudioOutputFormat::Raw44100Hz16BitMonoPcm => "raw-44100hz-16bit-mono-pcm",
            AudioOutputFormat::Riff44100Hz16BitMonoPcm => "riff-44100hz-16bit-mono-pcm",
        }
    }
}
