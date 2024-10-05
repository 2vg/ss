use anyhow::Context as _;
use dashmap::DashMap;
use serenity::{all::{ChannelId, EventHandler, GuildId, Interaction, Message, Ready}, async_trait, prelude::*};
use songbird::{input::cached::Memory, tracks::Track};
use std::{ffi::OsString, sync::Arc};

use crate::{commands, utils::get_manager};

pub(crate) struct Handler {
    pub(crate) connections: Arc<DashMap<GuildId, ChannelId>>,
    pub(crate) sounds: Arc<DashMap<OsString, Memory>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        tracing::info!("{} is ready", ready.user.name);

        for guild in ready.guilds {
            let commands = guild
                .id
                .set_commands(
                    &context.http,
                    vec![
                        commands::join::register(),
                        commands::leave::register(),
                    ],
                )
                .await;

            if let Err(error) = commands {
                tracing::error!("failed to regeister slash commands\nError: {error:?}");
            }
        }
    }

    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let result = match command.data.name.as_str() {
                "join" => {
                    let connections = &self.connections;
                    commands::join::run(&context, connections, &command).await
                },
                "leave" => {
                    commands::leave::run(&context, &command).await
                },
                _ => Ok(()),
            }
            .with_context(|| format!("failed to execute /{}", command.data.name));

            if let Err(error) = result {
                tracing::error!("failed to handle slash command\nError: {error:?}");
            }
        }
    }

    async fn message(&self, context: Context, message: Message) {
        if message.author.bot {
            return;
        }

        let Some(guild_id) = message.guild_id else {
            return;
        };

        let manager = match get_manager(&context).await {
            Ok(manager) => manager,
            Err(error) => {
                tracing::error!("{error:?}");
                return;
            },
        };
        let call = manager.get_or_insert(guild_id);
        let mut call = call.lock().await;

        let (Some(_), Some(channel_id_bot_at)) = (call.current_connection(), call.current_channel()) else {
            return;
        };
        let channel_id_bot_at = ChannelId::from(channel_id_bot_at.0);

        let is_voice_channel_bot_at = {
            self.connections
                .get(&guild_id)
                .is_some_and(|channel_id| &message.channel_id == channel_id.value())
        };
        let is_text_channel_binded_to_bot = message.channel_id == channel_id_bot_at;

        if !is_voice_channel_bot_at && !is_text_channel_binded_to_bot {
            return;
        }

        let channel_bot_at = match channel_id_bot_at.to_channel(&context.http).await {
            Ok(channel_bot_at) => channel_bot_at,
            Err(error) => {
                tracing::error!("failed to get channel: {channel_id_bot_at:?}\nError: {error:?}");
                return;
            },
        };

        let serenity::all::Channel::Guild(_) = channel_bot_at else {
            return;
        };

        let os_string: OsString = message.content.into();
        if let Some(sound) = self.sounds.get(&os_string) {
            call.play(Track::from(sound.value().clone()).volume(0.05));
        }
    }
}