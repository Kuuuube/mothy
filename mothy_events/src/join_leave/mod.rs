use std::sync::Arc;

use chrono::{DateTime, Datelike, Timelike, Utc};
use mothy_ansi::{RESET, YELLOW};
use mothy_core::structs::Data;
use serenity::all::{
    Context, CreateEmbed, CreateEmbedFooter, CreateMessage, GuildId, Member, Timestamp, User,
};

use crate::{NEGATIVE_COLOR_HEX, POSITIVE_COLOR_HEX, helper::get_guild_name_override};

pub async fn guild_member_addition(ctx: &Context, new_member: &Member, data: Arc<Data>) {
    let guild_id = new_member.guild_id;
    let joined_user_id = new_member.user.id;

    if let Some(join_logs_channel) = data.config.mothy_logs_channel.get(&new_member.guild_id) {
        let embed = CreateEmbed::new()
            .thumbnail(new_member.avatar_url().unwrap_or_default())
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
}

pub async fn guild_member_removal(ctx: &Context, guild_id: &GuildId, user: &User, data: Arc<Data>) {
    let guild_name = get_guild_name_override(ctx, &data, Some(*guild_id));

    println!(
        "{YELLOW}[{}] {} (ID:{}) has left!{RESET}",
        guild_name,
        user.tag(),
        user.id
    );

    if let Some(join_logs_channel) = data.config.mothy_logs_channel.get(&guild_id) {
        let embed = CreateEmbed::new()
            .thumbnail(user.avatar_url().unwrap_or_default())
            .colour(NEGATIVE_COLOR_HEX)
            .title("Member Left")
            .description(format!("<@{}> {}", user.id, user.name))
            .timestamp(Timestamp::now())
            .footer(CreateEmbedFooter::new(format!("ID: {}", user.id)));
        let _ = join_logs_channel
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await;
    }
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
    let month = datetime.month();
    let day = datetime.day();
    let hour = datetime.hour();
    let minute = datetime.minute();
    let second = datetime.second();
    if year > 0 {
        datetime_strings.push(format!("Years {}", year));
    }
    if month > 0 {
        datetime_strings.push(format!("Months {}", month));
    }
    if day > 0 {
        datetime_strings.push(format!("Days {}", day));
    }

    if hour > 0 {
        datetime_strings.push(format!("Hours {}", hour));
    }
    if minute > 0 {
        datetime_strings.push(format!("Minutes {}", minute));
    }
    if second > 0 {
        datetime_strings.push(format!("Seconds {}", second));
    }

    datetime_strings.truncate(3);
    return datetime_strings.join(", ");
}
