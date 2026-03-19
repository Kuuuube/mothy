use poise::CreateReply;
use serenity::all::CreateAttachment;

use crate::{Context, Error};

const MOTHY_ID: u64 = 1415021997133402213;

#[poise::command(rename = "get-guilds", prefix_command, hide_in_help, owners_only)]
async fn get_guilds(ctx: Context<'_>) -> Result<(), Error> {
    let guilds_list = ctx.cache().guilds();

    let guilds_list_formatted_futures = guilds_list.iter().map(async |guild_id| {
        format!(
            "{}\t{}\t{}",
            guild_id,
            guild_id
                .name(ctx.cache())
                .unwrap_or("Failed to get guild_name".to_string()),
            guild_id
                .member(ctx, MOTHY_ID.into())
                .await
                .unwrap_or_default()
                .joined_at
                .map(|x| x.to_string())
                .unwrap_or("Failed to get bot_join_date".to_string()),
        )
    });
    let guilds_list_formatted = serenity::futures::future::join_all(guilds_list_formatted_futures)
        .await
        .join("\n");

    let header = ["guild_id", "guild_name", "bot_join_date"].join("\t");
    ctx.send(CreateReply::new().attachment(CreateAttachment::bytes(
        format!("{header}\n{guilds_list_formatted}"),
        "guild_data.txt",
    )))
    .await?;
    Ok(())
}



#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [get_guilds()]
}
