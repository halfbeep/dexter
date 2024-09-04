// src/pool_lib.rs

#[derive(Debug)]
pub struct LiquidityPool {
    pub reserve_a: f64,
    pub reserve_b: f64,
    pub k: f64, // x * y = k formula
}

impl LiquidityPool {
    pub fn new(reserve_a: f64, reserve_b: f64) -> Self {
        let k = reserve_a * reserve_b;
        LiquidityPool {
            reserve_a,
            reserve_b,
            k,
        }
    }

    pub fn swap_a_for_b(&mut self, amount_in: f64) -> Option<f64> {
        // AMM logic for swapping token A for token B
        if amount_in <= 0.0 || self.reserve_a + amount_in == 0.0 {
            return None;
        }
        let new_reserve_a = self.reserve_a + amount_in;
        let new_reserve_b = self.k / new_reserve_a;
        let amount_out = self.reserve_b - new_reserve_b;
        if amount_out > 0.0 {
            self.reserve_a = new_reserve_a;
            self.reserve_b = new_reserve_b;
            Some(amount_out)
        } else {
            None
        }
    }

    pub fn swap_b_for_a(&mut self, amount_in: f64) -> Option<f64> {
        // AMM logic for swapping token B for token A
        if amount_in <= 0.0 || self.reserve_b + amount_in == 0.0 {
            return None;
        }
        let new_reserve_b = self.reserve_b + amount_in;
        let new_reserve_a = self.k / new_reserve_b;
        let amount_out = self.reserve_a - new_reserve_a;
        if amount_out > 0.0 {
            self.reserve_b = new_reserve_b;
            self.reserve_a = new_reserve_a;
            Some(amount_out)
        } else {
            None
        }
    }
}
