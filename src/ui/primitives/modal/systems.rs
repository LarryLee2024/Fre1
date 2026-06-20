//! Modal 交互系统
//!
//! 通过 Observer 模式监听 ButtonClicked 事件，
//! 当模态框中的按钮被点击时触发 ModalConfirmed/ModalCancelled 事件，
//! 并清除模态框。
//!
//! 采用 Observer 而非 EventReader 的原因：
//! Button 系统使用 commands.trigger() 发送 ButtonClicked（Observer 模式），
//! 而非 EventWriter。EventReader<ButtonClicked> 无法接收到该信号。

use bevy::ecs::observer::On;
use bevy::prelude::*;

use super::components::{ModalButtonRole, ModalState};
use super::events::{ModalCancelled, ModalConfirmed};
use crate::ui::primitives::button::events::ButtonClicked;

/// 模态框交互观察者
///
/// 监听 ButtonClicked 触发器，检查被点击按钮是否带有 ModalButtonRole 标记。
/// 如果是模态框内的按钮，则沿 ChildOf 链向上查找 ModalState 所在实体，
/// 根据按钮角色触发 ModalConfirmed / ModalCancelled 事件并从场景中移除模态框。
pub fn modal_interaction_observer(
    trigger: On<ButtonClicked>,
    mut commands: Commands,
    role_query: Query<&ModalButtonRole>,
    parent_query: Query<&ChildOf>,
    modal_query: Query<Entity, With<ModalState>>,
) {
    let button_entity = trigger.event().entity;

    // 只处理标记了 ModalButtonRole 的按钮
    let Ok(role) = role_query.get(button_entity) else {
        return;
    };

    // 沿 ChildOf 链向上查找模态框（ModalState 所在的浮层实体）
    let mut current = button_entity;
    let overlay = loop {
        match parent_query.get(current) {
            Ok(parent) => {
                let parent_entity = parent.parent();
                if modal_query.contains(parent_entity) {
                    break Some(parent_entity);
                }
                current = parent_entity;
            }
            Err(_) => break None,
        }
    };

    let Some(overlay) = overlay else {
        return;
    };

    // 根据按钮角色触发对应事件
    match role {
        ModalButtonRole::Confirm => {
            commands.trigger(ModalConfirmed { entity: overlay });
        }
        ModalButtonRole::Cancel => {
            commands.trigger(ModalCancelled { entity: overlay });
        }
    }

    // 清除整个模态框节点树
    commands.entity(overlay).despawn();
}
