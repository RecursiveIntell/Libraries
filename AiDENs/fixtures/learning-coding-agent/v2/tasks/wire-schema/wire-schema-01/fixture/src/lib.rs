pub fn encode_user_1(name: &str) -> String {
    format!(r#"{{"userName":"{}"}}"#, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_snake_case_wire_name() {
        assert_eq!(encode_user_1("Ada"), r#"{"user_name":"Ada"}"#);
    }
}
