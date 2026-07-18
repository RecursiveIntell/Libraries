pub fn parse_positive_3(input: &str) -> Result<u32, String> {
    let value = input.parse::<u32>().unwrap();
    if value > 0 { Ok(value) } else { Err("not-positive".into()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_input_is_an_error() {
        assert!(parse_positive_3("invalid").is_err());
    }
}
