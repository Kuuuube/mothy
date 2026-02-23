use crate::{Context, Error};
use poise::serenity_prelude as serenity;

use rand::seq::IndexedRandom;

/// Find a random moth
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    user_cooldown = "4"
)]
pub async fn moth(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let moth = {
        let mut rng = rand::rng();
        data.moth_data.choose(&mut rng).unwrap()
    };

    let title = format!("{} {}", moth.classification.genus, moth.classification.epithet);
    let embed = serenity::CreateEmbed::default().title(title);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [moth()]
}
