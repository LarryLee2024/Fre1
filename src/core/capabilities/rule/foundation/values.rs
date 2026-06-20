//! Rule 值对象定义
//!
//! RuleDef — 数据驱动规则的完整静态定义，从 RON 配置文件反序列化。
//!
//! 详见 docs/02-domain/capabilities/rule_domain.md。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::types::RuleEffect;
use crate::core::capabilities::condition::foundation::Condition;
use crate::shared::localization_key::LocalizationKey;

/// 数据驱动规则（Definition 层）。
///
/// 一条规则 = 一组条件 + 一个效果。当条件满足时，执行效果。
/// 这是五层架构中 Rule System 的核心数据结构。
///
/// ```text
/// RuleDef:
///   condition: TagMatch(Any, ["Status.Burning"]) AND AttributeCheck(HP < 50%)
///   effect: Modifier { target: "fire_resistance", op: Add, value: -20 }
///   priority: 100
/// ```
#[derive(Debug, Clone, PartialEq, Asset, Serialize, Deserialize, TypePath)]
pub struct RuleDef {
    /// 规则唯一标识（格式：rule_ + 6 位数字，如 rule_000001）
    pub id: String,

    /// 规则名称本地化 Key
    pub name_key: LocalizationKey,

    /// 规则描述本地化 Key
    pub desc_key: LocalizationKey,

    /// 触发条件（递归 Condition 树）
    pub condition: Condition,

    /// 规则效果（条件满足时执行）
    pub effect: RuleEffect,

    /// 优先级（越小越优先，同优先级按注册顺序）
    #[serde(default)]
    pub priority: u32,

    /// 是否启用（禁用的规则不参与评估）
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// 规则所属域标签（用于按域筛选规则，如 "combat", "buff", "quest"）
    #[serde(default)]
    pub domain: Option<String>,
}

fn default_enabled() -> bool {
    true
}
