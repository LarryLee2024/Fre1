//! TextInput 交互系统
//!
//! 处理 TextInput 的键盘输入，包括字符输入（通过 ButtonInput<KeyCode> 映射）、
//! 退格删除（Backspace）和提交（Enter）。
//! 基于 Bevy 0.19 的 ButtonInput<KeyCode> API，手动映射按键到字符。
//!
//! 注意：当前实现仅支持基本 ASCII 字符输入（字母、数字、空格、标点），
//! 不支持 IME/输入法。如需完整文本输入（中文/日文等），需要集成 IME 系统。

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::TextInputState;

/// 将 KeyCode 映射到基本 ASCII 字符
///
/// 根据当前的 Shift 状态决定大写/小写。返回 Option<char>，
/// 不可映射的按键（如 F1、ArrowUp）返回 None。
fn keycode_to_char(key: &KeyCode, shift_pressed: bool) -> Option<char> {
    match key {
        // 字母键
        KeyCode::KeyA => Some(if shift_pressed { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift_pressed { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift_pressed { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift_pressed { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift_pressed { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift_pressed { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift_pressed { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift_pressed { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift_pressed { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift_pressed { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift_pressed { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift_pressed { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift_pressed { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift_pressed { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift_pressed { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift_pressed { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift_pressed { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift_pressed { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift_pressed { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift_pressed { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift_pressed { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift_pressed { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift_pressed { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift_pressed { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift_pressed { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift_pressed { 'Z' } else { 'z' }),

        // 数字键
        KeyCode::Digit0 => Some(if shift_pressed { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift_pressed { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift_pressed { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift_pressed { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift_pressed { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift_pressed { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift_pressed { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift_pressed { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift_pressed { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift_pressed { '(' } else { '9' }),

        // 空格和符号键
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift_pressed { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift_pressed { '+' } else { '=' }),
        KeyCode::BracketLeft => Some(if shift_pressed { '{' } else { '[' }),
        KeyCode::BracketRight => Some(if shift_pressed { '}' } else { ']' }),
        KeyCode::Semicolon => Some(if shift_pressed { ':' } else { ';' }),
        KeyCode::Quote => Some(if shift_pressed { '"' } else { '\'' }),
        KeyCode::Comma => Some(if shift_pressed { '<' } else { ',' }),
        KeyCode::Period => Some(if shift_pressed { '>' } else { '.' }),
        KeyCode::Slash => Some(if shift_pressed { '?' } else { '/' }),
        KeyCode::Backslash => Some(if shift_pressed { '|' } else { '\\' }),
        KeyCode::IntlBackslash => Some(if shift_pressed { '~' } else { '`' }),

        // 不可映射按键
        _ => None,
    }
}

/// TextInput 键盘输入处理系统
///
/// 每帧检测所有聚焦的 TextInput 组件，处理字符输入和退格删除。
///
/// # 字符输入
/// 通过 `ButtonInput<KeyCode>::just_pressed()` 检测按键事件，
/// 再通过 `keycode_to_char()` 映射为字符。Shift 键影响大小写和符号。
///
/// # 退格删除
/// 通过 `ButtonInput<KeyCode>::just_pressed(KeyCode::Backspace)` 检测。
pub fn text_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<&mut TextInputState>,
) {
    // 只处理处于焦点状态的 TextInput
    for mut input in &mut text_inputs {
        if !input.is_focused {
            continue;
        }

        // 处理退格删除
        if keyboard.just_pressed(KeyCode::Backspace) {
            input.value.pop();
            continue;
        }

        // 处理字符输入：遍历所有刚按下的键，映射到字符
        let shift_pressed =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        for key in keyboard.get_just_pressed() {
            if input.value.len() >= input.max_length {
                break;
            }

            if let Some(ch) = keycode_to_char(key, shift_pressed) {
                input.value.push(ch);
            }
        }
    }
}
