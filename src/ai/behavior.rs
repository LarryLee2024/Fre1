// AI 行为配置：数据驱动的 AI 决策策略
// 替代硬编码的 AI 逻辑，支持不同单位使用不同行为模式
// 支持从 assets/ai/*.ron 外部配置文件加载

use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

/// 目标选择策略
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TargetStrategy {
    /// 选择最近的敌人
    Nearest,
    /// 选择血量最低的敌人
    Weakest,
    /// 选择攻击力最高的敌人
    MostDangerous,
    /// 选择血量百分比最低的敌人
    LowestHpPercent,
}

/// 移动策略
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MoveStrategy {
    /// 冲向目标（贪心最近）
    Aggressive,
    /// 保持攻击距离（不靠近超过攻击范围）
    Cautious,
    /// 优先靠近友军（辅助型）
    Support,
}

/// 技能选择策略
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SkillStrategy {
    /// 优先使用特殊技能
    PreferSpecial,
    /// 优先使用基础攻击
    PreferBasic,
    /// 按技能优先级排序
    ByPriority,
}

/// AI 行为定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct AiBehaviorDef {
    pub id: String,
    pub name: String,
    pub target_strategy: TargetStrategy,
    pub move_strategy: MoveStrategy,
    pub skill_strategy: SkillStrategy,
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
            // enum variant 名转为字符串，与 trait strategy_name 对应
            target_strategy: match def.target_strategy {
                TargetStrategy::Nearest => "Nearest",
                TargetStrategy::Weakest => "Weakest",
                TargetStrategy::MostDangerous => "MostDangerous",
                TargetStrategy::LowestHpPercent => "LowestHpPercent",
            }
            .to_string(),
            move_strategy: match def.move_strategy {
                MoveStrategy::Aggressive => "Aggressive",
                MoveStrategy::Cautious => "Cautious",
                MoveStrategy::Support => "Support",
            }
            .to_string(),
            skill_strategy: match def.skill_strategy {
                SkillStrategy::PreferSpecial => "PreferSpecial",
                SkillStrategy::PreferBasic => "PreferBasic",
                SkillStrategy::ByPriority => "ByPriority",
            }
            .to_string(),
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

    /// 获取默认行为（找不到指定行为时回退）
    pub fn default_behavior(&self) -> &AiBehavior {
        self.behaviors.get("default").unwrap_or_else(|| {
            self.behaviors
                .values()
                .next()
                .expect("至少需要一个 AI 行为定义")
        })
    }

    /// 从 assets/ai/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = AiBehaviorRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("AI 行为目录不存在: {}", dir);
            registry.register_defaults();
            return registry;
        };

        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<AiBehaviorDef>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.behaviors.insert(id.clone(), def.into());
                            bevy::log::info!("加载 AI 行为: {}", id);
                            loaded = true;
                        }
                        Err(e) => {
                            bevy::log::error!("解析 AI 行为文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取 AI 行为文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }

        if !loaded {
            bevy::log::warn!("AI 行为目录为空，使用默认行为");
            registry.register_defaults();
        }

        registry
    }

    /// 注册内置默认 AI 行为
    fn register_defaults(&mut self) {
        let defaults = vec![
            AiBehaviorDef {
                id: "default".into(),
                name: "默认".into(),
                target_strategy: TargetStrategy::Nearest,
                move_strategy: MoveStrategy::Aggressive,
                skill_strategy: SkillStrategy::PreferSpecial,
                skill_priority: vec![],
            },
            AiBehaviorDef {
                id: "aggressive".into(),
                name: "激进".into(),
                target_strategy: TargetStrategy::Weakest,
                move_strategy: MoveStrategy::Aggressive,
                skill_strategy: SkillStrategy::PreferSpecial,
                skill_priority: vec![],
            },
            AiBehaviorDef {
                id: "cautious".into(),
                name: "谨慎".into(),
                target_strategy: TargetStrategy::Nearest,
                move_strategy: MoveStrategy::Cautious,
                skill_strategy: SkillStrategy::PreferSpecial,
                skill_priority: vec![],
            },
            AiBehaviorDef {
                id: "support".into(),
                name: "辅助".into(),
                target_strategy: TargetStrategy::Nearest,
                move_strategy: MoveStrategy::Support,
                skill_strategy: SkillStrategy::ByPriority,
                skill_priority: vec!["heal".into(), "cleanse_skill".into()],
            },
        ];

        for def in defaults {
            let id = def.id.clone();
            self.behaviors.insert(id, def.into());
        }
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

    #[test]
    fn ron_反序列化_ai行为() {
        let ron_str = r#"
            (
                id: "aggressive",
                name: "激进",
                target_strategy: Weakest,
                move_strategy: Aggressive,
                skill_strategy: PreferSpecial,
                skill_priority: [],
            )
        "#;
        let def: AiBehaviorDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "aggressive");
        assert_eq!(def.target_strategy, TargetStrategy::Weakest);
        assert_eq!(def.move_strategy, MoveStrategy::Aggressive);
    }

    #[test]
    fn ai_behavior_def_转换为_ai_behavior() {
        let def = AiBehaviorDef {
            id: "test".into(),
            name: "测试".into(),
            target_strategy: TargetStrategy::MostDangerous,
            move_strategy: MoveStrategy::Cautious,
            skill_strategy: SkillStrategy::ByPriority,
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
                target_strategy: Nearest,
                move_strategy: Support,
                skill_strategy: ByPriority,
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
