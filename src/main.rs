#![warn(clippy::pedantic)]
#![allow(clippy::wildcard_imports, clippy::unused_async)]

use poise::serenity_prelude as serenity;
use std::sync::{atomic::AtomicBool, Arc};

pub use mothy_core::error::Error;
pub use mothy_core::structs::Command;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let options = poise::FrameworkOptions {
        commands: mothy_commands::commands(),
        prefix_options: poise::PrefixFrameworkOptions {
            stripped_dynamic_prefix: Some(|ctx, msg, _| {
                Box::pin(mothy_commands::try_strip_prefix(ctx, msg))
            }),
            mention_as_prefix: true,
            execute_untracked_edits: false,
            case_insensitive_commands: true,
            edit_tracker: None,
            ..Default::default()
        },

        skip_checks_for_owners: false,
        ..Default::default()
    };

    let framework = poise::Framework::new(options);

    // eventually only use the intents I need.
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let token = serenity::Token::from_env("MOTHY_TOKEN").expect("MOTHY_TOKEN is not set.");

    let mut http = serenity::Http::new(token.clone());
    http.default_allowed_mentions = Some(serenity::CreateAllowedMentions::new());

    let client = serenity::ClientBuilder::new_with_http(token, Arc::new(http), intents)
        .framework(framework)
        .event_handler(mothy_events::Handler)
        .data(Arc::new(mothy_core::structs::Data {
            time_started: std::time::Instant::now(),
            has_started: AtomicBool::new(false),
            database: mothy_core::database::Database::init().await,
            james_scores: mothy_core::score_data::init().unwrap_or_default(),
            regex_filters: mothy_core::regex_filters::init(),
            config: mothy_core::structs::MothyConfig::new(),
        }))
        .await;

    client.unwrap().start().await.unwrap();
}
