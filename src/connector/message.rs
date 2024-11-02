use crate::connector::utils::{
    extract_headers_and_data_from_binary_message, extract_headers_and_data_from_text_message,
    make_binary_payload, make_text_payload,
};

/// Data type for message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    Binary(Option<Vec<u8>>),
    Text(Option<String>),
}

/// Headers of the message.
pub type Headers = Vec<(String, String)>;
pub static REQUEST_ID_HEADER: &str = "X-RequestId";
pub static STREAM_ID_HEADER: &str = "X-StreamId";
pub static PATH_HEADER: &str = "Path";

/// Azure Speech service message struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: String,
    pub path: String,
    pub headers: Headers,
    pub data: Data,
}

impl Message {
    pub(crate) fn new(id: String, path: String, headers: Headers, data: Data) -> Self {
        Self {
            id,
            path: path.to_lowercase(),
            headers,
            data,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_header<I: Into<String> + Eq>(&self, key: I) -> Option<String> {
        let key = key.into();
        self.headers
            .iter()
            .find(|(k, _)| k == &key)
            .map(|(_, v)| v.clone())
    }

    pub(crate) fn from_headers_and_data(mut headers: Headers, data: Data) -> Self {
        Self::new(
            extract_header(&mut headers, REQUEST_ID_HEADER),
            extract_header(&mut headers, PATH_HEADER),
            headers,
            data,
        )
    }
}

impl From<Message> for tokio_websockets::Message {
    fn from(message: Message) -> Self {
        let headers = vec![
            (REQUEST_ID_HEADER.to_string(), message.id),
            (PATH_HEADER.to_string(), message.path),
        ];

        let headers = headers.into_iter().chain(message.headers).collect();

        match message.data {
            Data::Binary(data) => {
                tokio_websockets::Message::binary(make_binary_payload(headers, data.as_deref()))
            }
            Data::Text(data) => {
                tokio_websockets::Message::text(make_text_payload(headers, data.as_deref()))
            }
        }
    }
}

impl TryFrom<&str> for Message {
    type Error = crate::Error;
    fn try_from(value: &str) -> crate::Result<Self> {
        let (headers, text) = extract_headers_and_data_from_text_message(value)?;
        Ok(Message::from_headers_and_data(headers, Data::Text(text)))
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = crate::Error;
    fn try_from(value: &[u8]) -> crate::Result<Self> {
        let (headers, data) = extract_headers_and_data_from_binary_message(value)?;
        Ok(Message::from_headers_and_data(headers, Data::Binary(data)))
    }
}

impl TryFrom<tokio_websockets::Message> for Message {
    type Error = crate::Error;
    fn try_from(value: tokio_websockets::Message) -> crate::Result<Self> {
        if value.is_text() {
            return Message::try_from(value.as_text().unwrap());
        }
        if value.is_binary() {
            return Message::try_from(value.as_payload().to_vec().as_slice());
        }

        Err(crate::Error::InternalError(
            "Cannot convert message to binary".into(),
        ))
    }
}

pub(crate) fn extract_header(headers: &mut Headers, header_name: &str) -> String {
    match headers.iter().position(|(k, _)| k == header_name) {
        Some(index) => headers.remove(index).1,
        None => Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_to_twsmessage() {
        let message = Message::new(
            "id".to_string(),
            "path".to_string(),
            vec![("header".to_string(), "value".to_string())],
            Data::Text(Some("data".to_string())),
        );

        let ezmessage: tokio_websockets::Message = message.clone().into();
        let headers = vec![
            (REQUEST_ID_HEADER.to_string(), "id".to_string()),
            (PATH_HEADER.to_string(), "path".to_string()),
            ("header".to_string(), "value".to_string()),
        ];

        match ezmessage.as_text() {
            Some(text) => {
                let text_from_message = make_text_payload(headers, Some("data"));
                assert_eq!(text, text_from_message);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn test_string_to_message() {
        let text = make_text_payload(
            vec![
                (REQUEST_ID_HEADER.to_string(), "id".to_string()),
                (PATH_HEADER.to_string(), "path".to_string()),
                ("header".to_string(), "value".to_string()),
            ],
            Some("data"),
        );

        let message = Message::try_from(text.as_str()).unwrap();
        assert_eq!(
            message,
            Message::new(
                "id".to_string(),
                "path".to_string(),
                vec![("header".to_string(), "value".to_string())],
                Data::Text(Some("data".to_string()))
            )
        );
    }

    #[test]
    fn test_binary_to_message() {
        let data = make_binary_payload(
            vec![
                (REQUEST_ID_HEADER.to_string(), "id".to_string()),
                (PATH_HEADER.to_string(), "path".to_string()),
                ("header".to_string(), "value".to_string()),
            ],
            Some("data".as_bytes()),
        );

        let message = Message::try_from(data.as_slice()).unwrap();
        assert_eq!(
            message,
            Message::new(
                "id".to_string(),
                "path".to_string(),
                vec![("header".to_string(), "value".to_string())],
                Data::Binary(Some("data".as_bytes().to_vec()))
            )
        );
    }

    #[test]
    fn test_binary_to_message_binary_no_data() {
        let message = make_binary_payload(
            vec![
                (REQUEST_ID_HEADER.to_string(), "id".to_string()),
                (PATH_HEADER.to_string(), "path".to_string()),
                ("header".to_string(), "value".to_string()),
            ],
            None,
        );

        let message = Message::try_from(message.as_slice()).unwrap();
        assert_eq!(
            message,
            Message::new(
                "id".to_string(),
                "path".to_string(),
                vec![("header".to_string(), "value".to_string())],
                Data::Binary(None)
            )
        );
    }
}
