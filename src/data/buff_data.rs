// Buff 数据：数据驱动的 Buff/Debuff 定义，替代 StatusEffect 枚举
// 支持从 assets/buffs/*.ron 外部配置文件加载

use crate::core::attribute::{AttributeKind, AttributeModifierDef, BuffInstanceId, ModifierOp};
use crate::core::attribute::Attributes;
use crate::core::tag::{GameplayTag, GameplayTags, TagName};
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

/// Buff 数据定义（运行时）
#[derive(Clone, Debug)]
pub struct BuffData {
    pub id: String,
    pub name: String,
    pub default_duration: u32,
    pub modifiers: Vec<AttributeModifierDef>,
    pub tags: Vec<GameplayTag>,
    pub dot_damage: i32,
    pub hot_heal: i32,
    pub is_stun: bool,
    pub is_cleanse: bool,
    pub is_buff: bool,
}

/// Buff 数据定义（RON 反序列化用，TagName 替代 GameplayTag）
#[derive(Clone, Debug, Deserialize)]
pub struct BuffDef {
    pub id: String,
    pub name: String,
    pub default_duration: u32,
    pub modifiers: Vec<AttributeModifierDef>,
    pub tags: Vec<TagName>,
    pub dot_damage: i32,
    pub hot_heal: i32,
    pub is_stun: bool,
    pub is_cleanse: bool,
    pub is_buff: bool,
}

impl From<BuffDef> for BuffData {
    fn from(def: BuffDef) -> Self {
        BuffData {
            id: def.id,
            name: def.name,
            default_duration: def.default_duration,
            modifiers: def.modifiers,
            tags: def.tags.iter().map(|t| t.to_tag()).collect(),
            dot_damage: def.dot_damage,
            hot_heal: def.hot_heal,
            is_stun: def.is_stun,
            is_cleanse: def.is_cleanse,
            is_buff: def.is_buff,
        }
    }
}

impl BuffData {
    pub fn is_debuff(&self) -> bool {
        !self.is_buff
    }
}

/// 运行时 Buff 实例
#[derive(Clone, Debug)]
pub struct BuffInstance {
    pub instance_id: BuffInstanceId,
    pub buff_id: String,
    pub name: String,
    pub remaining_turns: u32,
    pub source_entity: Option<Entity>,
    pub tags: Vec<GameplayTag>,
    pub is_buff: bool,
    pub dot_damage: i32,
    pub hot_heal: i32,
}

impl BuffInstance {
    pub fn is_debuff(&self) -> bool {
        !self.is_buff
    }

    pub fn label(&self) -> String {
        self.name.clone()
    }
}

/// 活跃 Buff 列表组件（替代原 StatusEffects）
#[derive(Component, Default, Debug, Clone)]
pub struct ActiveBuffs {
    pub instances: Vec<BuffInstance>,
    next_id: u64,
}

impl ActiveBuffs {
    /// 添加 Buff 实例，返回实例 ID
    pub fn add(&mut self, buff: BuffInstance) -> BuffInstanceId {
        let id = buff.instance_id;
        // 同源同 buff_id 刷新持续时间
        if let Some(source) = buff.source_entity {
            if let Some(existing) = self.instances.iter_mut().find(|b| {
                b.source_entity == Some(source) && b.buff_id == buff.buff_id
            }) {
                existing.remaining_turns = buff.remaining_turns;
                return existing.instance_id;
            }
        }
        self.instances.push(buff);
        id
    }

    /// 生成下一个唯一 ID
    pub fn next_instance_id(&mut self) -> BuffInstanceId {
        self.next_id += 1;
        BuffInstanceId(self.next_id)
    }

    /// 移除指定实例 ID 的 Buff
    pub fn remove(&mut self, instance_id: BuffInstanceId) -> Option<BuffInstance> {
        if let Some(idx) = self.instances.iter().position(|b| b.instance_id == instance_id) {
            Some(self.instances.remove(idx))
        } else {
            None
        }
    }

    /// 每回合结算：先移除已过期的，再递减剩余持续时间
    pub fn tick(&mut self) {
        self.instances.retain(|inst| inst.remaining_turns > 0);
        for inst in &mut self.instances {
            inst.remaining_turns -= 1;
        }
    }

    pub fn is_stunned(&self) -> bool {
        self.instances
            .iter()
            .any(|b| b.tags.contains(&GameplayTag::STUN))
    }

    /// 消耗晕眩：移除所有带 STUN 标签的 Buff，返回是否原本处于晕眩
    pub fn consume_stun(&mut self) -> bool {
        let was = self.is_stunned();
        self.instances.retain(|b| !b.tags.contains(&GameplayTag::STUN));
        was
    }

    pub fn dot_damage(&self) -> i32 {
        self.instances.iter().map(|b| b.dot_damage).sum()
    }

    pub fn hot_heal(&self) -> i32 {
        self.instances.iter().map(|b| b.hot_heal).sum()
    }

    /// 移除所有 Debuff
    pub fn remove_debuffs(&mut self) {
        self.instances.retain(|b| b.is_buff);
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BuffInstance> {
        self.instances.iter()
    }
}

/// Buff 注册表资源
#[derive(Resource, Default)]
pub struct BuffRegistry {
    pub buffs: HashMap<String, BuffData>,
}

impl BuffRegistry {
    pub fn get(&self, id: &str) -> Option<&BuffData> {
        self.buffs.get(id)
    }

    /// 从 assets/buffs/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = BuffRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("Buff 目录不存在，使用默认 Buff: {}", dir);
            registry.register_defaults();
            return registry;
        };

        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<BuffDef>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.buffs.insert(id.clone(), def.into());
                            bevy::log::info!("加载 Buff: {}", id);
                            loaded = true;
                        }
                        Err(e) => {
                            bevy::log::error!("解析 Buff 文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取 Buff 文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }

        // 目录存在但为空或全部解析失败，加载默认 Buff
        if !loaded {
            bevy::log::warn!("Buff 目录为空，使用默认 Buff");
            registry.register_defaults();
        }

        registry
    }

    /// 注册内置默认 Buff（确保基础功能可用）
    fn register_defaults(&mut self) {
        // 攻击力增加
        self.buffs.insert("attack_up".into(), BuffData {
            id: "attack_up".into(),
            name: "攻+5".into(),
            default_duration: 3,
            modifiers: vec![AttributeModifierDef { kind: AttributeKind::Atk, op: ModifierOp::Add, value: 5.0 }],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0, hot_heal: 0, is_stun: false, is_cleanse: false, is_buff: true,
        });

        // 攻击力减少
        self.buffs.insert("attack_down".into(), BuffData {
            id: "attack_down".into(),
            name: "攻-5".into(),
            default_duration: 3,
            modifiers: vec![AttributeModifierDef { kind: AttributeKind::Atk, op: ModifierOp::Add, value: -5.0 }],
            tags: vec![GameplayTag::DEBUFF],
            dot_damage: 0, hot_heal: 0, is_stun: false, is_cleanse: false, is_buff: false,
        });

        // 防御力增加
        self.buffs.insert("defense_up".into(), BuffData {
            id: "defense_up".into(),
            name: "防+5".into(),
            default_duration: 3,
            modifiers: vec![AttributeModifierDef { kind: AttributeKind::Def, op: ModifierOp::Add, value: 5.0 }],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0, hot_heal: 0, is_stun: false, is_cleanse: false, is_buff: true,
        });

        // 防御力减少
        self.buffs.insert("defense_down".into(), BuffData {
            id: "defense_down".into(),
            name: "防-5".into(),
            default_duration: 3,
            modifiers: vec![AttributeModifierDef { kind: AttributeKind::Def, op: ModifierOp::Add, value: -5.0 }],
            tags: vec![GameplayTag::DEBUFF],
            dot_damage: 0, hot_heal: 0, is_stun: false, is_cleanse: false, is_buff: false,
        });

        // 灼烧
        self.buffs.insert("burn".into(), BuffData {
            id: "burn".into(),
            name: "灼-2".into(),
            default_duration: 2,
            modifiers: vec![AttributeModifierDef { kind: AttributeKind::Def, op: ModifierOp::Add, value: -2.0 }],
            tags: vec![GameplayTag::DEBUFF, GameplayTag::BURN, GameplayTag::FIRE],
            dot_damage: 2, hot_heal: 0, is_stun: false, is_cleanse: false, is_buff: false,
        });

        // 中毒
        self.buffs.insert("poison".into(), BuffData {
            id: "poison".into(),
            name: "毒-3".into(),
            default_duration: 3,
            modifiers: vec![],
            tags: vec![GameplayTag::DEBUFF, GameplayTag::POISON],
            dot_damage: 3, hot_heal: 0, is_stun: false, is_cleanse: false, is_buff: false,
        });

        // 再生
        self.buffs.insert("regen".into(), BuffData {
            id: "regen".into(),
            name: "愈+4".into(),
            default_duration: 3,
            modifiers: vec![],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0, hot_heal: 4, is_stun: false, is_cleanse: false, is_buff: true,
        });

        // 眩晕
        self.buffs.insert("stun".into(), BuffData {
            id: "stun".into(),
            name: "晕眩".into(),
            default_duration: 1,
            modifiers: vec![],
            tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
            dot_damage: 0, hot_heal: 0, is_stun: true, is_cleanse: false, is_buff: false,
        });
    }
}

/// 给目标施加 Buff（修改 ActiveBuffs + Attributes + GameplayTags）
pub fn apply_buff(
    active_buffs: &mut ActiveBuffs,
    attributes: &mut Attributes,
    gameplay_tags: &mut GameplayTags,
    buff_data: &BuffData,
    source_entity: Option<Entity>,
    duration: u32,
) -> BuffInstanceId {
    // Cleanse 特殊处理：立即驱散所有 debuff
    if buff_data.is_cleanse {
        remove_all_debuffs(active_buffs, attributes, gameplay_tags);
        return BuffInstanceId(0);
    }

    let instance_id = active_buffs.next_instance_id();

    let instance = BuffInstance {
        instance_id,
        buff_id: buff_data.id.clone(),
        name: buff_data.name.clone(),
        remaining_turns: duration,
        source_entity,
        tags: buff_data.tags.clone(),
        is_buff: buff_data.is_buff,
        dot_damage: buff_data.dot_damage,
        hot_heal: buff_data.hot_heal,
    };

    // 添加修饰符到 Attributes
    attributes.add_modifiers_from_def(&buff_data.modifiers, instance_id);

    // 添加标签到 GameplayTags
    for tag in &buff_data.tags {
        gameplay_tags.add(*tag);
    }

    active_buffs.add(instance);
    instance_id
}

/// 移除指定 Buff 实例（清理 Attributes + GameplayTags）
pub fn remove_buff(
    active_buffs: &mut ActiveBuffs,
    attributes: &mut Attributes,
    gameplay_tags: &mut GameplayTags,
    instance_id: BuffInstanceId,
) {
    if let Some(removed) = active_buffs.remove(instance_id) {
        // 移除修饰符
        attributes.remove_modifiers_from(instance_id);

        // 移除标签（仅当没有其他 Buff 提供相同标签时）
        let remaining_tags: Vec<GameplayTag> = active_buffs
            .instances
            .iter()
            .flat_map(|b| b.tags.iter())
            .copied()
            .collect();
        for tag in &removed.tags {
            if !remaining_tags.contains(tag) {
                gameplay_tags.remove(*tag);
            }
        }
    }
}

/// 移除所有 Debuff
pub fn remove_all_debuffs(
    active_buffs: &mut ActiveBuffs,
    attributes: &mut Attributes,
    gameplay_tags: &mut GameplayTags,
) {
    let debuff_ids: Vec<BuffInstanceId> = active_buffs
        .instances
        .iter()
        .filter(|b| b.is_debuff())
        .map(|b| b.instance_id)
        .collect();
    for id in debuff_ids {
        remove_buff(active_buffs, attributes, gameplay_tags, id);
    }
}

/// Buff 数据插件
pub struct BuffDataPlugin;

impl Plugin for BuffDataPlugin {
    fn build(&self, app: &mut App) {
        let registry = BuffRegistry::load_from_dir("assets/buffs");
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::{AttributeKind, ModifierOp};

    /// 辅助：创建一个简单的 BuffData
    fn make_buff(id: &str, is_buff: bool, modifiers: Vec<AttributeModifierDef>, tags: Vec<GameplayTag>) -> BuffData {
        BuffData {
            id: id.into(),
            name: id.into(),
            default_duration: 2,
            modifiers,
            tags,
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff,
        }
    }

    /// 辅助：创建一个带 DoT/HoT 的 BuffData
    fn make_dot_buff(id: &str, dot: i32, hot: i32) -> BuffData {
        BuffData {
            id: id.into(),
            name: id.into(),
            default_duration: 2,
            modifiers: vec![],
            tags: vec![GameplayTag::DEBUFF, GameplayTag::POISON],
            dot_damage: dot,
            hot_heal: hot,
            is_stun: false,
            is_cleanse: false,
            is_buff: false,
        }
    }

    // ── ActiveBuffs 基础操作 ──

    #[test]
    fn 活跃buff_添加和查询() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        assert_eq!(buffs.len(), 1);
        assert!(!buffs.is_empty());
    }

    #[test]
    fn 活跃buff_移除() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        let removed = buffs.remove(id);
        assert!(removed.is_some());
        assert_eq!(buffs.len(), 0);
        assert!(buffs.is_empty());
    }

    #[test]
    fn 活跃buff_移除不存在的返回none() {
        let mut buffs = ActiveBuffs::default();
        let result = buffs.remove(BuffInstanceId(999));
        assert!(result.is_none());
    }

    #[test]
    fn 活跃buff_同源同id刷新持续时间() {
        let mut buffs = ActiveBuffs::default();
        let source = Entity::from_bits(42);

        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 1,
            source_entity: Some(source),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 3,
            source_entity: Some(source),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].remaining_turns, 3);
        assert_eq!(buffs.instances[0].instance_id, id1);
    }

    #[test]
    fn 活跃buff_不同源同id不刷新() {
        let mut buffs = ActiveBuffs::default();
        let source_a = Entity::from_bits(1);
        let source_b = Entity::from_bits(2);

        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 1,
            source_entity: Some(source_a),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 3,
            source_entity: Some(source_b),
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });

        assert_eq!(buffs.len(), 2);
    }

    // ── Tick ──

    #[test]
    fn 活跃buff_tick_递减持续时间() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        buffs.tick();
        assert_eq!(buffs.instances[0].remaining_turns, 1);
    }

    #[test]
    fn 活跃buff_tick_递减后过期() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 1,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        buffs.tick();
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].remaining_turns, 0);
        buffs.tick();
        assert!(buffs.is_empty());
    }

    // ── 晕眩 ──

    #[test]
    fn 活跃buff_晕眩检测() {
        let mut buffs = ActiveBuffs::default();
        assert!(!buffs.is_stunned());

        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "stun".into(),
            name: "晕".into(),
            remaining_turns: 1,
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
            is_buff: false,
            dot_damage: 0,
            hot_heal: 0,
        });
        assert!(buffs.is_stunned());
    }

    #[test]
    fn 活跃buff_消耗晕眩() {
        let mut buffs = ActiveBuffs::default();
        let id = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id,
            buff_id: "stun".into(),
            name: "晕".into(),
            remaining_turns: 1,
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
            is_buff: false,
            dot_damage: 0,
            hot_heal: 0,
        });
        let was_stunned = buffs.consume_stun();
        assert!(was_stunned);
        assert!(!buffs.is_stunned());
        assert!(buffs.is_empty());
    }

    // ── DoT / HoT ──

    #[test]
    fn 活跃buff_dot_hot汇总() {
        let mut buffs = ActiveBuffs::default();
        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 2,
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });
        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "regen".into(),
            name: "愈".into(),
            remaining_turns: 2,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 4,
        });
        assert_eq!(buffs.dot_damage(), 3);
        assert_eq!(buffs.hot_heal(), 4);
    }

    // ── 移除所有 Debuff ──

    #[test]
    fn 活跃buff_移除所有debuff() {
        let mut buffs = ActiveBuffs::default();
        let id1 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id1,
            buff_id: "attack_up".into(),
            name: "攻+5".into(),
            remaining_turns: 2,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
        let id2 = buffs.next_instance_id();
        buffs.add(BuffInstance {
            instance_id: id2,
            buff_id: "poison".into(),
            name: "毒".into(),
            remaining_turns: 2,
            source_entity: None,
            tags: vec![GameplayTag::DEBUFF],
            is_buff: false,
            dot_damage: 3,
            hot_heal: 0,
        });
        buffs.remove_debuffs();
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].buff_id, "attack_up");
    }

    // ── apply_buff / remove_buff 集成测试 ──

    #[test]
    fn apply_buff_添加修饰符和标签() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        let mut tags = GameplayTags::default();
        attrs.set_base(AttributeKind::Atk, 10.0);

        let buff_data = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Atk,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );

        apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_data, None, 2);

        assert_eq!(buffs.len(), 1);
        assert_eq!(attrs.get(AttributeKind::Atk), 15.0);
        assert!(tags.has(GameplayTag::BUFF));
    }

    #[test]
    fn remove_buff_清理修饰符和标签() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        let mut tags = GameplayTags::default();
        attrs.set_base(AttributeKind::Atk, 10.0);

        let buff_data = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Atk,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );

        let instance_id = apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_data, None, 2);
        assert_eq!(attrs.get(AttributeKind::Atk), 15.0);

        remove_buff(&mut buffs, &mut attrs, &mut tags, instance_id);
        assert_eq!(attrs.get(AttributeKind::Atk), 10.0);
        assert!(!tags.has(GameplayTag::BUFF));
        assert!(buffs.is_empty());
    }

    #[test]
    fn remove_buff_共享标签不被误删() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        let mut tags = GameplayTags::default();

        let buff_a = make_buff(
            "buff_a",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Atk,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF, GameplayTag::FIRE],
        );
        let buff_b = make_buff(
            "buff_b",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Def,
                op: ModifierOp::Add,
                value: 3.0,
            }],
            vec![GameplayTag::BUFF, GameplayTag::FIRE],
        );

        let id_a = apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_a, None, 2);
        let _id_b = apply_buff(&mut buffs, &mut attrs, &mut tags, &buff_b, None, 2);

        remove_buff(&mut buffs, &mut attrs, &mut tags, id_a);
        assert!(tags.has(GameplayTag::BUFF));
        assert!(tags.has(GameplayTag::FIRE));
    }

    #[test]
    fn apply_buff_cleanse_驱散所有debuff() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        let mut tags = GameplayTags::default();
        attrs.set_base(AttributeKind::Atk, 10.0);

        let debuff = make_buff(
            "attack_down",
            false,
            vec![AttributeModifierDef {
                kind: AttributeKind::Atk,
                op: ModifierOp::Add,
                value: -5.0,
            }],
            vec![GameplayTag::DEBUFF],
        );
        apply_buff(&mut buffs, &mut attrs, &mut tags, &debuff, None, 2);
        assert_eq!(attrs.get(AttributeKind::Atk), 5.0);

        let cleanse = BuffData {
            id: "cleanse".into(),
            name: "驱散".into(),
            default_duration: 0,
            modifiers: vec![],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: true,
            is_buff: true,
        };
        apply_buff(&mut buffs, &mut attrs, &mut tags, &cleanse, None, 0);

        assert!(buffs.is_empty());
        assert_eq!(attrs.get(AttributeKind::Atk), 10.0);
    }

    #[test]
    fn remove_all_debuffs_只移除debuff保留buff() {
        let mut buffs = ActiveBuffs::default();
        let mut attrs = Attributes::default();
        let mut tags = GameplayTags::default();
        attrs.set_base(AttributeKind::Atk, 10.0);
        attrs.set_base(AttributeKind::Def, 5.0);

        let buff = make_buff(
            "attack_up",
            true,
            vec![AttributeModifierDef {
                kind: AttributeKind::Atk,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            vec![GameplayTag::BUFF],
        );
        let debuff = make_buff(
            "defense_down",
            false,
            vec![AttributeModifierDef {
                kind: AttributeKind::Def,
                op: ModifierOp::Add,
                value: -3.0,
            }],
            vec![GameplayTag::DEBUFF],
        );

        apply_buff(&mut buffs, &mut attrs, &mut tags, &buff, None, 2);
        apply_buff(&mut buffs, &mut attrs, &mut tags, &debuff, None, 2);
        assert_eq!(attrs.get(AttributeKind::Atk), 15.0);
        assert_eq!(attrs.get(AttributeKind::Def), 2.0);

        remove_all_debuffs(&mut buffs, &mut attrs, &mut tags);
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].buff_id, "attack_up");
        assert_eq!(attrs.get(AttributeKind::Atk), 15.0);
        assert_eq!(attrs.get(AttributeKind::Def), 5.0);
    }

    // ── BuffDef → BuffData 转换 ──

    #[test]
    fn buff_def_转换为_buff_data() {
        let def = BuffDef {
            id: "test_buff".into(),
            name: "测试增益".into(),
            default_duration: 3,
            modifiers: vec![AttributeModifierDef {
                kind: AttributeKind::Atk,
                op: ModifierOp::Add,
                value: 10.0,
            }],
            tags: vec![TagName::Buff, TagName::Fire],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: true,
        };
        let data: BuffData = def.into();
        assert_eq!(data.id, "test_buff");
        assert_eq!(data.tags, vec![GameplayTag::BUFF, GameplayTag::FIRE]);
    }

    // ── RON 反序列化 ──

    #[test]
    fn ron_反序列化_buff定义() {
        let ron_str = r#"
            (
                id: "test_buff",
                name: "测试增益",
                default_duration: 2,
                modifiers: [
                    (kind: Atk, op: Add, value: 5.0),
                ],
                tags: [BUFF, FIRE],
                dot_damage: 0,
                hot_heal: 3,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            )
        "#;
        let def: BuffDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "test_buff");
        assert_eq!(def.tags, vec![TagName::Buff, TagName::Fire]);
        assert_eq!(def.hot_heal, 3);
        assert_eq!(def.modifiers.len(), 1);
    }
}
