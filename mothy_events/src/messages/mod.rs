use std::{fmt::Write, sync::Arc};
use mothy_ansi::{CYAN, HI_BLACK, RESET, HI_RED};
use mothy_core::structs::Data;
use serenity::all::{Context, Message, Role};

use crate::helper::{get_channel_name, get_guild_name_override};

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

    let _ = tokio::join!(
        image_spambot_filter(ctx, msg),
        regex_blacklist_filter(ctx, &data, msg, guild_name, channel_name, author_string),
    );

    let Some(_) = msg.guild_id else { return };
}

async fn regex_blacklist_filter(ctx: &Context, data: &Data, msg: &Message, guild_name: String, channel_name: String, author_string: String) {
    let regex_filters = &data.regex_filters;
    let content = &msg.content;
    for regex_filter in regex_filters {
        if regex_filter.is_match(&content) {
            match msg.delete(&ctx.http, None).await {
                Ok(_) => {
                    println!(
                        "{HI_RED}DELETED [{guild_name}] [#{channel_name}]{RESET} {author_string}: \
                        {content}{RESET}{CYAN}{RESET}"
                    );
                },
                Err(err) => {
                    println!(
                        "FAILED TO DELETE {HI_RED}[{guild_name}] [#{channel_name}]{RESET} {author_string}: \
                        {content}{RESET}{CYAN}{RESET}");
                    dbg!(err);
                },
            }
            break;
        }
    }
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
