#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Default)]
pub enum AudioFormat {
    // with header
    Riff8Khz8BitMonoALaw,
    Riff8Khz8BitMonoMULaw,
    Riff8Khz16BitMonoPcm,
    #[default]
    Riff16Khz16BitMonoPcm,
    Riff22050Hz16BitMonoPcm,
    Riff24Khz16BitMonoPcm,
    Riff44100Hz16BitMonoPcm,
    Riff48Khz16BitMonoPcm,

    // without header at the beginning
    Raw8Khz8BitMonoMULaw,
    Raw8Khz8BitMonoALaw,

    Raw8Khz16BitMonoPcm,
    Raw16Khz16BitMonoPcm,
    Raw16Khz16BitMonoTrueSilk,
    Raw22050Hz16BitMonoPcm,
    Raw24Khz16BitMonoPcm,
    Raw24Khz16BitMonoTrueSilk,
    Raw44100Hz16BitMonoPcm,
    Raw48Khz16BitMonoPcm,

    Riff16Khz16KbpsMonoSiren,
    Audio16Khz16KbpsMonoSiren,

    Audio16Khz32KBitRateMonoMp3,
    Audio16Khz128KBitRateMonoMp3,
    Audio16Khz64KBitRateMonoMp3,
    Audio24Khz48KBitRateMonoMp3,
    Audio24Khz96KBitRateMonoMp3,
    Audio24Khz160KBitRateMonoMp3,
    Audio48Khz96KBitRateMonoMp3,
    Audio48Khz192KBitRateMonoMp3,

    Ogg48Khz16BitMonoOpus,
    Ogg16Khz16BitMonoOpus,
    Ogg24Khz16BitMonoOpus,
    Webm16Khz16BitMonoOpus,
    Webm24Khz16BitMonoOpus,
    Webm24Khz16Bit24KbpsMonoOpus,
    Audio16Khz16Bit32KbpsMonoOpus,
    Audio24Khz16Bit48KbpsMonoOpus,
    Audio24Khz16Bit24KbpsMonoOpus,
    Custom(&'static str)
}

impl AudioFormat {
    pub fn as_str(&self) -> &'static str {
        match *self {
            AudioFormat::Raw8Khz8BitMonoMULaw => "raw-8khz-8bit-mono-mulaw",
            AudioFormat::Raw22050Hz16BitMonoPcm => "raw-22050hz-16bit-mono-pcm",
            AudioFormat::Riff22050Hz16BitMonoPcm => "riff-22050hz-16bit-mono-pcm",
            AudioFormat::Raw44100Hz16BitMonoPcm => "raw-44100hz-16bit-mono-pcm",
            AudioFormat::Riff44100Hz16BitMonoPcm => "riff-44100hz-16bit-mono-pcm",
            AudioFormat::Riff16Khz16BitMonoPcm => "riff-16khz-16bit-mono-pcm",
            AudioFormat::Riff8Khz16BitMonoPcm => "riff-8khz-16bit-mono-pcm",
            AudioFormat::Riff24Khz16BitMonoPcm => "riff-24khz-16bit-mono-pcm",
            AudioFormat::Raw16Khz16BitMonoPcm => "raw-16khz-16bit-mono-pcm",
            AudioFormat::Raw24Khz16BitMonoPcm => "raw-24khz-16bit-mono-pcm",
            AudioFormat::Raw8Khz16BitMonoPcm => "raw-8khz-16bit-mono-pcm",
            AudioFormat::Riff8Khz8BitMonoMULaw => "riff-8khz-8bit-mono-mulaw",
            AudioFormat::Raw48Khz16BitMonoPcm => "raw-48khz-16bit-mono-pcm",
            AudioFormat::Riff48Khz16BitMonoPcm => "riff-48khz-16bit-mono-pcm",
            AudioFormat::Raw24Khz16BitMonoTrueSilk => "raw-24khz-16bit-mono-truesilk",
            AudioFormat::Raw8Khz8BitMonoALaw => "raw-8khz-8bit-mono-alaw",
            AudioFormat::Riff8Khz8BitMonoALaw => "riff-8khz-8bit-mono-alaw",

            AudioFormat::Riff16Khz16KbpsMonoSiren => "riff-16khz-16kbps-mono-siren",
            AudioFormat::Audio16Khz16KbpsMonoSiren => "audio-16khz-16kbps-mono-siren",
            AudioFormat::Audio16Khz32KBitRateMonoMp3 => "audio-16khz-32kbitrate-mono-mp3",
            AudioFormat::Audio16Khz128KBitRateMonoMp3 => "audio-16khz-128kbitrate-mono-mp3",
            AudioFormat::Audio16Khz64KBitRateMonoMp3 => "audio-16khz-64kbitrate-mono-mp3",
            AudioFormat::Audio24Khz48KBitRateMonoMp3 => "audio-24khz-48kbitrate-mono-mp3",
            AudioFormat::Audio24Khz96KBitRateMonoMp3 => "audio-24khz-96kbitrate-mono-mp3",
            AudioFormat::Audio24Khz160KBitRateMonoMp3 => "audio-24khz-160kbitrate-mono-mp3",
            AudioFormat::Raw16Khz16BitMonoTrueSilk => "raw-16khz-16bit-mono-truesilk",
            AudioFormat::Ogg16Khz16BitMonoOpus => "ogg-16khz-16bit-mono-opus",
            AudioFormat::Ogg24Khz16BitMonoOpus => "ogg-24khz-16bit-mono-opus",
            AudioFormat::Audio48Khz96KBitRateMonoMp3 => "audio-48khz-96kbitrate-mono-mp3",
            AudioFormat::Audio48Khz192KBitRateMonoMp3 => "audio-48khz-192kbitrate-mono-mp3",
            AudioFormat::Ogg48Khz16BitMonoOpus => "ogg-48khz-16bit-mono-opus",
            AudioFormat::Webm16Khz16BitMonoOpus => "webm-16khz-16bit-mono-opus",
            AudioFormat::Webm24Khz16BitMonoOpus => "webm-24khz-16bit-mono-opus",
            AudioFormat::Webm24Khz16Bit24KbpsMonoOpus => "webm-24khz-16bit-24kbps-mono-opus",
            AudioFormat::Audio16Khz16Bit32KbpsMonoOpus => "audio-16khz-16bit-32kbps-mono-opus",
            AudioFormat::Audio24Khz16Bit48KbpsMonoOpus => "audio-24khz-16bit-48kbps-mono-opus",
            AudioFormat::Audio24Khz16Bit24KbpsMonoOpus => "audio-24khz-16bit-24kbps-mono-opus",
            AudioFormat::Custom(s) => s,
        }
    }
}
