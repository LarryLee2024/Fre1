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
    use super::*;
    use crate::core::attribute::{AttributeKind, ModifierOp};

    // ── GrantTagHandler ──

    #[test]
    fn grant_tag_handler_类型名() {
        let handler = GrantTagHandler;
        assert_eq!(handler.type_name(), "GrantTag");
    }

    #[test]
    fn grant_tag_handler_授予标签() {
        let handler = GrantTagHandler;
        let effect = TraitEffect::GrantTag(GameplayTag::WARRIOR);
        let tags = handler.granted_tags(&effect);
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&GameplayTag::WARRIOR));
    }

    #[test]
    fn grant_tag_handler_非grant_tag返回空() {
        let handler = GrantTagHandler;
        let effect = TraitEffect::ModifyAttribute(AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 5.0,
        });
        let tags = handler.granted_tags(&effect);
        assert!(tags.is_empty());
    }

    #[test]
    fn grant_tag_handler_无属性修饰() {
        let handler = GrantTagHandler;
        let effect = TraitEffect::GrantTag(GameplayTag::FIRE);
        let mods = handler.attribute_modifiers(&effect);
        assert!(mods.is_empty());
    }

    // ── ModifyAttributeHandler ──

    #[test]
    fn modify_attribute_handler_类型名() {
        let handler = ModifyAttributeHandler;
        assert_eq!(handler.type_name(), "ModifyAttribute");
    }

    #[test]
    fn modify_attribute_handler_返回属性修饰() {
        let handler = ModifyAttributeHandler;
        let mod_def = AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 10.0,
        };
        let effect = TraitEffect::ModifyAttribute(mod_def);
        let mods = handler.attribute_modifiers(&effect);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].kind, AttributeKind::Attack);
        assert_eq!(mods[0].value, 10.0);
    }

    #[test]
    fn modify_attribute_handler_非modify返回空() {
        let handler = ModifyAttributeHandler;
        let effect = TraitEffect::GrantTag(GameplayTag::MAGE);
        let mods = handler.attribute_modifiers(&effect);
        assert!(mods.is_empty());
    }

    #[test]
    fn modify_attribute_handler_无标签授予() {
        let handler = ModifyAttributeHandler;
        let effect = TraitEffect::ModifyAttribute(AttributeModifierDef {
            kind: AttributeKind::Defense,
            op: ModifierOp::Multiply,
            value: 1.5,
        });
        let tags = handler.granted_tags(&effect);
        assert!(tags.is_empty());
    }

    // ── ApplyBuffHandler ──

    #[test]
    fn apply_buff_handler_类型名() {
        let handler = ApplyBuffHandler;
        assert_eq!(handler.type_name(), "ApplyBuff");
    }

    #[test]
    fn apply_buff_handler_无标签授予() {
        let handler = ApplyBuffHandler;
        let effect = TraitEffect::ApplyBuff {
            buff_id: "burn".into(),
            duration: 3,
        };
        let tags = handler.granted_tags(&effect);
        assert!(tags.is_empty());
    }

    #[test]
    fn apply_buff_handler_无属性修饰() {
        let handler = ApplyBuffHandler;
        let effect = TraitEffect::ApplyBuff {
            buff_id: "poison".into(),
            duration: 2,
        };
        let mods = handler.attribute_modifiers(&effect);
        assert!(mods.is_empty());
    }

    // ── TraitEffectHandlerRegistry ──

    #[test]
    fn registry_默认包含三个处理器() {
        let registry = TraitEffectHandlerRegistry::with_defaults();
        assert!(registry.get("GrantTag").is_some());
        assert!(registry.get("ModifyAttribute").is_some());
        assert!(registry.get("ApplyBuff").is_some());
    }

    #[test]
    fn registry_查询不存在返回none() {
        let registry = TraitEffectHandlerRegistry::with_defaults();
        assert!(registry.get("NonExistent").is_none());
    }

    #[test]
    fn registry_注册自定义处理器() {
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
        registry.register(Box::new(CustomHandler));
        assert!(registry.get("Custom").is_some());
        let handler = registry.get("Custom").unwrap();
        let effect = TraitEffect::GrantTag(GameplayTag::FIRE);
        let tags = handler.granted_tags(&effect);
        assert!(tags.contains(&GameplayTag::BUFF));
    }

    #[test]
    fn registry_default等于with_defaults() {
        let r1 = TraitEffectHandlerRegistry::default();
        let r2 = TraitEffectHandlerRegistry::with_defaults();
        assert!(r1.get("GrantTag").is_some());
        assert!(r2.get("GrantTag").is_some());
    }
}
