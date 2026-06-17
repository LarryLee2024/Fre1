//! 确定性测试 RNG
//!
//! 提供固定种子的伪随机数生成，确保测试可复现。
//! Seed = 42 为默认值。

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// 确定性 RNG 包装器，固定种子保证跨平台一致。
pub struct DeterministicRng {
    rng: StdRng,
}

impl DeterministicRng {
    /// 使用默认种子（42）创建。
    pub fn new() -> Self {
        Self::with_seed(42)
    }

    /// 使用指定种子创建。
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// 生成 u32 范围内的随机数。
    pub fn gen_range(&mut self, low: u32, high: u32) -> u32 {
        self.rng.random_range(low..high)
    }

    /// 生成 f32 随机数 [0.0, 1.0)。
    pub fn gen_f32(&mut self) -> f32 {
        self.rng.random()
    }

    /// 生成 bool，指定概率为 true。
    pub fn gen_bool(&mut self, probability: f64) -> bool {
        self.rng.random_bool(probability)
    }

    /// 填充字节缓冲区。
    pub fn fill_bytes(&mut self, buf: &mut [u8]) {
        self.rng.fill(buf);
    }
}

impl Default for DeterministicRng {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = DeterministicRng::with_seed(42);
        let mut b = DeterministicRng::with_seed(42);
        for _ in 0..100 {
            assert_eq!(a.gen_range(1, 21), b.gen_range(1, 21));
        }
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut a = DeterministicRng::with_seed(42);
        let mut b = DeterministicRng::with_seed(43);
        assert_ne!(a.gen_range(1, 1001), b.gen_range(1, 1001));
    }

    #[test]
    fn default_seed_is_42() {
        let mut a = DeterministicRng::new();
        let mut b = DeterministicRng::with_seed(42);
        assert_eq!(a.gen_range(0, 100), b.gen_range(0, 100));
    }
}
