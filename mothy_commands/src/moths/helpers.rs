use crate::Error;

use moth_filter::{ButterflyBlacklist, SpeciesData};

const BUTTERFLY_SUPERFAMILY: &str = "Papilionoidea";

pub fn moth_query<'a>(
    moth_data: &'a Vec<SpeciesData>,
    query_data: &MothQuery,
) -> Result<Vec<&'a SpeciesData>, Error> {
    let mut moths_found: Vec<&SpeciesData> = moth_data
        .iter()
        .filter(|moth| {
            if !search_classification_valid(
                &query_data.superfamily,
                &moth.classification.superfamily,
            ) || !search_classification_valid(&query_data.family, &moth.classification.family)
                || !search_classification_valid(
                    &query_data.subfamily,
                    &moth.classification.subfamily,
                )
                || !search_classification_valid(&query_data.tribe, &moth.classification.tribe)
                || !search_classification_valid(&query_data.subtribe, &moth.classification.subtribe)
                || !search_classification_valid(
                    &query_data.genus,
                    &Some(&moth.classification.genus),
                )
                || !search_classification_valid(
                    &query_data.specific,
                    &Some(&moth.classification.specific),
                )
                || !search_classification_valid(
                    &query_data.subspecific,
                    &moth.classification.subspecific,
                )
            {
                return false;
            }
            true
        })
        .collect();

    if moths_found.len() == 0 {
        return Err(Error::Custom("Search found 0 moths".into()));
    }

    moths_found.sort_by(|a, b| {
        format!(
            "{} {} {}",
            a.classification.genus,
            a.classification.specific,
            a.classification.subspecific.as_deref().unwrap_or_default()
        )
        .cmp(&format!(
            "{} {} {}",
            b.classification.genus,
            b.classification.specific,
            a.classification.subspecific.as_deref().unwrap_or_default()
        ))
    });

    Ok(moths_found)
}

pub fn get_moth_rank_vec(input_strings: &[Option<String>]) -> Vec<String> {
    let mut ranks_vec = Vec::new();
    for some in input_strings.iter().flatten() {
        ranks_vec.push(some.to_string());
    }
    ranks_vec
}

pub fn search_classification_valid<A: AsRef<str>, B: AsRef<str>>(
    search_input: &Option<A>,
    check_against: &Option<B>,
) -> bool {
    if let Some(search_input_string) = &search_input {
        return check_against
            .as_ref()
            .map(|check_against_string| {
                let check_against_str: &str = check_against_string.as_ref();
                let search_input_str: &str = search_input_string.as_ref();
                check_against_str.eq_ignore_ascii_case(search_input_str)
            })
            .unwrap_or(false); // search requested on classification but moth doesnt contain classification = invalid (`false`)
    }
    true // no search requested (`search_input` == None)
}

pub fn assemble_scientific_name(genus: &str, specific: &str, subspecific: Option<&str>) -> String {
    format!("{} {} {}", genus, specific, subspecific.unwrap_or_default())
        .trim()
        .to_string()
}

pub fn is_butterfly(
    butterfly_blacklist: &ButterflyBlacklist,
    superfamily: &Option<String>,
    family: &Option<String>,
    subfamily: &Option<String>,
    tribe: &Option<String>,
    subtribe: &Option<String>,
    genus: &Option<String>,
    specific: &Option<String>,
    subspecific: &Option<String>,
) -> bool {
    if let Some(superfamily) = superfamily
        && superfamily == BUTTERFLY_SUPERFAMILY
    {
        return true;
    }
    if let Some(family) = family
        && butterfly_blacklist
            .families
            .contains(&family.to_lowercase())
    {
        return true;
    }
    if let Some(subfamily) = subfamily
        && butterfly_blacklist
            .subfamilies
            .contains(&subfamily.to_lowercase())
    {
        return true;
    }
    if let Some(tribe) = tribe
        && butterfly_blacklist.tribes.contains(&tribe.to_lowercase())
    {
        return true;
    }
    if let Some(subtribe) = subtribe
        && butterfly_blacklist
            .subtribes
            .contains(&subtribe.to_lowercase())
    {
        return true;
    }
    if let Some(genus) = genus
        && butterfly_blacklist.genera.contains(&genus.to_lowercase())
    {
        return true;
    }
    if let Some(specific) = specific
        && butterfly_blacklist
            .specifics
            .contains(&specific.to_lowercase())
    {
        return true;
    }
    if let Some(subspecific) = subspecific
        && butterfly_blacklist
            .subspecifics
            .contains(&subspecific.to_lowercase())
    {
        return true;
    }
    false
}

#[test]
fn test_is_butterfly() {
    let butterfly_blacklist = mothy_core::moth_data::moth_data_init()
        .unwrap()
        .butterfly_blacklist;
    // full database search
    assert!(
        is_butterfly(
            &butterfly_blacklist,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None
        ) == false
    );
    // moths
    assert!(
        is_butterfly(
            &butterfly_blacklist,
            &None,
            &None,
            &None,
            &None,
            &None,
            &Some("Urapteroides".to_string()),
            &Some("astheniata".to_string()),
            &None
        ) == false
    );
    assert!(
        is_butterfly(
            &butterfly_blacklist,
            &None,
            &Some("Saturniidae".to_string()),
            &None,
            &None,
            &None,
            &None,
            &None,
            &None
        ) == false
    );
    // butterflies
    assert!(
        is_butterfly(
            &butterfly_blacklist,
            &Some(BUTTERFLY_SUPERFAMILY.to_string()),
            &None,
            &None,
            &None,
            &None,
            &None,
            &None,
            &None
        ) == true
    );
    assert!(
        is_butterfly(
            &butterfly_blacklist,
            &None,
            &None,
            &None,
            &None,
            &None,
            &Some("Danaus".to_string()),
            &Some("plexippus".to_string()),
            &None
        ) == true
    );
    assert!(
        is_butterfly(
            &butterfly_blacklist,
            &None,
            &Some("Hesperiidae".to_string()),
            &None,
            &None,
            &None,
            &None,
            &None,
            &None
        ) == true
    );
    assert!(
        is_butterfly(
            &butterfly_blacklist,
            &None,
            &Some("Lycaenidae".to_string()),
            &None,
            &None,
            &None,
            &None,
            &None,
            &None
        ) == true
    );
}

pub struct MothQuery {
    pub superfamily: Option<String>,
    pub family: Option<String>,
    pub subfamily: Option<String>,
    pub tribe: Option<String>,
    pub subtribe: Option<String>,
    pub genus: Option<String>,
    pub specific: Option<String>,
    pub subspecific: Option<String>,
}
