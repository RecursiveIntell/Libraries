pub fn duplicate_2(input: &str) -> (String, String) {
    let owned = input.to_string();
    let moved = owned;
    (owned, moved)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicates_owned_value() {
        assert_eq!(duplicate_2("v"), ("v".into(), "v".into()));
    }
}
