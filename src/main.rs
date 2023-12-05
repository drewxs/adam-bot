mod cfg;
mod chat;
mod handler;
mod logging;

use cfg::*;
use handler::Handler;
use log::{error, info};
use logging::setup_logging;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use songbird::SerenityInit;

#[async_trait]
impl EventHandler for Handler {
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
        let adam_dm = msg.author.id == ADAM_ID && msg.is_private();

        if !mentioned && !adam_dm {
            return;
        }

        if adam_dm {
            self.gen_adam_dm(&ctx, &msg).await;
        } else if content.contains("fight") {
            self.send_dm(&ctx, &msg, "no").await;
        } else if content.contains("explain") {
            let res = "what do you meeeeeeean";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("thank") {
            let res = "your WELcome";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("hate") {
            let res = "i hate myself too";
            self.send_msg(&ctx, &msg, res).await;
        } else if matches_any(&content, &["night", "bye", "goodnight"]) {
            let res = "gooodniiiight";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("join") {
            self.join_channel(&ctx, &msg).await;
        } else if content.contains("leave") {
            self.leave_channel(&ctx, &msg).await;
            let res = "fine then";
            self.send_msg(&ctx, &msg, res).await;
        } else {
            self.gen_msg(&ctx, &msg).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

fn matches_any(content: &str, strs: &[&str]) -> bool {
    for s in strs {
        if content.contains(s) {
            return true;
        }
    }
    false
}

#[tokio::main]
async fn main() {
    setup_logging();

    let token = std::env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' not found");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler::new())
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(error) = client.start().await {
        error!("Error occurred while running client: {:?}", error)
    }
}
