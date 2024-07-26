
/// WavSpec type from hound crate
pub type Spec = hound::WavSpec;
/// SampleFormat type from hound crate
pub type SampleFormat = hound::SampleFormat;

#[derive(Debug, Clone)]
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

