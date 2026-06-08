// Trait 系统：统一抽象，种族/职业/天赋/装备均可引用 traits
// 替代硬编码的 class_tag 单标签，支持多 trait 组合
// 支持从 assets/traits/*.ron 外部配置文件加载

use crate::gameplay::attribute::{AttributeModifierDef, AttributeModifierInstance, BuffInstanceId};
use crate::gameplay::tag::{GameplayTag, GameplayTags, TagName};
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

/// Trait 触发时机
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TraitTrigger {
    /// 被动：始终生效（授予标签/属性修饰）
    Passive,
    /// 回合开始时触发
    OnTurnStart,
    /// 回合结束时触发
    OnTurnEnd,
    /// 攻击时触发
    OnAttack,
    /// 被攻击时触发
    OnHit,
    /// 击杀时触发
    OnKill,
}

/// Trait 效果定义（RON 反序列化用，TagName 替代 GameplayTag）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TraitEffectDef {
    /// 授予标签
    GrantTag(TagName),
    /// 属性修饰（永久，作为基础值的一部分）
    ModifyAttribute(AttributeModifierDef),
    /// 触发时施加 Buff
    ApplyBuff { buff_id: String, duration: u32 },
}

/// Trait 效果（运行时，GameplayTag 替代 TagName）
#[derive(Clone, Debug)]
pub enum TraitEffect {
    GrantTag(GameplayTag),
    ModifyAttribute(AttributeModifierDef),
    ApplyBuff { buff_id: String, duration: u32 },
}

impl TraitEffect {
    /// 返回效果类型名（与 variant 名对应，用于 Handler 查找）
    pub fn type_name(&self) -> &'static str {
        match self {
            TraitEffect::GrantTag(_) => "GrantTag",
            TraitEffect::ModifyAttribute(_) => "ModifyAttribute",
            TraitEffect::ApplyBuff { .. } => "ApplyBuff",
        }
    }
}

// ── TraitEffectHandler trait 及内置实现 ──────────────────────────────

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

impl From<TraitEffectDef> for TraitEffect {
    fn from(def: TraitEffectDef) -> Self {
        match def {
            TraitEffectDef::GrantTag(tag_name) => TraitEffect::GrantTag(tag_name.to_tag()),
            TraitEffectDef::ModifyAttribute(mod_def) => TraitEffect::ModifyAttribute(mod_def),
            TraitEffectDef::ApplyBuff { buff_id, duration } => {
                TraitEffect::ApplyBuff { buff_id, duration }
            }
        }
    }
}

/// Trait 定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct TraitDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: TraitTrigger,
    pub effects: Vec<TraitEffectDef>,
}

/// Trait 数据（运行时）
#[derive(Clone, Debug)]
pub struct TraitData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: TraitTrigger,
    pub effects: Vec<TraitEffect>,
}

impl From<TraitDefinition> for TraitData {
    fn from(def: TraitDefinition) -> Self {
        TraitData {
            id: def.id,
            name: def.name,
            description: def.description,
            trigger: def.trigger,
            effects: def.effects.into_iter().map(Into::into).collect(),
        }
    }
}

impl TraitData {
    /// 收集此 trait 授予的所有标签（通过 TraitEffectHandlerRegistry 分发）
    pub fn granted_tags(&self, handlers: &TraitEffectHandlerRegistry) -> Vec<GameplayTag> {
        self.effects
            .iter()
            .flat_map(|e| {
                handlers
                    .get(e.type_name())
                    .map(|h| h.granted_tags(e))
                    .unwrap_or_default()
            })
            .collect()
    }

    /// 收集此 trait 的所有属性修饰（通过 TraitEffectHandlerRegistry 分发）
    pub fn attribute_modifiers<'a>(
        &'a self,
        handlers: &'a TraitEffectHandlerRegistry,
    ) -> Vec<&'a AttributeModifierDef> {
        self.effects
            .iter()
            .flat_map(|e| {
                handlers
                    .get(e.type_name())
                    .map(|h| h.attribute_modifiers(e))
                    .unwrap_or_default()
            })
            .collect()
    }
}

/// 单位上的 Trait 集合组件
#[derive(Component, Default, Debug, Clone)]
pub struct TraitCollection {
    pub trait_ids: Vec<String>,
}

impl TraitCollection {
    pub fn new(trait_ids: Vec<String>) -> Self {
        Self { trait_ids }
    }

    /// 是否拥有指定 trait
    pub fn has(&self, trait_id: &str) -> bool {
        self.trait_ids.iter().any(|t| t == trait_id)
    }
}

/// 将一组 trait 的被动效果应用到单位的标签和属性上
/// 返回 (granted_tags, attribute_modifiers)
pub fn apply_passive_traits(
    trait_ids: &[String],
    registry: &TraitRegistry,
    handlers: &TraitEffectHandlerRegistry,
) -> (GameplayTags, Vec<AttributeModifierInstance>) {
    let mut tags = GameplayTags::default();
    let mut modifiers = Vec::new();
    // 使用固定的 source id 标识 trait 来源的修饰符
    let trait_source = BuffInstanceId(u64::MAX);

    for trait_id in trait_ids {
        if let Some(trait_data) = registry.get(trait_id) {
            if trait_data.trigger != TraitTrigger::Passive {
                continue;
            }
            for tag in trait_data.granted_tags(handlers) {
                tags.add(tag);
            }
            for mod_def in trait_data.attribute_modifiers(handlers) {
                modifiers.push(AttributeModifierInstance {
                    kind: mod_def.kind,
                    op: mod_def.op,
                    value: mod_def.value,
                    source: trait_source,
                });
            }
        }
    }

    (tags, modifiers)
}

/// Trait 注册表资源
#[derive(Resource, Default)]
pub struct TraitRegistry {
    pub traits: HashMap<String, TraitData>,
}

impl TraitRegistry {
    pub fn get(&self, id: &str) -> Option<&TraitData> {
        self.traits.get(id)
    }

    /// 从 assets/traits/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = TraitRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("Trait 目录不存在: {}", dir);
            registry.register_defaults();
            return registry;
        };

        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<TraitDefinition>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.traits.insert(id.clone(), def.into());
                            bevy::log::info!("加载 Trait: {}", id);
                            loaded = true;
                        }
                        Err(e) => {
                            bevy::log::error!("解析 Trait 文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取 Trait 文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }

        if !loaded {
            bevy::log::warn!("Trait 目录为空，使用默认 Traits");
            registry.register_defaults();
        }

        registry
    }

    /// 注册内置默认 Traits
    fn register_defaults(&mut self) {
        let defaults = vec![
            TraitDefinition {
                id: "warrior_mastery".into(),
                name: "战士精通".into(),
                description: "近战职业，擅长正面作战".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::GrantTag(TagName::Warrior),
                    TraitEffectDef::GrantTag(TagName::Melee),
                ],
            },
            TraitDefinition {
                id: "archer_mastery".into(),
                name: "弓手精通".into(),
                description: "远程职业，擅长远距离攻击".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffectDef::GrantTag(TagName::Archer),
                    TraitEffectDef::GrantTag(TagName::Ranged),
                ],
            },
            TraitDefinition {
                id: "mage_mastery".into(),
                name: "法师精通".into(),
                description: "魔法职业，擅长元素攻击".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::GrantTag(TagName::Mage)],
            },
            TraitDefinition {
                id: "fire_affinity".into(),
                name: "火焰亲和".into(),
                description: "拥有火焰力量".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::GrantTag(TagName::Fire)],
            },
            TraitDefinition {
                id: "heavy_armor".into(),
                name: "重甲".into(),
                description: "装备重甲，防御+3".into(),
                trigger: TraitTrigger::Passive,
                effects: vec![TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                    kind: crate::gameplay::attribute::AttributeKind::Def,
                    op: crate::gameplay::attribute::ModifierOp::Add,
                    value: 3.0,
                })],
            },
        ];

        for def in defaults {
            let id = def.id.clone();
            self.traits.insert(id, def.into());
        }
    }
}

/// Trait 插件
pub struct TraitPlugin;

impl Plugin for TraitPlugin {
    fn build(&self, app: &mut App) {
        let registry = TraitRegistry::load_from_dir("assets/traits");
        app.insert_resource(registry);
        app.insert_resource(TraitEffectHandlerRegistry::with_defaults());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::attribute::{AttributeKind, ModifierOp};

    #[test]
    fn ron_反序列化_trait定义() {
        let ron_str = r#"
            (
                id: "warrior_mastery",
                name: "战士精通",
                description: "近战职业",
                trigger: Passive,
                effects: [
                    GrantTag(WARRIOR),
                    GrantTag(MELEE),
                ],
            )
        "#;
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "warrior_mastery");
        assert_eq!(def.trigger, TraitTrigger::Passive);
        assert_eq!(def.effects.len(), 2);
    }

    #[test]
    fn trait_def_转换为_trait_data() {
        let def = TraitDefinition {
            id: "test".into(),
            name: "测试".into(),
            description: "测试trait".into(),
            trigger: TraitTrigger::OnAttack,
            effects: vec![
                TraitEffectDef::GrantTag(TagName::Fire),
                TraitEffectDef::ModifyAttribute(AttributeModifierDef {
                    kind: AttributeKind::Atk,
                    op: ModifierOp::Add,
                    value: 5.0,
                }),
            ],
        };
        let data: TraitData = def.into();
        assert_eq!(data.id, "test");
        assert_eq!(data.trigger, TraitTrigger::OnAttack);
        assert_eq!(data.effects.len(), 2);

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let tags = data.granted_tags(&handlers);
        assert_eq!(tags, vec![GameplayTag::FIRE]);

        let mods = data.attribute_modifiers(&handlers);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].kind, AttributeKind::Atk);
        assert_eq!(mods[0].value, 5.0);
    }

    #[test]
    fn trait_collection_查询() {
        let collection = TraitCollection::new(vec!["warrior_mastery".into(), "heavy_armor".into()]);
        assert!(collection.has("warrior_mastery"));
        assert!(collection.has("heavy_armor"));
        assert!(!collection.has("mage_mastery"));
    }

    #[test]
    fn apply_passive_traits_授予标签和修饰符() {
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "warrior_mastery".into(),
            TraitData {
                id: "warrior_mastery".into(),
                name: "战士精通".into(),
                description: String::new(),
                trigger: TraitTrigger::Passive,
                effects: vec![
                    TraitEffect::GrantTag(GameplayTag::WARRIOR),
                    TraitEffect::GrantTag(GameplayTag::MELEE),
                    TraitEffect::ModifyAttribute(AttributeModifierDef {
                        kind: AttributeKind::Def,
                        op: ModifierOp::Add,
                        value: 2.0,
                    }),
                ],
            },
        );

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let (tags, modifiers) =
            apply_passive_traits(&["warrior_mastery".into()], &registry, &handlers);

        assert!(tags.has(GameplayTag::WARRIOR));
        assert!(tags.has(GameplayTag::MELEE));
        assert_eq!(modifiers.len(), 1);
        assert_eq!(modifiers[0].value, 2.0);
    }

    #[test]
    fn apply_passive_traits_跳过非被动触发() {
        let mut registry = TraitRegistry::default();
        registry.traits.insert(
            "on_attack_trait".into(),
            TraitData {
                id: "on_attack_trait".into(),
                name: "攻击触发".into(),
                description: String::new(),
                trigger: TraitTrigger::OnAttack,
                effects: vec![TraitEffect::GrantTag(GameplayTag::FIRE)],
            },
        );

        let handlers = TraitEffectHandlerRegistry::with_defaults();
        let (tags, modifiers) =
            apply_passive_traits(&["on_attack_trait".into()], &registry, &handlers);

        assert!(!tags.has(GameplayTag::FIRE));
        assert!(modifiers.is_empty());
    }

    #[test]
    fn ron_反序列化_触发型trait() {
        let ron_str = r#"
            (
                id: "leader_aura",
                name: "领袖光环",
                description: "回合开始时为友军施加增益",
                trigger: OnTurnStart,
                effects: [
                    ApplyBuff(buff_id: "attack_up", duration: 1),
                ],
            )
        "#;
        let def: TraitDefinition = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "leader_aura");
        assert_eq!(def.trigger, TraitTrigger::OnTurnStart);
        assert_eq!(def.effects.len(), 1);
    }
}
