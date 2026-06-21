//! 全屏视图的导航栈
//!
//! 实现 ScreenType 值的 LIFO 栈，跟踪导航历史。
//! 防止重复的栈顶入栈和空栈出栈操作。
//! 设计为注册为 Bevy Resource。

use bevy::prelude::*;

use super::screen_type::ScreenType;

/// 屏幕类型的 LIFO 导航栈。
///
/// 作为 Bevy Resource 存储以供 ECS 访问。此处仅跟踪屏幕类型标识；
/// 屏幕实体及其生命周期由 UiScreenState 和屏幕特定系统管理。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ScreenStack {
    /// 屏幕类型栈，从底部（最早）到顶部（当前）。
    stack: Vec<ScreenType>,
}

impl ScreenStack {
    /// 创建空的导航栈。
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// 将屏幕压入栈中。
    ///
    /// 如果 `screen` 已在栈顶，则不执行任何操作，
    /// 以防止重复压入。
    pub fn push(&mut self, screen: ScreenType) {
        if self.peek() == Some(&screen) {
            return;
        }
        self.stack.push(screen);
    }

    /// 从栈中弹出顶部屏幕并返回。
    ///
    /// 如果栈中只剩一个元素（根屏幕被保留），返回 `None`。
    /// 如果栈为空，返回 `None`。
    pub fn pop(&mut self) -> Option<ScreenType> {
        if self.stack.len() <= 1 {
            return None;
        }
        self.stack.pop()
    }

    /// 用新屏幕替换栈顶屏幕。
    ///
    /// 返回之前的栈顶屏幕，如果栈为空则返回 `None`。
    /// 如果栈为空，此操作退化为压入操作。
    pub fn replace(&mut self, screen: ScreenType) -> Option<ScreenType> {
        let old = self.stack.pop();
        self.stack.push(screen);
        old
    }

    /// 返回顶部屏幕的引用，如果为空则返回 `None`。
    pub fn peek(&self) -> Option<&ScreenType> {
        self.stack.last()
    }

    /// 如果栈不包含任何屏幕，返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// 返回栈中屏幕的数量。
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// 如果栈包含给定的屏幕类型，返回 `true`。
    pub fn contains(&self, screen: ScreenType) -> bool {
        self.stack.contains(&screen)
    }

    /// 移除栈中的所有屏幕。
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// 返回从底部到顶部的屏幕迭代器。
    pub fn iter(&self) -> impl Iterator<Item = &ScreenType> {
        self.stack.iter()
    }
}

impl Default for ScreenStack {
    fn default() -> Self {
        Self::new()
    }
}
