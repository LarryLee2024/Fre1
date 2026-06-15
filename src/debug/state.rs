// 调试面板状态定义：DebugView 枚举、DebugPanelState、WorldInspectorState
// 所有调试面板的共享状态集中在此

use bevy::prelude::*;

/// 调试视图枚举
#[derive(Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, Debug)]
pub enum DebugView {
    #[default]
    Battle,
    Buff,
    Overlay,
    DamageAttribute,
    TurnQueue,
    Stepping,
    Grid,
    Ai,
    Equipment,
    Settings,
}

impl DebugView {
    /// 所有视图及其标签和快捷键
    pub fn all() -> &'static [(DebugView, &'static str, &'static str)] {
        &[
            (Self::Battle, "Battle", "F1"),
            (Self::Buff, "Buff", "F2"),
            (Self::Overlay, "Overlay", "F3"),
            (Self::DamageAttribute, "Damage", "F4"),
            (Self::TurnQueue, "Turn", "F5"),
            (Self::Stepping, "Stepping", "F6"),
            (Self::Grid, "Grid", ""),
            (Self::Ai, "AI", ""),
            (Self::Equipment, "Equip", ""),
            (Self::Settings, "Settings", ""),
        ]
    }
}

/// 统一调试面板状态
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DebugPanelState {
    /// 主面板显隐（F1 控制）
    pub show_panel: bool,
    /// 当前选中的导航项
    pub active_view: DebugView,
    /// Damage & Attribute 面板内的 Tab 切换（0=Damage, 1=Attribute）
    pub damage_attribute_tab: u32,
}

impl Default for DebugPanelState {
    fn default() -> Self {
        Self {
            show_panel: true,
            active_view: DebugView::Battle,
            damage_attribute_tab: 0,
        }
    }
}

/// World Inspector 显隐状态（默认展开，F12 切换）
#[derive(Resource)]
pub struct WorldInspectorState {
    pub open: bool,
}

impl Default for WorldInspectorState {
    fn default() -> Self {
        Self { open: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_panel_state_默认全部关闭() {
        let state = DebugPanelState::default();
        assert!(state.show_panel);
        assert_eq!(state.active_view, DebugView::Battle);
        assert_eq!(state.damage_attribute_tab, 0);
    }

    #[test]
    fn debug_view_all_返回正确数量() {
        assert_eq!(DebugView::all().len(), 10);
    }

    #[test]
    fn debug_view_默认是battle() {
        assert_eq!(DebugView::default(), DebugView::Battle);
    }

    #[test]
    fn debug_view_相等性() {
        assert_eq!(DebugView::Battle, DebugView::Battle);
        assert_ne!(DebugView::Battle, DebugView::Buff);
    }

    #[test]
    fn debug_view_哈希一致性() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for &(view, _, _) in DebugView::all() {
            set.insert(view);
        }
        assert_eq!(set.len(), 10);
    }
}
