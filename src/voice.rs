use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{env, fs};

use anyhow::Error;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use hound::{SampleFormat, WavSpec, WavWriter};
use log::info;
use reqwest::multipart::{Form, Part};
use serenity::all::GuildId;
use serenity::async_trait;
use serenity::client::Context;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use songbird::input::codecs::{CODEC_REGISTRY, PROBE};
use songbird::input::Input;
use songbird::model::id::UserId;
use songbird::model::payload::{ClientDisconnect, Speaking};
use songbird::{CoreEvent, Event, EventContext as Ctx, EventHandler};

use crate::bot::Bot;
use crate::cfg::SYS_PROMPT;
use crate::music::find_song;
use crate::openai::{
    build_json_client, build_multipart_client, ChatMessage, ChatRequest, SpeechRequest,
    OPENAI_API_URL,
};

#[derive(Clone)]
struct Receiver {
    ctx: Context,
    guild_id: GuildId,
    chat_model: String,
    json_client: reqwest::Client,
    multipart_client: reqwest::Client,
    controller: Arc<VoiceController>,
}

struct VoiceReply {
    timestamp: DateTime<Utc>,
    duration: Duration,
}

struct VoiceController {
    last_tick_was_empty: AtomicBool,
    known_ssrcs: DashMap<u32, UserId>,
    accumulator: DashMap<u32, Slice>,
    last_reply: Mutex<Option<VoiceReply>>,
}

struct Slice {
    user_id: u64,
    bytes: Vec<i16>,
    timestamp: DateTime<Utc>,
}

impl Receiver {
    pub fn new(ctx: Context, guild_id: GuildId) -> Self {
        let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let chat_model = env::var("MODEL").expect("MODEL not set");
        let json_client = build_json_client(&openai_api_key).unwrap();
        let multipart_client = build_multipart_client(&openai_api_key).unwrap();

        Self {
            ctx,
            guild_id,
            chat_model,
            json_client,
            multipart_client,
            controller: Arc::new(VoiceController {
                last_tick_was_empty: AtomicBool::default(),
                known_ssrcs: DashMap::new(),
                accumulator: DashMap::new(),
                last_reply: Mutex::new(None),
            }),
        }
    }

    async fn process(&self, slice: &mut Slice) -> Result<(), Error> {
        if let Ok(mut last_reply) = self.controller.last_reply.lock() {
            if let Some(reply) = last_reply.take() {
                let elapsed = Utc::now() - reply.timestamp;
                let remaining = reply.duration - elapsed;

                if remaining > Duration::milliseconds(0) {
                    slice.timestamp = Utc::now();
                    slice.bytes.clear();

                    return Ok(());
                }
            }
        }

        let filename = format!("cache/{}_{}.wav", slice.user_id, Utc::now().timestamp());

        self.save(&slice.bytes, &filename);

        slice.timestamp = Utc::now();
        slice.bytes.clear();

        if let Ok(text) = self.transcribe(&filename).await {
            let text: String = text
                .chars()
                .filter(|&c| c != ',' && c != '.' && c != '!')
                .collect();

            match text.to_lowercase().as_str() {
                t if t.starts_with("play") => {
                    let search = t.trim_start_matches("play").trim();

                    info!("Searching for {}", search);

                    let guild_id = self.guild_id;
                    let manager = songbird::get(&self.ctx).await.unwrap().clone();

                    if let Some(handler_lock) = manager.get(guild_id) {
                        let mut handler = handler_lock.lock().await;

                        let (youtube_dl, url) = find_song(&self.ctx, search).await?;

                        info!("Queueing {}", url);

                        let (input, _) =
                            self.gen_audio(&format!("Queueing up, {}", search)).await?;
                        let _ = handler.play_input(input).set_volume(0.5);

                        let handle = handler.enqueue_input(youtube_dl.into()).await;
                        let _ = handle.set_volume(0.05);
                    }
                }
                "stop" => {
                    let guild_id = self.guild_id;
                    let manager = songbird::get(&self.ctx).await.unwrap().clone();

                    if let Some(handler_lock) = manager.get(guild_id) {
                        let mut handler = handler_lock.lock().await;
                        let _ = handler.stop();

                        let queue = handler.queue();
                        queue.stop();

                        let (input, _) = self
                            .gen_audio("Just say the word and I'll be back to play some tunes")
                            .await?;
                        let _ = handler.play_input(input).set_volume(0.5);
                    }
                }
                t if ["adam", "and", "i don't know"]
                    .iter()
                    .any(|s| t.to_lowercase().contains(s)) =>
                {
                    let text = text.replace("adam", "");
                    let res = self.gen_response(&text).await?;
                    let (input, duration) = self.gen_audio(&res).await?;
                    self.play_audio(input, duration).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn save(&self, pcm_samples: &[i16], filename: &str) {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let _ = fs::create_dir_all("cache");
        let mut writer = WavWriter::create(filename, spec).unwrap();

        for &sample in pcm_samples {
            let _ = writer.write_sample(sample);
        }

        let _ = writer.finalize();
    }

    async fn transcribe(&self, filename: &str) -> Result<String, Error> {
        let file = fs::read(&filename)?;
        let form = Form::new()
            .part(
                "file",
                Part::bytes(file)
                    .file_name(filename.to_string())
                    .mime_str("audio/wav")
                    .unwrap(),
            )
            .part("model", Part::text("whisper-1"));

        let res = self
            .multipart_client
            .post(format!("{OPENAI_API_URL}/audio/transcriptions"))
            .multipart(form)
            .send()
            .await?;

        let data = res.json::<serde_json::Value>().await?;
        if let Some(text) = data["text"].as_str() {
            info!("Transcription: {:?}", text);
            return Ok(text.to_string());
        }

        Err(Error::msg("Failed to transcribe audio"))
    }

    async fn gen_response(&self, text: &str) -> Result<String, Error> {
        let data = self
            .json_client
            .post(format!("{OPENAI_API_URL}/chat/completions"))
            .json(&ChatRequest {
                model: self.chat_model.clone(),
                messages: vec![
                    ChatMessage::new("system", &SYS_PROMPT),
                    ChatMessage::new("user", &text),
                ],
            })
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let res = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("idk")
            .to_string();

        info!("Response: {:?}", res);

        Ok(res)
    }

    async fn gen_audio(&self, text: &str) -> Result<(Input, u64), Error> {
        let res = self
            .json_client
            .post(format!("{OPENAI_API_URL}/audio/speech"))
            .json(&SpeechRequest {
                model: "tts-1".to_string(),
                input: text.to_string(),
                voice: "onyx".to_string(),
            })
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(Error::msg("Failed to generate audio"));
        }

        let bytes = res.bytes().await?;

        let mut input: Input = bytes.clone().into();
        input = input.make_playable_async(&CODEC_REGISTRY, &PROBE).await?;

        let duration = (bytes.len() / 48) as u64;

        if !input.is_playable() {
            return Err(Error::msg("Generated audio is not playable"));
        }

        Ok((input, duration))
    }

    async fn play_audio(&self, input: Input, duration: u64) -> Result<(), Error> {
        let manager = songbird::get(&self.ctx).await.unwrap();

        if let Some(handler_lock) = manager.get(self.guild_id.clone()) {
            let mut handler = handler_lock.lock().await;
            let _ = handler.play_input(input).set_volume(0.5);

            if let Ok(mut last_reply) = self.controller.last_reply.lock() {
                *last_reply = Some(VoiceReply {
                    timestamp: Utc::now(),
                    duration: Duration::milliseconds(duration as i64),
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Receiver {
    async fn act(&self, ctx: &Ctx<'_>) -> Option<Event> {
        match ctx {
            Ctx::SpeakingStateUpdate(Speaking {
                speaking: _,
                ssrc,
                user_id: Some(user_id),
                ..
            }) => {
                info!("{:?} speaking", ssrc);

                self.controller.known_ssrcs.insert(*ssrc, *user_id);

                self.controller.accumulator.entry(*ssrc).or_insert(Slice {
                    user_id: user_id.0,
                    bytes: Vec::new(),
                    timestamp: Utc::now(),
                });
            }
            Ctx::VoiceTick(tick) => {
                let speaking = tick.speaking.len();
                let last_tick_was_empty =
                    self.controller.last_tick_was_empty.load(Ordering::SeqCst);

                if speaking == 0 && !last_tick_was_empty {
                    self.controller
                        .last_tick_was_empty
                        .store(true, Ordering::SeqCst);

                    for mut slice in self.controller.accumulator.iter_mut() {
                        if slice.bytes.is_empty() {
                            continue;
                        }
                        if let Err(e) = self.process(&mut slice).await {
                            info!("Processing error: {:?}", e);
                        }
                    }
                } else if speaking != 0 {
                    self.controller
                        .last_tick_was_empty
                        .store(false, Ordering::SeqCst);

                    for (ssrc, data) in &tick.speaking {
                        if let Some(decoded_voice) = data.decoded_voice.as_ref() {
                            let mut bytes = decoded_voice.to_owned();

                            if let Some(mut slice) = self.controller.accumulator.get_mut(&ssrc) {
                                slice.bytes.append(&mut bytes);
                            } else if let Some(user_id) = self.controller.known_ssrcs.get(ssrc) {
                                self.controller.accumulator.insert(
                                    *ssrc,
                                    Slice {
                                        user_id: user_id.0,
                                        bytes,
                                        timestamp: Utc::now(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
            Ctx::ClientDisconnect(ClientDisconnect { user_id, .. }) => {
                info!("{:?} disconnected", user_id);
            }
            _ => {}
        }

        None
    }
}

impl Bot {
    pub async fn join_channel(&self, ctx: &Context, msg: &Message) {
        if msg.guild_id.is_none() {
            self.send_msg(&ctx, &msg, "no").await;
            return;
        }

        let (guild_id, channel_id) = {
            let guild = msg.guild(&ctx.cache).unwrap();
            let channel_id = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);
            (guild.id, channel_id)
        };

        if let Some(channel_id) = channel_id {
            info!("Joining voice channel");

            ctx.set_activity(Some(ActivityData::listening("youtube music")));

            let manager = songbird::get(&ctx).await.unwrap().clone();

            if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
                let mut handler = handler_lock.lock().await;

                let receiver = Receiver::new(ctx.to_owned(), guild_id.into());

                handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), receiver.clone());
                handler.add_global_event(CoreEvent::VoiceTick.into(), receiver.clone());
                handler.add_global_event(CoreEvent::ClientDisconnect.into(), receiver);
            }
        }
    }

    pub async fn leave_channel(&self, ctx: &Context, msg: &Message) {
        if msg.guild_id.is_none() {
            self.send_msg(&ctx, &msg, "no").await;
            return;
        }

        ctx.set_activity(None);

        let guild_id = msg.guild_id.unwrap();
        let manager = songbird::get(&ctx).await.unwrap().clone();

        if manager.get(guild_id).is_some() {
            info!("Leaving voice channel");
            let _ = manager.remove(guild_id).await;
        }

        let _ = fs::remove_dir_all("cache");
    }
}
