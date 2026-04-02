use crate::structs::RegexFilters;

pub fn init() -> RegexFilters {
    let regex_strs = include_str!("regex_link_filters.txt").split("\n");
    let regex_comps: Vec<regex::Regex> = regex_strs
        .filter(|x| !x.trim().is_empty())
        .map(|x| {
            regex::RegexBuilder::new(x)
                .case_insensitive(true)
                .build()
                .unwrap()
        })
        .collect();

    let links_regex = regex::RegexBuilder::new(r"https?://[^\s]*?(\s|$)")
        .case_insensitive(true)
        .build()
        .unwrap();

    RegexFilters {
        links_detector: links_regex,
        links_blacklist: regex_comps,
    }
}
