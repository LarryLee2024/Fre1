//! Spec 生命周期管理
//!
//! SpecRegistry Resource 管理已注册的 Def 元数据，并提供 Def→Spec 的工厂转换
//! 以及 Spec 生命周期操作的纯函数（授予、移除、等级变更）。
//!
//! 详见 docs/02-domain/capabilities/spec_domain.md §5（流程定义）。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::spec::events::{
    SpecGranted, SpecLevelChanged, SpecRemovalReason, SpecRemoved,
};
use crate::core::capabilities::spec::foundation::{
    AbilitySpec, EffectSource, EffectSpec, SpecError, SpecId, SpecRegistryConfig, SpecType,
};

use crate::core::capabilities::spec::mechanism::components::SpecContainer;

/// 已注册 Def 的元数据条目。
#[derive(Debug, Clone)]
pub struct DefEntry {
    /// Def 类型
    pub def_type: SpecType,
    /// 最大等级（仅 AbilitySpec 有效）
    pub max_level: u8,
}

/// Spec 注册中心 Resource。
///
/// 管理已知 Def 的注册信息，提供 Def→Spec 的工厂转换。
/// 在 Content 层未实现时，通过 `register()` 手动注册 Def 元数据以通过不变量 V1。
#[derive(Resource, Debug, Clone)]
pub struct SpecRegistry {
    /// def_id → DefEntry 映射
    pub registered_defs: HashMap<String, DefEntry>,
    /// 注册中心配置
    pub config: SpecRegistryConfig,
}

impl SpecRegistry {
    /// 创建一个新的空 SpecRegistry。
    pub fn new(config: SpecRegistryConfig) -> Self {
        Self {
            registered_defs: HashMap::new(),
            config,
        }
    }

    /// 注册一个 Def 条目。
    pub fn register(&mut self, def_id: impl Into<String>, def_type: SpecType, max_level: u8) {
        self.registered_defs.insert(
            def_id.into(),
            DefEntry {
                def_type,
                max_level,
            },
        );
    }

    /// 检查 Def 是否已注册。
    pub fn is_registered(&self, def_id: &str) -> bool {
        self.registered_defs.contains_key(def_id)
    }

    /// 获取指定 Def 的条目信息。
    pub fn get_entry(&self, def_id: &str) -> Option<&DefEntry> {
        self.registered_defs.get(def_id)
    }

    /// 基于 Def 创建 AbilitySpec 实例。
    ///
    /// 校验：
    /// - V1: Def 必须已注册
    /// - V2: 等级必须在 [1, max_level] 范围内
    pub fn create_ability_spec(
        &self,
        def_id: impl Into<String>,
        level: u8,
    ) -> Result<AbilitySpec, SpecError> {
        let def_id = def_id.into();

        // V1: Def 必须已注册
        let entry = self
            .registered_defs
            .get(&def_id)
            .ok_or_else(|| SpecError::DefNotRegistered(def_id.clone()))?;

        // V2: 等级必须在合法范围
        validate_level(level, entry.max_level)?;

        Ok(AbilitySpec::new(def_id, level, entry.max_level))
    }

    /// 基于 Def 创建 EffectSpec 实例。
    ///
    /// 校验：
    /// - V1: Def 必须已注册
    pub fn create_effect_spec(
        &self,
        def_id: impl Into<String>,
        source: EffectSource,
        frame: u64,
    ) -> Result<EffectSpec, SpecError> {
        let def_id = def_id.into();

        // V1: Def 必须已注册
        self.registered_defs
            .get(&def_id)
            .ok_or_else(|| SpecError::DefNotRegistered(def_id.clone()))?;

        let spec = EffectSpec::new(def_id, source, frame);
        if self.config.enable_snapshot {
            // snapshot 已由 EffectSpec::new 初始化为 empty(frame)
            // 完整快照需在应用时由调用方填充属性值（不变量 V4）
        }

        Ok(spec)
    }
}

impl Default for SpecRegistry {
    fn default() -> Self {
        Self::new(SpecRegistryConfig::default())
    }
}

// ── 不变量校验函数 ──────────────────────────────────────────────

/// 校验等级是否在合法范围内（不变量 V2）。
///
/// # Errors
/// - `SpecError::LevelOutOfRange` 当 level 不在 [1, max_level] 范围内时。
pub fn validate_level(level: u8, max_level: u8) -> Result<(), SpecError> {
    if level < 1 || level > max_level {
        return Err(SpecError::LevelOutOfRange {
            level,
            min: 1,
            max: max_level,
        });
    }
    Ok(())
}

/// 校验同一实体是否已有同 Def 的 AbilitySpec（不变量 V3）。
///
/// # Errors
/// - `SpecError::DuplicateSpec` 当容器中已存在同 def_id 的 AbilitySpec 时。
pub fn validate_no_duplicate_ability(
    container: &SpecContainer,
    def_id: &str,
) -> Result<(), SpecError> {
    if container.has_ability_for_def(def_id)
        && let Some(existing_id) = container.find_ability_by_def(def_id)
    {
        return Err(SpecError::DuplicateSpec {
            def_id: def_id.to_string(),
            spec_id: existing_id.to_string(),
        });
    }
    Ok(())
}

// ── Spec 生命周期操作 ───────────────────────────────────────────

/// 授予一个 AbilitySpec 到实体的 SpecContainer。
///
/// 完整流程（领域规则 §5.1）：
/// 1. 校验 Def 已注册（V1）
/// 2. 校验无重复（V3）
/// 3. 通过 SpecRegistry 创建 Spec 实例
/// 4. 注册 Spec 到容器的 abilities + 索引
///
/// # Errors
/// 透传 SpecError 的错误变体。
pub fn grant_ability_spec(
    container: &mut SpecContainer,
    registry: &SpecRegistry,
    def_id: &str,
    level: u8,
    entity: Entity,
    commands: &mut Commands,
) -> Result<SpecId, SpecError> {
    // 1. 校验 Def 已注册 + 创建（含 V2 等级校验）
    let spec = registry.create_ability_spec(def_id, level)?;

    // 2. 校验无重复
    validate_no_duplicate_ability(container, def_id)?;

    // 3. 注册到容器
    let spec_id = spec.spec_id.clone();
    container.insert_ability(spec);

    commands.trigger(SpecGranted {
        entity,
        spec_id: spec_id.clone(),
        spec_type: SpecType::Ability,
        def_id: def_id.to_string(),
    });

    Ok(spec_id)
}

/// 从一个实体移除 AbilitySpec。
///
/// 完整流程（领域规则 §5.3）：
/// 1. 校验 Spec 存在
/// 2. 从容器移除 + 清理索引
///
/// # Errors
/// - `SpecError::SpecNotFound` 当指定 spec_id 不存在时。
pub fn remove_ability_spec(
    container: &mut SpecContainer,
    spec_id: &SpecId,
    entity: Entity,
    commands: &mut Commands,
) -> Result<(), SpecError> {
    if !container.abilities.contains_key(spec_id) {
        return Err(SpecError::SpecNotFound(spec_id.to_string()));
    }
    container.remove_ability(spec_id);

    commands.trigger(SpecRemoved {
        entity,
        spec_id: spec_id.clone(),
        reason: SpecRemovalReason::Manual,
    });

    Ok(())
}

/// 修改 AbilitySpec 的等级。
///
/// 完整流程（领域规则 §5.2）：
/// 1. 校验新等级在 [1, MaxLevel] 范围内（V2）
/// 2. 更新 Spec 等级
///
/// 注意：活跃 Instance 的检查由外部调用方负责（该检查需要 Ability 领域状态）。
///
/// # Errors
/// - `SpecError::SpecNotFound` 当指定 spec_id 不存在时。
/// - `SpecError::LevelOutOfRange` 当新等级越界时。
pub fn change_ability_level(
    container: &mut SpecContainer,
    spec_id: &SpecId,
    new_level: u8,
    entity: Entity,
    commands: &mut Commands,
) -> Result<(), SpecError> {
    let spec = container
        .get_ability_mut(spec_id)
        .ok_or_else(|| SpecError::SpecNotFound(spec_id.to_string()))?;

    // V2: 等级越界校验
    validate_level(new_level, spec.max_level)?;

    let old_level = spec.level;
    spec.level = new_level;

    commands.trigger(SpecLevelChanged {
        entity,
        spec_id: spec_id.clone(),
        old_level,
        new_level,
    });

    Ok(())
}

/// 授予一个 EffectSpec 到实体的 SpecContainer。
///
/// 流程：
/// 1. 校验 Def 已注册（V1）
/// 2. 通过 SpecRegistry 创建 EffectSpec 实例
/// 3. 注册 EffectSpec 到容器的 effects + 索引
///
/// # Errors
/// 透传 SpecError 的错误变体。
pub fn grant_effect_spec(
    container: &mut SpecContainer,
    registry: &SpecRegistry,
    def_id: &str,
    source: EffectSource,
    frame: u64,
    entity: Entity,
    commands: &mut Commands,
) -> Result<SpecId, SpecError> {
    let spec = registry.create_effect_spec(def_id, source, frame)?;

    let spec_id = spec.spec_id.clone();
    container.insert_effect(spec);

    commands.trigger(SpecGranted {
        entity,
        spec_id: spec_id.clone(),
        spec_type: SpecType::Effect,
        def_id: def_id.to_string(),
    });

    Ok(spec_id)
}

/// 从一个实体移除 EffectSpec。
///
/// # Errors
/// - `SpecError::SpecNotFound` 当指定 spec_id 不存在时。
pub fn remove_effect_spec(
    container: &mut SpecContainer,
    spec_id: &SpecId,
    entity: Entity,
    commands: &mut Commands,
) -> Result<(), SpecError> {
    if !container.effects.contains_key(spec_id) {
        return Err(SpecError::SpecNotFound(spec_id.to_string()));
    }
    container.remove_effect(spec_id);

    commands.trigger(SpecRemoved {
        entity,
        spec_id: spec_id.clone(),
        reason: SpecRemovalReason::Manual,
    });

    Ok(())
}
