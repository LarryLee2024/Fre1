//! QueryExt -- Extension trait for [`Query`].
//!
//! Provides common SRPG query filters as method-syntax sugar on top of Bevy's `Query`.
//!
//! # Stub Status
//!
//! This is an initial stub implementation. Methods will be wired to actual
//! integration facade functions as Phase C2 progresses.

use bevy::ecs::query::QueryData;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::Entity;
use bevy::prelude::Query;

/// Extension trait for [`Query`] providing domain-specific filter methods.
///
/// # Usage
///
/// ```ignore
/// use crate::core::domains::combat::integration::ext::QueryExt;
///
/// fn my_system(query: Query<&Health>) {
///     for entity in query.alive() {
///         // process alive entities
///     }
/// }
/// ```
/// Combat 域 Query 扩展。
///
/// 存在原因：战斗系统频繁需要"存活单位"和"敌对单位"筛选，
/// 将常用过滤模式封装为方法，避免每个系统重复写 `Without<Dead>` 等过滤器。
pub trait QueryExt<'w, 's, D: QueryData, F: QueryFilter> {
    /// Filter entities that are alive (not dead/removed).
    fn alive(&self) -> impl Iterator<Item = Entity>;

    /// Filter entities that are hostile to the given faction.
    fn hostile_to(&self, faction: &str) -> impl Iterator<Item = Entity>;
}

impl<'w, 's, D: QueryData, F: QueryFilter> QueryExt<'w, 's, D, F> for Query<'w, 's, D, F> {
    fn alive(&self) -> impl Iterator<Item = Entity> {
        // Stub: returns empty iterator.
        // Will be wired to a Dead component exclusion in future Phase C2 work.
        std::iter::empty()
    }

    fn hostile_to(&self, _faction: &str) -> impl Iterator<Item = Entity> {
        // Stub: returns empty iterator.
        // Will be wired to Faction component filtering in future Phase C2 work.
        std::iter::empty()
    }
}
