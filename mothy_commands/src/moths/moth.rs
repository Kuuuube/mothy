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
    user_cooldown = "30"
)]
pub async fn moth(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let moth = {
        let mut rng = rand::rng();
        data.moth_data.choose(&mut rng).unwrap()
    };
    let classifications = moth.classification.clone();

    let title = format!("{} {}", classifications.genus, classifications.epithet);

    let mut fields = vec![];
    if let Some(common_names) = &moth.common_names {
        fields.push(("Common Names", common_names.join(", "), false));
    }
    let moth_rank_flow = [
        get_moth_rank_vec(&[
            classifications.superfamily,
            classifications.family,
            classifications.subfamily,
            classifications.tribe,
            classifications.subtribe,
        ]),
        vec![classifications.genus, classifications.epithet],
    ]
    .concat()
    .join(" -> ");
    fields.push(("Taxa", moth_rank_flow, false));

    let footer = serenity::CreateEmbedFooter::new(format!(
        "Catalog of Life ID {}",
        moth.catalogue_of_life_taxon_id
    ));

    let embed = serenity::CreateEmbed::default()
        .title(title)
        .url(format!(
            "https://www.catalogueoflife.org/data/taxon/{}",
            moth.catalogue_of_life_taxon_id
        ))
        .fields(fields)
        .footer(footer);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn get_moth_rank_vec(input_strings: &[Option<String>]) -> Vec<String> {
    let mut ranks_vec = Vec::new();
    for input_string in input_strings {
        if let Some(some) = input_string {
            ranks_vec.push(some.to_string());
        }
    }
    return ranks_vec;
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [moth()]
}
