use mothy_core::{error::Error, structs::Data};
use serenity::all::{
    Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Timestamp,
    VoiceState,
};

use crate::{NEGATIVE_COLOR_HEX, NEUTRAL_ACTION_COLOR_HEX, POSITIVE_COLOR_HEX};

pub async fn voice_state_update(
    ctx: &Context,
    data: &Data,
    old: &Option<VoiceState>,
    new: &VoiceState,
) -> Result<(), Error> {
    let embed = match (old, new.channel_id) {
        (None, _) => handle_join(new),
        (Some(old_voice_state), None) => handle_leave(old_voice_state, new),
        (Some(old_voice_state), Some(new_channel_id)) => {
            if old_voice_state.channel_id != Some(new_channel_id) {
                handle_switch(old_voice_state, new)
            } else {
                dbg!("misc stuff");
                // handle_misc()
                return Ok(());
            }
        }
    };

    if let Some(logs_channel) = data
        .config
        .mothy_voice_logs_channel
        .get(&new.guild_id.unwrap_or_default())
    {
        logs_channel
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await?;
    };

    Ok(())
}

fn handle_join(new_voice_state: &'_ VoiceState) -> CreateEmbed<'_> {
    let (username, avatar_url) = if let Some(member) = &new_voice_state.member {
        (member.user.name.to_string(), member.user.avatar_url())
    } else {
        ("Unknown".to_string(), None)
    };
    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(username).icon_url(avatar_url.unwrap_or_default()))
        .colour(POSITIVE_COLOR_HEX)
        .description(format!(
            "<@{}> joined the voice channel <#{}>",
            new_voice_state.user_id,
            new_voice_state.channel_id.unwrap_or_default()
        ))
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!(
            "ID: {}",
            new_voice_state.user_id
        )));

    return embed;
}

fn handle_leave<'a>(old_voice_state: &VoiceState, new_voice_state: &VoiceState) -> CreateEmbed<'a> {
    let (username, avatar_url) = if let Some(member) = &new_voice_state.member {
        (member.user.name.to_string(), member.user.avatar_url())
    } else {
        ("Unknown".to_string(), None)
    };
    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(username).icon_url(avatar_url.unwrap_or_default()))
        .colour(NEGATIVE_COLOR_HEX)
        .description(format!(
            "<@{}> left the voice channel <#{}>",
            new_voice_state.user_id,
            old_voice_state.channel_id.unwrap_or_default()
        ))
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!(
            "ID: {}",
            new_voice_state.user_id
        )));

    return embed;
}

fn handle_switch<'a>(
    old_voice_state: &VoiceState,
    new_voice_state: &VoiceState,
) -> CreateEmbed<'a> {
    let (username, avatar_url) = if let Some(member) = &new_voice_state.member {
        (member.user.name.to_string(), member.user.avatar_url())
    } else {
        ("Unknown".to_string(), None)
    };
    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(username).icon_url(avatar_url.unwrap_or_default()))
        .colour(NEUTRAL_ACTION_COLOR_HEX)
        .description(format!(
            "<@{}> switched voice channel from <#{}> to <#{}>",
            new_voice_state.user_id,
            old_voice_state.channel_id.unwrap_or_default(),
            new_voice_state.channel_id.unwrap_or_default()
        ))
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!(
            "ID: {}",
            new_voice_state.user_id
        )));

    return embed;
}
