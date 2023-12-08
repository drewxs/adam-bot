use log::{error, info};
use reqwest::Client as HttpClient;
use serenity::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use songbird::input::YoutubeDl;
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use std::time::Duration;

use crate::state::HttpKey;

struct SongFader {}

#[async_trait]
impl VoiceEventHandler for SongFader {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(state, track)]) = ctx {
            let _ = track.set_volume(state.volume / 2.0);

            if state.volume < 1e-2 {
                let _ = track.stop();
                Some(Event::Cancel)
            } else {
                None
            }
        } else {
            None
        }
    }
}

struct SongEndNotifier {}

#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        None
    }
}

#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = args.single::<String>()?;

    if !url.starts_with("http") {
        error!("Invalid URL: {}", url);
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Http client not found")
    };

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        info!("Attempting to play: {}", url);

        let src = YoutubeDl::new(http_client, url);

        let _ = handler.play_input(src.into()).set_volume(0.5);

        info!("Playing song");
    } else {
        error!("Not in a voice channel");
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn play_fade(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = args.single::<String>()?;

    if !url.starts_with("http") {
        error!("Invalid URL: {}", url);
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    let http_client = get_http_client(ctx).await;

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let src = YoutubeDl::new(http_client, url);
        let song = handler.play_input(src.into());

        // Periodically make a track quieter until it can be no longer heard.
        let _ = song.add_event(
            Event::Periodic(Duration::from_secs(5), Some(Duration::from_secs(7))),
            SongFader {},
        );

        // Fire an event once an audio track completes,
        // either due to hitting the end of the bytestream or stopped by user code.
        let _ = song.add_event(Event::Track(TrackEvent::End), SongEndNotifier {});
    } else {
        error!("Not in a voice channel");
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = args.single::<String>()?;

    if !url.starts_with("http") {
        error!("Invalid URL: {}", url);
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    let http_client = get_http_client(ctx).await;

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        info!("Queueing {}", url);

        // Use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let src = YoutubeDl::new(http_client, url);

        handler.enqueue_input(src.into()).await;

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

        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        queue.stop();

        let _ = msg.channel_id.say(&ctx.http, "Queue cleared.").await;
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
