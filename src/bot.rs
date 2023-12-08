use std::env;
use std::sync::{Arc, Mutex};

use crate::history::History;
use crate::openai::build_openai_client;

#[derive(Debug)]
pub struct Bot {
    pub history: Arc<Mutex<History>>,
    pub client: reqwest::Client,
    pub model: String,
}

impl Bot {
    pub fn new() -> Self {
        let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let model = env::var("MODEL").expect("MODEL not set");

        let client = build_openai_client(openai_api_key).expect("Failed to build OpenAI client");

        Self {
            history: Arc::new(Mutex::new(Vec::new())),
            client,
            model,
        }
    }
}
