//! DefinitionType trait — 所有可加载 Definition 的公共接口
//!
//! 详见 ADR-047 §1

use bevy::prelude::{Asset, TypePath};

use super::errors::{ConfigError, ValidationError};

/// Sealed trait — 防止外部实现破坏 DefinitionType 的不变量。
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Definition 类型元信息 trait。
///
/// 存在原因：Content 层需要统一加载各种 Def 类型（SpellDef、EffectDef 等），
/// 此 trait 提供桶名、文件扩展名、校验逻辑等元信息，驱动通用加载管线。
pub trait DefinitionType: sealed::Sealed + Asset + TypePath {
    /// 在 Registry 中的桶名（如 "spells", "effects"）。
    const BUCKET_NAME: &'static str;

    /// 配置文件扩展名。
    const EXTENSION: &'static str;

    /// 从反序列化后的数据创建 Definition。
    ///
    /// 默认实现直接返回 Ok(data)，子类型可覆盖以执行转换逻辑。
    fn from_deserialized(data: Self) -> Result<Self, ConfigError>
    where
        Self: Sized,
    {
        Ok(data)
    }

    /// 加载后校验 Definition 的完整性。
    ///
    /// 校验内容包括：
    /// - ID 格式合法性
    /// - 必填字段完整性
    /// - 数值范围合理性
    fn validate(&self) -> Result<(), ValidationError>;

    /// 返回此 Definition 类型对应的文件路径前缀。
    ///
    /// 例如 "config/spells/" 或 "config/abilities/"。
    fn config_dir() -> &'static str {
        static CONFIG_DIR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        CONFIG_DIR.get_or_init(|| format!("config/{}", Self::BUCKET_NAME))
    }
}

/// 校验单个 Definition ID 格式。
///
/// 合法格式：`{prefix}_{digits}`（如 `spl_000001`）。
pub fn validate_id_format(id: &str, prefix: &str) -> Result<(), ValidationError> {
    if id.is_empty() {
        return Err(ValidationError::EmptyId);
    }

    if !id.starts_with(prefix) {
        return Err(ValidationError::InvalidIdPrefix {
            id: id.to_string(),
            expected_prefix: prefix.to_string(),
        });
    }

    let after_prefix = &id[prefix.len()..];
    if after_prefix.is_empty() || !after_prefix.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError::InvalidIdFormat {
            id: id.to_string(),
            detail: format!(
                "after prefix '{}', expected digits, got '{}'",
                prefix, after_prefix
            ),
        });
    }

    Ok(())
}
