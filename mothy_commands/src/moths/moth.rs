use std::time::Duration;

use crate::{Context, Error, moths::api_callers::*, moths::helpers::*};
use moth_filter::SpeciesData;
use poise::serenity_prelude as serenity;

use ::serenity::{
    all::{
        ComponentInteractionCollector, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
    },
    futures::StreamExt,
};
use rand::seq::IndexedRandom;
use reqwest::Client as ReqwestClient;

const CATALOGUE_OF_LIFE_TAXON_URL: &str = "https://www.catalogueoflife.org/data/taxon/";
const GBIF_SPECIES_URL: &str = "https://www.gbif.org/species/";

const MOTHS_PER_PAGE: usize = 10;
const MOTH_SEARCH_INTERACTION_TIMEOUT: u64 = 300; // interaction tokens are only valid for 15 minutes max, this should never exceed `900` (realistically a bit lower to have a bit of safety buffer)
const BUTTON_ID_FIRST: &str = "First";
const BUTTON_ID_BACK: &str = "Back";
const BUTTON_ID_FORWARD: &str = "Forward";
const BUTTON_ID_LAST: &str = "Last";

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
    // this command's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
    ctx.defer()
        .await
        .expect("moth command response defer fail, this shouldn't happen");

    let data = ctx.data();
    let moth = {
        let mut rng = rand::rng();
        data.moth_data.choose(&mut rng).unwrap()
    };
    let embed = assemble_moth_embed(moth).await;
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Search for a moth
#[poise::command(
    rename = "moth-search",
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    user_cooldown = "30"
)]
pub async fn moth_search(
    ctx: Context<'_>,
    superfamily: Option<String>,
    family: Option<String>,
    subfamily: Option<String>,
    tribe: Option<String>,
    subtribe: Option<String>,
    genus: Option<String>,
    epithet: Option<String>,
) -> Result<(), Error> {
    // this command's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
    ctx.defer()
        .await
        .expect("moth_search command response defer fail, this shouldn't happen");

    // ugly lepidoptera searching is not allowed (butteryflies)
    if is_butterfly(
        &ctx.data().butterfly_blacklist,
        &superfamily,
        &family,
        &subfamily,
        &tribe,
        &subtribe,
        &genus,
        &epithet,
    ) {
        let embed = serenity::CreateEmbed::default()
            .description("Attempted butterfly search detected. This incident will be reported.")
            .color(serenity::Colour::from_rgb(255, 0, 0));
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let data = ctx.data();

    // specific species search
    if let Some(genus_some) = &genus
        && let Some(epithet_some) = &epithet
    {
        if let Some(found_moth) = &data.moth_data.iter().find(|moth| {
            moth.classification.genus.to_lowercase() == genus_some.to_lowercase()
                && moth.classification.epithet.to_lowercase() == epithet_some.to_lowercase()
        }) {
            let embed = assemble_moth_embed(*found_moth).await;
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        } else {
            let mut uppercase_genus = genus_some.clone();
            uppercase_genus
                .get_mut(0..1)
                .and_then(|x| Some(x.make_ascii_uppercase()));
            let embed = serenity::CreateEmbed::default().description(format!(
                "Failed to find moth `{} {}`.",
                uppercase_genus,
                epithet_some.to_lowercase()
            ));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        return Ok(());
    }

    // wide search
    let mut moths_found: Vec<&SpeciesData> = data
        .moth_data
        .iter()
        .filter(|moth| {
            if !search_classification_valid(&superfamily, &moth.classification.superfamily)
                || !search_classification_valid(&family, &moth.classification.family)
                || !search_classification_valid(&subfamily, &moth.classification.subfamily)
                || !search_classification_valid(&tribe, &moth.classification.tribe)
                || !search_classification_valid(&subtribe, &moth.classification.subtribe)
                || !search_classification_valid(&genus, &Some(&moth.classification.genus))
                || !search_classification_valid(&epithet, &Some(&moth.classification.epithet))
            {
                return false;
            }
            return true;
        })
        .collect();

    let moth_count = moths_found.len();

    if moth_count == 0 {
        let embed = serenity::CreateEmbed::default().title("Search found 0 moths");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    moths_found.sort_by(|a, b| {
        format!("{} {}", a.classification.genus, a.classification.epithet).cmp(&format!(
            "{} {}",
            b.classification.genus, b.classification.epithet
        ))
    });

    let mut page_number = 0;
    let pagecount = (moth_count + MOTHS_PER_PAGE - 1) / MOTHS_PER_PAGE; // int division that rounds up
    let embed =
        assemble_paginated_moth_search_embed(&moths_found, moth_count, page_number, pagecount);

    let bot_message = ctx
        .send(
            poise::CreateReply::default()
                .embed(embed.clone())
                .components(&[get_pagination_buttons(page_number, pagecount)]),
        )
        .await?;

    let mut interaction_collector = ComponentInteractionCollector::new(&ctx.serenity_context())
        .timeout(Duration::from_secs(MOTH_SEARCH_INTERACTION_TIMEOUT))
        .message_id(bot_message.message().await?.id)
        .stream();

    while let Some(interaction) = interaction_collector.next().await {
        // this interactions's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
        interaction
            .defer(ctx.http())
            .await
            .expect("Interaction defer fail, this shouldn't happen");
        match interaction.data.custom_id.clone().into_string().as_str() {
            BUTTON_ID_FIRST => {
                if page_number == 0 {
                    continue;
                }
                page_number = 0;
            }
            BUTTON_ID_BACK => {
                if page_number == 0 {
                    continue;
                }
                page_number -= 1;
            }
            BUTTON_ID_FORWARD => {
                if page_number == pagecount - 1 {
                    continue;
                }
                page_number += 1;
            }
            BUTTON_ID_LAST => {
                if page_number == pagecount - 1 {
                    continue;
                }
                page_number = pagecount - 1;
            }
            _ => continue,
        };

        bot_message
            .edit(
                ctx,
                poise::CreateReply::default()
                    .embed(assemble_paginated_moth_search_embed(
                        &moths_found,
                        moth_count,
                        page_number,
                        pagecount,
                    ))
                    .components(&[get_pagination_buttons(page_number, pagecount)]),
            )
            .await?;
        interaction
            .create_response(
                &ctx.serenity_context().http,
                CreateInteractionResponse::Acknowledge,
            )
            .await?;
    }

    // edit out buttons after timeout
    // redoing the embed here is inefficient and unnecessary but it also doesn't really matter
    bot_message
        .edit(
            ctx,
            poise::CreateReply::default()
                .embed(assemble_paginated_moth_search_embed(
                    &moths_found,
                    moth_count,
                    page_number,
                    pagecount,
                ))
                .components(&[]),
        )
        .await?;

    Ok(())
}

fn assemble_paginated_moth_search_embed<'a>(
    moths: &Vec<&SpeciesData>,
    moth_count: usize,
    page_number: usize,
    pagecount: usize,
) -> CreateEmbed<'a> {
    let title = format!("Search found {moth_count} moths");

    let start = page_number * MOTHS_PER_PAGE;
    let mut end = start + MOTHS_PER_PAGE;
    if end >= moth_count {
        end = moth_count;
    }

    let footer = format!(
        "Page {}/{} - Showing moths {}-{}/{}",
        page_number + 1,
        pagecount,
        start + 1,
        end,
        moth_count
    );

    let moths = moths[start..end]
        .iter()
        .map(|x| {
            format!(
                "[{} {}]({}{})",
                x.classification.genus.clone(),
                x.classification.epithet.clone(),
                CATALOGUE_OF_LIFE_TAXON_URL,
                x.catalogue_of_life_taxon_id
            )
        })
        .collect::<Vec<String>>();

    return serenity::CreateEmbed::default()
        .title(title)
        .footer(CreateEmbedFooter::new(footer))
        .description(moths.join("\n"));
}

fn get_pagination_buttons<'a>(
    current_page: usize,
    last_page: usize,
) -> serenity::CreateComponent<'a> {
    let first_button = serenity::CreateButton::new(BUTTON_ID_FIRST)
        .label("⏮️")
        .disabled(current_page == 0);
    let back_button = serenity::CreateButton::new(BUTTON_ID_BACK)
        .label("◀️")
        .disabled(current_page == 0);
    let forward_button = serenity::CreateButton::new(BUTTON_ID_FORWARD)
        .label("▶️")
        .disabled(current_page == last_page - 1);
    let last_button = serenity::CreateButton::new(BUTTON_ID_LAST)
        .label("⏭️")
        .disabled(current_page == last_page - 1);
    return serenity::CreateComponent::ActionRow(serenity::CreateActionRow::Buttons(
        vec![first_button, back_button, forward_button, last_button].into(),
    ));
}

async fn assemble_moth_embed(moth: &moth_filter::SpeciesData) -> CreateEmbed<'_> {
    let reqwest_client = ReqwestClient::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
    let classifications = moth.classification.clone();

    let title = format!(
        "{} {}",
        moth.classification.genus, moth.classification.epithet
    );

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
    fields.push(("Classification", moth_rank_flow, false));

    if let Some(synonyms) = &moth.synonyms {
        let synonyms_formatted = synonyms
            .iter()
            .map(|x| {
                format!(
                    "[{} {}]({CATALOGUE_OF_LIFE_TAXON_URL}{})",
                    x.genus, x.epithet, x.catalogue_of_life_taxon_id
                )
            })
            .collect::<Vec<String>>()
            .join(", ");
        fields.push(("Synonyms", synonyms_formatted, false));
    }

    if let Some(published_in) = &moth.published_in {
        fields.push(("Published In", published_in.to_string(), false));
    }

    let species_formatted = format!(
        "{} {}",
        moth.classification.genus, moth.classification.epithet
    );

    let mut more_info_field_urls: Vec<String> = Vec::new();

    let (inaturalist_data_result, gbif_data_result) = tokio::join!(
        try_get_inaturalist_data(&reqwest_client, &species_formatted),
        try_get_gbif_data(&reqwest_client, &species_formatted),
    );

    if let Ok(inaturalist_data) = &inaturalist_data_result {
        // the iNaturalist ID will always be present but don't bother linking photos if there are none
        if inaturalist_data.photo_url.is_some() {
            fields.push((
                "Photos",
                format!("[iNaturalist]({})", inaturalist_data.inaturalist_url),
                false,
            ));
        }
        if let Some(wikipedia_url) = &inaturalist_data.wikipedia_url {
            more_info_field_urls.push(format!("[Wikipedia]({})", wikipedia_url));
        }
    }
    let thumbnail_url = match inaturalist_data_result {
        Ok(ok) => ok.photo_url.unwrap_or_default(),
        Err(_err) => "".to_string(),
    };

    if let Ok(gbif_data) = gbif_data_result {
        more_info_field_urls.push(format!("[GBIF]({GBIF_SPECIES_URL}{})", gbif_data.usage_key));
    }

    if more_info_field_urls.len() > 0 {
        fields.push(("More Info", more_info_field_urls.join("\n"), false));
    }

    let footer = serenity::CreateEmbedFooter::new(moth.catalogue_of_life_taxon_id.clone());

    return serenity::CreateEmbed::default()
        .title(title)
        .url(format!(
            "{CATALOGUE_OF_LIFE_TAXON_URL}{}",
            moth.catalogue_of_life_taxon_id
        ))
        .fields(fields)
        .thumbnail(thumbnail_url)
        .footer(footer);
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [moth(), moth_search()]
}
