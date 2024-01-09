use std::env;

use log::{error, info};
use reqwest::Client as HttpClient;
use reqwest::Error;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use songbird::input::YoutubeDl;

use crate::state::HttpKey;

#[command]
#[only_in(guilds)]
pub async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let search = args.message();

    info!("Searching for {}", search);

    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let (youtube_dl, url) = find_song(&ctx, search).await?;

        info!("Queueing {}", url);

        // Use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let handle = handler.enqueue_input(youtube_dl.into()).await;
        let _ = handle.set_volume(0.05);

        let _ = msg
            .channel_id
            .say(
                &ctx.http,
                format!("Added song to queue: position {}", handler.queue().len()),
            )
            .await;
    } else {
        error!("Not in a voice channel");
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    info!("Music: skip");

    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        let _ = msg
            .channel_id
            .say(
                &ctx.http,
                format!("Song skipped: {} in queue.", queue.len()),
            )
            .await;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        info!("Stopping");

        let mut handler = handler_lock.lock().await;
        handler.stop();

        let queue = handler.queue();
        queue.stop();

        let _ = msg.channel_id.say(&ctx.http, "Queue cleared.").await;
    } else {
        error!("Not in a voice channel");
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn vol(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let volume = args.message().parse::<f32>().unwrap() / 100.0;

    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(call) = manager.get(guild_id) {
        call.lock()
            .await
            .queue()
            .current_queue()
            .iter()
            .for_each(|t| {
                let _ = t.set_volume(volume);
            });
    } else {
        error!("Not in a voice channel");
    }

    Ok(())
}

async fn get_http_client(ctx: &Context) -> HttpClient {
    let data = ctx.data.read().await;
    data.get::<HttpKey>()
        .cloned()
        .expect("Http client not found")
}

async fn find_song(ctx: &Context, search: &str) -> Result<(YoutubeDl, String), Error> {
    let client = get_http_client(ctx).await;

    if search.starts_with("https://") {
        let youtube_dl = YoutubeDl::new(client, search.to_string());
        return Ok((youtube_dl, search.to_string()));
    }

    let yt_api_key = env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY not set");

    let search_results = client
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&[
            ("key", yt_api_key.as_str()),
            ("type", "video"),
            ("maxResults", "1"),
            ("q", search),
        ])
        .send()
        .await?;
    let search_results = search_results.json::<serde_json::Value>().await?;

    let video_id = search_results["items"][0]["id"]["videoId"]
        .as_str()
        .expect("No video found")
        .to_string();

    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    let youtube_dl = YoutubeDl::new(client, url.clone());

    Ok((youtube_dl, url))
}
