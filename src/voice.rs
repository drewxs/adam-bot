use log::info;
use serenity::client::Context;
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;

use crate::bot::Bot;

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
            let _ = manager.join(guild_id, channel_id).await;
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
    }
}
