use crate::chat::build_openai_client;
use log::{error, info};
use serenity::gateway::ActivityData;
use serenity::prelude::*;
use serenity::{builder::CreateMessage, model::channel::Message};

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub struct Handler {
    history: Arc<Mutex<Vec<String>>>,
    client: reqwest::Client,
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

impl Handler {
    pub fn new() -> Self {
        let client = build_openai_client().expect("Failed to build OpenAI client");

        Self {
            client,
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn _get_history(&self) -> Vec<String> {
        self.history.lock().unwrap().clone()
    }

    pub fn add_history(&self, msg: &str) {
        if let Ok(mut history) = self.history.lock() {
            history.push(msg.to_string());
        } else {
            error!("Failed to acquire lock for history");
        }
    }

    pub fn _clear_history(&self) {
        info!("Clearing history");

        self.history.lock().unwrap().clear();
    }

    pub async fn gen_res(&self, ctx: &Context, msg: &Message) {
        let model = std::env::var("MODEL").expect("MODEL not set");
        let content = msg.content.as_str();

        let res = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&ChatRequest {
                model,
                messages: vec![
                    ChatMessage {
                    role: "system".to_string(),
                    content: "You are a 27 year old human named 'Adam' who likes cars, anime, plays a lot of video games, and plays Ramattra in Overwatch 2.
                    You also like to quack occasionally, and you normally have short replies.
                    You work as a lab technician with Richard.".to_string(),
                },
                    ChatMessage {
                    role: "user".to_string(),
                    content: content.to_string(),
                }
                ],
            })
            .send()
            .await
            .unwrap();

        let data = res.json::<serde_json::Value>().await.unwrap();
        let text = data["choices"][0]["message"]["content"].as_str().unwrap();

        self.send_msg(&ctx, &msg, text).await;
    }

    pub async fn send_msg(&self, ctx: &Context, msg: &Message, res: &str) {
        info!("Sending message: {}", res);

        self.add_history(res);

        if let Err(e) = msg.channel_id.say(&ctx, res).await {
            error!("Failed to send message: {}", e);
        }
    }

    pub async fn send_dm(&self, ctx: &Context, msg: &Message, res: impl AsRef<str>) {
        let res = res.as_ref();

        info!("Sending DM: {}", res);

        self.add_history(res);

        msg.author
            .direct_message(ctx, CreateMessage::new().content(res))
            .await
            .unwrap();
    }

    pub async fn join_channel(&self, ctx: &Context, msg: &Message) {
        if msg.guild_id.is_none() {
            self.send_msg(&ctx, &msg, "no").await;
            return;
        }

        info!("Joining voice channel");

        let (guild_id, channel_id) = {
            let guild = msg.guild(&ctx.cache).unwrap();
            let channel_id = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);
            (guild.id, channel_id)
        };

        if let Some(channel_id) = channel_id {
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

        info!("Leaving voice channel");

        ctx.set_activity(None);

        let guild_id = msg.guild_id.unwrap();
        let manager = songbird::get(&ctx).await.unwrap().clone();

        if manager.get(guild_id).is_some() {
            manager.remove(guild_id).await.unwrap();
        }
    }
}
