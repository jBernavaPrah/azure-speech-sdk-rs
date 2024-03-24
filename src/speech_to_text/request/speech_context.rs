use serde::{Deserialize, Serialize};
use crate::speech_to_text::config::{LanguageDetectMode, RecognitionConfig};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SpeechContext {
    #[serde(rename = "languageId")]
    language: Option<Language>,
    #[serde(rename = "phraseOutput")]
    phrase_output: Option<PhraseOutput>,
    #[serde(rename = "phraseDetection")]
    phrase_detection: Option<PhraseDetection>,
    dgi: Option<Dgi>,
}

impl SpeechContext {
    pub(crate) fn from_config(config: RecognitionConfig) -> Self {
        let mut c = SpeechContext {
            language: None,
            dgi: None,
            phrase_output: None,
            phrase_detection: None,
        };

        if let Some(grammars) = config.phrases {
            c.dgi = Some(Dgi {
                groups: vec![Group {
                    type_group: "Generic".to_string(),
                    items: grammars.iter().map(|g| Item { text: g.to_string() }).collect(),
                }]
            })
        }

        if config.languages.len() > 1 {
            c.language = Some(Language {
                mode: config.language_detect_mode.unwrap(),
                priority: Priority::PrioritizeLatency,
                languages: config.languages,
                success_action: Action {
                    action: LanguageAction::Recognize
                },
                unknown_action: Action {
                    action: LanguageAction::None
                },
            });

            c.phrase_detection = Some(PhraseDetection {
                custom_models: if let Some(custom_models) = config.custom_models {
                    Some(custom_models.iter().map(|(l, e)| CustomModel {
                        language: l.to_string(),
                        endpoint: e.to_string(),
                    }).collect())
                } else { None },
                // todo: when translation, this are set to { action: "Translate" }
                on_interim: None,
                // todo: when translation, this are set to { action: "Translate" }
                on_success: None,
            });

            // todo: when translation, this are set to None
            c.phrase_output = Some(PhraseOutput {
                interim_results: ResultType {
                    result_type: ResultTypeType::Auto
                },
                phrase_results: ResultType {
                    result_type: ResultTypeType::Always
                },
            }
            )
        }

        c
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Priority {
    #[serde(rename = "PrioritizeLatency")]
    PrioritizeLatency
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum LanguageAction {
    Recognize,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Language {
    #[serde(rename = "Priority")]
    pub priority: Priority,
    pub languages: Vec<String>,
    pub mode: LanguageDetectMode,
    // todo: Investigate how to instruct serde to map LanguageAction to `success_action.action` = "language_action" directly.
    #[serde(rename = "onSuccess")]
    pub success_action: Action<LanguageAction>,
    #[serde(rename = "onUnknown")]
    pub unknown_action: Action<LanguageAction>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct Action<T> {
    pub action: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PhraseOutput {
    #[serde(rename = "interimResults")]
    pub interim_results: ResultType,
    #[serde(rename = "phraseResults")]
    pub phrase_results: ResultType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum ResultTypeType {
    None,
    Auto,
    Always,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ResultType {
    #[serde(rename = "resultType")]
    pub result_type: ResultTypeType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum PhraseDetectionAction {
    Translate,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CustomModel {
    language: String,
    endpoint: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PhraseDetection {
    #[serde(rename = "customModels")]
    custom_models: Option<Vec<CustomModel>>,
    #[serde(rename = "onInterim")]
    on_interim: Option<Action<PhraseDetectionAction>>,
    #[serde(rename = "onSuccess")]
    on_success: Option<Action<PhraseDetectionAction>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(self) struct Dgi {
    #[serde(rename = "Groups")]
    pub groups: Vec<Group>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(self) struct Group {
    #[serde(rename = "Type")]
    pub type_group: String,
    #[serde(rename = "Items")]
    pub items: Vec<Item>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Item {
    #[serde(rename = "Text")]
    pub text: String,
}