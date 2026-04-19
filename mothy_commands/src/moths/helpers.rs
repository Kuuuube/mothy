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
                || query_data.common_name.is_some() && moth.common_names.is_none()
            {
                return false;
            }
            if let Some(query_common_name) = &query_data.common_name
                && let Some(moth_common_names) = &moth.common_names
            {
                if query_data.exact_common_name_search {
                    return moth_common_names.iter().any(|common_name| {
                        search_classification_valid(&Some(query_common_name), &Some(common_name))
                    });
                } else {
                    return moth_common_names.iter().any(|common_name| {
                        common_name
                            .to_ascii_lowercase()
                            .contains(&query_common_name.to_ascii_lowercase())
                    });
                }
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
    if search_input.is_none() {
        return true;
    }
    if check_against.is_none() {
        return false;
    }

    if let Some(search_input_string) = search_input
        && let Some(check_against_string) = check_against
        && check_against_string
            .as_ref()
            .eq_ignore_ascii_case(search_input_string.as_ref())
    {
        return true;
    }

    false
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
    pub common_name: Option<String>,
    pub exact_common_name_search: bool,
}

const SMALL_WORDS: &[(&str, &str)] = &[
    (" A ", " a "),
    (" An ", " an "),
    (" And ", " and "),
    (" As ", " as "),
    (" At ", " at "),
    (" But ", " but "),
    (" By ", " by "),
    (" En ", " en "),
    (" For ", " for "),
    (" If ", " if "),
    (" In ", " in "),
    (" Of ", " of "),
    (" On ", " on "),
    (" Or ", " or "),
    (" The ", " the "),
    (" To ", " to "),
    (" Via ", " via "),
    (" Vs ", " vs "),
];

pub fn title_case(input_string: String) -> String {
    let mut input_string_title_naive = input_string;
    title_case_ascii_mut(&mut input_string_title_naive);

    for small_word in SMALL_WORDS {
        input_string_title_naive = input_string_title_naive.replace(small_word.0, small_word.1);
    }

    input_string_title_naive
}

/// Naive title case, does not correctly handle small words such as `a`, `an`, `the`
fn title_case_ascii_mut(input_str: &mut str) {
    let bytes = unsafe { input_str.as_bytes_mut() };
    let mut last_whitespace = true;
    for byte in bytes.iter_mut() {
        // Below 0x80 is invalid when used in a unicode multi byte sequence
        // This ensures ascii characters cannot be contained in unicode bytes
        // `is_ascii` is safe even on a byte-by-byte level over a unicode string
        if !byte.is_ascii() {
            last_whitespace = false;
            continue;
        }

        let current_whitespace = byte.is_ascii_whitespace() || byte.is_ascii_punctuation();
        if last_whitespace && !current_whitespace {
            *byte = byte.to_ascii_uppercase();
        } else {
            // `to_ascii_lowercase` runs `is_ascii_uppercase` for us, no need to check
            *byte = byte.to_ascii_lowercase();
        }

        last_whitespace = current_whitespace;
    }
}

#[test]
fn test_title_case() {
    let test_cases = vec![
        ("test", "Test"),
        ("this is a title", "This Is a Title"),
        (
            "a letter a should be capitalized at the start only",
            "A Letter a Should Be Capitalized at the Start Only",
        ),
        (
            "fabulous green sphinx of kauai",
            "Fabulous Green Sphinx of Kauai",
        ),
        ("TOTALLY UPPERCASE STRING!!!", "Totally Uppercase String!!!"),
        (
            "what if the letter \"a\" is quoted?",
            "What if the Letter \"A\" Is Quoted?",
        ), // not totally sure about the handling of this one
        ("silkworm", "Silkworm"),
        (
            "?did \"you\" know\" that\' \'some\' moths~ can. swim! underwater?",
            "?Did \"You\" Know\" That\' \'Some\' Moths~ Can. Swim! Underwater?",
        ),
        (
            "one moth. two moth. three moth.",
            "One Moth. Two Moth. Three Moth.",
        ),
        ("uPdOwNaNdUpDoWn AnDuPdOwN", "Updownandupdown Andupdown"),
    ];

    for test_case in test_cases {
        dbg!(test_case);
        let title_case_string = title_case(test_case.0.to_string());
        dbg!(&title_case_string);
        assert!(title_case_string == test_case.1.to_string());
    }
}

pub fn dequote(input_str: &str) -> String {
    return input_str
        .trim_start_matches('\'')
        .trim_end_matches('\'')
        .trim_start_matches('\"')
        .trim_end_matches('\"')
        .to_string();
}
