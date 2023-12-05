use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};

pub fn build_openai_client() -> Result<reqwest::Client, ()> {
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let mut default_client_headers = HeaderMap::new();

    default_client_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    default_client_headers.insert(
        AUTHORIZATION,
        format!("Bearer {openai_api_key}").parse().unwrap(),
    );

    let res = reqwest::Client::builder()
        .default_headers(default_client_headers)
        .build();

    res.ok().ok_or(())
}
