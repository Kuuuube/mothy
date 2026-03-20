use std::time::Duration;

use reqwest::Client as ReqwestClient;

use crate::{moths::api_callers::*, moths::helpers::*};
use moth_filter::SpeciesData;
use poise::serenity_prelude as serenity;

use ::serenity::all::{CreateEmbed, CreateEmbedFooter};

const CATALOGUE_OF_LIFE_TAXON_URL: &str = "https://www.catalogueoflife.org/data/taxon/";
const GBIF_SPECIES_URL: &str = "https://www.gbif.org/species/";

const MOTHS_PER_PAGE: usize = 10;

pub async fn assemble_moth_embed<'a>(moth: &moth_filter::SpeciesData) -> CreateEmbed<'a> {
    let reqwest_client = ReqwestClient::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
    let classifications = moth.classification.clone();

    let species_formatted = assemble_scientific_name(
        &moth.classification.genus,
        &moth.classification.specific,
        moth.classification.subspecific.as_deref(),
    );

    let (inaturalist_data_result, gbif_data_result) = tokio::join!(
        try_get_inaturalist_data(&reqwest_client, &species_formatted),
        try_get_gbif_data(&reqwest_client, &species_formatted),
    );

    let title = species_formatted;

    let mut fields: Vec<(String, String, bool)> = vec![];

    if let Some(common_names) = &moth.common_names {
        fields.push(("Common Names".to_string(), common_names.join(", "), false));
    } else if let Ok(ref inaturalist_data) = inaturalist_data_result
        && let Some(common_name) = &inaturalist_data.preferred_common_name
    {
        fields.push(("Common Names".to_string(), common_name.to_string(), false));
    }

    let moth_rank_flow = get_moth_rank_vec(&[
        classifications.superfamily,
        classifications.family,
        classifications.subfamily,
        classifications.tribe,
        classifications.subtribe,
        Some(classifications.genus),
        Some(classifications.specific),
        classifications.subspecific,
    ])
    .join(" -> ");
    fields.push(("Classification".to_string(), moth_rank_flow, false));

    if let Some(synonyms) = &moth.synonyms {
        let synonyms_formatted = synonyms
            .iter()
            .map(|x| {
                format!(
                    "[{}]({CATALOGUE_OF_LIFE_TAXON_URL}{})",
                    assemble_scientific_name(&x.genus, &x.specific, x.subspecific.as_deref()),
                    x.catalogue_of_life_taxon_id
                )
            })
            .collect::<Vec<String>>();

        let synonym_fields = create_sized_fields("Synonyms", synonyms_formatted, ", ");
        fields = [fields, synonym_fields].concat();
    }

    if let Some(subspecies) = &moth.subspecies {
        let subspecies_formatted = subspecies
            .iter()
            .map(|x| {
                format!(
                    "[{}]({CATALOGUE_OF_LIFE_TAXON_URL}{})",
                    assemble_scientific_name(&x.genus, &x.specific, Some(&x.subspecific)),
                    x.catalogue_of_life_taxon_id
                )
            })
            .collect::<Vec<String>>()
            .join(", ");
        fields.push(("Subspecies".to_string(), subspecies_formatted, false));
    }

    if let Some(published_in) = &moth.published_in {
        fields.push(("Published In".to_string(), published_in.to_string(), false));
    }

    let mut more_info_field_urls: Vec<String> = Vec::new();

    if let Ok(inaturalist_data) = &inaturalist_data_result {
        // the iNaturalist ID will always be present but don't bother linking photos if there are none
        if inaturalist_data.photo_url.is_some() {
            fields.push((
                "Photos".to_string(),
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
        fields.push((
            "More Info".to_string(),
            more_info_field_urls.join("\n"),
            false,
        ));
    }

    println!("{:?}", fields);

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

pub fn assemble_paginated_moth_search_embed<'a>(
    moths: &Vec<&SpeciesData>,
    moth_count: usize,
    page_number: usize,
    pagecount: usize,
    selected_moth: Option<usize>,
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

    let mut moths = moths[start..end]
        .iter()
        .map(|x| {
            format!(
                "[{}]({}{})",
                assemble_scientific_name(
                    &x.classification.genus,
                    &x.classification.specific,
                    x.classification.subspecific.as_deref()
                ),
                CATALOGUE_OF_LIFE_TAXON_URL,
                x.catalogue_of_life_taxon_id
            )
        })
        .collect::<Vec<String>>();

    if let Some(selected_moth) = selected_moth {
        if let Some(moth_entry) = moths.get_mut(selected_moth) {
            *moth_entry = format!("**{moth_entry}** ⬅︎");
        }
    }

    return serenity::CreateEmbed::default()
        .title(title)
        .footer(CreateEmbedFooter::new(footer))
        .description(moths.join("\n"));
}

const MAX_FIELD_SIZE: usize = 1024;
fn create_sized_fields<'a>(
    field_name: &'a str,
    field_contents_split: Vec<String>,
    delimiter: &str,
) -> Vec<(String, String, bool)> {
    let mut fields = Vec::new();
    let mut field_count = 0;
    let mut current_field_size = 0;

    let mut current_field_content = Vec::new();

    for field_content in field_contents_split {
        if field_content.len() > MAX_FIELD_SIZE {
            continue;
        }

        if current_field_size + field_content.len() + delimiter.len() > MAX_FIELD_SIZE {
            let field_name = match field_count {
                0 => field_name,
                _ => "",
            };
            fields.push((
                field_name.to_string(),
                current_field_content.join(delimiter),
                false,
            ));
            field_count += 1;
            current_field_content = Vec::new();
            current_field_size = 0;
        } else {
            current_field_size += field_content.len() + delimiter.len();
            current_field_content.push(field_content);
        }
    }

    return fields;
}
