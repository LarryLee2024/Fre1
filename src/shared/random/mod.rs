//! 确定性随机数 Generator
//!
//! 使用基于种子的确定性 PRNG，确保回放兼容。
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md`
//!
//! # 核心类型
//! - [`SeededRng`]: 从 u64 种子初始化的确定性 PRNG，包装 `ChaCha12Rng`

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;

/// 种子驱动的确定性 PRNG。
///
/// 包装 `ChaCha12Rng`，从 u64 种子初始化。
/// 同一种子总是产生完全相同的随机数序列。
///
/// # 使用
///
/// ```ignore
/// use fre_shared::random::SeededRng;
/// use rand::Rng;
///
/// let mut rng = SeededRng::new(42);
/// let roll = rng.gen_range(1..=20);
/// let crit = rng.gen_bool(0.05);
/// ```
#[derive(Debug, Clone)]
pub struct SeededRng(ChaCha12Rng);

impl SeededRng {
    /// 从 u64 种子创建新的确定性 RNG。
    ///
    /// 同一种子总是产生完全相同的结果序列。
    pub fn new(seed: u64) -> Self {
        Self(ChaCha12Rng::seed_from_u64(seed))
    }

    /// 从 32 字节种子创建新的确定性 RNG。
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(ChaCha12Rng::from_seed(seed))
    }

    /// 获取内部 `ChaCha12Rng` 的可变引用。
    pub fn as_mut(&mut self) -> &mut ChaCha12Rng {
        &mut self.0
    }

    /// 消费自身，返回内部 `ChaCha12Rng`。
    pub fn into_inner(self) -> ChaCha12Rng {
        self.0
    }
}

impl RngCore for SeededRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_fill_bytes(dest)
    }
}

/// `SeededRng` 使用 ChaCha12（密码学安全 PRNG），标记为 `CryptoRng`。
///
/// 虽然游戏 RNG 不需要密码学安全性，ChaCha12 的确定性保证
/// 和平台一致性使其成为回放确定性 RNG 的理想选择。
impl rand::CryptoRng for SeededRng {}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = SeededRng::new(42);
        let mut b = SeededRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seed_produces_different_sequence() {
        let mut a = SeededRng::new(42);
        let mut b = SeededRng::new(43);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn gen_range_works() {
        let mut rng = SeededRng::new(99);
        for _ in 0..100 {
            let val = rng.gen_range(1..=20);
            assert!((1..=20).contains(&val));
        }
    }

    #[test]
    fn gen_bool_works() {
        let mut rng = SeededRng::new(7);
        // Just ensure it doesn't panic - produces both true/false
        let mut had_true = false;
        let mut had_false = false;
        for _ in 0..1000 {
            if rng.gen_bool(0.5) {
                had_true = true;
            } else {
                had_false = true;
            }
        }
        assert!(had_true && had_false);
    }

    #[test]
    fn fill_bytes_is_deterministic() {
        let mut a = SeededRng::new(0);
        let mut b = SeededRng::new(0);
        let mut buf_a = [0u8; 32];
        let mut buf_b = [0u8; 32];
        a.fill_bytes(&mut buf_a);
        b.fill_bytes(&mut buf_b);
        assert_eq!(buf_a, buf_b);
    }
}
