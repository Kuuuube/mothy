pub mod fun;
pub mod meta;
pub mod utility;

pub use mothy_core::{
    error::Error,
    structs::{Command, Context, Data},
};

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    let mut commands: Vec<crate::Command> = meta::commands()
        .into_iter()
        .chain(fun::commands())
        .chain(utility::commands())
        .collect();

    if std::env::var("DEV_COMMANDS")
        .map(|e| e.parse::<bool>())
        .is_ok_and(Result::unwrap_or_default)
    {
        for command in &mut commands {
            command.name = std::borrow::Cow::Owned(format!("dev-{}", command.name));
        }
    }

    commands
}

pub async fn try_strip_prefix<'a>(
    ctx: &serenity::all::Context,
    msg: &'a serenity::all::Message,
) -> Result<Option<(&'a str, &'a str)>, Error> {
    let stock_prefix = "m";

    let Some(guild_id) = msg.guild_id else {
        if msg.content.strip_prefix(stock_prefix).is_some() {
            return Ok(Some(msg.content.split_at(stock_prefix.len())));
        } else {
            return Ok(None);
        }
    };

    let data = ctx.data_ref::<Data>();

    if let Some(prefix) = data
        .database
        .guild_handler
        .get(guild_id)
        .await
        .expect("TODO: anyhow to error?")
        .prefix
        && msg.content.strip_prefix(prefix.as_str()).is_some()
    {
        return Ok(Some(msg.content.split_at(prefix.len())));
    };

    if msg.content.strip_prefix(stock_prefix).is_some() {
        return Ok(Some(msg.content.split_at(stock_prefix.len())));
    }

    Ok(None)
}
