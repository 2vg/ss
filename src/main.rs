use anyhow::{bail, Context as _};
use dashmap::DashMap;
use event_handler::Handler;
use serenity::{all::GatewayIntents, Client};
use songbird::{input::{cached::Memory, File}, SerenityInit};
use jwalk::WalkDir;
use utils::initialize_logging;
use std::{env, ffi::OsString, path::Path, process::exit, sync::Arc};

mod commands;
mod event_handler;
mod utils;

#[tokio::main]
async fn main()-> anyhow::Result<()> {
    initialize_logging();

    let token = env::var("DISCORD_TOKEN").context("failed to fetch environment variable DISCORD_TOKEN\nError: {error:?}")?;
    let ss_direcotry = env::var("SS_DIRECTORY").context("failed to fetch environment variable SS_DIRECTORY\nError: {error:?}")?;

    if !Path::new(&ss_direcotry).exists() {
        bail!("{} is not exists.", ss_direcotry);
    };

    let sounds: DashMap<OsString, Memory> = DashMap::new();

    for entry in WalkDir::new(ss_direcotry).into_iter().flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "mp3" || ext == "wav" || ext == "opus" || path.file_stem().is_some() {
                let file = File::new(path.clone());
                match Memory::new(file.into()).await {
                    Ok(memory) => {
                        sounds.insert(path.file_stem().unwrap().to_owned(), memory);
                    },
                    Err(error) => {
                        tracing::error!("{error:?}");
                        continue;
                    },
                };
            }
        }
    }

    tracing::info!("{} files found!", sounds.len());

    let mut client = Client::builder(token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Handler {
            connections: Arc::new(DashMap::new()),
            sounds: Arc::new(sounds)
        })
        .register_songbird()
        .await
        .context("failed to build serenity client\nError: {error:?}")?;

    if let Err(error) = client.start().await {
        tracing::error!("failed to start client\nError: {error:?}");
        exit(1);
    }

    Ok(())
}
