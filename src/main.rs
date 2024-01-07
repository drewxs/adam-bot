extern crate dotenv;

mod bot;
mod cfg;
mod history;
mod logging;
mod message;
mod music;
mod openai;
mod state;
mod voice;

use std::collections::HashSet;
use std::env;

use chrono::Utc;
use dotenv::dotenv;
use log::{error, info};
use reqwest::Client as HttpClient;
use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::{Configuration, StandardFramework};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use songbird::driver::DecodeMode;
use songbird::SerenityInit;

use crate::bot::Bot;
use crate::cfg::{ADAM_ID, BOT_ID};
use crate::logging::setup_logging;
use crate::music::*;
use crate::state::{HttpKey, ShardManagerContainer};

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == BOT_ID {
            return;
        }

        if let Ok(mut user_limits) = self.user_limits.lock() {
            if let Some((last_time, count)) = user_limits.get(&msg.author.id.into()) {
                let current_time = Utc::now().timestamp();
                if current_time - last_time < 60 && *count >= 10 {
                    info!("Exceeded rate limit: {}", msg.author.name);
                    return;
                }
            }

            let (timestamp, count) = user_limits.entry(msg.author.id.into()).or_insert((0, 0));
            let current_time = Utc::now().timestamp();

            if current_time - *timestamp >= 60 {
                *timestamp = current_time;
                *count = 0;
            }

            *count += 1;
        }

        if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            self.send_msg(&ctx, &msg, "?").await;
        }

        let content = msg.content.as_str().to_lowercase();

        let mentioned = content.contains("adam");
        let dm = msg.is_private();
        let reply = if let Some(last) = self.get_last_2_msgs() {
            last.0.author == msg.author.name && last.1.author == "adam"
        } else {
            false
        };

        if !mentioned && !dm && !reply {
            self.add_history(&msg.author.name, &msg.content);
            return;
        }

        if dm {
            if msg.author.id == ADAM_ID {
                self.gen_adam_dm(&ctx, &msg).await;
            } else {
                self.gen_msg(&ctx, &msg).await;
            }
        } else if content.contains("join") {
            self.join_channel(&ctx, &msg).await;
        } else if content.contains("leave") {
            self.send_msg(&ctx, &msg, "fine then").await;
            self.leave_channel(&ctx, &msg).await;
        } else {
            self.gen_msg(&ctx, &msg).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(play, play_fade, queue, skip, stop)]
struct General;

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        dotenv().ok();
    }

    setup_logging();

    let token = env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' not found");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;
    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }

            (owners, info.id)
        }
        Err(error) => panic!("Could not access application info: {:?}", error),
    };

    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().owners(owners).prefix("~"));

    let yt_client = HttpClient::new();
    let songbird_cfg = songbird::Config::default().decode_mode(DecodeMode::Decode);

    let mut client = Client::builder(token, intents)
        .event_handler(Bot::new())
        .framework(framework)
        .register_songbird_from_config(songbird_cfg)
        .type_map_insert::<HttpKey>(yt_client)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.shutdown_all().await;
    });

    if let Err(error) = client.start().await {
        error!("Client error: {:?}", error)
    }
}
