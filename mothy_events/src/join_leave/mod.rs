use std::sync::Arc;

use chrono::{DateTime, Datelike, Timelike, Utc};
use mothy_ansi::{RESET, YELLOW};
use mothy_core::{error::Error, structs::Data};
use serenity::all::{
    Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, GuildId, Member,
    Timestamp, User,
};

use crate::{NEGATIVE_COLOR_HEX, POSITIVE_COLOR_HEX, helper::get_guild_name_override};

pub async fn guild_member_addition(
    ctx: &Context,
    new_member: &Member,
    data: Arc<Data>,
) -> Result<(), Error> {
    let guild_id = new_member.guild_id;
    let joined_user_id = new_member.user.id;

    if let Some(join_logs_channel) = data
        .config
        .mothy_join_logs_channel
        .get(&new_member.guild_id)
    {
        let embed = CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&new_member.user.name)
                    .icon_url(new_member.avatar_url().unwrap_or_default()),
            )
            .colour(POSITIVE_COLOR_HEX)
            .title("Member Joined")
            .description(format!(
                "<@{}> {}",
                new_member.user.id, new_member.user.name
            ))
            .field(
                "Account Age",
                get_member_joined_at(new_member).unwrap_or_else(|| "Date Unknown".to_string()),
                false,
            )
            .timestamp(Timestamp::now())
            .footer(CreateEmbedFooter::new(format!(
                "ID: {}",
                new_member.user.id
            )));
        let _ = join_logs_channel
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await;
    }

    let guild_name = get_guild_name_override(ctx, &data, Some(guild_id));

    println!(
        "{YELLOW}[{}] {} (ID:{}) has joined!{RESET}",
        guild_name,
        new_member.user.tag(),
        joined_user_id
    );

    Ok(())
}

pub async fn guild_member_removal(
    ctx: &Context,
    guild_id: &GuildId,
    user: &User,
    data: Arc<Data>,
) -> Result<(), Error> {
    let guild_name = get_guild_name_override(ctx, &data, Some(*guild_id));

    println!(
        "{YELLOW}[{}] {} (ID:{}) has left!{RESET}",
        guild_name,
        user.tag(),
        user.id
    );

    if let Some(join_logs_channel) = data.config.mothy_join_logs_channel.get(&guild_id) {
        let embed = CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&user.name).icon_url(user.avatar_url().unwrap_or_default()),
            )
            .colour(NEGATIVE_COLOR_HEX)
            .title("Member Left")
            .description(format!("<@{}> {}", user.id, user.name))
            .timestamp(Timestamp::now())
            .footer(CreateEmbedFooter::new(format!("ID: {}", user.id)));
        let _ = join_logs_channel
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await;
    }

    Ok(())
}

fn get_member_joined_at(new_member: &Member) -> Option<String> {
    let user_account_creation_date = &new_member.user.id.created_at();

    let time_since = DateTime::from_timestamp(
        Timestamp::now().unix_timestamp() - user_account_creation_date.unix_timestamp(),
        0,
    )?;
    let time_since_adjusted = time_since.with_year(time_since.year() - 1970)?;

    return Some(format!("{}", truncate_datetime_string(time_since_adjusted)));
}

fn truncate_datetime_string(datetime: DateTime<Utc>) -> String {
    let mut datetime_strings: Vec<String> = vec![];

    let year = datetime.year();
    let month = datetime.month() - 1; // starts at 1
    let day = datetime.day() - 1; // starts at 1
    let hour = datetime.hour();
    let minute = datetime.minute();
    let second = datetime.second();
    if year > 0 {
        datetime_strings.push(format!("{}y", year));
    }
    if month > 0 {
        datetime_strings.push(format!("{}M", month));
    }
    if day > 0 {
        datetime_strings.push(format!("{}d", day));
    }

    if hour > 0 {
        datetime_strings.push(format!("{}h", hour));
    }
    if minute > 0 {
        datetime_strings.push(format!("{}m", minute));
    }
    if second > 0 {
        datetime_strings.push(format!("{}s", second));
    }

    return datetime_strings.join(" ");
}
