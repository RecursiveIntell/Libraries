mod internal {
    fn token_2() -> &'static str { "ok" }
}

pub fn public_token_2() -> &'static str {
    internal::token_2()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_wrapper_works() { assert_eq!(public_token_2(), "ok"); }
}
