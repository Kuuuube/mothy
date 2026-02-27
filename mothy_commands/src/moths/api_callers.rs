use crate::meta::KUUUBE_SOURCE_URL;
use mothy_core::error::Error;
use reqwest::{Client as ReqwestClient, header::USER_AGENT};
use serde::Deserialize;

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
pub struct INaturalistData {
    pub inaturalist_url: String,
    pub photo_url: Option<String>,
    pub wikipedia_url: Option<String>,
}

// https://api.inaturalist.org/v1/docs/#!/Search/get_search
pub async fn try_get_inaturalist_data(
    reqwest: &ReqwestClient,
    species: &str,
) -> Result<INaturalistData, Error> {
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
    return Err(Error::Custom(
        format!("No iNaturalist results found for {species}").into(),
    ));
}

#[derive(Debug, Deserialize)]
struct GBIFResponse {
    usage: Option<GBIFResponseUsage>,
}

#[derive(Debug, Deserialize)]
struct GBIFResponseUsage {
    key: String,
}

#[derive(Debug)]
pub struct GBIFData {
    pub usage_key: String,
}

// https://techdocs.gbif.org/en/openapi/v1/species#/
pub async fn try_get_gbif_data(reqwest: &ReqwestClient, species: &str) -> Result<GBIFData, Error> {
    let response = reqwest
        .get("https://api.gbif.org/v2/species/match")
        .query(&[("scientificName", species)])
        // user agent with identifying url as requested by GBIF at https://techdocs.gbif.org/en/openapi/#rate-limits
        // "If you are integrating the GBIF API into a website or app, we highly recommend you set the HTTP `User-Agent` to a URL or email address. We can then contact you if there is a problem."
        .header("User-Agent", format!("{USER_AGENT} {KUUUBE_SOURCE_URL}"))
        .send()
        .await?
        .json::<GBIFResponse>()
        .await?;

    if let Some(gbif_usage) = response.usage {
        return Ok(GBIFData {
            usage_key: gbif_usage.key,
        });
    }

    return Err(Error::Custom(
        format!("No GBIF results found for {species}").into(),
    ));
}
