//! 测试用确定性 RNG
//!
//! 提供固定种子的伪随机数生成，确保测试可复现。
//! Seed = 42 为默认值。
//!
//! # 与 `DeterministicRng`（生产用四流版本）的区别
//!
//! | 特性 | `TestRng` | `DeterministicRng`（四流版本） |
//! |------|-----------|-------------------------------|
//! | 用途 | 单元测试 | 生产代码 |
//! | 底层 | `StdRng`（ChaCha12） | `ChaCha12Rng`（每流独立实例） |
//! | 位置 | `shared/testing/` | `shared/random/` |
//! | 流隔离 | 无（单流） | 四流（Combat/AI/Drop/World） |
//! | Replay 兼容 | 否 | 是 |

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// 测试用确定性 RNG，固定种子保证跨平台一致。
///
/// 设计用于单元测试：同一个 seed 永远产生相同的序列。
/// 不要在集成测试或生产代码中用来写随机判定逻辑——那里要用四流 `DeterministicRng`。
pub struct TestRng {
    rng: StdRng,
}

impl TestRng {
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

impl Default for TestRng {
    fn default() -> Self {
        Self::new()
    }
}
