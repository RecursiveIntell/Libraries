//! Scope-test fixture — provides diverse scope targets for CEA edge generation.

/// A public struct to test struct-scope patches.
pub struct TestStruct {
    pub value: i32,
}

impl TestStruct {
    /// Creates a new TestStruct.
    pub fn new(value: i32) -> Self {
        TestStruct { value }
    }

    /// Returns the value.
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Adds to the value.
    pub fn add(&mut self, delta: i32) {
        self.value += delta;
    }
}

/// A public enum for enum-scope patches.
pub enum TestEnum {
    Alpha,
    Beta(i32),
    Gamma { name: String },
}

/// A public trait for trait-scope patches.
pub trait TestTrait {
    fn describe(&self) -> String;
    fn is_valid(&self) -> bool;
}

impl TestTrait for TestStruct {
    fn describe(&self) -> String {
        format!("TestStruct({})", self.value)
    }

    fn is_valid(&self) -> bool {
        self.value > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_struct() {
        let ts = TestStruct::new(42);
        assert_eq!(ts.get_value(), 42);
    }

    #[test]
    fn test_add() {
        let mut ts = TestStruct::new(10);
        ts.add(5);
        assert_eq!(ts.get_value(), 15);
    }

    #[test]
    fn test_trait() {
        let ts = TestStruct::new(1);
        assert!(ts.is_valid());
        assert_eq!(ts.describe(), "TestStruct(1)");
    }
}
