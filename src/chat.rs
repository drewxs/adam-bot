use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Error};

pub fn build_openai_client(api_key: String) -> Result<Client, Error> {
    let mut default_client_headers = HeaderMap::new();

    default_client_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    default_client_headers.insert(AUTHORIZATION, format!("Bearer {api_key}").parse().unwrap());

    reqwest::Client::builder()
        .default_headers(default_client_headers)
        .build()
}
