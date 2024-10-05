use std::sync::Arc;

use anyhow::{Context as _, Result};
use serenity::{
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
    model::{application::CommandInteraction, guild::Guild},
};
use songbird::Songbird;
use time::macros::format_description;
use tracing_subscriber::{fmt::time::UtcTime, EnvFilter};

pub(crate) async fn get_manager(context: &Context) -> Result<Arc<Songbird>> {
    songbird::get(context)
        .await
        .context("failed to get songbird voice client: it placed in at initialisation")
}

pub(crate) fn get_guild(context: &Context, interaction: &CommandInteraction) -> Option<Guild> {
    let guild_id = interaction.guild_id?;
    guild_id.to_guild_cached(&context.cache).map(|guild| guild.to_owned())
}

pub(crate) async fn respond(
    context: &Context,
    interaction: &CommandInteraction,
    message: &CreateInteractionResponseMessage,
) -> Result<()> {
    let builder = CreateInteractionResponse::Message(message.clone());
    interaction
        .create_response(&context.http, builder)
        .await
        .with_context(|| format!("failed to create interaction response with message: {message:?}"))?;

    Ok(())
}

pub fn initialize_logging() {
    let local_timer = UtcTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]+[offset_hour]:[offset_minute]"
    ));
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(local_timer)
        .with_file(true)
        .with_line_number(true)
        .init();
}
