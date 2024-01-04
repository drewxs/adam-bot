use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

use crate::history::History;
use crate::openai::build_json_client;

#[derive(Clone, Debug)]
pub struct Bot {
    pub history: Arc<Mutex<History>>,
    pub client: reqwest::Client,
    pub model: String,
    pub user_limits: Arc<Mutex<HashMap<u64, (u64, u64)>>>,
}

impl Bot {
    pub fn new() -> Self {
        let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let model = env::var("MODEL").expect("MODEL not set");

        let client = build_json_client(&openai_api_key).expect("Failed to build OpenAI client");

        Self {
            history: Arc::new(Mutex::new(Vec::new())),
            client,
            model,
            user_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
