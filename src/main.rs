extern crate dotenv;

mod bot;
mod cfg;
mod chat;
mod logging;

use bot::Bot;
use cfg::{ADAM_ID, BOT_ID};
use dotenv::dotenv;
use log::{error, info};
use logging::setup_logging;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::{async_trait, Client};
use songbird::SerenityInit;
use std::env;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == BOT_ID {
            return;
        }

        let mentioned = msg.mentions_me(&ctx.http).await.unwrap_or(false);
        if mentioned {
            self.send_msg(&ctx, &msg, "?").await;
        }

        let content = msg.content.as_str().to_lowercase();

        let mentioned = content.contains("adam");
        let dm = msg.is_private();
        let reply = if let Some(last) = self.get_last_2_msgs() {
            last.0.author == msg.author.name && last.1.author == "bot"
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
            self.leave_channel(&ctx, &msg).await;
            self.send_msg(&ctx, &msg, "fine then").await;
        } else {
            self.gen_msg(&ctx, &msg).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        dotenv().ok();
    }

    setup_logging();

    let token = env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' not found");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Bot::new())
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(error) = client.start().await {
        error!("Error occurred while running client: {:?}", error)
    }
}
