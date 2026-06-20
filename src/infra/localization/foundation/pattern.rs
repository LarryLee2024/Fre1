//! Pattern — 解析后的文本模板
//!
//! 含原始文本和预提取的变量名列表。

use bevy::prelude::*;

/// 解析后的 Pattern，含原始文本和预提取的变量名列表
#[derive(Debug, Clone, Reflect)]
pub struct Pattern {
    /// 原始模式文本（带 {$var} 占位符）
    pub template: String,
    /// 从 template 中提取的变量名（按出现顺序）
    pub variables: Vec<String>,
}
