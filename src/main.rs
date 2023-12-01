use songbird::SerenityInit;

use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;

struct Handler;

const ADAM_ID: u64 = 281207443105644544;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mentioned = msg.mentions_me(&ctx.http).await.unwrap_or(false);
        if mentioned {
            msg.channel_id.say(&ctx.http, "?").await.unwrap();
        }

        let content = msg.content.as_str();
        let mentioned = content.contains("adam");

        if msg.author.id == ADAM_ID {
            let res = "dattebayo";
            msg.channel_id.say(&ctx.http, res).await.unwrap();
        } else if mentioned {
            if content.contains("you") {
                let res = "nO";
                msg.channel_id.say(&ctx.http, res).await.unwrap();
            } else {
                let res = "QUACK!";
                msg.channel_id.say(&ctx.http, res).await.unwrap();
            }
        } else if content.contains("ADAM") {
            let res = "WHAT";
            msg.channel_id.say(&ctx.http, res).await.unwrap();
        } else if content.contains("join") {
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

                let _guild = msg.guild_id.unwrap();
            }
        } else if content.contains("leave") {
            ctx.set_activity(None);

            let guild_id = msg.guild_id.unwrap();
            let manager = songbird::get(&ctx).await.unwrap().clone();

            if manager.get(guild_id).is_some() {
                manager.remove(guild_id).await.unwrap();
            }

            msg.channel_id.say(&ctx.http, "fine then").await.unwrap();
        } else if content.contains("explain") {
            let res = "what do you meeeeeeean";
            msg.channel_id.say(&ctx.http, res).await.unwrap();
        } else if content.contains("fite") || content.contains("fight") {
            let res = "whAT do U wANT FrOM mE";
            msg.author
                .direct_message(&ctx, CreateMessage::new().content(res))
                .await
                .unwrap();
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
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(error) = client.start().await {
        println!("Error occurred while running client: {:?}", error)
    }
}
