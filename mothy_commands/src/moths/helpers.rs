use moth_filter::ButterflyBlacklist;

const BUTTERFLY_SUPERFAMILY: &str = "Papilionoidea";

pub fn get_moth_rank_vec(input_strings: &[Option<String>]) -> Vec<String> {
    let mut ranks_vec = Vec::new();
    for input_string in input_strings {
        if let Some(some) = input_string {
            ranks_vec.push(some.to_string());
        }
    }
    return ranks_vec;
}

pub fn search_classification_valid<A: AsRef<str>, B: AsRef<str>>(
    search_input: &Option<A>,
    check_against: &Option<B>,
) -> bool {
    if let Some(search_input_string) = &search_input {
        return check_against
            .as_ref()
            .and_then(|check_against_string| {
                let check_against_str: &str = check_against_string.as_ref();
                let search_input_str: &str = search_input_string.as_ref();
                Some(check_against_str.eq_ignore_ascii_case(search_input_str))
            })
            .unwrap_or(false); // search requested on classification but moth doesnt contain classification = invalid (`false`)
    }
    return true; // no search requested (`search_input` == None)
}

pub fn is_butterfly(
    butterfly_blacklist: &ButterflyBlacklist,
    superfamily: &Option<String>,
    family: &Option<String>,
    subfamily: &Option<String>,
    tribe: &Option<String>,
    subtribe: &Option<String>,
    genus: &Option<String>,
    epithet: &Option<String>,
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
    if let Some(epithet) = epithet
        && butterfly_blacklist
            .epithets
            .contains(&epithet.to_lowercase())
    {
        return true;
    }
    return false;
}
