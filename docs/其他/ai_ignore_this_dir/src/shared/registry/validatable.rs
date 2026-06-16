//! 校验型 Registry（ADR-030 §2.2）
//!
//! 提供 [`ValidatableRegistry`] trait，支持注册表在加载后执行自校验
//! （重复 ID 检查、业务约束验证等），以及跨注册表引用完整性检查的框架。
//!
//! # 验证时机
//! 所有验证在启动阶段（`PreStartup`）完成，验证失败阻止游戏启动。
//!
//! # 验证等级（ADR-029~035 Data Architecture §7）
//! - **Level 1** — Schema 验证（RON 反序列化时自动完成）
//! - **Level 2** — 引用完整性验证（跨 Registry 的 ID 引用检查）
//! - **Level 3** — 跨域一致性验证（互斥关系、初始化顺序等）

use std::fmt;

/// 注册表验证错误。
///
/// 包含定位信息（哪个 Registry、哪个 Key）、严重级别和详细描述。
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 来源注册表名称
    pub registry: &'static str,
    /// 关联的 Key（如果适用）
    pub key: Option<String>,
    /// 错误描述
    pub message: String,
    /// 严重级别
    pub severity: ValidationSeverity,
}

/// 验证严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// 错误：必须修复，阻止启动
    Error,
    /// 警告：建议修复，不影响启动
    Warning,
    /// 信息：仅记录
    Info,
}

impl ValidationError {
    /// 创建 Error 级别的验证错误
    pub fn error(registry: &'static str, message: impl Into<String>) -> Self {
        Self {
            registry,
            key: None,
            message: message.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// 创建包含 Key 的 Error 级别验证错误
    pub fn error_for(
        registry: &'static str,
        key: impl fmt::Display,
        message: impl Into<String>,
    ) -> Self {
        Self {
            registry,
            key: Some(key.to_string()),
            message: message.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// 创建 Warning 级别的验证错误
    pub fn warning(registry: &'static str, message: impl Into<String>) -> Self {
        Self {
            registry,
            key: None,
            message: message.into(),
            severity: ValidationSeverity::Warning,
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.key {
            Some(key) => write!(
                f,
                "[{:?}][{}] {}: {}",
                self.severity, self.registry, key, self.message
            ),
            None => write!(
                f,
                "[{:?}][{}] {}",
                self.severity, self.registry, self.message
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

/// 支持自校验的注册表。
///
/// 实现者应检查内部数据的一致性：
/// - 字段值是否在有效范围内
/// - 不变量是否满足（如 min ≤ max）
/// - 是否存在配置错误
///
/// 跨注册表引用完整性检查通过独立的 [`CrossRegistryValidator`] 完成。
pub trait ValidatableRegistry: super::Registry {
    /// 执行自校验，返回所有发现的错误。
    fn validate(&self) -> Vec<ValidationError>;
}

/// 跨注册表引用验证器。
///
/// 在多个 Registry 加载完成后执行，检查 ID 引用是否有效。
/// 例如：EffectRegistry 的 `execution` 字段是否引用了 ExecutionRegistry 中存在的 ID。
///
/// # 用法
/// 各域验证器应实现此 trait，并在所有依赖 Registry 加载完成后执行。
/// 实际交叉验证在 Steps 3-7 中按领域逐步实现。
///
/// ```ignore
/// struct EffectReferenceValidator;
///
/// impl CrossRegistryValidator for EffectReferenceValidator {
///     fn validate(
///         &self,
///         registries: &dyn RegistryProvider,
///     ) -> Vec<ValidationError> {
///         // 通过 registries 获取 EffectRegistry, ExecutionRegistry 等
///         // 检查所有 effect.execution 引用是否有效
///     }
/// }
/// ```
pub trait CrossRegistryValidator: Send + Sync + 'static {
    /// 执行跨注册表引用完整性检查。
    fn validate(&self, provider: &dyn RegistryProvider) -> Vec<ValidationError>;
}

/// Registry 提供者：按类型 ID 提供运行时 Registry 的只读访问。
///
/// 用于跨注册表验证（Level 2 引用完整性检查）。
/// 实际提供者由 App 在启动阶段组装，Steps 7-12 实现完整集成。
pub trait RegistryProvider {
    /// 通过 TypeId 获取类型擦除的 Registry 引用。
    fn get_registry(&self, type_id: std::any::TypeId) -> Option<&dyn std::any::Any>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ── ValidationError tests ──

    #[test]
    fn validation_error_display_with_key() {
        let err = ValidationError::error_for("TestRegistry", "item_1", "Value out of range");
        let msg = err.to_string();
        assert!(msg.contains("TestRegistry"));
        assert!(msg.contains("item_1"));
        assert!(msg.contains("Value out of range"));
    }

    #[test]
    fn validation_error_display_without_key() {
        let err = ValidationError::error("TestRegistry", "Registry is empty");
        let msg = err.to_string();
        assert!(msg.contains("TestRegistry"));
        assert!(msg.contains("Registry is empty"));
    }

    #[test]
    fn validation_error_severity_levels() {
        let err = ValidationError::error("R", "err");
        assert_eq!(err.severity, ValidationSeverity::Error);

        let warn = ValidationError::warning("R", "warn");
        assert_eq!(warn.severity, ValidationSeverity::Warning);
    }

    // ── ValidatableRegistry trait test ──

    #[derive(Default)]
    struct TestValidRegistry {
        items: HashMap<String, i32>,
    }

    use crate::shared::registry::Registry;

    impl Registry for TestValidRegistry {
        type Key = String;
        type Data = i32;

        fn len(&self) -> usize {
            self.items.len()
        }

        fn get(&self, key: &Self::Key) -> Option<&Self::Data> {
            self.items.get(key)
        }

        fn keys(&self) -> Vec<&Self::Key> {
            self.items.keys().collect()
        }

        fn iter(&self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Data)> + '_> {
            Box::new(self.items.iter())
        }
    }

    impl ValidatableRegistry for TestValidRegistry {
        fn validate(&self) -> Vec<ValidationError> {
            let mut errors = Vec::new();
            for (key, val) in &self.items {
                if *val < 0 {
                    errors.push(ValidationError::error_for(
                        "TestValidRegistry",
                        key,
                        format!("Value {} is negative, must be >= 0", val),
                    ));
                }
            }
            if self.items.is_empty() {
                errors.push(ValidationError::warning(
                    "TestValidRegistry",
                    "Registry is empty",
                ));
            }
            errors
        }
    }

    #[test]
    fn validatable_registry_valid_data() {
        let mut reg = TestValidRegistry::default();
        reg.items.insert("a".into(), 10);
        reg.items.insert("b".into(), 20);

        let errors = reg.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn validatable_registry_negative_value_error() {
        let mut reg = TestValidRegistry::default();
        reg.items.insert("bad".into(), -5);

        let errors = reg.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Error);
        assert!(errors[0].to_string().contains("negative"));
    }

    #[test]
    fn validatable_registry_empty_warning() {
        let reg = TestValidRegistry::default();
        let errors = reg.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Warning);
    }

    // ── CrossRegistryValidator + RegistryProvider tests ──

    struct TestProvider;

    impl RegistryProvider for TestProvider {
        fn get_registry(&self, _type_id: std::any::TypeId) -> Option<&dyn std::any::Any> {
            None
        }
    }

    struct TestValidator;

    impl CrossRegistryValidator for TestValidator {
        fn validate(&self, _provider: &dyn RegistryProvider) -> Vec<ValidationError> {
            vec![]
        }
    }

    #[test]
    fn cross_registry_validator_returns_empty() {
        let validator = TestValidator;
        let provider = TestProvider;
        let errors = validator.validate(&provider);
        assert!(errors.is_empty());
    }

    #[test]
    fn registry_provider_returns_none_for_unknown() {
        let provider = TestProvider;
        assert!(
            provider
                .get_registry(std::any::TypeId::of::<TestValidRegistry>())
                .is_none()
        );
    }
}
