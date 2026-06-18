//! Summon Domain — 测试辅助
//!
//! 提供 Builder 模式和标准测试数据。

use crate::core::domains::summon::components::{
    GridSize, SummonAIMode, SummonCost, SummonTemplateDef,
};

/// 创建一个标准的召唤物模板。
pub fn summon_template(id: &str, concentration: bool) -> SummonTemplateDef {
    SummonTemplateDef {
        id: id.into(),
        name_key: "summon.test.name".into(),
        base_attributes: vec![],
        tags: vec![],
        abilities: vec![],
        modifiers: vec![],
        grid_size: GridSize::Medium,
        default_ai_mode: SummonAIMode::Autonomous,
        summon_cost: SummonCost {
            ability_id: None,
            spell_level: Some(3),
            requires_concentration: concentration,
        },
    }
}
