mod internal {
    fn token_3() -> &'static str { "ok" }
}

pub fn public_token_3() -> &'static str {
    internal::token_3()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_wrapper_works() { assert_eq!(public_token_3(), "ok"); }
}
