//! TextInput 组件的类型定义
//!
//! 定义 TextInputState（Widget Contract 的本地状态）。

use bevy::prelude::*;

/// TextInput 本地状态（Widget Contract Local State）
///
/// 包含当前输入值、最大长度、占位符 Key 和焦点状态。
/// Props 字段由 spawn_text_input 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TextInputState {
    /// 当前输入文本
    pub value: String,
    /// 最大可输入字符数
    pub max_length: usize,
    /// 占位符文本的本地化 Key
    pub placeholder_key: &'static str,
    /// 是否处于输入焦点状态
    pub is_focused: bool,
}
