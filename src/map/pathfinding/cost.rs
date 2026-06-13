// 地形成本计算规则

use crate::core::tag::{GameplayTag, GameplayTags};
use bevy::prelude::*;
use std::collections::HashMap;

// ── 地形成本计算 trait ──

/// 地形移动成本计算规则 trait：描述不同单位类型的地形通行能力
/// terrain_id: 地形 ID 字符串（如 "plain", "forest"），由 TerrainRegistry 定义
/// base_cost: 从 TerrainRegistry 加载的基础成本（None 表示基础不可通行）
pub trait TerrainCostCalculator: Send + Sync + 'static {
    /// 计算器名称（用于注册和查找）
    fn name(&self) -> &'static str;
    /// 计算指定地形的移动成本，None 表示不可通行
    fn cost(&self, terrain_id: &str, base_cost: Option<u32>) -> Option<u32>;
}

// ── 内置实现 ──

/// 步兵成本计算器：使用基础成本
pub struct GroundCostCalculator;

impl TerrainCostCalculator for GroundCostCalculator {
    fn name(&self) -> &'static str {
        "ground"
    }

    fn cost(&self, _terrain_id: &str, base_cost: Option<u32>) -> Option<u32> {
        base_cost
    }
}

/// 飞行成本计算器：所有地形成本为1
pub struct FlyingCostCalculator;

impl TerrainCostCalculator for FlyingCostCalculator {
    fn name(&self) -> &'static str {
        "flying"
    }

    fn cost(&self, _terrain_id: &str, _base_cost: Option<u32>) -> Option<u32> {
        Some(1)
    }
}

/// 骑兵成本计算器：平原成本1，森林成本3，山地/水域不可通行
pub struct MountedCostCalculator;

impl TerrainCostCalculator for MountedCostCalculator {
    fn name(&self) -> &'static str {
        "mounted"
    }

    fn cost(&self, terrain_id: &str, _base_cost: Option<u32>) -> Option<u32> {
        match terrain_id {
            "plain" => Some(1),
            "forest" => Some(3),
            _ => None, // 骑兵无法进入山地和水域
        }
    }
}

/// 水生成本计算器：水域成本1，平原成本2，山地不可通行
pub struct SwimmingCostCalculator;

impl TerrainCostCalculator for SwimmingCostCalculator {
    fn name(&self) -> &'static str {
        "swimming"
    }

    fn cost(&self, terrain_id: &str, _base_cost: Option<u32>) -> Option<u32> {
        match terrain_id {
            "water" => Some(1),
            "plain" => Some(2),
            "forest" => Some(3),
            _ => None, // 水生单位无法进入山地
        }
    }
}

// ── 地形成本注册表 ──

/// 地形成本计算器注册表资源
#[derive(Resource)]
pub struct TerrainCostRegistry {
    calculators: HashMap<String, Box<dyn TerrainCostCalculator>>,
}

impl Default for TerrainCostRegistry {
    fn default() -> Self {
        let mut registry = Self {
            calculators: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }
}

impl TerrainCostRegistry {
    /// 注册内置默认计算器
    fn register_defaults(&mut self) {
        self.register(Box::new(GroundCostCalculator));
        self.register(Box::new(FlyingCostCalculator));
        self.register(Box::new(MountedCostCalculator));
        self.register(Box::new(SwimmingCostCalculator));
    }

    /// 注册一个计算器
    pub fn register(&mut self, calculator: Box<dyn TerrainCostCalculator>) {
        self.calculators
            .insert(calculator.name().to_string(), calculator);
    }

    /// 按名称获取计算器
    pub fn get(&self, name: &str) -> Option<&dyn TerrainCostCalculator> {
        self.calculators.get(name).map(|c| c.as_ref())
    }

    /// 获取默认（步兵）计算器
    pub fn ground(&self) -> &dyn TerrainCostCalculator {
        match self.get("ground") {
            Some(calc) => calc,
            None => {
                bevy::log::error!(
                    target: "map",
                    event = "calculator_missing",
                    name = "ground",
                    "GroundCostCalculator 缺失，使用 FlyingCostCalculator 作为回退"
                );
                // 回退到 flying 计算器（所有地形成本为1）
                match self.get("flying") {
                    Some(fallback) => fallback,
                    None => panic!("GroundCostCalculator 和 FlyingCostCalculator 均未注册"),
                }
            }
        }
    }

    /// 根据单位标签解析对应的成本计算器
    /// 优先级：SWIMMING > FLYING > MOUNTED > 默认(ground)
    pub fn resolve_from_tags(&self, tags: &GameplayTags) -> &dyn TerrainCostCalculator {
        if tags.has(GameplayTag::SWIMMING) {
            return self.get("swimming").unwrap_or(self.ground());
        }
        if tags.has(GameplayTag::FLYING) {
            return self.get("flying").unwrap_or(self.ground());
        }
        if tags.has(GameplayTag::MOUNTED) {
            return self.get("mounted").unwrap_or(self.ground());
        }
        self.ground()
    }
}
