//! Module Name: Modal Widget — 模态弹窗原子组件
//!
//! 提供 Alert/Confirm/Custom 三种变体的模态弹窗。
//! 使用 Factory 模式创建，唯一入口为 spawn_modal()。
//! 交互通过 Observer 模式监听 ButtonClicked 事件驱动。
//!
//! Contract:
//!   Props (input):    variant, title, message（通过 ModalState）
//!   Events (output):  ModalConfirmed, ModalCancelled（Observer 模式）
//!   Local State:      无（Modal 是 Ephemeral 模式，关闭即销毁）
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §8

pub mod components;
pub mod events;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::ModalButtonRole;
use self::components::ModalState;
use self::events::ModalCancelled;
use self::events::ModalConfirmed;
use self::systems::modal_interaction_observer;

/// ModalPlugin — 注册 Modal Widget 所需的 Component/Event/System
///
/// 注意：ModalPlugin 需在 ButtonPlugin 之后注册，
/// 因为 Modal 依赖 Button 系统的 ButtonClicked 触发器。
///
/// 在 Bevy 0.19 中，Event 类型通过 #[derive(Event)] 自动注册，
/// 无需手动调用 add_event。
pub struct ModalPlugin;

impl Plugin for ModalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ModalState>()
            .register_type::<ModalButtonRole>()
            // ModalConfirmed / ModalCancelled 是 Event，Bevy 0.19 自动注册
            // 使用 Observer 模式监听按钮点击
            .add_observer(modal_interaction_observer);
    }
}
