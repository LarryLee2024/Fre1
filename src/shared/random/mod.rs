//! 确定性随机数 Generator
//!
//! 使用基于种子的确定性 PRNG，确保回放兼容。
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md`
//!
//! # 核心类型
//! - [`SeededRng`]: 从 u64 种子初始化的确定性 PRNG，包装 `ChaCha12Rng`
//! - [`RngStream`]: RNG 流枚举——按用途拆分独立流
//! - [`RngSeeds`]: 4 流种子集合
//! - [`DeterministicRng`]: 4 流确定性 RNG，每流一个独立 ChaCha12 实例

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

/// RNG 流枚举——按用途拆分独立流，互不干扰。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum RngStream {
    /// 战斗（命中/暴击/伤害浮动）
    Combat,
    /// 掉落/制造随机
    Drop,
    /// AI 决策随机
    AI,
    /// 世界事件随机
    World,
}

impl RngStream {
    /// 返回流名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Combat => "Combat",
            Self::Drop => "Drop",
            Self::AI => "AI",
            Self::World => "World",
        }
    }

    /// 所有流的列表。
    pub fn all() -> [Self; 4] {
        [Self::Combat, Self::Drop, Self::AI, Self::World]
    }
}

/// RNG 种子集合——每个流独立种子。
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct RngSeeds {
    /// 战斗种子
    pub combat_seed: u64,
    /// 掉落种子
    pub drop_seed: u64,
    /// AI 种子
    pub ai_seed: u64,
    /// 世界种子
    pub world_seed: u64,
}

impl RngSeeds {
    /// 创建统一的种子集合（所有流使用偏移种子确保互不干扰）。
    pub fn uniform(seed: u64) -> Self {
        Self {
            combat_seed: seed,
            drop_seed: seed.wrapping_add(1),
            ai_seed: seed.wrapping_add(2),
            world_seed: seed.wrapping_add(3),
        }
    }

    /// 创建独立种子的集合。
    pub fn new(combat: u64, drop: u64, ai: u64, world: u64) -> Self {
        Self {
            combat_seed: combat,
            drop_seed: drop,
            ai_seed: ai,
            world_seed: world,
        }
    }

    /// 获取指定流的种子。
    pub fn get(&self, stream: RngStream) -> u64 {
        match stream {
            RngStream::Combat => self.combat_seed,
            RngStream::Drop => self.drop_seed,
            RngStream::AI => self.ai_seed,
            RngStream::World => self.world_seed,
        }
    }

    /// 设置指定流的种子。
    pub fn set(&mut self, stream: RngStream, seed: u64) {
        match stream {
            RngStream::Combat => self.combat_seed = seed,
            RngStream::Drop => self.drop_seed = seed,
            RngStream::AI => self.ai_seed = seed,
            RngStream::World => self.world_seed = seed,
        }
    }
}

/// 4 流确定性 RNG——每流一个独立 ChaCha12 实例。
///
/// 所有业务随机操作通过此资源进行，确保回放确定性。
/// 使用经过验证的 ChaCha12 CSPRNG，而非自制 hash PRNG。
///
/// 详见 ADR-041 §3
#[derive(Debug, Clone, Resource)]
pub struct DeterministicRng {
    /// 各流当前种子
    seeds: RngSeeds,
    /// 战斗流
    combat: SeededRng,
    /// 掉落流
    drop: SeededRng,
    /// AI 流
    ai: SeededRng,
    /// 世界流
    world: SeededRng,
}

impl DeterministicRng {
    /// 使用 RngSeeds 创建确定性 RNG。
    pub fn new(seeds: RngSeeds) -> Self {
        Self {
            seeds,
            combat: SeededRng::new(seeds.combat_seed),
            drop: SeededRng::new(seeds.drop_seed),
            ai: SeededRng::new(seeds.ai_seed),
            world: SeededRng::new(seeds.world_seed),
        }
    }

    /// 使用统一初始种子创建。
    pub fn with_seed(seed: u64) -> Self {
        Self::new(RngSeeds::uniform(seed))
    }

    /// 获取指定流的可变 SeededRng 引用。
    pub fn stream(&mut self, stream: RngStream) -> &mut SeededRng {
        match stream {
            RngStream::Combat => &mut self.combat,
            RngStream::Drop => &mut self.drop,
            RngStream::AI => &mut self.ai,
            RngStream::World => &mut self.world,
        }
    }

    /// 获取指定流的种子。
    pub fn get_seed(&self, stream: RngStream) -> u64 {
        self.seeds.get(stream)
    }

    /// 设置指定流的种子并重置该流状态。
    pub fn set_seed(&mut self, stream: RngStream, seed: u64) {
        self.seeds.set(stream, seed);
        *self.stream(stream) = SeededRng::new(seed);
    }

    /// 获取所有流种子。
    pub fn get_all_seeds(&self) -> RngSeeds {
        self.seeds
    }

    /// 同步设置所有流种子（回放模式），重置所有流状态。
    pub fn set_all_seeds(&mut self, seeds: RngSeeds) {
        self.seeds = seeds;
        self.combat = SeededRng::new(seeds.combat_seed);
        self.drop = SeededRng::new(seeds.drop_seed);
        self.ai = SeededRng::new(seeds.ai_seed);
        self.world = SeededRng::new(seeds.world_seed);
    }

    /// 生成指定流的下一个 u64 伪随机数。
    pub fn next_u64(&mut self, stream: RngStream) -> u64 {
        use rand::RngExt;
        self.stream(stream).as_mut().random()
    }

    /// 生成指定流的 f32 伪随机数（0.0..1.0）。
    pub fn next_f32(&mut self, stream: RngStream) -> f32 {
        use rand::RngExt;
        self.stream(stream).as_mut().random_range(0.0f32..1.0)
    }

    /// 生成指定流的 bool 伪随机数（给定概率）。
    pub fn gen_bool(&mut self, stream: RngStream, probability: f32) -> bool {
        use rand::RngExt;
        self.stream(stream).as_mut().random_bool(probability as f64)
    }

    /// 生成指定流在 [min, max) 范围内的整数。
    pub fn gen_range(&mut self, stream: RngStream, min: u64, max: u64) -> u64 {
        use rand::RngExt;
        if min >= max {
            return min;
        }
        self.stream(stream).as_mut().random_range(min..max)
    }
}

impl FromWorld for DeterministicRng {
    fn from_world(_: &mut World) -> Self {
        Self::with_seed(0)
    }
}

#[cfg(test)]
mod tests;
