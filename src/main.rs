mod commands;
use commands::*;

mod event_handler;

use poise::serenity_prelude as serenity;
use std::{env::var, time::Duration};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


pub struct Data {
    time_started: std::time::Instant,
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}



// Define the 'dmuser' command
#[poise::command(prefix_command, track_edits, owners_only)]
pub async fn dmuser(
    ctx: Context<'_>,
    #[description = "ID"] user_id: poise::serenity_prelude::UserId,
    #[rest]
    #[description = "Message"]
    messages: String,
) -> Result<(), Error> {
    let user = user_id.to_user(&ctx).await?;
    user.direct_message(&ctx, |m| m.content(messages)).await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {

    let options = poise::FrameworkOptions {
        commands: vec![
            register(),
            dmuser(),
            meta::source(),
            meta::shutdown(),
            meta::uptime(),
            meta::help(),
            fun::hug::hug(),
            utility::urban::urban(),
            utility::roll::roll(),
            general::avatar::avatar(),
            general::userinfo::userinfo(),
            utility::colour::hex(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },

        on_error: |error| Box::pin(on_error(error)),

        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },

        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },

        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler::event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(
            var("JAMEBOT_TOKEN")
                .expect("The JAMEBOT_TOKEN is not set. aborting..."),
        )
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    time_started: std::time::Instant::now(),
                })
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
        .unwrap();


}
