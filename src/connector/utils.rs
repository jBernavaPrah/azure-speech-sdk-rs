static CRLF: &str = "\r\n";
static HEADER_JSON_SEPARATOR: &str = "\r\n\r\n";

pub fn make_text_payload(headers: Vec<(String, String)>, data: Option<String>) -> String {
    let headers = transform_headers_to_string(headers);
    let data = data.map_or(String::new(), |d| d);

    format!("{}{CRLF}{}", headers, data)
}

pub fn make_binary_payload(headers: Vec<(String, String)>, data: Option<Vec<u8>>) -> Vec<u8> {
    let headers = transform_headers_to_string(headers);

    let data_length = if let Some(ref d) = data { d.len() } else { 0 };

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

pub fn extract_headers_and_data_from_binary_message(
    data: Vec<u8>,
) -> Result<(Vec<(String, String)>, Option<Vec<u8>>), crate::Error> {
    let header_length = ((data[0] as usize) << 8) + data[1] as usize;
    let headers = String::from_utf8(data[2..2 + header_length].to_vec())
        .map_err(|_| crate::Error::ParseError("Error parsing headers".to_string()))?;
    let data = if header_length + 2 < data.len() {
        Some(data[2 + header_length..].to_vec())
    } else {
        None
    };

    Ok((explode_headers_message(headers.as_str()), data))
}

pub fn extract_headers_and_data_from_text_message(
    text: String,
) -> Result<(Vec<(String, String)>, Option<String>), crate::Error> {
    let mut split_response = text.split(HEADER_JSON_SEPARATOR);

    let headers = explode_headers_message(split_response.next().unwrap_or_default());

    Ok((headers, split_response.next().map(|x| x.to_string())))
}

fn transform_headers_to_string(map: Vec<(String, String)>) -> String {
    let mut headers = String::new();
    for (content_type, value) in map {
        headers.push_str(format!("{content_type}:{value}{CRLF}").as_str());
    }

    headers
}

// Example of message received:
// X-RequestId:5FF045681350489AAF1CD740EE5ACDDD
// Path:turn.start
// Content-Type:application/json; charset=utf-8
fn explode_headers_message(headers: &str) -> Vec<(String, String)> {
    headers
        .split(CRLF)
        .map(|x| {
            let mut split = x.split(":");
            (
                split.next().unwrap_or("").to_string(),
                split.next().unwrap_or("").to_string(),
            )
        })
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explode_message() {
        let text = "X-RequestId:91067ed0-bd0d-4682-891f-446a95996c19\r\nContent-Type:application/json; charset=utf-8\r\nPath:audio.metadata\r\n\r\n{\"Metadata\": [{\"Type\": \"SessionEnd\",\"Data\": {\"Offset\": 11250000}}]}";
        let result = extract_headers_and_data_from_text_message(text.to_string());
        match result {
            Ok((headers, data)) => {
                assert_eq!(
                    headers,
                    vec![
                        (
                            "X-RequestId".to_string(),
                            "91067ed0-bd0d-4682-891f-446a95996c19".to_string()
                        ),
                        (
                            "Content-Type".to_string(),
                            "application/json; charset=utf-8".to_string()
                        ),
                        ("Path".to_string(), "audio.metadata".to_string()),
                    ]
                );
                assert_eq!(data, Some("{\"Metadata\": [{\"Type\": \"SessionEnd\",\"Data\": {\"Offset\": 11250000}}]}".to_string()));
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        }
    }

    #[test]
    fn explode_headers_message_returns_correct_pairs_for_valid_input() {
        let headers = "X-RequestId:5FF045681350489AAF1CD740EE5ACDDD\r\nPath:turn.start\r\nContent-Type:application/json; charset=utf-8";
        let result = explode_headers_message(headers);
        assert_eq!(
            result,
            vec![
                (
                    "X-RequestId".to_string(),
                    "5FF045681350489AAF1CD740EE5ACDDD".to_string()
                ),
                ("Path".to_string(), "turn.start".to_string()),
                (
                    "Content-Type".to_string(),
                    "application/json; charset=utf-8".to_string()
                ),
            ]
        );
    }

    #[test]
    fn explode_headers_message_returns_empty_vector_for_empty_input() {
        let headers = "";
        let result = explode_headers_message(headers);
        assert_eq!(result, Vec::<(String, String)>::new());
    }

    #[test]
    fn explode_headers_message_ignores_lines_without_colon() {
        let headers =
            "X-RequestId:5FF045681350489AAF1CD740EE5ACDDD\r\nInvalidLine\r\nPath:turn.start";
        let result = explode_headers_message(headers);
        assert_eq!(
            result,
            vec![
                (
                    "X-RequestId".to_string(),
                    "5FF045681350489AAF1CD740EE5ACDDD".to_string()
                ),
                ("InvalidLine".to_string(), "".to_string()),
                ("Path".to_string(), "turn.start".to_string()),
            ]
        );
    }

    #[test]
    fn explode_headers_message_handles_lines_with_multiple_colons() {
        let headers = "X-RequestId:5FF045681350489AAF1CD740EE5ACDDD\r\nPath:turn.start\r\nContent-Type:application/json; charset=utf-8\r\nMulti:Part:Header";
        let result = explode_headers_message(headers);
        assert_eq!(
            result,
            vec![
                (
                    "X-RequestId".to_string(),
                    "5FF045681350489AAF1CD740EE5ACDDD".to_string()
                ),
                ("Path".to_string(), "turn.start".to_string()),
                (
                    "Content-Type".to_string(),
                    "application/json; charset=utf-8".to_string()
                ),
                ("Multi".to_string(), "Part".to_string()),
            ]
        );
    }
}
