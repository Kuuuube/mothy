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
