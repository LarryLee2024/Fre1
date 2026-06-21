//! QueryExt — [`Query`] 的扩展 trait。
//!
//! 在 Bevy 的 `Query` 之上提供常用 SRPG 查询过滤器的语法糖。
//!
//! # 桩实现状态
///
/// 这是初始桩实现。随着 Phase C2 的推进，方法将接入实际的
/// 集成 Facade 函数。

use bevy::ecs::query::QueryData;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::Entity;
use bevy::prelude::Query;

/// [`Query`] 的扩展 trait，提供领域特定的过滤方法。
///
/// # 用法
///
/// ```ignore
/// use crate::core::domains::combat::integration::ext::QueryExt;
///
/// fn my_system(query: Query<&Health>) {
///     for entity in query.alive() {
///         // 处理存活实体
///     }
/// }
/// ```
/// Combat 域 Query 扩展。
///
/// 存在原因：战斗系统频繁需要"存活单位"和"敌对单位"筛选，
/// 将常用过滤模式封装为方法，避免每个系统重复写 `Without<Dead>` 等过滤器。
pub trait QueryExt<'w, 's, D: QueryData, F: QueryFilter> {
    /// 过滤存活实体（非 Dead/已移除）。
    fn alive(&self) -> impl Iterator<Item = Entity>;

    /// 过滤与指定 Faction 敌对的实体。
    fn hostile_to(&self, faction: &str) -> impl Iterator<Item = Entity>;
}

impl<'w, 's, D: QueryData, F: QueryFilter> QueryExt<'w, 's, D, F> for Query<'w, 's, D, F> {
    fn alive(&self) -> impl Iterator<Item = Entity> {
        // Stub：返回空迭代器。
        // 未来 Phase C2 将接入 Dead 组件排除逻辑。
        std::iter::empty()
    }

    fn hostile_to(&self, _faction: &str) -> impl Iterator<Item = Entity> {
        // Stub：返回空迭代器。
        // 未来 Phase C2 将接入 Faction 组件过滤逻辑。
        std::iter::empty()
    }
}
