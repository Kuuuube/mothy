use ::serenity::all::CreateEmbed;
use mothy_core::NEGATIVE_COLOR_HEX;

use crate::{
    Context, Error,
    moths::{embed_assemblers::*, helpers::*, interaction_helpers::*},
};
use poise::serenity_prelude as serenity;

use rand::seq::IndexedRandom;

const MOTHS_PER_PAGE: usize = 10;

/// Find a random moth
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    user_cooldown = "10"
)]
pub async fn moth(ctx: Context<'_>) -> Result<(), Error> {
    // this command's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
    ctx.defer()
        .await
        .expect("moth command response defer fail, this shouldn't happen");

    let data = ctx.data();
    let moth = {
        let mut rng = rand::rng();
        data.moth_data.moth_data.choose(&mut rng).unwrap()
    };
    let embed = assemble_moth_embed(moth).await;
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Find a random named moth
#[poise::command(
    rename = "moth-named",
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    user_cooldown = "10"
)]
pub async fn moth_named(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    const MAX_TRIES: usize = 1000; // the chance this does not find a named moth in 1000 tries is about 0.000000679%
    let mut i = 0;
    while i < MAX_TRIES {
        let moth = {
            let mut rng = rand::rng();
            data.moth_data.moth_data.choose(&mut rng).unwrap()
        };
        if moth.common_names.is_none() {
            i += 1;
            continue;
        }
        let embed = assemble_moth_embed(moth).await;
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let embed = CreateEmbed::new()
        .description(format!("Failed to find named moth in {MAX_TRIES} tries."))
        .color(NEGATIVE_COLOR_HEX);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Search for a moth
#[poise::command(
    rename = "moth-search",
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    // This command may appear heavy on memory, it is not. All the moth data besides what is actively displayed is borrowed
    user_cooldown = "10"
)]
pub async fn moth_search(
    ctx: Context<'_>,
    #[description = "The superfamily to search for moths in"] superfamily: Option<String>,
    #[description = "The family to search for moths in"] family: Option<String>,
    #[description = "The subfamily to search for moths in"] subfamily: Option<String>,
    #[description = "The tribe to search for moths in"] tribe: Option<String>,
    #[description = "The subtribe to search for moths in"] subtribe: Option<String>,
    #[description = "The genus to search for moths in"] genus: Option<String>,
    #[description = "The specific name to search for moths in"] specific: Option<String>,
    #[description = "The subspecific name to search for moths in"] subspecific: Option<String>,
) -> Result<(), Error> {
    // this command's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
    ctx.defer()
        .await
        .expect("moth_search command response defer fail, this shouldn't happen");

    let data = ctx.data();
    let moth_data = &data.moth_data.moth_data;
    let moth_synonyms = &data.moth_data.moth_synonyms;
    let butterfly_blacklist = &data.moth_data.butterfly_blacklist;

    // ugly lepidoptera searching is not allowed (butteryflies)
    if is_butterfly(
        butterfly_blacklist,
        &superfamily,
        &family,
        &subfamily,
        &tribe,
        &subtribe,
        &genus,
        &specific,
        &subspecific,
    ) {
        let embed = serenity::CreateEmbed::default()
            .description("Attempted butterfly search detected. This incident will be reported.")
            .color(serenity::Colour::from_rgb(255, 0, 0));
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // specific species search
    if let Some(genus_some) = &genus
        && let Some(specific_some) = &specific
    {
        let lowercase_scientific_name = assemble_scientific_name(
            &genus_some.to_lowercase(),
            &specific_some.to_lowercase(),
            subspecific.as_deref(),
        );
        let mut possible_subspecific_as_specific = Vec::new();
        let found_synonym_id = moth_synonyms.get(&lowercase_scientific_name);
        let found_moth = if let Some(found_synonym_id) = found_synonym_id {
            moth_data
                .iter()
                .find(|moth| &moth.catalogue_of_life_taxon_id == found_synonym_id)
        } else {
            moth_data.iter().find(|moth| {
                if moth.classification.genus.to_lowercase() != genus_some.to_lowercase() {
                    return false;
                }
                if moth.classification.specific.to_lowercase() == specific_some.to_lowercase()
                    && moth.classification.subspecific
                        == subspecific.as_ref().map(|x| x.to_lowercase())
                {
                    return true;
                }

                // sometimes subspecies can be abbreviated to `Genus subspecific` rather than `Genus specific subspecific`
                // this can make it appear as if `Genus specific` has been written rather than a subspecies identifier
                // check if user's input `specific` matches moth's `subspecific`
                if let Some(moth_subspecific) = &moth.classification.subspecific
                    && moth_subspecific.to_lowercase() == specific_some.to_lowercase()
                {
                    possible_subspecific_as_specific.push(format!(
                        "{} {} {}",
                        moth.classification.genus, moth.classification.specific, moth_subspecific
                    ));
                }
                false
            })
        };

        if let Some(found_moth) = found_moth {
            let embed = assemble_moth_embed(found_moth).await;
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        } else {
            let mut capitalized_scientific_name = lowercase_scientific_name;
            if let Some(first_char) = capitalized_scientific_name.get_mut(0..1) {
                first_char.make_ascii_uppercase();
            }

            let mut embed_text = format!("Failed to find moth `{capitalized_scientific_name}`.");
            if !possible_subspecific_as_specific.is_empty() {
                let formatted_subspecific_as_specific = possible_subspecific_as_specific
                    .iter()
                    .map(|x| format!("`{x}`"))
                    .collect::<Vec<String>>()
                    .join("\n");
                embed_text = format!(
                    "{embed_text}\nFound similarly named subspecies:\n{formatted_subspecific_as_specific}"
                );
            }
            let embed = serenity::CreateEmbed::default().description(embed_text);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        return Ok(());
    }

    // wide search
    let Ok(moths_found) = moth_query(
        &moth_data,
        &MothQuery {
            superfamily,
            family,
            subfamily,
            tribe,
            subtribe,
            genus,
            specific,
            subspecific,
            common_name: None,
            exact_common_name_search: false,
        },
    ) else {
        let embed = serenity::CreateEmbed::default().title("Search found 0 moths");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    pagination_embed(
        ctx,
        &moths_found,
        MOTHS_PER_PAGE,
        assemble_paginated_moth_search_embed,
        assemble_moth_embed,
    )
    .await
}

/// Search for a moth by name
#[poise::command(
    rename = "moth-search-name",
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    // This command may appear heavy on memory, it is not. All the moth data besides what is actively displayed is borrowed
    user_cooldown = "10"
)]
pub async fn moth_search_named(
    ctx: Context<'_>,
    #[description = "The common name for a moth"] name: String,
    #[rename = "exact-match"]
    #[flag]
    exact_match: bool,
) -> Result<(), Error> {
    let name = dequote(&name);

    let data = ctx.data();
    let moth_data = &data.moth_data.moth_data;

    let Ok(moths_found) = moth_query(
        &moth_data,
        &MothQuery {
            superfamily: None,
            family: None,
            subfamily: None,
            tribe: None,
            subtribe: None,
            genus: None,
            specific: None,
            subspecific: None,
            common_name: Some(name),
            exact_common_name_search: exact_match,
        },
    ) else {
        let embed = serenity::CreateEmbed::default().title("Search found 0 moths");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    return pagination_embed(
        ctx,
        &moths_found,
        MOTHS_PER_PAGE,
        assemble_paginated_moth_search_embed_named,
        assemble_moth_embed,
    )
    .await;
}

#[must_use]
pub fn commands() -> [crate::Command; 4] {
    [moth(), moth_search(), moth_named(), moth_search_named()]
}
