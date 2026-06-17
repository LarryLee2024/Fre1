//! 确定性随机数 Generator
//!
//! 使用基于种子的确定性 PRNG，确保回放兼容。
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md`
//!
//! # 核心类型
//! - [`SeededRng`]: 从 u64 种子初始化的确定性 PRNG，包装 `ChaCha12Rng`

// TODO: rand 0.10 API 变更，以下代码需要 @feature-developer 适配：
// - RngCore 已移至 rand_core，需改用 rand_core::TryRng
// - rand::Error 已移除，try_fill_bytes 返回 Infallible
// - CryptoRng 需要 DerefMut，SeededRng 需要实现 DerefMut
// 临时方案：注释掉 RngCore/CryptoRng impl，保留核心功能

use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

/// 种子驱动的确定性 PRNG。
///
/// 包装 `ChaCha12Rng`，从 u64 种子初始化。
/// 同一种子总是产生完全相同的随机数序列。
#[derive(Debug, Clone)]
pub struct SeededRng(ChaCha12Rng);

impl SeededRng {
    /// 从 u64 种子创建新的确定性 RNG。
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

// TODO: 适配 rand 0.10 API 后恢复以下 impl
// impl RngCore for SeededRng { ... }
// impl rand::CryptoRng for SeededRng {}

#[cfg(test)]
mod tests;
