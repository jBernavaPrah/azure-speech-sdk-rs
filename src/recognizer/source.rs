use tokio::sync::mpsc::{Receiver, Sender};

pub type WavSpec = hound::WavSpec;
pub type SampleFormat = hound::SampleFormat;

/// Sample trait. This trait is implemented for the following types:
/// - u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, u128, i128, usize, isize
pub trait Sample: ToLeBytes + Send + 'static {}

impl Sample for u8 {}

impl Sample for i8 {}

impl Sample for i16 {}

impl Sample for i32 {}

impl Sample for f32 {}

impl Sample for u16 {}

impl Sample for u32 {}

impl Sample for f64 {}

impl Sample for i64 {}

impl Sample for u64 {}

impl Sample for i128 {}

impl Sample for u128 {}

impl Sample for isize {}

impl Sample for usize {}

pub trait ToLeBytes {
    fn to_le_bytes(self) -> Vec<u8>;
}

macro_rules! impl_to_le_bytes {
    ($($t:ty),*) => {
        $(
            impl ToLeBytes for $t {
                fn to_le_bytes(self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }
            }
        )*
    };
}

impl_to_le_bytes!(u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, u128, i128, usize, isize);


#[derive(Debug)]
/// Audio source
pub struct Source<T: Sample> {
    receiver: Receiver<Vec<T>>,
    pub(crate) spec: WavSpec,
    pub(crate) details: Details,
}

impl<T: Sample> Source<T> {
    /// Create a new Source instance
    pub fn new(spec: WavSpec, details: Details) -> (Self, Sender<Vec<T>>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(32 * 1024);

        (Self {
            spec,
            details,
            receiver,
        }, sender)
    }

    pub(crate) async fn next(&mut self) -> Option<Vec<T>> {
        self.receiver.recv().await
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

    pub fn unknown() -> Self {
        Details::new("Unknown", "Unknown", "Unknown", "Unknown")
    }

    pub fn stream(manufacture: impl Into<String>, connectivity: impl Into<String>) -> Self {
        Details::new("Stream", "Stream", manufacture, connectivity)
    }

    pub fn microphone(manufacture: impl Into<String>, connectivity: impl Into<String>) -> Self {
        Details::new("Microphone", "Stream", manufacture, connectivity)
    }

    pub fn file() -> Self {
        Details::new("File", "File", "Unknown", "Unknown")
    }
}
