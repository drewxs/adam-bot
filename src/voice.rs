use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{env, fs};

use anyhow::Error;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use hound::{SampleFormat, WavSpec, WavWriter};
use log::info;
use reqwest::multipart::{Form, Part};
use serenity::async_trait;
use serenity::client::Context;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use songbird::model::id::UserId;
use songbird::model::payload::{ClientDisconnect, Speaking};
use songbird::{CoreEvent, Event, EventContext as Ctx, EventHandler};

use crate::bot::Bot;
use crate::cfg::SYS_PROMPT;
use crate::openai::{
    build_audio_client, build_chat_client, ChatMessage, ChatRequest, OPENAI_API_URL,
};

#[derive(Clone)]
struct Receiver {
    chat_model: String,
    chat_client: reqwest::Client,
    whisper_client: reqwest::Client,
    controller: Arc<VoiceController>,
}

struct VoiceController {
    last_tick_was_empty: AtomicBool,
    known_ssrcs: DashMap<u32, UserId>,
    accumulator: DashMap<u32, Slice>,
}

struct Slice {
    user_id: u64,
    bytes: Vec<i16>,
    #[allow(dead_code)]
    timestamp: DateTime<Utc>,
}

impl Receiver {
    pub fn new() -> Self {
        let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let chat_model = env::var("MODEL").expect("MODEL not set");
        let chat_client = build_chat_client(&openai_api_key).unwrap();
        let whisper_client = build_audio_client(&openai_api_key).unwrap();

        Self {
            chat_model,
            chat_client,
            whisper_client,
            controller: Arc::new(VoiceController {
                last_tick_was_empty: AtomicBool::default(),
                known_ssrcs: DashMap::new(),
                accumulator: DashMap::new(),
            }),
        }
    }

    async fn transcribe_slice(&self, slice: &mut Slice) {
        let filename = format!("cache/{}_{}.wav", slice.user_id, Utc::now().timestamp());

        self.save(&slice.bytes, &filename);
        slice.bytes.clear();

        if let Ok(text) = self.transcribe(&filename).await {
            if let Ok(res) = self.gen_response(&text).await {
                info!("Response: {:?}", res);
            }
        }
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
                    .file_name(filename.to_owned())
                    .mime_str("audio/wav")
                    .unwrap(),
            )
            .part("model", Part::text("whisper-1"));

        let res = self
            .whisper_client
            .post(format!("{OPENAI_API_URL}/audio/transcriptions"))
            .multipart(form)
            .send()
            .await?;

        let data = res.json::<serde_json::Value>().await?;
        if let Some(text) = data["text"].as_str() {
            info!("Transcription: {:?}", text);
            return Ok(text.to_string());
        }

        fs::remove_file(filename)?;

        Err(Error::msg("no text"))
    }

    async fn gen_response(&self, text: &str) -> Result<String, Error> {
        let data = self
            .chat_client
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

        Ok(res)
    }
}

#[async_trait]
impl EventHandler for Receiver {
    async fn act(&self, ctx: &Ctx<'_>) -> Option<Event> {
        match ctx {
            Ctx::SpeakingStateUpdate(Speaking {
                speaking: _,
                ssrc,
                user_id,
                ..
            }) => {
                if let Some(user) = user_id {
                    info!("{:?}: Speaking", ssrc);
                    self.controller.known_ssrcs.insert(*ssrc, *user);

                    match self.controller.accumulator.get(ssrc) {
                        Some(_) => {}
                        None => {
                            self.controller.accumulator.insert(
                                *ssrc,
                                Slice {
                                    user_id: user.0,
                                    bytes: Vec::new(),
                                    timestamp: Utc::now(),
                                },
                            );
                        }
                    }
                }
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
                        if slice.bytes.len() == 0 {
                            continue;
                        }

                        self.transcribe_slice(&mut slice).await;
                    }
                } else if speaking != 0 {
                    self.controller
                        .last_tick_was_empty
                        .store(false, Ordering::SeqCst);

                    for (ssrc, data) in &tick.speaking {
                        if let Some(decoded_voice) = data.decoded_voice.as_ref() {
                            let mut bytes = decoded_voice.to_owned();

                            match self.controller.accumulator.get_mut(&ssrc) {
                                Some(mut slice) => {
                                    slice.bytes.append(&mut bytes);
                                }
                                None => {
                                    let slice = Slice {
                                        user_id: self.controller.known_ssrcs.get(&ssrc).unwrap().0,
                                        bytes,
                                        timestamp: Utc::now(),
                                    };
                                    self.controller.accumulator.insert(*ssrc, slice);
                                }
                            }
                        } else {
                            info!("{}: Decode disabled", ssrc);
                        }
                    }
                }
            }
            Ctx::RtpPacket(_) => {}
            Ctx::RtcpPacket(_) => {}
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

            ctx.set_activity(Some(ActivityData::listening("richard's music")));

            let manager = songbird::get(&ctx).await.unwrap().clone();

            if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
                let mut handler = handler_lock.lock().await;

                let receiver = Receiver::new();

                handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), receiver.clone());
                handler.add_global_event(CoreEvent::RtpPacket.into(), receiver.clone());
                handler.add_global_event(CoreEvent::RtcpPacket.into(), receiver.clone());
                handler.add_global_event(CoreEvent::ClientDisconnect.into(), receiver.clone());
                handler.add_global_event(CoreEvent::VoiceTick.into(), receiver);
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
