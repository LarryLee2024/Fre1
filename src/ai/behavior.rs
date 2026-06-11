// AI 行为配置：数据驱动的 AI 决策策略
// 替代硬编码的 AI 逻辑，支持不同单位使用不同行为模式
// 支持从 assets/ai/*.ron 外部配置文件加载
// 策略字段使用字符串名称，运行时通过 AiStrategyRegistry 查找 trait 对象分发
// 新增策略只需实现 trait 并注册，无需修改本文件（规则1/规则5合规）

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// AI 行为定义（RON 反序列化用）
/// 策略字段为字符串，与 trait 的 strategy_name() 对应
#[derive(Clone, Debug, Deserialize)]
pub struct AiBehaviorDef {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    /// 目标选择策略名称（对应 TargetSelector trait 实现的 strategy_name）
    pub target_strategy: String,
    /// 移动策略名称（对应 MoveSelector trait 实现的 strategy_name）
    pub move_strategy: String,
    /// 技能选择策略名称（对应 SkillSelector trait 实现的 strategy_name）
    pub skill_strategy: String,
    /// 技能使用优先级（从高到低），空则使用 skill_strategy 默认逻辑
    pub skill_priority: Vec<String>,
}

/// AI 行为数据（运行时）
/// 策略字段存储策略名称字符串，运行时通过 AiStrategyRegistry 查找 trait 对象分发
#[derive(Clone, Debug)]
pub struct AiBehavior {
    pub id: String,
    pub name: String,
    /// 目标选择策略名称（对应 TargetSelector trait 实现的 strategy_name）
    pub target_strategy: String,
    /// 移动策略名称（对应 MoveSelector trait 实现的 strategy_name）
    pub move_strategy: String,
    /// 技能选择策略名称（对应 SkillSelector trait 实现的 strategy_name）
    pub skill_strategy: String,
    pub skill_priority: Vec<String>,
}

impl From<AiBehaviorDef> for AiBehavior {
    fn from(def: AiBehaviorDef) -> Self {
        AiBehavior {
            id: def.id,
            name: def.name,
            target_strategy: def.target_strategy,
            move_strategy: def.move_strategy,
            skill_strategy: def.skill_strategy,
            skill_priority: def.skill_priority,
        }
    }
}

/// AI 行为注册表资源
#[derive(Resource, Default)]
pub struct AiBehaviorRegistry {
    pub behaviors: HashMap<String, AiBehavior>,
}

impl AiBehaviorRegistry {
    pub fn get(&self, id: &str) -> Option<&AiBehavior> {
        self.behaviors.get(id)
    }

    /// 注册一个 AI 行为
    pub fn register(&mut self, behavior: AiBehavior) {
        self.behaviors.insert(behavior.id.clone(), behavior);
    }

    /// 获取默认行为（找不到指定行为时回退）
    pub fn default_behavior(&self) -> &AiBehavior {
        self.behaviors.get("default").unwrap_or_else(|| {
            self.behaviors
                .values()
                .next()
                .expect("至少需要一个 AI 行为定义")
        })
    }

    /// 注册内置默认 AI 行为
    fn register_defaults(&mut self) {
        if !self.behaviors.is_empty() {
            return;
        }
        let defaults = vec![
            AiBehaviorDef {
                version: 0,
                id: "default".into(),
                name: "默认".into(),
                target_strategy: "Nearest".into(),
                move_strategy: "Aggressive".into(),
                skill_strategy: "PreferSpecial".into(),
                skill_priority: vec![],
            },
            AiBehaviorDef {
                version: 0,
                id: "aggressive".into(),
                name: "激进".into(),
                target_strategy: "Weakest".into(),
                move_strategy: "Aggressive".into(),
                skill_strategy: "PreferSpecial".into(),
                skill_priority: vec![],
            },
            AiBehaviorDef {
                version: 0,
                id: "cautious".into(),
                name: "谨慎".into(),
                target_strategy: "Nearest".into(),
                move_strategy: "Cautious".into(),
                skill_strategy: "PreferSpecial".into(),
                skill_priority: vec![],
            },
            AiBehaviorDef {
                version: 0,
                id: "support".into(),
                name: "辅助".into(),
                target_strategy: "Nearest".into(),
                move_strategy: "Support".into(),
                skill_strategy: "ByPriority".into(),
                skill_priority: vec!["heal".into(), "cleanse_skill".into()],
            },
        ];

        for def in defaults {
            let id = def.id.clone();
            self.behaviors.insert(id, def.into());
        }
    }
}

impl RegistryLoader for AiBehaviorRegistry {
    type Item = AiBehaviorDef;

    fn register_item(&mut self, item: AiBehaviorDef) {
        let id = item.id.clone();
        self.register(item.into());
        bevy::log::info!(target: "ai", id = %id, "AI行为已加载");
    }

    fn register_defaults(&mut self) {
        AiBehaviorRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }

    fn registry_name() -> &'static str {
        "AI行为"
    }
}

/// AI 行为插件
pub struct AiBehaviorPlugin;

impl Plugin for AiBehaviorPlugin {
    fn build(&self, app: &mut App) {
        let registry = AiBehaviorRegistry::load_from_dir("assets/ai");
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::BASIC_ATTACK_ID;
    use ron::de::from_bytes;

    #[test]
    fn ron_反序列化_ai行为() {
        let ron_str = r#"
            (
                id: "aggressive",
                name: "激进",
                target_strategy: "Weakest",
                move_strategy: "Aggressive",
                skill_strategy: "PreferSpecial",
                skill_priority: [],
            )
        "#;
        let def: AiBehaviorDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "aggressive");
        assert_eq!(def.target_strategy, "Weakest");
        assert_eq!(def.move_strategy, "Aggressive");
    }

    #[test]
    fn ai_behavior_def_转换为_ai_behavior() {
        let def = AiBehaviorDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            target_strategy: "MostDangerous".into(),
            move_strategy: "Cautious".into(),
            skill_strategy: "ByPriority".into(),
            skill_priority: vec!["fireball".into(), BASIC_ATTACK_ID.into()],
        };
        let behavior: AiBehavior = def.into();
        assert_eq!(behavior.id, "test");
        assert_eq!(behavior.target_strategy, "MostDangerous");
        assert_eq!(behavior.move_strategy, "Cautious");
        assert_eq!(behavior.skill_strategy, "ByPriority");
        assert_eq!(behavior.skill_priority, vec!["fireball", BASIC_ATTACK_ID]);
    }

    #[test]
    fn ai_behavior_registry_默认行为() {
        let mut registry = AiBehaviorRegistry::default();
        registry.register_defaults();

        let default = registry.default_behavior();
        assert_eq!(default.id, "default");
        assert_eq!(default.target_strategy, "Nearest");
    }

    #[test]
    fn ron_反序列化_带技能优先级() {
        let ron_str = format!(
            r#"
            (
                id: "support",
                name: "辅助",
                target_strategy: "Nearest",
                move_strategy: "Support",
                skill_strategy: "ByPriority",
                skill_priority: ["heal", "cleanse_skill", "{}"],
            )
        "#,
            BASIC_ATTACK_ID
        );
        let def: AiBehaviorDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(
            def.skill_priority,
            vec!["heal", "cleanse_skill", BASIC_ATTACK_ID]
        );
    }
}
