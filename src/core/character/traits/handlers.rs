// Trait 效果处理器：TraitEffectHandler trait 及内置实现

use super::types::TraitEffect;
use crate::core::attribute::AttributeModifierDef;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use std::collections::HashMap;

/// 特性效果处理规则 trait：描述如何应用一种特性效果
/// 新增效果类型时只需实现此 trait 并注册到 TraitEffectHandlerRegistry，
/// 无需修改 TraitData 的 granted_tags/attribute_modifiers 方法
pub trait TraitEffectHandler: Send + Sync + 'static {
    /// 效果类型名（与 TraitEffect variant 名对应）
    fn type_name(&self) -> &'static str;
    /// 收集此效果授予的标签
    fn granted_tags(&self, effect: &TraitEffect) -> Vec<GameplayTag>;
    /// 收集此效果的属性修饰（返回引用的生命周期与 effect 绑定）
    fn attribute_modifiers<'a>(&self, effect: &'a TraitEffect) -> Vec<&'a AttributeModifierDef>;
}

/// GrantTag 效果处理器
pub struct GrantTagHandler;

impl TraitEffectHandler for GrantTagHandler {
    fn type_name(&self) -> &'static str {
        "GrantTag"
    }

    fn granted_tags(&self, effect: &TraitEffect) -> Vec<GameplayTag> {
        match effect {
            TraitEffect::GrantTag(tag) => vec![*tag],
            _ => vec![],
        }
    }

    fn attribute_modifiers<'a>(&self, _effect: &'a TraitEffect) -> Vec<&'a AttributeModifierDef> {
        vec![]
    }
}

/// ModifyAttribute 效果处理器
pub struct ModifyAttributeHandler;

impl TraitEffectHandler for ModifyAttributeHandler {
    fn type_name(&self) -> &'static str {
        "ModifyAttribute"
    }

    fn granted_tags(&self, _effect: &TraitEffect) -> Vec<GameplayTag> {
        vec![]
    }

    fn attribute_modifiers<'a>(&self, effect: &'a TraitEffect) -> Vec<&'a AttributeModifierDef> {
        match effect {
            TraitEffect::ModifyAttribute(mod_def) => vec![mod_def],
            _ => vec![],
        }
    }
}

/// ApplyBuff 效果处理器
pub struct ApplyBuffHandler;

impl TraitEffectHandler for ApplyBuffHandler {
    fn type_name(&self) -> &'static str {
        "ApplyBuff"
    }

    fn granted_tags(&self, _effect: &TraitEffect) -> Vec<GameplayTag> {
        vec![]
    }

    fn attribute_modifiers<'a>(&self, _effect: &'a TraitEffect) -> Vec<&'a AttributeModifierDef> {
        vec![]
    }
}

/// 特性效果处理器注册表（Bevy Resource）
/// 通过 type_name 查找对应的 TraitEffectHandler，实现效果分发的扩展点
#[derive(Resource)]
pub struct TraitEffectHandlerRegistry {
    handlers: HashMap<&'static str, Box<dyn TraitEffectHandler>>,
}

impl TraitEffectHandlerRegistry {
    /// 创建包含所有内置处理器的注册表
    pub fn with_defaults() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };
        registry.register(Box::new(GrantTagHandler));
        registry.register(Box::new(ModifyAttributeHandler));
        registry.register(Box::new(ApplyBuffHandler));
        registry
    }

    /// 注册一个效果处理器
    pub fn register(&mut self, handler: Box<dyn TraitEffectHandler>) {
        self.handlers.insert(handler.type_name(), handler);
    }

    /// 根据类型名查找处理器
    pub fn get(&self, type_name: &str) -> Option<&dyn TraitEffectHandler> {
        self.handlers.get(type_name).map(|h| h.as_ref())
    }
}

impl Default for TraitEffectHandlerRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;
    use crate::core::attribute::ModifierOp;

    // ── GrantTagHandler ──

    /// Test ID: CHR-HDL-001
    /// Title: GrantTagHandler type_name 返回 "GrantTag"
    ///
    /// Given: 一个 GrantTagHandler 实例
    /// When: 调用 type_name()
    /// Then: 返回 "GrantTag"
    ///
    /// Assertions: type_name == "GrantTag"
    #[test]
    fn 授予标签处理器_类型名() {
        // Given
        let handler = GrantTagHandler;

        // When
        let name = handler.type_name();

        // Then
        assert_eq!(name, "GrantTag");
    }

    /// Test ID: CHR-HDL-002
    /// Title: GrantTagHandler 授予标签
    ///
    /// Given: 一个 GrantTagHandler 和 GrantTag(WARRIOR) 效果
    /// When: 调用 granted_tags()
    /// Then: 返回包含 WARRIOR 的标签列表
    ///
    /// Assertions: tags.len() == 1, tags.contains(WARRIOR)
    #[test]
    fn 授予标签处理器_授予标签() {
        // Given
        let handler = GrantTagHandler;
        let effect = TraitEffect::GrantTag(GameplayTag::ALLY);

        // When
        let tags = handler.granted_tags(&effect);

        // Then
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&GameplayTag::ALLY));
    }

    /// Test ID: CHR-HDL-003
    /// Title: GrantTagHandler 对非 GrantTag 效果返回空标签
    ///
    /// Given: 一个 GrantTagHandler 和 ModifyAttribute 效果
    /// When: 调用 granted_tags()
    /// Then: 返回空列表
    ///
    /// Assertions: tags.is_empty()
    #[test]
    fn 授予标签处理器_非授予标签返回空() {
        // Given
        let handler = GrantTagHandler;
        let effect = TraitEffect::ModifyAttribute(AttributeModifierDef {
            config_id: "phys_atk".to_string(),
            op: ModifierOp::Add,
            value: 5,
        });

        // When
        let tags = handler.granted_tags(&effect);

        // Then
        assert!(tags.is_empty());
    }

    /// Test ID: CHR-HDL-004
    /// Title: GrantTagHandler 无属性修饰
    ///
    /// Given: 一个 GrantTagHandler 和 GrantTag(FIRE) 效果
    /// When: 调用 attribute_modifiers()
    /// Then: 返回空列表
    ///
    /// Assertions: mods.is_empty()
    #[test]
    fn 授予标签处理器_无属性修饰符() {
        // Given
        let handler = GrantTagHandler;
        let effect = TraitEffect::GrantTag(GameplayTag::DMG_FIRE);

        // When
        let mods = handler.attribute_modifiers(&effect);

        // Then
        assert!(mods.is_empty());
    }

    // ── ModifyAttributeHandler ──

    /// Test ID: CHR-HDL-005
    /// Title: ModifyAttributeHandler type_name 返回 "ModifyAttribute"
    ///
    /// Given: 一个 ModifyAttributeHandler 实例
    /// When: 调用 type_name()
    /// Then: 返回 "ModifyAttribute"
    ///
    /// Assertions: type_name == "ModifyAttribute"
    #[test]
    fn 修改属性处理器_类型名() {
        // Given
        let handler = ModifyAttributeHandler;

        // When
        let name = handler.type_name();

        // Then
        assert_eq!(name, "ModifyAttribute");
    }

    /// Test ID: CHR-HDL-006
    /// Title: ModifyAttributeHandler 返回属性修饰
    ///
    /// Given: 一个 ModifyAttributeHandler 和 ModifyAttribute(Attack+10) 效果
    /// When: 调用 attribute_modifiers()
    /// Then: 返回包含 Attack 修饰符的列表
    ///
    /// Assertions: mods.len() == 1, kind == Attack, value == 10.0
    #[test]
    fn 修改属性处理器_返回修饰符() {
        // Given
        let handler = ModifyAttributeHandler;
        let effect = TraitEffect::ModifyAttribute(AttributeModifierDef {
            config_id: "phys_atk".to_string(),
            op: ModifierOp::Add,
            value: 10,
        });

        // When
        let mods = handler.attribute_modifiers(&effect);

        // Then
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].config_id, "phys_atk");
        assert_eq!(mods[0].value, 10);
    }

    /// Test ID: CHR-HDL-007
    /// Title: ModifyAttributeHandler 对非 ModifyAttribute 效果返回空修饰符
    ///
    /// Given: 一个 ModifyAttributeHandler 和 GrantTag(MAGE) 效果
    /// When: 调用 attribute_modifiers()
    /// Then: 返回空列表
    ///
    /// Assertions: mods.is_empty()
    #[test]
    fn 修改属性处理器_非修改返回空() {
        // Given
        let handler = ModifyAttributeHandler;
        let effect = TraitEffect::GrantTag(GameplayTag::SPECIAL_STATE);

        // When
        let mods = handler.attribute_modifiers(&effect);

        // Then
        assert!(mods.is_empty());
    }

    /// Test ID: CHR-HDL-008
    /// Title: ModifyAttributeHandler 无标签授予
    ///
    /// Given: 一个 ModifyAttributeHandler 和 ModifyAttribute(Defense*1.5) 效果
    /// When: 调用 granted_tags()
    /// Then: 返回空列表
    ///
    /// Assertions: tags.is_empty()
    #[test]
    fn 修改属性处理器_无标签() {
        // Given
        let handler = ModifyAttributeHandler;
        let effect = TraitEffect::ModifyAttribute(AttributeModifierDef {
            config_id: "phys_def".to_string(),
            op: ModifierOp::Multiply,
            value: 15000,
        });

        // When
        let tags = handler.granted_tags(&effect);

        // Then
        assert!(tags.is_empty());
    }

    // ── ApplyBuffHandler ──

    /// Test ID: CHR-HDL-009
    /// Title: ApplyBuffHandler type_name 返回 "ApplyBuff"
    ///
    /// Given: 一个 ApplyBuffHandler 实例
    /// When: 调用 type_name()
    /// Then: 返回 "ApplyBuff"
    ///
    /// Assertions: type_name == "ApplyBuff"
    #[test]
    fn 施加buff处理器_类型名() {
        // Given
        let handler = ApplyBuffHandler;

        // When
        let name = handler.type_name();

        // Then
        assert_eq!(name, "ApplyBuff");
    }

    /// Test ID: CHR-HDL-010
    /// Title: ApplyBuffHandler 无标签授予
    ///
    /// Given: 一个 ApplyBuffHandler 和 ApplyBuff(burn, 3) 效果
    /// When: 调用 granted_tags()
    /// Then: 返回空列表
    ///
    /// Assertions: tags.is_empty()
    #[test]
    fn 施加buff处理器_无标签() {
        // Given
        let handler = ApplyBuffHandler;
        let effect = TraitEffect::ApplyBuff {
            buff_id: "burn".into(),
            duration: 3,
        };

        // When
        let tags = handler.granted_tags(&effect);

        // Then
        assert!(tags.is_empty());
    }

    /// Test ID: CHR-HDL-011
    /// Title: ApplyBuffHandler 无属性修饰
    ///
    /// Given: 一个 ApplyBuffHandler 和 ApplyBuff(poison, 2) 效果
    /// When: 调用 attribute_modifiers()
    /// Then: 返回空列表
    ///
    /// Assertions: mods.is_empty()
    #[test]
    fn 施加buff处理器_无修饰符() {
        // Given
        let handler = ApplyBuffHandler;
        let effect = TraitEffect::ApplyBuff {
            buff_id: "poison".into(),
            duration: 2,
        };

        // When
        let mods = handler.attribute_modifiers(&effect);

        // Then
        assert!(mods.is_empty());
    }

    // ── TraitEffectHandlerRegistry ──

    /// Test ID: CHR-HDL-012
    /// Title: Registry 默认包含三个处理器
    ///
    /// Given: 一个使用 with_defaults() 创建的 Registry
    /// When: 查询三种处理器类型
    /// Then: 全部存在
    ///
    /// Assertions: get("GrantTag").is_some(), get("ModifyAttribute").is_some(), get("ApplyBuff").is_some()
    #[test]
    fn 注册表_默认包含三个处理器() {
        // Given
        let registry = TraitEffectHandlerRegistry::with_defaults();

        // When & Then
        assert!(registry.get("GrantTag").is_some());
        assert!(registry.get("ModifyAttribute").is_some());
        assert!(registry.get("ApplyBuff").is_some());
    }

    /// Test ID: CHR-HDL-013
    /// Title: Registry 查询不存在的处理器返回 None
    ///
    /// Given: 一个使用 with_defaults() 创建的 Registry
    /// When: 查询 "NonExistent"
    /// Then: 返回 None
    ///
    /// Assertions: get("NonExistent").is_none()
    #[test]
    fn 注册表_查询不存在返回none() {
        // Given
        let registry = TraitEffectHandlerRegistry::with_defaults();

        // When
        let result = registry.get("NonExistent");

        // Then
        assert!(result.is_none());
    }

    /// Test ID: CHR-HDL-014
    /// Title: Registry 注册自定义处理器
    ///
    /// Given: 一个 Registry 和一个自定义 Handler
    /// When: 注册并查询
    /// Then: 自定义 Handler 可用并正确执行
    ///
    /// Assertions: get("Custom").is_some(), granted_tags 返回 BUFF
    #[test]
    fn 注册表_注册自定义处理器() {
        // Given
        let mut registry = TraitEffectHandlerRegistry::with_defaults();
        struct CustomHandler;
        impl TraitEffectHandler for CustomHandler {
            fn type_name(&self) -> &'static str {
                "Custom"
            }
            fn granted_tags(&self, _effect: &TraitEffect) -> Vec<GameplayTag> {
                vec![GameplayTag::BUFF]
            }
            fn attribute_modifiers<'a>(
                &self,
                _effect: &'a TraitEffect,
            ) -> Vec<&'a AttributeModifierDef> {
                vec![]
            }
        }

        // When
        registry.register(Box::new(CustomHandler));
        let handler = registry.get("Custom").unwrap();
        let effect = TraitEffect::GrantTag(GameplayTag::DMG_FIRE);
        let tags = handler.granted_tags(&effect);

        // Then
        assert!(registry.get("Custom").is_some());
        assert!(tags.contains(&GameplayTag::BUFF));
    }

    /// Test ID: CHR-HDL-015
    /// Title: Registry default() 等于 with_defaults()
    ///
    /// Given: 两个 Registry，分别用 default() 和 with_defaults() 创建
    /// When: 查询处理器
    /// Then: 两者功能相同
    ///
    /// Assertions: 两者均包含 "GrantTag"
    #[test]
    fn 注册表_默认等于带默认值() {
        // Given
        let r1 = TraitEffectHandlerRegistry::default();
        let r2 = TraitEffectHandlerRegistry::with_defaults();

        // When & Then
        assert!(r1.get("GrantTag").is_some());
        assert!(r2.get("GrantTag").is_some());
    }
}
