mod constants;
mod handler;

use constants::*;
use handler::*;

use rand::{thread_rng, Rng};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use songbird::SerenityInit;
use std::env;

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

        let content_og = msg.content.as_str();
        let content = content_og.to_lowercase();
        let mentioned = content.contains("adam");

        if msg.author.id != ADAM_ID && msg.mentions.len() > 0 {
            return;
        } else if msg.author.id == ADAM_ID {
            if content == "?" {
                self.send_msg(&ctx, &msg, "?").await;
            } else {
                if msg.guild_id.is_none() {
                    println!("Adam: {}", content_og);
                    let res = ADAM_MESSAGES[thread_rng().gen_range(0..10)];
                    self.send_msg(&ctx, &msg, res).await;
                } else if msg.attachments.len() > 0 {
                    let res = "look";
                    self.send_msg(&ctx, &msg, res).await;
                } else {
                    let res = format!("{} dattebayo", content_og);
                    msg.delete(&ctx.http).await.unwrap();
                    self.send_msg(&ctx, &msg, res).await;
                }
            }
        } else if msg.attachments.len() > 0 {
            let res = "pikchur";
            self.send_msg(&ctx, &msg, res).await;
        } else if mentioned {
            if content.contains("you") || content.contains("u") || content.contains("can") {
                let res = "NO";
                self.send_msg(&ctx, &msg, res).await;
            } else {
                let res = "QUACK!";
                self.send_msg(&ctx, &msg, res).await;
            }
        } else if content.contains("join") {
            self.join_channel(&ctx, &msg).await;
        } else if content.contains("leave") {
            self.leave_channel(&ctx, &msg).await;
            let res = "fine then";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("explain") {
            msg.channel_id
                .say(&ctx, "what do you meeeeeeean")
                .await
                .unwrap();
        } else if content.contains("anime")
            || content.contains("kimono")
            || content.contains("japan")
        {
            let res = WEEB_MESSAGES[thread_rng().gen_range(0..7)];
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("work") {
            let res = "work chan uwu";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("car") {
            let res = "guys, i am more than just a car guy";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("food") {
            let res = "you can eat cars";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("fat") {
            let res = "...";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("thank") {
            let res = "your WELcome";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("why") {
            let res = "cuz the lord of cars said so";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("sure") {
            let res = "absolutely";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("hate") {
            let res = "i hate myself too";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("night") {
            let res = "gooodniiiight";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("fite") || content.contains("fight") {
            let res = "whAT do U wANT FrOM mE";
            self.send_dm(&ctx, &msg, res).await;
        } else if content_og.contains("ADAM") {
            let res = "WHAT";
            self.send_msg(&ctx, &msg, res).await;
        } else if content_og.contains("fact") || content_og.contains("trivia") {
            let rand_idx = thread_rng().gen_range(0..100);
            let res = CAR_FACTS[rand_idx % CAR_FACTS.len()];
            self.send_msg(&ctx, &msg, res).await;
        } else {
            let res = "car";
            self.send_msg(&ctx, &msg, res).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' not found");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler::new())
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(error) = client.start().await {
        println!("Error occurred while running client: {:?}", error)
    }
}
