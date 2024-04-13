use log::{error, info};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message as TMessage;

#[derive(Debug)]
pub enum Message {
    UnknownPath(String, Value),
    TurnStart(Value),
    SpeechStartDetected(Value),
    SpeechEndDetected(Value),
    TurnEnd(Value),
    SpeechHypothesis(Value),
    SpeechPhrase(Value),
}

impl From<TMessage> for Message {
    fn from(message: TMessage) -> Self {

        let mut split_response = message.to_text().unwrap().split("\r\n\r\n");

        let headers = explode_headers_message(split_response.nth(0).unwrap().into());
        let path = headers.iter().find(|x| x.0 == "Path").unwrap().1.clone();

        let json = serde_json::from_str::<Value>(split_response.nth(0).unwrap()).unwrap_or_else(|e| {
            error!("Error parsing json: {:?}", e);
            Value::Null
        });

        return match path.as_str() {
            "turn.start" => Message::TurnStart(json),
            "speech.fragment" => Message::SpeechHypothesis(json),
            "speech.startDetected" => Message::SpeechStartDetected(json),
            "speech.hypothesis" => Message::SpeechHypothesis(json),
            "speech.phrase" => Message::SpeechPhrase(json),
            "speech.endDetected" => Message::SpeechEndDetected(json),
            "turn.end" => Message::TurnEnd(json),
            e => {
                error!("Unknown path: {}", e.clone());
                Message::UnknownPath(e.to_string(), json)
            }
        };
    }
}


// Example of message received:
// X-RequestId:5FF045681350489AAF1CD740EE5ACDDD
// Path:turn.start
// Content-Type:application/json; charset=utf-8
//
// {
//   "context": {
//     "serviceTag": "94dbb8f712c84981b4b69e062494f8a3"
//   }
// }
fn explode_headers_message(headers: String) -> Vec<(String, String)> {
    headers.split("\r\n")
        .map(|x| {
            let mut split = x.split(":");
            (split.nth(0).unwrap().to_string(), split.nth(0).unwrap().to_string())
        })
        .collect()
}
