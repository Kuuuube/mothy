pub fn init() -> Vec<regex::Regex> {
    let regex_strs = include_str!("regex_filters.txt").split("\n");
    let regex_comps: Vec<regex::Regex> = regex_strs
        .clone()
        .filter(|x| x.trim().len() > 0)
        .map(|x| {
            regex::RegexBuilder::new(x)
                .case_insensitive(true)
                .build()
                .unwrap()
        })
        .collect();

    return regex_comps;
}
