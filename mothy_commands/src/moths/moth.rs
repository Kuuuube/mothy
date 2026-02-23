use crate::{Context, Error};
use poise::serenity_prelude as serenity;

use ::serenity::all::CreateEmbed;
use rand::seq::IndexedRandom;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;

const CATALOGUE_OF_LIFE_TAXON_URL: &str = "https://www.catalogueoflife.org/data/taxon/";
const BUTTERFLY_SUPERFAMILY: &str = "Papilionoidea";

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
    // ugly lepidoptera searching is not allowed (butteryflies)
    if let Some(superfamily_some) = &superfamily
        && superfamily_some.to_lowercase() == BUTTERFLY_SUPERFAMILY.to_lowercase()
    {
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
            uppercase_genus.get_mut(0..1).and_then(|x| Some(x.make_ascii_uppercase()));
            let embed = serenity::CreateEmbed::default()
                .description(format!(
                    "Failed to find moth `{} {}`.",
                    uppercase_genus, epithet_some.to_lowercase()
                ))
                .color(serenity::Colour::from_rgb(255, 0, 0));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        return Ok(());
    }

    // wide search
    let matches = &data.moth_data.iter().filter(|moth| true);

    Ok(())
}

async fn assemble_moth_embed(moth: &moth_filter::SpeciesData) -> CreateEmbed<'_> {
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

    let inaturalist_data_result = try_get_inaturalist_data(&format!(
        "{} {}",
        moth.classification.genus, moth.classification.epithet
    ))
    .await;
    if let Ok(inaturalist_data) = &inaturalist_data_result {
        fields.push((
            "Photos",
            format!("[iNaturalist]({})", inaturalist_data.inaturalist_url),
            false,
        ));
        if let Some(wikipedia_url) = &inaturalist_data.wikipedia_url {
            fields.push((
                "More Info",
                format!("[Wikipedia]({})", wikipedia_url),
                false,
            ));
        }
    }
    let thumbnail_url = match inaturalist_data_result {
        Ok(ok) => ok.photo_url.unwrap_or_default(),
        Err(_err) => "".to_string(),
    };

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

#[derive(Debug, Deserialize)]
struct INaturalistResponse {
    results: Vec<INaturalistResponseRecord>,
}
#[derive(Debug, Deserialize)]
struct INaturalistResponseRecord {
    record: INaturalistResponsePhoto,
}
#[derive(Debug, Deserialize)]
struct INaturalistResponsePhoto {
    id: i128,
    default_photo: Option<INaturalistResponseDefaultPhoto>,
    wikipedia_url: Option<String>,
}
#[derive(Debug, Deserialize)]
struct INaturalistResponseDefaultPhoto {
    medium_url: String,
}
#[derive(Debug)]
struct INaturalistData {
    inaturalist_url: String,
    photo_url: Option<String>,
    wikipedia_url: Option<String>,
}

// https://api.inaturalist.org/v1/docs/#!/Search/get_search
async fn try_get_inaturalist_data(species: &str) -> Result<INaturalistData, Error> {
    let reqwest = ReqwestClient::new();
    let response = reqwest
        .get("https://api.inaturalist.org/v1/search")
        .query(&[
            ("q", species),
            ("sources", "taxa"),
            ("include_taxon_ancestors", "false"),
        ])
        .send()
        .await?
        .json::<INaturalistResponse>()
        .await?;

    if let Some(first_result) = response.results.get(0) {
        return Ok(INaturalistData {
            inaturalist_url: format!(
                "https://www.inaturalist.org/taxa/{}",
                first_result.record.id
            )
            .to_string(),
            photo_url: first_result
                .record
                .default_photo
                .as_ref()
                .map(|x| x.medium_url.clone()),
            wikipedia_url: first_result.record.wikipedia_url.clone(),
        });
    }
    return Err(Error::Custom("No results found".into()));
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
pub fn commands() -> [crate::Command; 2] {
    [moth(), moth_search()]
}
