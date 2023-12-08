use crate::cfg::SYS_PROMPT;
use crate::chat::build_openai_client;
use log::{error, info};
use reqwest::Error;
use serde::{Deserialize, Serialize};
use serenity::builder::CreateMessage;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SavedMessage {
    pub author: String,
    pub content: String,
}

pub type History = Vec<SavedMessage>;

#[derive(Debug)]
pub struct Bot {
    history: Arc<Mutex<History>>,
    client: reqwest::Client,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
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

    pub fn _get_history(&self) -> History {
        self.history.lock().unwrap().clone()
    }

    pub fn add_history(&self, author_id: &str, msg: &str) {
        if let Ok(mut history) = self.history.lock() {
            history.push(SavedMessage {
                author: author_id.to_string(),
                content: msg.to_string(),
            });
        } else {
            error!("Failed to acquire lock for history");
        }
    }

    pub fn _clear_history(&self) {
        info!("Clearing history");

        self.history.lock().unwrap().clear();
    }

    pub fn get_last_2_msgs(&self) -> Option<(SavedMessage, SavedMessage)> {
        if let Ok(history) = self.history.lock() {
            if history.len() > 1 {
                return Some((
                    history[history.len() - 2].clone(),
                    history[history.len() - 1].clone(),
                ));
            }
        }

        None
    }

    pub async fn gen_msg(&self, ctx: &Context, msg: &Message) {
        let text = self.gen_with_prompt(&msg, SYS_PROMPT).await;

        if let Ok(text) = text {
            self.send_msg(&ctx, &msg, &text).await;
        }
    }

    pub async fn gen_adam_dm(&self, ctx: &Context, msg: &Message) {
        let prompt = format!("{} You are currently being messaged by yourself, reply with snarky out of pocket responses.", SYS_PROMPT);
        let text = self.gen_with_prompt(&msg, &prompt).await;

        if let Ok(text) = text {
            self.send_msg(&ctx, &msg, &text).await;
        }
    }

    pub async fn gen_with_prompt(&self, msg: &Message, sys_prompt: &str) -> Result<String, Error> {
        let res = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&ChatRequest {
                model: self.model.clone(),
                messages: vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: sys_prompt.to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    },
                ],
            })
            .send()
            .await?;

        let data = res.json::<serde_json::Value>().await?;
        let text = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("idk")
            .to_string();

        Ok(text)
    }

    pub async fn handle_msg(&self, msg: &Message, res: &str) {
        info!("{}: {}", msg.author.name, msg.content);
        info!("{}: {}", "bot", res);

        self.add_history(&msg.author.name, &msg.content);
        self.add_history("bot", &res);
    }

    pub async fn send_msg(&self, ctx: &Context, msg: &Message, res: &str) {
        self.handle_msg(&msg, &res).await;

        if let Err(e) = msg.channel_id.say(&ctx, res).await {
            error!("Failed to send message: {}", e);
        }
    }

    pub async fn _send_dm(&self, ctx: &Context, msg: &Message, res: &str) {
        self.handle_msg(&msg, &res).await;

        if let Err(e) = msg
            .author
            .direct_message(ctx, CreateMessage::new().content(res))
            .await
        {
            error!("Failed to send DM: {}", e);
        }
    }

    pub async fn join_channel(&self, ctx: &Context, msg: &Message) {
        if msg.guild_id.is_none() {
            self.send_msg(&ctx, &msg, "no").await;
            return;
        }

        let (guild_id, channel_id) = {
            let guild = msg.guild(&ctx.cache).unwrap();
            let channel_id = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);
            (guild.id, channel_id)
        };

        if let Some(channel_id) = channel_id {
            info!("Joining voice channel");

            ctx.set_activity(Some(ActivityData::listening("richard's music")));

            let manager = songbird::get(&ctx).await.unwrap().clone();
            manager.join(guild_id, channel_id).await.unwrap();
        }
    }

    pub async fn leave_channel(&self, ctx: &Context, msg: &Message) {
        if msg.guild_id.is_none() {
            self.send_msg(&ctx, &msg, "no").await;
            return;
        }

        ctx.set_activity(None);

        let guild_id = msg.guild_id.unwrap();
        let manager = songbird::get(&ctx).await.unwrap().clone();

        if manager.get(guild_id).is_some() {
            info!("Leaving voice channel");

            manager.remove(guild_id).await.unwrap();
        }
    }
}
