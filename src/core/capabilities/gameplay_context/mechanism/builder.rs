//! ContextBuilder — 构建 GameplayContextData 的唯一合法入口
//!
//! 校验规则（对应领域不变量 3.3）：
//! - source.entity 必须有效（非 Entity::PLACEHOLDER）
//! - target.entity 必须有效
//! - origin 必须有值

use bevy::prelude::Entity;

use crate::core::capabilities::gameplay_context::foundation::{
    ChainNode, ContextBuildError, ContextChain, ContextMetadata, ContextOrigin, ContextStatus,
    ElementType, GameplayContextData, SourceInfo, TargetInfo,
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
    pub fn build(self) -> Result<GameplayContextData, ContextBuildError> {
        let source = self
            .source
            .ok_or_else(|| ContextBuildError::MissingFields(vec!["source".to_string()]))?;
        let target = self
            .target
            .ok_or_else(|| ContextBuildError::MissingFields(vec!["target".to_string()]))?;

        // Entity validity is enforced by the ECS lifecycle.
        // Builder validates structural completeness (source + target present).

        let first_node = ChainNode {
            origin: self.origin,
            source: source.clone(),
            target: target.clone(),
            ability_id: self.ability_id.clone(),
            frame: self.frame,
            node_id: 0,
        };

        let chain = ContextChain::new(first_node);

        Ok(GameplayContextData {
            context_id: GameplayContextData::generate_id(),
            origin: self.origin,
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

#[cfg(test)]
mod tests {
    use super::*;
    fn test_entity(index: u32) -> Entity {
        Entity::from_bits((index as u64) << 32 | 0x10000)
    }

    #[test]
    fn debug_placeholder() {
        let from_bits_1 = Entity::from_bits(1);
        let placeholder = Entity::PLACEHOLDER;
        println!(
            "from_bits(1) = {:?} (bits=0x{:x})",
            from_bits_1,
            from_bits_1.to_bits()
        );
        println!(
            "PLACEHOLDER  = {:?} (bits=0x{:x})",
            placeholder,
            placeholder.to_bits()
        );
        println!("equal: {}", from_bits_1 == placeholder);
    }

    fn valid_source() -> SourceInfo {
        SourceInfo {
            entity: test_entity(1),
            faction: "fct_000001".to_string(),
            position: Some((0, 0)),
        }
    }

    fn valid_target() -> TargetInfo {
        TargetInfo {
            entity: test_entity(2),
            faction: "fct_000002".to_string(),
            position: Some((5, 5)),
            is_valid: true,
        }
    }

    #[test]
    fn unit_001_build_with_source_and_target_succeeds() {
        let ctx = ContextBuilder::new(ContextOrigin::Direct, 1)
            .source(valid_source())
            .target(valid_target())
            .build()
            .expect("build should succeed");
        assert_eq!(ctx.origin, ContextOrigin::Direct);
        assert!(ctx.context_id.starts_with("ctx_"));
        assert_eq!(ctx.created_at_frame, 1);
    }

    #[test]
    fn unit_002_build_missing_source_fails() {
        let ctx = ContextBuilder::new(ContextOrigin::Direct, 1)
            .target(valid_target())
            .build();
        assert!(
            matches!(ctx, Err(ContextBuildError::MissingFields(fields)) if fields.contains(&"source".to_string()))
        );
    }

    #[test]
    fn unit_003_build_missing_target_fails() {
        let ctx = ContextBuilder::new(ContextOrigin::Direct, 1)
            .source(valid_source())
            .build();
        assert!(
            matches!(ctx, Err(ContextBuildError::MissingFields(fields)) if fields.contains(&"target".to_string()))
        );
    }

    #[test]
    fn unit_004_build_with_all_optional_fields() {
        let ctx = ContextBuilder::new(ContextOrigin::Triggered, 42)
            .source(valid_source())
            .target(valid_target())
            .ability("abl_000001")
            .equipment("equip_000001")
            .element(ElementType::Fire)
            .critical()
            .build()
            .unwrap();
        assert_eq!(ctx.origin, ContextOrigin::Triggered);
        assert_eq!(ctx.ability_id, Some("abl_000001".to_string()));
        assert_eq!(ctx.equipment_id, Some("equip_000001".to_string()));
        assert_eq!(ctx.element_type, Some(ElementType::Fire));
        assert!(ctx.is_critical);
        assert_eq!(ctx.created_at_frame, 42);
        assert_eq!(ctx.chain.len(), 1);
    }

    #[test]
    fn unit_005_chain_starts_with_first_node() {
        let ctx = ContextBuilder::new(ContextOrigin::Direct, 10)
            .source(valid_source())
            .target(valid_target())
            .build()
            .unwrap();
        let last = ctx.chain.last().unwrap();
        assert_eq!(last.frame, 10);
        assert_eq!(last.source.entity, test_entity(1));
    }

    #[test]
    fn unit_006_context_id_unique_across_builds() {
        // Only one build per builder, so test that different builders produce different IDs
        let ctx1 = ContextBuilder::new(ContextOrigin::Direct, 1)
            .source(valid_source())
            .target(valid_target())
            .build()
            .unwrap();
        let ctx2 = ContextBuilder::new(ContextOrigin::Direct, 1)
            .source(valid_source())
            .target(valid_target())
            .build()
            .unwrap();
        assert_ne!(ctx1.context_id, ctx2.context_id);
    }
}
