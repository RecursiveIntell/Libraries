//! Budget, retry, fanout, and stop-rule guards.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetV1 {
    pub max_tool_calls: u32,
    pub max_retries: u32,
    pub max_turn_millis: u64,
}

impl Default for BudgetV1 {
    fn default() -> Self {
        Self {
            max_tool_calls: 8,
            max_retries: 2,
            max_turn_millis: 30_000,
        }
    }
}

impl BudgetV1 {
    pub fn allows_tool_call(&self, tool_calls_so_far: u32, requested_tool_calls: u32) -> bool {
        tool_calls_so_far.saturating_add(requested_tool_calls) <= self.max_tool_calls
    }

    pub fn allows_retry(&self, retries_so_far: u32) -> bool {
        retries_so_far < self.max_retries
    }

    pub fn deadline_exceeded(&self, elapsed_millis: u64) -> bool {
        elapsed_millis > self.max_turn_millis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_call_budget_is_cumulative() {
        let budget = BudgetV1 {
            max_tool_calls: 2,
            max_retries: 1,
            max_turn_millis: 1000,
        };

        assert!(budget.allows_tool_call(1, 1));
        assert!(!budget.allows_tool_call(2, 1));
    }

    #[test]
    fn deadline_check_is_explicit() {
        let budget = BudgetV1 {
            max_tool_calls: 1,
            max_retries: 1,
            max_turn_millis: 10,
        };

        assert!(!budget.deadline_exceeded(10));
        assert!(budget.deadline_exceeded(11));
    }
}
