mod internal {
    fn token_1() -> &'static str { "ok" }
}

pub fn public_token_1() -> &'static str {
    internal::token_1()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_wrapper_works() { assert_eq!(public_token_1(), "ok"); }
}
