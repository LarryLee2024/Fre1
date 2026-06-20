//! ProgressionQueryParam — Bevy SystemParam，封装所有 Progression 组件查询。
//!
//! Systems 通过此 param 获取 Progression 域数据，完全不知道
//! Experience / ClassLevels / TalentTree / SubclassChoice 组件内部细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     mut progression_query: ProgressionQueryParam,
//!     // ...
//! ) {
//!     let xp = progression_query.experiences.get(entity);
//!     if let Ok(xp) = xp {
//!         println!("Level: {}, XP: {}", xp.level, xp.current_xp);
//!     }
//! }
//! ```

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::progression::components::{
    ClassLevels, Experience, LevelProgressionTable, ProgressionMarker, SubclassChoice, TalentTree,
};

/// Progression 查询参数 — 封装所有 Progression 组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&Experience>` + `Res<LevelProgressionTable>`。
/// 函数体内所有 Progression 数据访问都通过此 param 的字段完成。
#[derive(SystemParam)]
pub struct ProgressionQueryParam<'w, 's> {
    /// Experience 组件查询。
    pub experiences: Query<'w, 's, &'static Experience>,
    /// ClassLevels 组件查询。
    pub class_levels_query: Query<'w, 's, &'static ClassLevels>,
    /// TalentTree 组件查询。
    pub talent_trees: Query<'w, 's, &'static TalentTree>,
    /// SubclassChoice 组件查询。
    pub subclass_choices: Query<'w, 's, &'static SubclassChoice>,
    /// ProgressionMarker 标记查询。
    pub progression_markers: Query<'w, 's, &'static ProgressionMarker>,
    /// 等级成长配置表资源。
    pub level_table: Res<'w, LevelProgressionTable>,
}

impl<'w, 's> ProgressionQueryParam<'w, 's> {
    /// 获取实体的当前等级。
    pub fn level(&self, entity: Entity) -> Option<u32> {
        self.experiences.get(entity).ok().map(|xp| xp.level)
    }

    /// 获取实体的当前经验值。
    pub fn current_xp(&self, entity: Entity) -> Option<u64> {
        self.experiences.get(entity).ok().map(|xp| xp.current_xp)
    }

    /// 获取实体的总获得经验值。
    pub fn total_xp_earned(&self, entity: Entity) -> Option<u64> {
        self.experiences
            .get(entity)
            .ok()
            .map(|xp| xp.total_xp_earned)
    }

    /// 检查实体是否为满级。
    pub fn is_max_level(&self, entity: Entity) -> Option<bool> {
        self.experiences.get(entity).ok().map(|xp| xp.is_max_level)
    }

    /// 获取实体的总等级（所有职业等级之和）。
    pub fn total_level(&self, entity: Entity) -> Option<u32> {
        self.class_levels_query
            .get(entity)
            .ok()
            .map(|cls| cls.total_level())
    }

    /// 获取实体可用的天赋点数。
    pub fn available_talent_points(&self, entity: Entity) -> Option<u32> {
        self.talent_trees
            .get(entity)
            .ok()
            .map(|tree| tree.available_points)
    }

    /// 检查实体是否拥有 ProgressionMarker。
    pub fn has_marker(&self, entity: Entity) -> bool {
        self.progression_markers.get(entity).is_ok()
    }
}
