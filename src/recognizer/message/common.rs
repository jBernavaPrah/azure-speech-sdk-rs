use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Language {
    #[serde(rename = "Language")]
    pub(crate) language: String,
    #[serde(rename = "Confidence")]
    pub(crate) confidence: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub(crate) enum RecognitionStatus {
    Success,
    NoMatch,
    InitialSilenceTimeout,
    BabbleTimeout,
    Error,
    EndOfDictation,
    TooManyRequests,
    BadRequest,
    Forbidden,
}

impl From<&RecognitionStatus> for Option<crate::Error> {
    fn from(value: &RecognitionStatus) -> Option<crate::Error> {
        match value {
            RecognitionStatus::Error => Some(crate::Error::RuntimeError("An error occurred during the recognition.".to_string())),
            RecognitionStatus::TooManyRequests => Some(crate::Error::TooManyRequests),
            RecognitionStatus::BadRequest => Some(crate::Error::BadRequest),
            RecognitionStatus::Forbidden => Some(crate::Error::Forbidden),
            _ => None,
        }
    }
}

#[allow(dead_code)]
impl RecognitionStatus {
    pub(crate) fn is_cancelled(&self) -> bool {
        matches!(self, RecognitionStatus::Error
            | RecognitionStatus::BadRequest
            | RecognitionStatus::Forbidden)
    }

    pub(crate) fn is_success(&self) -> bool {
        matches!(self, RecognitionStatus::Success)
    }

    pub(crate) fn is_no_match(&self) -> bool {
        matches!(self, RecognitionStatus::NoMatch
             | RecognitionStatus::InitialSilenceTimeout
             | RecognitionStatus::BabbleTimeout)
    }

    pub(crate) fn is_end_of_dictation(&self) -> bool {
        matches!(self, RecognitionStatus::EndOfDictation)
    }
}