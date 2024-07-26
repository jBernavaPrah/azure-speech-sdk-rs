use std::io::SeekFrom;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::{json};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;
use crate::synthesizer::config::Config;
use crate::utils::make_text_payload;

/// Creates a speech configuration message.
pub(crate) fn create_speech_config_message(session_id: Uuid,
                                           synthesizer_config: &Config,
) -> String {
    make_text_payload(
        vec![
            ("X-RequestId".to_string(), session_id.to_string()),
            ("Path".to_string(), "speech.config".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ],
        Some(json!({"context":{"system":&synthesizer_config.device.system,"os":&synthesizer_config.device.os}}).to_string()),
    )
}


/// Creates a speech context message.
pub(crate) fn create_synthesis_context_message(session_id: Uuid, config: &Config) -> String {
    
    make_text_payload(vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ("X-RequestId".to_string(), session_id.to_string()),
        ("Path".to_string(), "synthesis.context".to_string()),
    ],Some(json!({"synthesis":
        {"audio":
            {"metadataOptions":
                {
                    "bookmarkEnabled": config.bookmark_enabled,
                    "punctuationBoundaryEnabled": config.punctuation_boundary_enabled,
                    "sentenceBoundaryEnabled": config.sentence_boundary_enabled,
                    "sessionEndEnabled": config.session_end_enabled,
                    "visemeEnabled": config.viseme_enabled,
                    "wordBoundaryEnabled": config.word_boundary_enabled
                },
                "outputFormat": config.output_format.as_str()
            },
            "language": {"autoDetection": config.auto_detect_language}
        }}).to_string()))
    
}

pub(crate) fn create_ssml_message(session_id: Uuid, ssml: String) -> String {
    
    make_text_payload(vec![
        ("Content-Type".to_string(), "application/ssml+xml".to_string()),
        ("X-Timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()),
        ("X-RequestId".to_string(), session_id.to_string()),
        ("Path".to_string(), "ssml".to_string()),
    ], Some(ssml))
}

// todo: move this to separate crate!
pub(crate) struct StreamMediaSource {
    inner: Receiver<Vec<u8>>,
    buffer: Vec<u8>,
}

impl StreamMediaSource {
    pub fn new(inner: Receiver<Vec<u8>>) -> Self {
        Self {
            inner,
            buffer: Vec::with_capacity(1024),
        }
    }

    fn read_inner(&mut self, len: usize) -> Vec<u8> {
        while self.buffer.len() < len {
            match self.inner.blocking_recv() {
                Some(data) => {
                    self.buffer.extend(data);
                }
                None => break,
            }
        }
        let len = std::cmp::min(len, self.buffer.len());
        self.buffer.drain(..len).collect()
    }
}

impl std::io::Seek for StreamMediaSource {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        unreachable!("StreamMediaSource does not support seeking")
    }
}

impl std::io::Read for StreamMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.read_inner(buf.len());
        let len = std::cmp::min(buf.len(), data.len());
        buf[..len].copy_from_slice(&data[..len]);

        Ok(len)
    }
}


#[cfg(test)]
mod tests {
    use std::io::Read;

    #[test]
    fn test_stream_media_source() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let mut source = super::StreamMediaSource::new(rx);
        drop(tx);

        assert_eq!(source.read(&mut [0u8; 10]).unwrap(), 0);
    }

    #[test]
    fn test_stream_media_source_with_data() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let mut source = super::StreamMediaSource::new(rx);

        tx.blocking_send(vec![1, 2, 3, 4, 5]).unwrap();
        drop(tx);

        let mut buffer = [0u8; 10];
        assert_eq!(source.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer[..5], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_stream_media_source_with_data_larger_than_buffer() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let mut source = super::StreamMediaSource::new(rx);

        tx.blocking_send(vec![1, 2, 3, 4, 5, 6, 7]).unwrap();
        tx.blocking_send(vec![8, 9, 10]).unwrap();
        tx.blocking_send(vec![]).unwrap();
        drop(tx);

        let mut buffer = [0u8; 5];
        assert_eq!(source.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer, &[1, 2, 3, 4, 5]);

        let mut buffer = [0u8; 5];
        assert_eq!(source.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer, &[6, 7, 8, 9, 10]);

        let mut buffer = [0u8; 5];
        assert_eq!(source.read(&mut buffer).unwrap(), 0);
        assert_eq!(&buffer, &[0, 0, 0, 0, 0]);
    }
}
