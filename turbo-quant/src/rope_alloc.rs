use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RopeBlockBudget {
    pub layer: usize,
    pub head: usize,
    pub block_bits: Vec<u8>,
    pub energy: Vec<f32>,
}

impl RopeBlockBudget {
    pub fn allocate(
        layer: usize,
        head: usize,
        energy: &[f32],
        min_bits: u8,
        max_bits: u8,
        total_budget_bits: usize,
    ) -> Self {
        assert!(min_bits <= max_bits);
        if energy.is_empty() {
            return Self {
                layer,
                head,
                block_bits: Vec::new(),
                energy: Vec::new(),
            };
        }
        let mut block_bits = vec![min_bits; energy.len()];
        let min_total = min_bits as usize * energy.len();
        let mut remaining = total_budget_bits.saturating_sub(min_total);
        let mut order: Vec<usize> = (0..energy.len()).collect();
        order.sort_by(|&a, &b| {
            energy[b]
                .partial_cmp(&energy[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        while remaining > 0 {
            let mut changed = false;
            for &idx in &order {
                if remaining == 0 {
                    break;
                }
                if block_bits[idx] < max_bits {
                    block_bits[idx] += 1;
                    remaining -= 1;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        Self {
            layer,
            head,
            block_bits,
            energy: energy.to_vec(),
        }
    }

    pub fn total_bits(&self) -> usize {
        self.block_bits.iter().map(|&b| b as usize).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_energy_blocks_receive_at_least_low_energy_bits() {
        let budget = RopeBlockBudget::allocate(0, 1, &[0.1, 2.0, 0.4, 1.0], 2, 6, 14);
        assert_eq!(budget.total_bits(), 14);
        assert!(budget.block_bits[1] >= budget.block_bits[0]);
        assert!(budget.block_bits[1] >= budget.block_bits[2]);
    }
}
