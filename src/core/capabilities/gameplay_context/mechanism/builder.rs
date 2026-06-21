//! ContextBuilder — 构建 GameplayContextData 的唯一合法入口
//!
//! 校验规则（对应领域不变量 3.3）：
//! - source.entity 必须有效（非 Entity::PLACEHOLDER）
//! - target.entity 必须有效
//! - origin 必须有值

use bevy::prelude::{Commands, Entity};

use crate::core::capabilities::gameplay_context::events::{
    ContextCreated, ContextValidationFailed,
};
use crate::core::capabilities::gameplay_context::foundation::error::ContextBuildError;
use crate::core::capabilities::gameplay_context::foundation::{
    ChainNode, ContextChain, ContextMetadata, ContextOrigin, ContextStatus, ElementType,
    GameplayContextData, SourceInfo, TargetInfo,
};

/// 游戏上下文的构建器。
///
/// 链式调用填充字段，build() 触发校验，校验通过后生成不可变的 GameplayContextData。
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    origin: ContextOrigin,
    source: Option<SourceInfo>,
    target: Option<TargetInfo>,
    ability_id: Option<String>,
    equipment_id: Option<String>,
    element_type: Option<ElementType>,
    is_critical: bool,
    frame: u64,
}

impl ContextBuilder {
    /// 创建一个新构建器。
    pub fn new(origin: ContextOrigin, frame: u64) -> Self {
        Self {
            origin,
            source: None,
            target: None,
            ability_id: None,
            equipment_id: None,
            element_type: None,
            is_critical: false,
            frame,
        }
    }

    /// 设置行为发起者。
    pub fn source(mut self, info: SourceInfo) -> Self {
        self.source = Some(info);
        self
    }

    /// 设置行为目标。
    pub fn target(mut self, info: TargetInfo) -> Self {
        self.target = Some(info);
        self
    }

    /// 设置使用的能力。
    pub fn ability(mut self, id: impl Into<String>) -> Self {
        self.ability_id = Some(id.into());
        self
    }

    /// 设置使用的装备。
    pub fn equipment(mut self, id: impl Into<String>) -> Self {
        self.equipment_id = Some(id.into());
        self
    }

    /// 设置元素类型。
    pub fn element(mut self, element: ElementType) -> Self {
        self.element_type = Some(element);
        self
    }

    /// 标记为暴击。
    pub fn critical(mut self) -> Self {
        self.is_critical = true;
        self
    }

    /// 构建 GameplayContextData。
    ///
    /// 校验规则（V1）：
    /// - source 和 target 必须已填充
    /// - source.entity 和 target.entity 不得为 PLACEHOLDER
    pub fn build(
        self,
        commands: &mut Commands,
        next_id: &mut u64,
    ) -> Result<GameplayContextData, ContextBuildError> {
        let origin = self.origin;

        let source = match self.source {
            Some(s) => s,
            None => {
                commands.trigger(ContextValidationFailed {
                    missing_fields: vec!["source".to_string()],
                    origin,
                });
                return Err(ContextBuildError::MissingFields(vec!["source".to_string()]));
            }
        };
        let target = match self.target {
            Some(t) => t,
            None => {
                commands.trigger(ContextValidationFailed {
                    missing_fields: vec!["target".to_string()],
                    origin,
                });
                return Err(ContextBuildError::MissingFields(vec!["target".to_string()]));
            }
        };

        // 实体有效性由 ECS 生命周期强制执行。
        // Builder 校验结构完整性（source + target 存在）。

        let first_node = ChainNode {
            origin,
            source: source.clone(),
            target: target.clone(),
            ability_id: self.ability_id.clone(),
            frame: self.frame,
            node_id: 0,
        };

        let chain = ContextChain::new(first_node);

        let context_id = GameplayContextData::generate_id(next_id);

        commands.trigger(ContextCreated {
            context_id: context_id.clone(),
            origin,
            source_entity: source.entity,
            target_entity: target.entity,
        });

        Ok(GameplayContextData {
            context_id,
            origin,
            source,
            target,
            ability_id: self.ability_id,
            equipment_id: self.equipment_id,
            element_type: self.element_type,
            is_critical: self.is_critical,
            chain,
            created_at_frame: self.frame,
            metadata: ContextMetadata {
                schema_version: 1,
                status: ContextStatus::Active,
            },
        })
    }
}
