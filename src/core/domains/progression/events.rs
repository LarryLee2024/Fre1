//! 领域事件 — Progression 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/progression_domain.md §6

use bevy::prelude::*;

use super::components::ClassId;
use crate::shared::diagnostics::{Domain, FieldCollector, LogCode, ObservableEvent};

/// 角色获得经验值时触发。
///
/// 订阅者：
/// - Progression：检查经验阈值 → 触发升级
/// - UI：显示经验获取动画/进度条
/// - Narrative：检查经验相关的对话条件
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ExperienceGained {
    /// 获得经验的实体
    pub entity: Entity,
    /// 获得的经验量
    pub amount: u64,
    /// 经验来源描述
    pub source: String,
    /// 获得后的总经验值
    pub total_xp: u64,
    /// 获得后的当前等级
    pub current_level: u32,
}

/// 角色升级时触发。
///
/// 订阅者：
/// - Attribute：应用升级带来的属性变化（通过 Modifier Pipeline）
/// - Progression：解锁天赋选项、检查 ASI
/// - UI：显示升级选择界面
#[derive(Event, Debug, Clone, PartialEq)]
pub struct LevelUp {
    /// 升级的实体
    pub entity: Entity,
    /// 旧等级
    pub old_level: u32,
    /// 新等级
    pub new_level: u32,
    /// 提升的职业 ID（多职业场景）
    pub class_id: ClassId,
    /// 是否达到 ASI 等级
    pub is_asi_level: bool,
}

impl ObservableEvent for LevelUp {
    const DOMAIN: Domain = Domain::Progression;
    const CODE: LogCode = LogCode::PRG002;

    fn record_fields(&self, collector: &mut FieldCollector) {
        collector.add_field("entity", format_args!("{:?}", self.entity));
        collector.add_field("old", self.old_level);
        collector.add_field("new", self.new_level);
        collector.add_field("class", &self.class_id);
        collector.add_field("asi", self.is_asi_level);
    }
}

/// 天赋解锁时触发。
///
/// 订阅者：
/// - Modifier：应用天赋的被动效果
/// - Ability：注册天赋提供的主动能力
/// - UI：更新天赋树显示
/// - Cue：天赋解锁特效
#[derive(Event, Debug, Clone, PartialEq)]
pub struct TalentUnlocked {
    /// 解锁天赋的实体
    pub entity: Entity,
    /// 天赋 ID
    pub talent_id: String,
}

/// 子职选择完成时触发。
///
/// 订阅者：
/// - Ability：注册子职能力
/// - Spell：更新法术列表（如适用）
/// - UI：更新角色面板
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SubclassChosen {
    /// 选择子职的实体
    pub entity: Entity,
    /// 职业 ID
    pub class_id: ClassId,
    /// 子职 ID
    pub subclass_id: String,
}

/// 属性提升（ASI）完成时触发。
///
/// 订阅者：
/// - Attribute：修改属性 BaseValue
/// - Aggregator：触发属性重算
/// - UI：显示属性变化
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ASICompleted {
    /// 完成 ASI 的实体
    pub entity: Entity,
    /// 触发 ASI 的等级
    pub level: u32,
    /// 选择结果描述
    pub choices: Vec<String>,
}

/// 获得新职业等级时触发（多职业用）。
///
/// 订阅者：
/// - Spell：更新法术位/已知法术
/// - Ability：注册职业特性
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ClassGained {
    /// 获得职业等级的实体
    pub entity: Entity,
    /// 职业 ID
    pub class_id: ClassId,
    /// 在该职业上的新等级
    pub new_level: u32,
}
