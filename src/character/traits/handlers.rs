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
