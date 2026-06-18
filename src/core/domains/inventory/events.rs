//! 领域事件 — Inventory 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/inventory_domain.md §6

use bevy::prelude::*;

use super::components::{EquipSlot, RemovalReason};

/// 物品进入背包时触发。
///
/// 订阅者：
/// - UI：更新背包显示
/// - Quest：检查物品收集任务进度
/// - Narrative：触发物品相关对话
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ItemAcquired {
    /// 获得物品的实体
    pub entity: Entity,
    /// 物品模板 ID
    pub item_template_id: String,
    /// 获得数量
    pub quantity: u32,
    /// 来源描述
    pub source: String,
}

/// 消耗品使用完成时触发。
///
/// 订阅者：
/// - Effect：执行消耗品效果（如治疗药水 → HealEffect）
/// - UI：更新背包显示
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ItemUsed {
    /// 使用物品的实体
    pub entity: Entity,
    /// 物品模板 ID
    pub item_template_id: String,
    /// 消耗的数量
    pub quantity_consumed: u32,
    /// 剩余的物品数量
    pub remaining: u32,
}

/// 装备穿戴/卸下时触发。
///
/// 订阅者：
/// - Modifier：注册/移除装备 Modifier
/// - Attribute：触发属性重算
/// - UI：更新角色装备预览
/// - Condition：重新检查装备相关条件
#[derive(Event, Debug, Clone, PartialEq)]
pub struct EquipmentChanged {
    /// 装备变更的实体
    pub entity: Entity,
    /// 变更的槽位
    pub slot: EquipSlot,
    /// 旧物品模板 ID（可为空）
    pub old_item_template_id: Option<String>,
    /// 新物品模板 ID（可为空）
    pub new_item_template_id: Option<String>,
}

/// 物品从背包移除时触发。
///
/// 订阅者：
/// - UI：更新背包显示
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ItemRemoved {
    /// 物品所属实体
    pub entity: Entity,
    /// 物品模板 ID
    pub item_template_id: String,
    /// 移除数量
    pub quantity: u32,
    /// 移除原因
    pub reason: RemovalReason,
}

/// 战利品生成时触发。
///
/// 订阅者：
/// - Inventory：添加物品到背包
/// - UI：显示战利品界面
#[derive(Event, Debug, Clone, PartialEq)]
pub struct LootGenerated {
    /// 战利品来源实体（如击杀的敌人）
    pub source_entity: Option<Entity>,
    /// 生成的物品列表（模板 ID → 数量）
    pub items: Vec<(String, u32)>,
}
