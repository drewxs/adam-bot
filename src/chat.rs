use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Error};

pub fn build_openai_client(api_key: String) -> Result<Client, Error> {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, format!("Bearer {api_key}").parse().unwrap());

    Client::builder().default_headers(headers).build()
}
