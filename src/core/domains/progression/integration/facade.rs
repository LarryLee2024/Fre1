//! ProgressionFacade — Progression 域读写的业务语义 API。
//!
//! 所有 Progression 域组件（Experience、ClassLevels、TalentTree、SubclassChoice 等）的
//! 字段访问都在此文件中完成。Systems 通过 ReadFacade / WriteFacade 交互，
//! 永远不直接访问 Progression 组件的字段。
//!
//! # 职责边界
//!
//! - ✅ 封装对 Experience / ClassLevels / TalentTree / SubclassChoice 的读写
//! - ✅ 提供 spawn_progression_entity 快捷创建
//! - 🟥 禁止：在此模块之外直接 import Experience / ClassLevels / TalentTree / SubclassChoice 进行字段访问

use bevy::prelude::*;

use crate::core::domains::progression::components::{
    ClassId, ClassLevels, Experience, LevelProgressionTable, ProgressionMarker, SubclassChoice,
    SubclassId, TalentId, TalentTree,
};

// ─── 只读操作 ────────────────────────────────────────────────────────────

/// Progression 域只读查询接口。
///
/// 外部通过此结构体的 static 方法读取 Progression 域数据。
/// 所有方法接受 `&World`，返回只读引用或拷贝值。
pub struct ProgressionReadFacade;

impl ProgressionReadFacade {
    /// 获取实体的 Experience 组件引用。
    pub fn experience(world: &World, entity: Entity) -> Option<&Experience> {
        world.get::<Experience>(entity)
    }

    /// 获取实体的 ClassLevels 组件引用。
    pub fn class_levels(world: &World, entity: Entity) -> Option<&ClassLevels> {
        world.get::<ClassLevels>(entity)
    }

    /// 获取实体的 TalentTree 组件引用。
    pub fn talent_tree(world: &World, entity: Entity) -> Option<&TalentTree> {
        world.get::<TalentTree>(entity)
    }

    /// 获取实体的 SubclassChoice 组件引用。
    pub fn subclass_choice(world: &World, entity: Entity) -> Option<&SubclassChoice> {
        world.get::<SubclassChoice>(entity)
    }

    /// 检查实体是否拥有 ProgressionMarker。
    ///
    /// 返回 `true` 仅当实体同时具有 ProgressionMarker 组件。
    pub fn has_marker(world: &World, entity: Entity) -> bool {
        world.get::<ProgressionMarker>(entity).is_some()
    }

    /// 获取 LevelProgressionTable 资源引用。
    pub fn level_table(world: &World) -> Option<&LevelProgressionTable> {
        world.get_resource::<LevelProgressionTable>()
    }

    /// 获取实体的当前等级。
    pub fn level(world: &World, entity: Entity) -> Option<u32> {
        world.get::<Experience>(entity).map(|xp| xp.level)
    }

    /// 获取实体的当前经验值。
    pub fn current_xp(world: &World, entity: Entity) -> Option<u64> {
        world.get::<Experience>(entity).map(|xp| xp.current_xp)
    }

    /// 获取实体的总获得经验值。
    pub fn total_xp_earned(world: &World, entity: Entity) -> Option<u64> {
        world.get::<Experience>(entity).map(|xp| xp.total_xp_earned)
    }

    /// 检查实体是否为满级。
    pub fn is_max_level(world: &World, entity: Entity) -> Option<bool> {
        world.get::<Experience>(entity).map(|xp| xp.is_max_level)
    }

    /// 获取实体的总等级（所有职业等级之和）。
    pub fn total_level(world: &World, entity: Entity) -> Option<u32> {
        world
            .get::<ClassLevels>(entity)
            .map(|cls| cls.total_level())
    }

    /// 获取实体指定职业的等级。
    pub fn level_in_class(world: &World, entity: Entity, class_id: &ClassId) -> Option<u32> {
        world
            .get::<ClassLevels>(entity)
            .map(|cls| cls.level_in_class(class_id))
    }

    /// 检查实体的天赋是否已解锁。
    pub fn is_talent_unlocked(world: &World, entity: Entity, talent_id: &TalentId) -> Option<bool> {
        world
            .get::<TalentTree>(entity)
            .map(|tree| tree.is_unlocked(talent_id))
    }

    /// 获取实体可用的天赋点数。
    pub fn available_talent_points(world: &World, entity: Entity) -> Option<u32> {
        world
            .get::<TalentTree>(entity)
            .map(|tree| tree.available_points)
    }

    /// 获取实体指定职业的子职。
    pub fn subclass_for_class(
        world: &World,
        entity: Entity,
        class_id: &ClassId,
    ) -> Option<SubclassId> {
        world
            .get::<SubclassChoice>(entity)
            .and_then(|sc| sc.get(class_id))
            .cloned()
    }
}

// ─── 写操作 ──────────────────────────────────────────────────────────────

/// Progression 域写入接口。
///
/// 外部通过此结构体的 static 方法修改 Progression 域数据。
/// 方法直接修改组件状态（通过 `&mut World` 或 `Commands`），
/// 不触发 Progression 域的 Observer 事件循环——调用方如需通知其他系统应手动触发事件。
pub struct ProgressionWriteFacade;

impl ProgressionWriteFacade {
    /// 为实体增加经验值。
    ///
    /// 直接调用 `Experience::add_xp()`。满级实体不会累积经验。
    /// 不触发 `ExperienceGained` 事件——调用方如需通知其他系统应自行触发。
    pub fn add_experience(world: &mut World, entity: Entity, amount: u64) {
        if let Some(mut xp) = world.get_mut::<Experience>(entity) {
            xp.add_xp(amount);
        }
    }

    /// 对实体应用升级消耗。
    ///
    /// 直接调用 `Experience::apply_level_up()`。扣除经验、等级 +1。
    /// 不触发 `LevelUp` 事件——调用方负责在需要时触发。
    pub fn apply_level_up(world: &mut World, entity: Entity, xp_cost: u64) {
        if let Some(mut xp) = world.get_mut::<Experience>(entity) {
            xp.apply_level_up(xp_cost);
        }
    }

    /// 为实体解锁天赋。
    ///
    /// 直接调用 `TalentTree::unlock()`。重复解锁同一天赋为 no-op。
    /// 不触发 `TalentUnlocked` 事件——调用方负责在需要时触发。
    pub fn unlock_talent(world: &mut World, entity: Entity, talent_id: &str) {
        if let Some(mut tree) = world.get_mut::<TalentTree>(entity) {
            tree.unlock(TalentId::new(talent_id));
        }
    }

    /// 为实体选择子职。
    ///
    /// 直接调用 `SubclassChoice::choose()`。同一职业不可重复选择。
    ///
    /// # 错误
    /// 如果该职业已有子职，返回错误描述。
    pub fn choose_subclass(
        world: &mut World,
        entity: Entity,
        class_id: ClassId,
        subclass_id: &str,
    ) -> Result<(), String> {
        let mut sc = world
            .get_mut::<SubclassChoice>(entity)
            .ok_or_else(|| "实体缺少 SubclassChoice 组件".to_string())?;
        sc.choose(class_id, SubclassId::new(subclass_id))
    }

    /// 为实体增加天赋点数。
    pub fn add_talent_points(world: &mut World, entity: Entity, points: u32) {
        if let Some(mut tree) = world.get_mut::<TalentTree>(entity) {
            tree.add_points(points);
        }
    }

    /// 消耗实体的一点天赋点数。
    ///
    /// 返回 `true` 如果成功消耗，`false` 如果无可用点数。
    pub fn spend_talent_point(world: &mut World, entity: Entity) -> bool {
        world
            .get_mut::<TalentTree>(entity)
            .map(|mut tree| tree.spend_point())
            .unwrap_or(false)
    }

    /// 推进实体的指定职业等级（多职业用）。
    ///
    /// 直接调用 `ClassLevels::advance_class()`。如果角色已有该职业则等级 +1，
    /// 否则添加新职业条目。
    pub fn advance_class(world: &mut World, entity: Entity, class_id: ClassId) {
        if let Some(mut cls) = world.get_mut::<ClassLevels>(entity) {
            cls.advance_class(class_id);
        }
    }

    /// 生成一个完整的 Progression 实体。
    ///
    /// 创建的实体包含所有 Progression 组件：
    /// - Experience（等级 1）
    /// - ClassLevels（初始职业等级 1）
    /// - TalentTree（空）
    /// - SubclassChoice（空）
    /// - ProgressionMarker
    pub fn spawn_progression_entity(
        commands: &mut Commands,
        initial_class: impl Into<ClassId>,
    ) -> Entity {
        commands
            .spawn((
                Experience::new(),
                ClassLevels::new(initial_class),
                TalentTree::new(),
                SubclassChoice::new(),
                ProgressionMarker,
            ))
            .id()
    }
}
