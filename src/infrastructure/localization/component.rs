//! LocalizedText 组件

use bevy::prelude::*;

/// 标记一个 UI 元素需要本地化显示
/// 存储本地化 Key，由系统自动解析为当前语言文本
#[derive(Component, Debug, Clone)]
pub struct LocalizedText {
    /// 本地化 Key，如 "battle.turn.start"
    pub key: String,
    /// 可选的变量参数 Key（用于运行时动态值）
    pub args_keys: Option<Vec<(String, String)>>,
}

impl LocalizedText {
    /// 创建无参数的本地化文本
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            args_keys: None,
        }
    }

    /// 创建带参数的本地化文本
    pub fn with_args(key: impl Into<String>, args: Vec<(String, String)>) -> Self {
        Self {
            key: key.into(),
            args_keys: Some(args),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localized_text_new() {
        let lt = LocalizedText::new("ui.test");
        assert_eq!(lt.key, "ui.test");
        assert!(lt.args_keys.is_none());
    }

    #[test]
    fn localized_text_with_args() {
        let lt = LocalizedText::with_args(
            "ui.test",
            vec![("damage".into(), "100".into())],
        );
        assert_eq!(lt.key, "ui.test");
        assert!(lt.args_keys.is_some());
    }
}
