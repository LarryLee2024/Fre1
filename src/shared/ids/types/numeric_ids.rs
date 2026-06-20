//! Numeric ID 类型 — 运行时实例标识。
//!
//! 使用 `InstanceId<T>` 泛型包装 `RuntimeId`，提供 generation 保护。
//!
//! # 当前状态
//!
//! - `ModifierInstanceId`: ✅ 已迁移至 `InstanceId<ModifierInstanceMarker>`（带 Generation 保护）
//! - 其他运行时 ID 待迁移

use super::runtime_id::InstanceId;
use bevy::prelude::Reflect;

// ============================================================================
// ModifierInstanceId
// ============================================================================

/// ModifierInstanceId 的 PhantomData marker。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Reflect)]
pub struct ModifierInstanceMarker;

/// 修改器运行时实例 ID（带 Generation 保护）。
///
/// 内部包装 `RuntimeId`（index + generation），防止 ID 复用导致的引用悬空。
/// 通过 `ModifierIdGenerator`（`RuntimeIdAllocator`）分配。
pub type ModifierInstanceId = InstanceId<ModifierInstanceMarker>;
