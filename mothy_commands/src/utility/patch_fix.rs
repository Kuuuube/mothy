#![expect(clippy::cast_possible_truncation)]

use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, CreateAllowedMentions};

/// Convert bad git patches created on Windows to patches easily readable on Linux
#[poise::command(
    slash_command,
    category = "Utility",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    user_cooldown = "5"
)]
pub async fn patch_fix(
    ctx: Context<'_>,
    #[description = "Bugged git patch file generated from Windows"]
    #[rest]
    patch: serenity::Attachment,
) -> Result<(), Error> {
    const EIGHT_MB: u32 = 8000000;
    let mentions = CreateAllowedMentions::new()
        .everyone(false)
        .all_roles(false)
        .all_users(false);

    if patch.size > EIGHT_MB {
        ctx.send(
            poise::CreateReply::new()
                .content(format!("Attachment too large"))
                .allowed_mentions(mentions),
        )
        .await?;
        return Ok(());
    }
    let patch_data = patch.download().await?;
    let fixed_patch = match fix_patch(patch_data) {
        Ok(ok) => ok,
        Err(_) => {
            ctx.send(
                poise::CreateReply::new()
                    .content(format!("Could not fix patch"))
                    .allowed_mentions(mentions),
            )
            .await?;
            return Ok(());
        },
    };

    let attachment = serenity::CreateAttachment::bytes(fixed_patch, format!("fixed_{}", patch.filename));
    ctx.send(poise::CreateReply::default().attachment(attachment))
        .await?;

    Ok(())
}

fn fix_patch(patch_bytes: Vec<u8>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    return Ok(String::from_utf16(&vec_u8_to_u16_le(patch_bytes))?.replace("\r", ""));
}

fn vec_u8_to_u16_le(vec_u8: Vec<u8>) -> Vec<u16> {
    let mut vec_u16: Vec<u16> = Default::default();
    let u8_pairs: Vec<[u8; 2]> = vec_u8.chunks(2).map(|x| [x[0], x[1]]).collect();

    for u8_pair in u8_pairs {
        vec_u16.push(u16::from_le_bytes(u8_pair));
    }

    return vec_u16;
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [patch_fix()]
}
