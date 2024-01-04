use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

pub const OPENAI_API_URL: &str = "https://api.openai.com/v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    role: String,
    content: String,
}

impl ChatMessage {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeechRequest {
    pub model: String,
    pub input: String,
    pub voice: String,
}

pub fn build_json_client(api_key: &str) -> Result<Client, Error> {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, format!("Bearer {api_key}").parse().unwrap());

    Client::builder().default_headers(headers).build()
}

pub fn build_multipart_client(api_key: &str) -> Result<Client, Error> {
    let mut headers = HeaderMap::new();

    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("multipart/form-data"),
    );
    headers.insert(AUTHORIZATION, format!("Bearer {api_key}").parse().unwrap());

    Client::builder().default_headers(headers).build()
}
