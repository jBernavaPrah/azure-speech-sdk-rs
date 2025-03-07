use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum BoundaryType {
    #[serde(rename = "SentenceBoundary")]
    Sentence,
    #[serde(rename = "WordBoundary")]
    Word,
    #[serde(rename = "PunctuationBoundary")]
    Punctuation,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Text {
    pub text: String,
    pub length: i64,
    pub boundary_type: BoundaryType,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "Type", content = "Data")]
pub enum Metadata {
    WordBoundary {
        /// Specifies the audio offset, in ticks (100 nanoseconds).
        #[serde(rename = "Offset")]
        offset: i64,
        /// Specifies the duration, in ticks (100 nanoseconds).
        #[serde(rename = "Duration")]
        duration: i64,
        #[serde(rename = "text")]
        text: Text,
    },
    SentenceBoundary {
        /// Specifies the audio offset, in ticks (100 nanoseconds).
        #[serde(rename = "Offset")]
        offset: i64,
        /// Specifies the duration, in ticks (100 nanoseconds).
        #[serde(rename = "Duration")]
        duration: i64,
        text: Text,
    },
    Viseme {
        #[serde(rename = "Offset")]
        offset: i64,
        #[serde(rename = "VisemeId")]
        viseme_id: i64,
        #[serde(rename = "IsLastAnimation")]
        is_last_animation: bool,
    },
    SessionEnd {
        /// Specifies the audio offset, in ticks (100 nanoseconds).
        #[serde(rename = "Offset")]
        offset: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Root {
    #[serde(rename = "Metadata")]
    pub metadata: Vec<Metadata>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_end_session() {
        let json = r#"{
  "Metadata": [
    {
      "Type": "SessionEnd",
      "Data": {
        "Offset": 16500000
      }
    }
  ]
}
"#;

        let root: Root = serde_json::from_str(json).unwrap();
        assert_eq!(root.metadata.len(), 1);
        match &root.metadata[0] {
            Metadata::SessionEnd { offset } => {
                assert_eq!(*offset, 16500000);
            }
            _ => panic!("Expected SessionEnd"),
        }
    }

    #[test]
    fn test_viseme() {
        let json = r#"{
  "Metadata": [
    {
      "Type": "Viseme",
      "Data": {
        "Offset": 500000,
        "VisemeId": 0,
        "IsLastAnimation": true
      }
    }
  ]
}
"#;

        let root: Root = serde_json::from_str(json).unwrap();
        assert_eq!(root.metadata.len(), 1);
        match &root.metadata[0] {
            Metadata::Viseme {
                offset,
                viseme_id,
                is_last_animation,
            } => {
                assert_eq!(*offset, 500000);
                assert_eq!(*viseme_id, 0);
                assert_eq!(*is_last_animation, true);
            }
            _ => panic!("Expected Viseme"),
        }
    }

    #[test]
    fn test_sentence_boundary() {
        let json = r#"{
  "Metadata": [
    {
      "Type": "SentenceBoundary",
      "Data": {
        "Offset": 500000,
        "Duration": 16000000,
        "text": {
          "Text": "Hello World!",
          "Length": 12,
          "BoundaryType": "SentenceBoundary"
        }
      }
    }
  ]
}
"#;

        let root: Root = serde_json::from_str(json).unwrap();
        assert_eq!(root.metadata.len(), 1);
        match &root.metadata[0] {
            Metadata::SentenceBoundary {
                offset,
                duration,
                text,
            } => {
                assert_eq!(*offset, 500000);
                assert_eq!(*duration, 16000000);
                assert_eq!(text.text, "Hello World!");
                assert_eq!(text.length, 12);
                assert_eq!(text.boundary_type, BoundaryType::Sentence);
            }
            _ => panic!("Expected SentenceBoundary"),
        }
    }

    #[test]
    fn test_word_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
                          "Metadata": [
                              {
                                  "Type": "WordBoundary",
                                  "Data": {
                                      "Offset": 500000,
                                      "Duration": 5125000,
                                      "text": {
                                          "Text": "Hello",
                                          "Length": 5,
                                          "BoundaryType": "WordBoundary"
                                      }
                                  }
                              }
                          ]
                      }"#;

        let root: Root = serde_json::from_str(json)?;
        assert_eq!(root.metadata.len(), 1);
        match &root.metadata[0] {
            Metadata::WordBoundary {
                offset,
                duration,
                text,
            } => {
                assert_eq!(*offset, 500000);
                assert_eq!(*duration, 5125000);
                assert_eq!(text.text, "Hello");
                assert_eq!(text.length, 5);
                assert_eq!(text.boundary_type, BoundaryType::Word);
            }
            _ => panic!("Expected WordBoundary"),
        }
        Ok(())
    }

    #[test]
    fn test_punctuation_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#" {
                              "Metadata": [
                                {
                                  "Type": "WordBoundary",
                                  "Data": {
                                    "Offset": 8500000,
                                    "Duration": 1000000,
                                    "text": {
                                      "Text": "!",
                                      "Length": 1,
                                      "BoundaryType": "PunctuationBoundary"
                                    }
                                  }
                                }
                              ]
                            }
"#;

        let root: Root = serde_json::from_str(json)?;
        assert_eq!(root.metadata.len(), 1);
        match &root.metadata[0] {
            Metadata::WordBoundary {
                offset,
                duration,
                text,
            } => {
                assert_eq!(*offset, 8500000);
                assert_eq!(*duration, 1000000);
                assert_eq!(text.text, "!");
                assert_eq!(text.length, 1);
                assert_eq!(text.boundary_type, BoundaryType::Punctuation);
            }
            _ => panic!("Expected WordBoundary"),
        }
        Ok(())
    }
}
