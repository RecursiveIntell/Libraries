pub fn value() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    #[test]
    fn value_is_one() {
        assert_eq!(super::value(), 1);
    }
}
