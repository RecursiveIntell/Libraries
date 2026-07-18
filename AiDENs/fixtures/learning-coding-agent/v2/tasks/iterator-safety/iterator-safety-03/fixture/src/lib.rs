pub fn even_sum_3(values: &[u32]) -> u32 {
    values.iter().copied().filter(|value| value % 2 == 1).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sums_only_even_values() {
        assert_eq!(even_sum_3(&[1, 2, 3, 4]), 6);
    }
}
