use log::error;
use serde::Serialize;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message as TMessage;


const CRLF: &str = "\r\n";

#[derive(Debug, Clone)]
pub enum Message {
    Binary {
        headers: Vec<(String, String)>,
        data: Option<Vec<u8>>,
    },
    Text {
        headers: Vec<(String, String)>,
        data: Option<Value>,
    },
}

impl Message {
    
    pub fn json(&self) -> Option<Value> {
        match self {
            Message::Text { data, .. } => data.clone(),
            _ => None,
        }
    }
    
    pub fn data(&self) -> Option<Vec<u8>> {
        match self {
            Message::Binary { data, .. } => data.clone(),
            _ => None,
        }
    }
    
    pub fn headers(&self) -> Vec<(String, String)> {
        match self {
            Message::Binary { headers, .. } => headers.clone(),
            Message::Text { headers, .. } => headers.clone(),
        }
    } 
    
    pub fn path(&self) -> Option<String> {
        let search = |headers: &Vec<(String, String)>| headers.iter().find(|x| x.0 == "Path").map(|x| x.1.clone().to_lowercase());

        match self {
            Message::Text { headers, .. } => {
                search(headers)
            }
            Message::Binary { headers, .. } => {
                search(headers)
            }
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            Message::Text { .. } => true,
            _ => false,
        }
    }
    pub fn is_binary(&self) -> bool {
        match self {
            Message::Binary { .. } => true,
            _ => false,
        }
    }
}

impl From<Message> for TMessage {
    fn from(message: Message) -> Self {
        match message {
            Message::Binary { data, headers } => TMessage::Binary(make_binary_payload(transform_headers_to_string(headers), data)),
            Message::Text { data, headers } => make_text_message(headers, data),
        }
    }
}

impl From<TMessage> for Message {
    fn from(value: TMessage) -> Self {
        match value {
            TMessage::Text(text) => {
                let (headers, json) = extract_headers_and_json_from_text_message(text);
                Message::Text { headers, data: Some(json) }
            }
            TMessage::Binary(data) => {
                let (headers, data) = extract_headers_and_data_from_binary_message(data);
                Message::Binary { headers, data }
            }
            _ => unreachable!("Only text messages are supported"),
        }
    }
}


fn make_text_message<T: Serialize>(headers: Vec<(String, String)>, data: Option<T>) -> TMessage {
    let headers = transform_headers_to_string(headers);
    let data = data.map_or(String::new(), |d| serde_json::to_string(&d).unwrap());

    TMessage::Text(format!("{}{CRLF}{}", headers, data))
}


fn transform_headers_to_string(map: Vec<(String, String)>) -> String {
    let mut headers = String::new();
    for (content_type, value) in map {
        headers.push_str(format!("{content_type}: {value}{CRLF}").as_str());
    }

    headers
}

fn make_binary_payload(headers: String, data: Option<Vec<u8>>) -> Vec<u8> {
    let data_length = if let Some(ref d) = data {
        d.len()
    } else {
        0
    };

    let header_buffer: Vec<_> = headers.bytes().collect();
    let header_length = header_buffer.len();
    let mut payload = vec![0; 2 + header_length + data_length];
    payload[0] = ((header_length >> 8) & 0xff) as u8;
    payload[1] = (header_length & 0xff) as u8;
    payload[2..2 + header_length].copy_from_slice(&header_buffer);

    if let Some(ref d) = data {
        payload[2 + header_length..].copy_from_slice(d);
    }

    payload
}


fn extract_headers_and_data_from_binary_message(data: Vec<u8>) -> (Vec<(String, String)>, Option<Vec<u8>>) {
    let header_length = ((data[0] as usize) << 8) + data[1] as usize;
    let headers = String::from_utf8(data[2..2 + header_length].to_vec()).unwrap();
    let data = if header_length + 2 < data.len() {
        Some(data[2 + header_length..].to_vec())
    } else {
        None
    };

    (explode_headers_message(headers), data)
}

fn extract_headers_and_json_from_text_message(text: String) -> (Vec<(String, String)>, Value) {
    let mut split_response = text.split("\r\n\r\n");
    let headers = explode_headers_message(split_response.nth(0).unwrap().into());
    let json = serde_json::from_str::<Value>(split_response.nth(0).unwrap()).unwrap_or_else(|e| {
        error!("Error parsing json: {:?}", e);
        Value::Null
    });

    (headers, json)
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