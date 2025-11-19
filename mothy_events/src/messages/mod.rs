use mothy_ansi::{CYAN, HI_BLACK, HI_RED, RESET};
use mothy_core::{error::Error, structs::Data};
use serenity::all::{
    Context, CreateEmbed, CreateEmbedFooter, CreateMessage, Message, Role, Timestamp,
};
use std::{fmt::Write, sync::Arc};

use crate::{
    NEGATIVE_COLOR_HEX,
    helper::{get_channel_name, get_guild_name_override},
};

pub async fn on_message(ctx: &Context, msg: &Message, data: Arc<Data>) {
    let dont_print = false;
    let content = {
        let maybe_flagged = &msg.content;
        // moth_filter::filter_content(&msg.content, &config.badlist, &config.fixlist);

        maybe_flagged
    };

    let guild_id = msg.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, msg.channel_id).await;

    let (attachments, embeds) = attachments_embed_fmt(msg);

    let author_string = author_string(ctx, msg);

    if !dont_print {
        println!(
            "{HI_BLACK}[{guild_name}] [#{channel_name}]{RESET} {author_string}: \
             {content}{RESET}{CYAN}{}{}{RESET}",
            attachments.as_deref().unwrap_or(""),
            embeds.as_deref().unwrap_or("")
        );
    }

    let user_roles = if let Some(member) = msg.member.as_ref() {
        member.roles.to_vec()
    } else {
        vec![]
    };

    let filters_valid_guild = data
        .config
        .filters_allowed_guilds
        .contains(&guild_id.unwrap_or_default());
    let filters_valid_author = !data
        .config
        .filter_bypass_roles
        .iter()
        .any(|x| user_roles.contains(x));

    if filters_valid_guild && filters_valid_author {
        let _ = tokio::join!(
            image_spambot_filter(ctx, msg),
            regex_blacklist_filter(ctx, &data, msg, guild_name, channel_name, author_string),
        );
    }

    let Some(_) = msg.guild_id else { return };
}

async fn regex_blacklist_filter(
    ctx: &Context,
    data: &Data,
    msg: &Message,
    guild_name: String,
    channel_name: String,
    author_string: String,
) -> Result<(), Error> {
    let regex_filters = &data.regex_filters;
    let content = &msg.content;

    let links = regex_filters
        .links_detector
        .captures_iter(content)
        .fold(vec![], |mut acc, x| {
            if let Some(x_some) = x.get(0) {
                acc.push(x_some.as_str());
            };
            acc
        })
        .join("\n");

    for regex_filter in &regex_filters.links_blacklist {
        if let Some(regex_match) = regex_filter.find(&links) {
            match msg.delete(&ctx.http, None).await {
                Ok(_) => {
                    println!(
                        "{HI_RED}REGEX DELETED [{guild_name}] [#{channel_name}]{RESET} {author_string}: \
                        {content}{RESET}{CYAN}{RESET}"
                    );
                    if let Some(blacklist_logs_channel) = data
                        .config
                        .mothy_blacklist_logs_channel
                        .get(&msg.guild_id.unwrap_or_default())
                    {
                        let embed = CreateEmbed::new()
                            .thumbnail(msg.author.avatar_url().unwrap_or_default())
                            .colour(NEGATIVE_COLOR_HEX)
                            .title("Message Filtered")
                            .description(format!(
                                "Message sent by <@{}> deleted in <#{}>\n```\n{}\n```",
                                msg.author.id,
                                msg.channel_id,
                                &msg.content_safe(&ctx.cache).replace("`", "\\`")
                            ))
                            .field(
                                "Reason",
                                format!("```\n{}\n```", regex_match.as_str().replace("`", "\\`")),
                                true,
                            )
                            .field(
                                "Rule",
                                format!("```\n{}\n```", regex_filter.as_str().replace("`", "\\`")),
                                true,
                            )
                            .timestamp(Timestamp::now())
                            .footer(CreateEmbedFooter::new(format!("ID: {}", msg.author.id)));
                        blacklist_logs_channel
                            .send_message(&ctx.http, CreateMessage::new().embed(embed))
                            .await?;
                    }
                }
                Err(err) => {
                    println!(
                        "FAILED TO REGEX DELETE {HI_RED}[{guild_name}] [#{channel_name}]{RESET} {author_string}: \
                        {content}{RESET}{CYAN}{RESET}"
                    );
                    dbg!(err);
                }
            }
            break;
        }
    }
    Ok(())
}

async fn image_spambot_filter(ctx: &Context, msg: &Message) {
    let mut image_count = 0;
    let mut not_image = 0;
    for attachment in &msg.attachments {
        if let Some(attachment_type) = &attachment.content_type {
            if attachment_type.contains("image") {
                image_count += 1;
            } else {
                not_image += 1;
            }
        }
    }
    if image_count >= 3 && not_image == 0 {
        // Dont do this yet
        // msg.delete(&ctx.http, None).await;
    }
}

#[must_use]
pub fn attachments_embed_fmt(new_message: &Message) -> (Option<String>, Option<String>) {
    let attachments = &new_message.attachments;
    let attachments_fmt: Option<String> = if attachments.is_empty() {
        None
    } else {
        let attachment_names: Vec<String> = attachments
            .iter()
            .map(|attachment| attachment.filename.to_string())
            .collect();
        Some(format!(" <{}>", attachment_names.join(", ")))
    };

    let embeds = &new_message.embeds;
    let embeds_fmt: Option<String> = if embeds.is_empty() {
        None
    } else {
        let embed_types: Vec<String> = embeds
            .iter()
            .map(|embed| embed.kind.clone().unwrap_or_default().into_string())
            .collect();

        Some(format!(" {{{}}}", embed_types.join(", ")))
    };

    (attachments_fmt, embeds_fmt)
}

#[must_use]
pub fn author_string(ctx: &Context, msg: &Message) -> String {
    // No member meaning no roles.
    let Some(member) = &msg.member else {
        return msg.author.tag();
    };

    let username = msg.author.tag();

    let guild = msg.guild(&ctx.cache).unwrap();

    let mut highest: Option<&Role> = None;

    for role_id in &member.roles {
        if let Some(role) = guild.roles.get(role_id) {
            if role.colour.0 == 000000 {
                continue;
            }

            // Skip this role if this role in iteration has:
            // - a position less than the recorded highest
            // - a position equal to the recorded, but a higher ID
            if let Some(r) = highest
                && (role.position < r.position || (role.position == r.position && role.id > r.id))
            {
                continue;
            }

            highest = Some(role);
        }
    }

    let mut prefix = String::new();
    if let Some(hr) = highest {
        let c = hr.colour;
        if hr.colour.0 != 0 {
            write!(prefix, "\x1B[38;2;{};{};{}m", c.r(), c.g(), c.b()).unwrap();
        }
    }

    format!("{prefix}{username}{RESET}")
}
