//! Text Widget 更新系统
//!
//! 每帧检测 TextWidget 组件的 content 字段变化，同步更新对应的 Text 组件。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §4

use bevy::prelude::*;

use super::components::TextWidget;

/// 文本内容同步系统
///
/// 当 TextWidget.content 发生变化时，将新内容写入 Text 组件。
/// 使用 `Changed<TextWidget>` 过滤，只在内容变化时执行。
pub fn text_update_system(mut query: Query<(&mut Text, &TextWidget), Changed<TextWidget>>) {
    for (mut text, widget) in query.iter_mut() {
        text.0 = widget.content.clone();
    }
}
