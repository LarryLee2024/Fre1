//! Registry DAG 初始化系统（ADR-029~035 Data Architecture §2.3）
//!
//! 定义 7 层 DAG 初始化顺序和 [`RegistryPlugin`]。
//! 所有注册表在 `PreStartup` 调度中按层序初始化：
//!
//! ```text
//! Layer 1 (独立):   Tag, Attribute, Terrain, Faction, Class, Race
//! Layer 2 (依赖 Tag): Modifier, Targeting
//! Layer 3 (依赖 Tag+Modifier): Execution, Stacking, Cue
//! Layer 4 (依赖多个): Effect
//! Layer 5 (依赖 Effect): Ability, Trigger
//! Layer 6 (依赖多项): Character, Equipment, Item, Trait
//! Layer 7 (场景): Campaign, Stage, AiBehavior
//! ```
//!
//! # 使用
//! ```ignore
//! app.add_systems(PreStartup, init_tag_registry.in_set(RegistryInitStage::Layer1));
//! app.add_systems(PreStartup, init_effect_registry.in_set(RegistryInitStage::Layer4));
//! ```

use bevy::app::{App, Plugin, PreStartup};
use bevy::ecs::schedule::SystemSet;
use bevy::prelude::IntoScheduleConfigs;

/// Registry 初始化阶段（严格 DAG 顺序）。
///
/// 每个阶段对应一层依赖关系。上层可以引用下层的 Registry Resource。
///
/// # 阶段间排序
/// `Layer1.before(Layer2).before(Layer3).before(Layer4).before(Layer5).before(Layer6).before(Layer7)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum RegistryInitStage {
    /// Layer 1：零依赖基础类型
    /// - TagRegistry, AttributeRegistry, TerrainRegistry
    /// - FactionRegistry, ClassRegistry, RaceRegistry
    Layer1,

    /// Layer 2：依赖 Tag
    /// - ModifierRegistry, TargetingRegistry
    Layer2,

    /// Layer 3：依赖 Tag + Modifier
    /// - ExecutionRegistry, StackingRegistry, CueRegistry
    Layer3,

    /// Layer 4：聚合层（依赖 Tag + Execution + Stacking + Cue）
    /// - EffectRegistry
    Layer4,

    /// Layer 5：逻辑层（依赖 Effect + Tag + Targeting）
    /// - AbilityRegistry, TriggerRegistry
    Layer5,

    /// Layer 6：实体层（依赖 Ability + Modifier + Trait 等）
    /// - CharacterRegistry, EquipmentRegistry, ItemRegistry, TraitRegistry
    Layer6,

    /// Layer 7：场景层（依赖所有下层）
    /// - CampaignRegistry, StageRegistry, AiBehaviorRegistry
    Layer7,
}

/// Registry 初始化插件。
///
/// 在 `PreStartup` 调度中注册 7 层 DAG 顺序约束。
/// 各层系统通过 `.in_set(RegistryInitStage::LayerN)` 注册。
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        // Layer 排序：1 < 2 < 3 < 4 < 5 < 6 < 7
        app.configure_sets(
            PreStartup,
            (
                RegistryInitStage::Layer1,
                RegistryInitStage::Layer2,
                RegistryInitStage::Layer3,
                RegistryInitStage::Layer4,
                RegistryInitStage::Layer5,
                RegistryInitStage::Layer6,
                RegistryInitStage::Layer7,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify all 7 stages exist and are distinct
    #[test]
    fn all_seven_layers_distinct() {
        let stages = [
            RegistryInitStage::Layer1,
            RegistryInitStage::Layer2,
            RegistryInitStage::Layer3,
            RegistryInitStage::Layer4,
            RegistryInitStage::Layer5,
            RegistryInitStage::Layer6,
            RegistryInitStage::Layer7,
        ];
        let mut set = std::collections::HashSet::new();
        for stage in &stages {
            assert!(set.insert(stage), "Duplicate stage: {:?}", stage);
        }
        assert_eq!(set.len(), 7);
    }

    #[test]
    fn registry_plugin_does_not_panic() {
        let mut app = bevy::app::App::new();
        let plugin = RegistryPlugin;
        // 验证插件构建成功
        plugin.build(&mut app);
    }
}
