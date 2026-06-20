//! 确定性随机数 Generator
//!
//! 使用基于种子的确定性 PRNG，确保回放兼容。
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md`
//!
//! # 核心类型
//! - [`SeededRng`]: 从 u64 种子初始化的确定性 PRNG，包装 `ChaCha12Rng`
//! - [`GameRng`]: Bevy Resource，全局可访问的确定性 RNG

use bevy::prelude::*;
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

/// 全局游戏确定性 RNG Resource。
///
/// 在 App 启动时初始化，所有需要随机数的系统通过 `Res<GameRng>` 访问。
/// 种子在创建战斗/存档时设定，确保 Replay 一致性。
///
/// # 使用示例
/// ```rust,ignore
/// fn damage_system(mut rng: ResMut<GameRng>) {
///     let roll = rng.gen_range(1, 21); // d20
/// }
/// ```
#[derive(Resource, Debug)]
pub struct GameRng {
    inner: SeededRng,
}

impl GameRng {
    /// 从种子创建 GameRng。
    pub fn new(seed: u64) -> Self {
        Self {
            inner: SeededRng::new(seed),
        }
    }

    /// 生成 u64 范围内的随机数。
    pub fn gen_range(&mut self, min: u64, max: u64) -> u64 {
        use rand::RngExt;
        self.inner.as_mut().random_range(min..max)
    }

    /// 生成 f32 范围内的随机数 [min, max)。
    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        use rand::RngExt;
        self.inner.as_mut().random_range(min..max)
    }

    /// 生成布尔值（给定概率为 true）。
    pub fn gen_bool(&mut self, probability: f32) -> bool {
        use rand::RngExt;
        self.inner.as_mut().random_bool(probability as f64)
    }

    /// 重置种子（用于存档加载后恢复状态）。
    pub fn reseed(&mut self, seed: u64) {
        self.inner = SeededRng::new(seed);
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new(42)
    }
}

// TODO[P2][SHARED][2026-06-20]: rand 0.10 API 变更，以下代码需要 @feature-developer 适配：
// - RngCore 已移至 rand_core，需改用 rand_core::TryRng
// - rand::Error 已移除，try_fill_bytes 返回 Infallible
// - CryptoRng 需要 DerefMut，SeededRng 需要实现 DerefMut
// 临时方案：注释掉 RngCore/CryptoRng impl，保留核心功能

#[cfg(test)]
mod tests;
