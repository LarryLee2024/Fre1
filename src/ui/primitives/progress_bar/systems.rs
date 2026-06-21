//! ProgressBar 更新系统
//!
//! 每帧更新所有进度条的填充条宽度和标签文本。
//! 根据 ProgressBarState 的 current/maximum 值重新计算比例，
//! 并同步更新填充条的 Node.width 和标签的 Text 内容。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §3

use bevy::prelude::*;

use super::components::{ProgressBarFill, ProgressBarLabel, ProgressBarState, ProgressBarVariant};

/// 进度条更新系统
///
/// 每帧对所有 ProgressBar 实体：
/// 1. 读取 ProgressBarState（current, maximum, variant, show_label）
/// 2. 计算填充比例 = clamp(current / maximum, 0.0, 1.0)
/// 3. 查找子实体中的 ProgressBarFill 并更新其 Node.width
/// 4. 如启用了标签，查找子实体中的 ProgressBarLabel 并更新文本内容
pub fn progress_bar_update_system(
    bar_query: Query<(&ProgressBarState, &Children)>,
    mut fill_query: Query<(&mut Node, &ProgressBarFill)>,
    mut label_query: Query<(&mut Text, &ProgressBarLabel)>,
) {
    for (state, children) in &bar_query {
        let ratio = if state.maximum > 0.0 {
            (state.current / state.maximum).clamp(0.0, 1.0)
        } else {
            0.0
        };

        for child in children.iter() {
            if let Ok((mut node, _)) = fill_query.get_mut(child) {
                node.width = Val::Percent(ratio * 100.0);
            }

            if let Ok((mut text, _)) = label_query.get_mut(child) {
                let prefix = match state.variant {
                    ProgressBarVariant::Hp => "HP ",
                    ProgressBarVariant::Mp => "MP ",
                    ProgressBarVariant::Xp => "XP ",
                    ProgressBarVariant::Generic => "",
                };
                text.0 = format!("{}{:.0}/{}", prefix, state.current, state.maximum as u32);
            }
        }
    }
}
