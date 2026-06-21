//! SelectList Factory — SelectList 的唯一创建入口
//!
//! 遵循 Factory 模式，禁止直接通过 commands.spawn 创建 SelectList。
//! 输入 Props + Theme → 输出 Entity。
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §9

use bevy::prelude::*;

use super::components::{SelectListItem, SelectListState};
use crate::ui::primitives::button::{
    components::{ButtonInteraction, ButtonState, ButtonVariant},
    factory::spawn_button,
};
use crate::ui::primitives::list::{components::ListVariant, factory::spawn_list};
use crate::ui::theme::Theme;

/// 工厂函数：生成一个完整配置的 SelectList Widget
///
/// # 参数
/// - `commands`: ECS 命令
/// - `theme`: 主题 Resource（提供颜色令牌）
/// - `items`: 条目文本列表
/// - `default_index`: 默认选中的条目索引
///
/// # 返回
/// SelectList 容器实体的 Entity。内部结构：
/// ```text
/// SelectList (List, Vertical)
///   ├── Button (item[0], Secondary) + SelectListItem { index: 0 }
///   ├── Button (item[1], Secondary) + SelectListItem { index: 1 }
///   └── ...
/// ```
pub fn spawn_select_list(
    commands: &mut Commands,
    theme: &Theme,
    items: Vec<&'static str>,
    default_index: usize,
) -> Entity {
    let list = spawn_list(commands, theme, ListVariant::Vertical);
    commands.entity(list).insert((
        SelectListState {
            selected_index: default_index,
            items: items.clone(),
        },
        Name::new("SelectList"),
    ));

    for (i, item) in items.iter().enumerate() {
        let is_selected = i == default_index;
        let btn = spawn_button(commands, theme, *item, ButtonVariant::Secondary);
        // Selected item gets accent_primary border
        if is_selected {
            commands
                .entity(btn)
                .insert(BorderColor::all(theme.colors.accent_primary));
        }
        commands.entity(btn).insert((
            SelectListItem { index: i },
            Name::new(format!("SelectListItem({})", item)),
        ));
        commands.entity(btn).set_parent_in_place(list);
    }

    list
}
