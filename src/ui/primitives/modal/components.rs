//! Modal 组件的类型定义
//!
//! 定义 ModalVariant 枚举和 ModalState（Widget Contract 的本地状态）
//! 及 ModalButtonRole（用于标记模态框内按钮的角色）。
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §8

use bevy::prelude::*;

/// 模态框样式变体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ModalVariant {
    /// 提示弹窗：单个"确定"按钮
    Alert,
    /// 确认弹窗："取消" + "确认"两个按钮
    Confirm,
    /// 自定义弹窗：无默认按钮，调用方自主添加
    Custom,
}

/// 模态框本地状态（Widget Contract Local State）
///
/// 包含变体、标题、正文和打开状态。
/// Props 字段由 spawn_modal 的入参决定。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ModalState {
    /// 模态框样式变体
    pub variant: ModalVariant,
    /// 模态框标题文本
    pub title: String,
    /// 模态框正文消息
    pub message: String,
    /// 模态框是否打开
    pub open: bool,
}

/// 模态框按钮角色标记
///
/// 标记按钮在模态框中的功能角色，由 modal_interaction_observer 读取。
/// 按钮本身仍通过 ButtonSystem 处理交互，ModalButtonRole 仅用于
/// 将按钮点击路由到模态框的确认/取消逻辑。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum ModalButtonRole {
    /// 确认/主操作（触发 ModalConfirmed 事件）
    Confirm,
    /// 取消/关闭操作（触发 ModalCancelled 事件）
    Cancel,
}
