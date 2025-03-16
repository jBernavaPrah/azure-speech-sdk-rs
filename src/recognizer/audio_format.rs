use std::ffi::OsStr;
use tokio::io::{AsyncRead, AsyncReadExt};

/// The WAV Audio Format Type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WavType {
    /// PCM
    Pcm,
    /// ALaw
    ALaw,
    /// MuLaw
    MuLaw,
}

impl WavType {
    pub(crate) fn as_u16(&self) -> u16 {
        match self {
            WavType::Pcm => 1,
            WavType::ALaw => 6,
            WavType::MuLaw => 7,
        }
    }
}

impl TryFrom<u16> for WavType {
    type Error = crate::Error;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(WavType::Pcm),
            6 => Ok(WavType::ALaw),
            7 => Ok(WavType::MuLaw),
            n => Err(Self::Error::IOError(format!(
                "Wav type ({n}) not supported."
            ))),
        }
    }
}

/// The Audio Format of the audio data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    /// The audio is in WAV format.
    Wav {
        sample_rate: u32,
        bits_per_sample: u16,
        channels: u16,
        wav_type: WavType,
    },
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

impl AudioFormat {
    /// Try to create an AudioFormat from a file.
    pub(crate) async fn try_from_reader(
        // accept anything that can be read
        reader: &mut (impl AsyncRead + Unpin),
        extension: &OsStr,
    ) -> Result<Self, crate::Error> {
        let ext = extension
            .to_str()
            .ok_or_else(|| crate::Error::IOError("Extension not valid UTF-8.".to_string()))?
            .to_lowercase();

        match ext.as_str() {
            "wav" | "wave" => {
                // Read the RIFF header (first 12 bytes)
                let mut riff_header = [0u8; 12];
                reader.read_exact(&mut riff_header).await?;
                if &riff_header[0..4] != b"RIFF" {
                    return Err(crate::Error::IOError(
                        "Invalid WAV file: missing RIFF header.".to_string(),
                    ));
                }
                if &riff_header[8..12] != b"WAVE" {
                    return Err(crate::Error::IOError(
                        "Invalid WAV file: missing WAVE format.".to_string(),
                    ));
                }

                // Variables to hold the extracted values.
                let mut wav_format: Option<u16> = None;
                let mut channels: Option<u16> = None;
                let mut sample_rate: Option<u32> = None;
                let mut bits_per_sample: Option<u16> = None;

                // Loop over chunks until we've read the necessary "fmt " chunk.
                loop {
                    // Read chunk header: 4 bytes for chunk ID and 4 bytes for chunk size.
                    let mut chunk_header = [0u8; 8];
                    // If we cannot read more, break out of the loop.
                    if reader.read_exact(&mut chunk_header).await.is_err() {
                        break;
                    }
                    let chunk_id = &chunk_header[0..4];
                    let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap());

                    match chunk_id {
                        b"fmt " => {
                            // Read the entire fmt chunk.
                            let mut fmt_data = vec![0u8; chunk_size as usize];
                            reader.read_exact(&mut fmt_data).await?;
                            // The minimum size for the fmt chunk is 16 bytes.
                            if chunk_size < 16 {
                                return Err(crate::Error::IOError(
                                    "Invalid fmt chunk size.".to_string(),
                                ));
                            }

                            // Parse the fields from the fmt chunk.
                            // Bytes 0-1: Audio format
                            wav_format = Some(u16::from_le_bytes([fmt_data[0], fmt_data[1]]));
                            // Bytes 2-3: Number of channels.
                            channels = Some(u16::from_le_bytes([fmt_data[2], fmt_data[3]]));
                            // Bytes 4-7: Sample rate.
                            sample_rate = Some(u32::from_le_bytes([
                                fmt_data[4],
                                fmt_data[5],
                                fmt_data[6],
                                fmt_data[7],
                            ]));
                            // Bytes 8-11: Byte rate (skipped)
                            // Bytes 12-13: Block align (skipped)
                            // Bytes 14-15: Bits per sample.
                            bits_per_sample =
                                Some(u16::from_le_bytes([fmt_data[14], fmt_data[15]]));
                            // If there are extra bytes (for non-PCM formats), you could process them here.
                        }
                        b"data" => {
                            // The data chunk contains the audio samples.
                            // You might want to store or process it later.
                            // Here we simply skip over it.
                            let mut _data = vec![0u8; chunk_size as usize];
                            reader.read_exact(&mut _data).await?;
                        }
                        _ => {
                            // Skip any unknown chunks.
                            let mut skip_buf = vec![0u8; chunk_size as usize];
                            reader.read_exact(&mut skip_buf).await?;
                        }
                    }

                    // If we've found all the necessary information from the fmt chunk, exit the loop.
                    if wav_format.is_some()
                        && channels.is_some()
                        && sample_rate.is_some()
                        && bits_per_sample.is_some()
                    {
                        break;
                    }
                }

                // Ensure that all required fields were found.
                let wav_format = wav_format
                    .ok_or_else(|| crate::Error::IOError("Missing fmt chunk.".to_string()))?;
                let channels = channels.ok_or_else(|| {
                    crate::Error::IOError("Missing channels information.".to_string())
                })?;
                let sample_rate = sample_rate.ok_or_else(|| {
                    crate::Error::IOError("Missing sample rate information.".to_string())
                })?;
                let bits_per_sample = bits_per_sample.ok_or_else(|| {
                    crate::Error::IOError("Missing bits per sample information.".to_string())
                })?;

                // Convert the WAV format code to our WavType.
                let wav_type = WavType::try_from(wav_format)?;

                Ok(AudioFormat::Wav {
                    sample_rate,
                    bits_per_sample,
                    channels,
                    wav_type,
                })
            }
            "flac" => Ok(AudioFormat::Flac),
            "opus" => Ok(AudioFormat::Opus),
            "mp3" => Ok(AudioFormat::Mp3),
            "webm" => Ok(AudioFormat::WebmOpus),
            "ogg" | "oga" => Ok(AudioFormat::OggOpus),
            "mpeg" | "mpg" => Ok(AudioFormat::Mpeg),
            _ => Err(crate::Error::IOError(
                "Unsupported audio format.".to_string(),
            )),
        }
    }

    pub(crate) fn as_header(&self) -> Option<Vec<u8>> {
        match self {
            //ContentType::Raw(header) => Some(header.clone()),
            AudioFormat::Wav {
                wav_type,
                channels,
                sample_rate,
                bits_per_sample,
            } => {
                // Create a 44-byte header initialized with zeros.
                let mut header = vec![0u8; 44];

                // RIFF chunk descriptor:
                // Bytes 0-3: "RIFF"
                header[0..4].copy_from_slice(b"RIFF");

                // Bytes 4-7: Chunk size (36 + Subchunk2Size).
                // Here we use 36 as a placeholder (i.e., assuming 0 bytes of audio data).
                let chunk_size: u32 = 36;
                header[4..8].copy_from_slice(&chunk_size.to_le_bytes());

                // Bytes 8-11: Format, must be "WAVE"
                header[8..12].copy_from_slice(b"WAVE");

                // "fmt " sub-chunk:
                // Bytes 12-15: Subchunk1 ID "fmt "
                header[12..16].copy_from_slice(b"fmt ");
                // Bytes 16-19: Subchunk1 size (16 for PCM)
                header[16..20].copy_from_slice(&16u32.to_le_bytes());
                // Bytes 20-21: Audio format (PCM = 1, etc.)
                header[20..22].copy_from_slice(&wav_type.as_u16().to_le_bytes());
                // Bytes 22-23: Number of channels
                header[22..24].copy_from_slice(&channels.to_le_bytes());
                // Bytes 24-27: Sample rate
                header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
                // Bytes 28-31: Byte rate = SampleRate * NumChannels * BitsPerSample / 8
                let byte_rate = sample_rate * (*channels as u32) * (*bits_per_sample as u32) / 8;
                header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
                // Bytes 32-33: Block align = NumChannels * BitsPerSample / 8
                let block_align = channels * bits_per_sample / 8;
                header[32..34].copy_from_slice(&block_align.to_le_bytes());
                // Bytes 34-35: Bits per sample
                header[34..36].copy_from_slice(&bits_per_sample.to_le_bytes());

                // "data" sub-chunk:
                // Bytes 36-39: Subchunk2 ID "data"
                header[36..40].copy_from_slice(b"data");
                // Bytes 40-43: Subchunk2 size (here set to 0 as a placeholder)
                header[40..44].copy_from_slice(&0u32.to_le_bytes());

                Some(header)
            }
            _ => None,
        }
    }

    pub(crate) fn as_content_type(&self) -> &str {
        match self {
            AudioFormat::Wav { .. } => "audio/wav",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Opus => "audio/ogg; codecs=opus",
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::WebmOpus => "audio/webm; codecs=opus",
            AudioFormat::OggOpus => "audio/ogg",
            AudioFormat::Mpeg => "audio/mpeg",
        }
    }
}
