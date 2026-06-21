//! Dirty<T> — ViewModel 变更跟踪标记
//!
//! Widget 每帧检测 Dirty<T> 标记，仅在标记为 true 时执行刷新。
//! consume() 后标记自动清除，避免同帧重复消费。
//!
//! 详见 `docs/06-ui/02-design-system/focus-binding.md` §3

use bevy::prelude::*;

/// Dirty 标记包装器，用于 ViewModel 变更跟踪。
///
/// # 使用方式
/// ```ignore
/// // Projection 层写入后标记
/// store.battle_hud.get_mut().hp = 80;  // 自动 mark_dirty()
///
/// // Widget 系统检测并消费
/// if dirty.consume() {
///     // 执行 UI 刷新
/// }
/// ```
#[derive(Component, Debug, Clone, Reflect)]
pub struct Dirty<T: Reflect + Default + Clone + Send + Sync + 'static> {
    /// 内部数据
    pub inner: T,
    /// 是否已被标记为脏
    is_dirty: bool,
}

impl<T: Reflect + Default + Clone + Send + Sync + 'static> Dirty<T> {
    /// 创建新的 Dirty 包装（初始状态为 dirty，触发首次刷新）
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            is_dirty: true,
        }
    }

    /// 手动标记为脏
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// 消费脏标记 —— 返回 true 表示需要刷新
    ///
    /// 消费后标记自动清除，避免同帧重复消费。
    pub fn consume(&mut self) -> bool {
        if self.is_dirty {
            self.is_dirty = false;
            return true;
        }
        false
    }

    /// 获取内部数据引用（不触发 dirty）
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// 获取内部数据可变引用（自动标记 dirty）
    pub fn get_mut(&mut self) -> &mut T {
        self.is_dirty = true;
        &mut self.inner
    }
}

impl<T: Reflect + Default + Clone + Send + Sync + 'static> Default for Dirty<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
