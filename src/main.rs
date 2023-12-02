use songbird::SerenityInit;

use rand::{thread_rng, Rng};
use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;

const BOT_ID: u64 = 1179957141688291498;
const ADAM_ID: u64 = 281207443105644544;

const CAR_FACTS: [&str; 20] = [
    "The first recorded car accident occurred in 1891 in Ohio, USA, when a steam-powered vehicle collided with a tree.",
    "The world's fastest production car is the Bugatti Chiron Super Sport 300+, reaching a top speed of 304 mph (490 km/h).",
    "The average car has about 30,000 parts.",
    "The first mass-produced car was the Model T by Ford, introduced in 1908.",
    "The Volkswagen Beetle is one of the best-selling cars in history, with over 21 million units sold.",
    "The first car with windshield wipers was the 1903 Cadillac Model A.",
    "The longest-lasting car model still in production is the Chevrolet Suburban, introduced in 1935.",
    "The Lamborghini company originally manufactured tractors before venturing into sports car production.",
    "The term \"horsepower\" was coined by James Watt, the inventor of the steam engine, to help market his invention.",
    "The world's largest collection of classic cars can be found at the Nethercutt Collection in Sylmar, California.",
    "The first recorded instance of a car race occurred in 1867 between two steam-powered vehicles in France.",
    "The Rolls-Royce Phantom has self-righting wheel centers to keep the brand's logo upright when the car is moving.",
    "The Mercedes-Benz G-Class (G-Wagon) was originally designed as a military vehicle.",
    "The first car radio was introduced by Chevrolet in 1922.",
    "The Porsche 911 is one of the few sports cars that has maintained its rear-engine design since its introduction in 1963.",
    "The average car spends about 95% of its time parked.",
    "The Toyota Corolla is the best-selling car model of all time, with over 44 million units sold.",
    "The first hybrid car was the Toyota Prius, introduced in Japan in 1997.",
    "The most expensive car ever sold at auction is a 1962 Ferrari 250 GTO, which fetched $48.4 million.",
    "The iconic \"Jeep\" name comes from the phonetic pronunciation of \"G.P.,\" which stands for General Purpose or Government Purpose vehicle."
];

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == BOT_ID {
            return;
        }

        let mentioned = msg.mentions_me(&ctx.http).await.unwrap_or(false);
        if mentioned {
            msg.channel_id.say(&ctx.http, "?").await.unwrap();
        }

        let content_og = msg.content.as_str();
        let content = content_og.to_lowercase();
        let mentioned = content.contains("adam");

        if msg.mentions.len() > 0 {
            return;
        } else if msg.author.id == ADAM_ID {
            if content == "?" {
                let res = "?";
                msg.channel_id.say(&ctx, res).await.unwrap();
            } else {
                let edited_msg = format!("{} dattebayo", content_og);
                msg.delete(&ctx.http).await.unwrap();
                msg.channel_id.say(&ctx, edited_msg).await.unwrap();
            }
        } else if mentioned {
            if content.contains("you") {
                let res = "NO";
                msg.channel_id.say(&ctx, res).await.unwrap();
            } else {
                let res = "QUACK!";
                msg.channel_id.say(&ctx, res).await.unwrap();
            }
        } else if content_og.contains("ADAM") {
            let res = "WHAT";
            msg.channel_id.say(&ctx, res).await.unwrap();
        } else if content.contains("join") {
            join_channel(&ctx, &msg).await;
        } else if content.contains("leave") {
            leave_channel(&ctx, &msg).await;
            let res = "fine then";
            msg.channel_id.say(&ctx, res).await.unwrap();
        } else if content.contains("explain") {
            msg.channel_id
                .say(&ctx, "what do you meeeeeeean")
                .await
                .unwrap();
        } else if content.contains("kimono") {
            let res = "I LOVE WEEB ROBE";
            msg.channel_id.say(&ctx, res).await.unwrap();
        } else if content.contains("thank") {
            let res = "your WELcome";
            msg.channel_id.say(&ctx, res).await.unwrap();
        } else if content.contains("night") {
            let res = "gooodniiiight";
            msg.channel_id.say(&ctx, res).await.unwrap();
        } else if content.contains("fite") || content.contains("fight") {
            let res = "whAT do U wANT FrOM mE";
            msg.author
                .direct_message(ctx, CreateMessage::new().content(res))
                .await
                .unwrap();
        } else {
            let rand_idx = thread_rng().gen_range(0..100);
            let res = CAR_FACTS[rand_idx % CAR_FACTS.len()];
            msg.channel_id.say(&ctx, res).await.unwrap();
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

async fn join_channel(ctx: &Context, msg: &Message) {
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
}

async fn leave_channel(ctx: &Context, msg: &Message) {
    ctx.set_activity(None);

    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(&ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await.unwrap();
    }
}
