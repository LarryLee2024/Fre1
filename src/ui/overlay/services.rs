//! 通用 Overlay 服务队列

use bevy::prelude::*;

/// 通用 Overlay 条目
#[derive(Debug, Clone)]
pub struct OverlayEntry<T: Clone> {
    pub data: T,
    pub lifetime: f32,
    pub priority: u32,
}

/// 通用 Overlay 服务队列
#[derive(Resource, Debug, Clone)]
pub struct OverlayQueue<T: Clone + Send + Sync + 'static> {
    pub entries: Vec<OverlayEntry<T>>,
}

impl<T: Clone + Send + Sync + 'static> Default for OverlayQueue<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> OverlayQueue<T> {
    /// 将覆盖层条目入队（按顺序处理）。
    pub fn push(&mut self, entry: OverlayEntry<T>) {
        self.entries.push(entry);
    }

    /// 取出所有待处理的覆盖层条目并清空队列。
    pub fn drain(&mut self) -> Vec<OverlayEntry<T>> {
        std::mem::take(&mut self.entries)
    }
}
