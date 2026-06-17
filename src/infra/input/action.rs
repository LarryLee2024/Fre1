//! InputAction — 语义化输入动作与按键映射配置
//!
//! Layer 1 的产出物：原始硬件输入通过 InputMap 翻译为语义化的 InputAction。
//! 业务代码不直接读取按键状态，只读取 InputAction。
//!
//! 详见 ADR-043 §6
//! 详见 docs/04-data/infrastructure/input_schema.md

use bevy::prelude::*;
use std::collections::HashMap;

/// 语义化输入动作 — 键盘/鼠标/手柄通过 InputMap 映射到此枚举。
///
/// 业务代码只匹配此枚举，不直接读取按键。这是输入抽象的核心。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    // ── 选择 ──
    /// 选择/确认
    Select,
    /// 取消/返回
    Cancel,

    // ── 方向 ──
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    // ── 摄像机 ──
    CameraUp,
    CameraDown,
    CameraLeft,
    CameraRight,
    CameraZoomIn,
    CameraZoomOut,

    // ── 快捷操作 ──
    QuickSave,
    QuickLoad,
    OpenMenu,
    EndTurn,

    // ── 技能槽 ──
    SkillSlot1,
    SkillSlot2,
    SkillSlot3,
    SkillSlot4,
}

impl InputAction {
    /// 返回动作的名称标识。
    pub fn name(&self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Cancel => "Cancel",
            Self::MoveUp => "MoveUp",
            Self::MoveDown => "MoveDown",
            Self::MoveLeft => "MoveLeft",
            Self::MoveRight => "MoveRight",
            Self::CameraUp => "CameraUp",
            Self::CameraDown => "CameraDown",
            Self::CameraLeft => "CameraLeft",
            Self::CameraRight => "CameraRight",
            Self::CameraZoomIn => "CameraZoomIn",
            Self::CameraZoomOut => "CameraZoomOut",
            Self::QuickSave => "QuickSave",
            Self::QuickLoad => "QuickLoad",
            Self::OpenMenu => "OpenMenu",
            Self::EndTurn => "EndTurn",
            Self::SkillSlot1 => "SkillSlot1",
            Self::SkillSlot2 => "SkillSlot2",
            Self::SkillSlot3 => "SkillSlot3",
            Self::SkillSlot4 => "SkillSlot4",
        }
    }
}

/// 按键绑定配置 — 将原始按键映射为语义化的 InputAction。
///
/// 可通过 RON 配置自定义（默认值在 Default impl 中定义）。
#[derive(Resource, Debug, Clone)]
pub struct InputMap {
    /// 键盘按键映射
    pub keyboard: HashMap<KeyCode, InputAction>,
    /// 鼠标按键映射
    pub mouse: HashMap<MouseButton, InputAction>,
}

impl InputMap {
    /// 根据按键查找对应的 InputAction。
    pub fn get_keyboard_action(&self, key: &KeyCode) -> Option<InputAction> {
        self.keyboard.get(key).copied()
    }

    /// 根据鼠标按键查找对应的 InputAction。
    pub fn get_mouse_action(&self, button: &MouseButton) -> Option<InputAction> {
        self.mouse.get(button).copied()
    }
}

impl Default for InputMap {
    /// 提供合理的默认按键绑定。
    fn default() -> Self {
        let mut keyboard = HashMap::new();

        // 方向键 — WASD
        keyboard.insert(KeyCode::KeyW, InputAction::MoveUp);
        keyboard.insert(KeyCode::KeyS, InputAction::MoveDown);
        keyboard.insert(KeyCode::KeyA, InputAction::MoveLeft);
        keyboard.insert(KeyCode::KeyD, InputAction::MoveRight);

        // 确认/取消
        keyboard.insert(KeyCode::Space, InputAction::Select);
        keyboard.insert(KeyCode::Escape, InputAction::Cancel);
        keyboard.insert(KeyCode::Enter, InputAction::Select);

        // 摄像机 — 方向键/小键盘
        keyboard.insert(KeyCode::ArrowUp, InputAction::CameraUp);
        keyboard.insert(KeyCode::ArrowDown, InputAction::CameraDown);
        keyboard.insert(KeyCode::ArrowLeft, InputAction::CameraLeft);
        keyboard.insert(KeyCode::ArrowRight, InputAction::CameraRight);
        keyboard.insert(KeyCode::Equal, InputAction::CameraZoomIn);
        keyboard.insert(KeyCode::Minus, InputAction::CameraZoomOut);

        // 快捷操作
        keyboard.insert(KeyCode::F5, InputAction::QuickSave);
        keyboard.insert(KeyCode::F9, InputAction::QuickLoad);
        keyboard.insert(KeyCode::Escape, InputAction::OpenMenu);
        keyboard.insert(KeyCode::KeyT, InputAction::EndTurn);

        // 技能槽
        keyboard.insert(KeyCode::Digit1, InputAction::SkillSlot1);
        keyboard.insert(KeyCode::Digit2, InputAction::SkillSlot2);
        keyboard.insert(KeyCode::Digit3, InputAction::SkillSlot3);
        keyboard.insert(KeyCode::Digit4, InputAction::SkillSlot4);

        // 鼠标映射
        let mut mouse = HashMap::new();
        mouse.insert(MouseButton::Left, InputAction::Select);
        mouse.insert(MouseButton::Right, InputAction::Cancel);

        Self { keyboard, mouse }
    }
}
