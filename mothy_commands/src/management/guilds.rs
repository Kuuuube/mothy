use poise::CreateReply;
use serenity::all::{CreateAttachment, MfaLevel, Timestamp, VerificationLevel};

use crate::{Context, Error};

const MOTHY_ID: u64 = 1415021997133402213;

struct GuildListEntry {
    id: u64,
    name: String,
    joined_at: Timestamp,
    description: String,
    member_count: u64,
    owner_id: u64,
    mfa_level: MfaLevel,
    verification_level: VerificationLevel,
    icon_url: String,
    banner_url: String,
    splash_url: String,
}

impl GuildListEntry {
    pub fn get_header() -> String {
        [
            "id",
            "name",
            "joined_at",
            "description",
            "member_count",
            "owner_id",
            "mfa_level",
            "verification_level",
            "icon_url",
            "banner_url",
            "splash_url",
        ]
        .join("\t")
    }

    pub fn format(self) -> String {
        [
            self.id.to_string(),
            self.name,
            self.joined_at.to_string(),
            self.description
                .replace("\n", "\\n")
                .replace("\t", "\\t")
                .to_string(),
            self.member_count.to_string(),
            self.owner_id.to_string(),
            self.mfa_level.0.to_string(),
            self.verification_level.0.to_string(),
            self.icon_url,
            self.banner_url,
            self.splash_url,
        ]
        .join("\t")
    }
}

#[poise::command(rename = "get-guilds", prefix_command, hide_in_help, owners_only)]
async fn get_guilds(ctx: Context<'_>) -> Result<(), Error> {
    let guilds_list = ctx.cache().guilds();
    let guilds_list_formatted_futures = guilds_list.iter().map(async |guild_id| {
        if let Some(guild_info) = ctx.cache().guild(guild_id.into()) {
            let guild_data = GuildListEntry {
                id: guild_info.id.into(),
                name: guild_info.name.to_string(),
                description: guild_info
                    .description
                    .clone()
                    .unwrap_or_default()
                    .to_string(),
                member_count: guild_info.member_count,
                joined_at: guild_info.joined_at,
                owner_id: guild_info.owner_id.into(),
                mfa_level: guild_info.mfa_level,
                verification_level: guild_info.verification_level,
                icon_url: guild_info.icon_url().unwrap_or_default(),
                banner_url: guild_info.banner_url().unwrap_or_default(),
                splash_url: guild_info.splash_url().unwrap_or_default(),
            };
            guild_data.format()
        } else {
            format!(
                "{}\t{}\t{}",
                guild_id,
                guild_id.name(ctx.cache()).unwrap_or("".to_string()),
                guild_id
                    .member(ctx, MOTHY_ID.into())
                    .await
                    .unwrap_or_default()
                    .joined_at
                    .map(|x| x.to_string())
                    .unwrap_or("".to_string()),
            )
        }
    });
    let guilds_list_formatted = serenity::futures::future::join_all(guilds_list_formatted_futures)
        .await
        .join("\n");

    ctx.send(CreateReply::new().attachment(CreateAttachment::bytes(
        format!("{}\n{guilds_list_formatted}\n", GuildListEntry::get_header()),
        "guild_data.txt",
    )))
    .await?;
    Ok(())
}


#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [get_guilds()]
}
