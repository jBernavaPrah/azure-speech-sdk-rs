use std::sync::Arc;

/// WavSpec type from hound crate
pub type WavSpec = hound::WavSpec;
/// SampleFormat type from hound crate
pub type SampleFormat = hound::SampleFormat;

/// Sample trait. This trait is implemented for the following types:
/// - u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, u128, i128, usize, isize
#[derive(Debug, Clone, Copy, PartialEq)]
/// The sample format of the audio data.
/// It's used to containing the audio data.
pub enum Sample {
    #[allow(missing_docs)]
    U8(u8),
    #[allow(missing_docs)]
    I8(i8),
    #[allow(missing_docs)]
    U16(u16),
    #[allow(missing_docs)]
    I16(i16),
    #[allow(missing_docs)]
    U32(u32),
    #[allow(missing_docs)]
    I32(i32),
    #[allow(missing_docs)]
    F32(f32),
    #[allow(missing_docs)]
    U64(u64),
    #[allow(missing_docs)]
    I64(i64),
    #[allow(missing_docs)]
    F64(f64),
}

impl Sample {
    pub(crate) fn to_bytes(self) -> Arc<[u8]> {
        match self {
            Sample::U8(s) => Arc::new([s]),
            Sample::I8(s) => Arc::new([s as u8]),
            Sample::U16(s) => Arc::new(s.to_le_bytes()),
            Sample::I16(s) => Arc::new(s.to_le_bytes()),
            Sample::U32(s) => Arc::new(s.to_le_bytes()),
            Sample::I32(s) => Arc::new(s.to_le_bytes()),
            Sample::F32(s) => Arc::new(s.to_le_bytes()),
            Sample::U64(s) => Arc::new(s.to_le_bytes()),
            Sample::I64(s) => Arc::new(s.to_le_bytes()),
            Sample::F64(s) => Arc::new(s.to_le_bytes()),
            
        }
    }
}
macro_rules! impl_from_for_sample {
    ($($t:ty => $variant:ident),*) => {
        $(
            impl From<$t> for Sample {
                fn from(s: $t) -> Self {
                    Sample::$variant(s)
                }
            }
        )*
    }
}

impl_from_for_sample!(
    u8 => U8,
    i8 => I8,
    u16 => U16,
    i16 => I16,
    u32 => U32,
    i32 => I32,
    f32 => F32,
    u64 => U64,
    i64 => I64,
    f64 => F64
);


#[derive(Debug)]
/// This will be used to indicate the source of the audio data.
pub struct Source
{
    pub(crate) stream: tokio::sync::mpsc::Receiver<Sample>,
    pub(crate) spec: WavSpec,
    pub(crate) details: Details,
}


impl Source
{
    /// Create new instance of Source
    pub fn new(spec: WavSpec, details: Details) -> (Self, tokio::sync::mpsc::Sender<Sample>) {
        let (tx, rx) = tokio::sync::mpsc::channel(1024 * 32);
        (Source { stream: rx, spec, details }, tx)
    }
}

#[derive(Debug)]
/// Details of the source. This is used to provide information about the source.
pub struct Details {
    /// Name of the source, e.g. "Microphone", "Stream", "File"
    pub name: String,
    /// Model of the source, e.g. "Stream", "File"
    pub model: String,
    /// Manufacturer of the source, e.g. "Unknown"
    pub connectivity: String,
    /// Connectivity of the source, e.g. "Unknown"
    pub manufacturer: String,
}

impl Details {
    /// Create a new Details instance
    pub fn new(name: impl Into<String>,
               model: impl Into<String>,
               manufacturer: impl Into<String>,
               connectivity: impl Into<String>) -> Self {
        Details {
            name: name.into(),
            model: model.into(),
            manufacturer: manufacturer.into(),
            connectivity: connectivity.into(),
        }
    }

    #[allow(missing_docs)]
    pub fn unknown() -> Self {
        Details::new("Unknown", "Unknown", "Unknown", "Unknown")
    }

    #[allow(missing_docs)]
    pub fn stream(manufacture: impl Into<String>, connectivity: impl Into<String>) -> Self {
        Details::new("Stream", "Stream", manufacture, connectivity)
    }
    #[allow(missing_docs)]
    pub fn microphone(manufacture: impl Into<String>, connectivity: impl Into<String>) -> Self {
        Details::new("Microphone", "Stream", manufacture, connectivity)
    }
    #[allow(missing_docs)]
    pub fn file() -> Self {
        Details::new("File", "File", "Unknown", "Unknown")
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    async fn test_source(mut source: Source, samples: Vec<Vec<u8>>)
    {
        let mut count = 0;
        while let Some(sample) = source.stream.recv().await {
            assert_eq!(sample.to_bytes().as_ref(), samples[count].as_slice(), "Sample {} is not equal", count);
            count += 1;
        }

        assert_eq!(count, samples.len());
    }

    #[tokio::test]
    async fn using_channel() {
        let (source, tx) = Source::new(WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        }, Details::unknown());

        tokio::spawn(async move {
            tx.send(Sample::F32(1f32)).await.unwrap();
            tx.send(Sample::F32(2f32)).await.unwrap();
            tx.send(Sample::F32(3f32)).await.unwrap();

            drop(tx);
        });

        test_source(source, vec![
            vec![0, 0, 128, 63],
            vec![0, 0, 0, 64],
            vec![0, 0, 64, 64],
        ]).await;
    }
}
