mod cfg;
mod handler;
mod logging;

use cfg::*;
use handler::Handler;
use logging::setup_logging;

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
                    let res = [
                        "oh hello there fake adam",
                        "im you",
                        "don't you see?",
                        "i'm you but stronger",
                        "i AM a car",
                        "you will never be a car",
                        "muahahaha",
                        "no one will believe you",
                        "adam listen",
                        "quack",
                    ][thread_rng().gen_range(0..10)];
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
            if ["you", "u", "can"].iter().any(|&s| content.contains(s)) {
                let res = "NO";
                self.send_msg(&ctx, &msg, res).await;
            } else {
                let res = "QUACK!";
                self.send_msg(&ctx, &msg, res).await;
            }
        } else if content.contains("explain") {
            let res = "what do you meeeeeeean";
            self.send_msg(&ctx, &msg, res).await;
        } else if ["anime", "japan", "kimono"]
            .iter()
            .any(|&s| content.contains(s))
        {
            let res = [
                "I LOVE WEEB ROBE",
                "its ramen time",
                "guys they do real jujutsu in jujutsu kaisen",
                "im not watching that",
                "oshiete yo",
                "lads",
                "guys, wanna go japann?",
            ][thread_rng().gen_range(0..7)];
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("work") {
            let res = "work chan uwu";
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("car") {
            let res = [
                "guys, i am more than just a car guy",
                "CARS",
                "caaaars whooo",
            ][thread_rng().gen_range(0..3)];
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
            let res = [
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
][thread_rng().gen_range(0..20)];
            self.send_msg(&ctx, &msg, res).await;
        } else if content.contains("join") {
            self.join_channel(&ctx, &msg).await;
        } else if content.contains("leave") {
            self.leave_channel(&ctx, &msg).await;
            let res = "fine then";
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
    setup_logging();

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
