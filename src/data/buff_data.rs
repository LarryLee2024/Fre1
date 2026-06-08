// Buff 数据：数据驱动的 Buff/Debuff 定义，替代 StatusEffect 枚举

use crate::core::attribute::{AttributeKind, AttributeModifierDef, BuffInstanceId, ModifierOp};
use crate::core::attribute::Attributes;
use crate::core::tag::{GameplayTag, GameplayTags};
use bevy::prelude::*;
use std::collections::HashMap;

/// Buff 数据定义
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
        self.instances.iter().map(|b| {
            // 需要从 registry 查 dot_damage，但这里只存了 buff_id
            // 改为在 BuffInstance 上直接存 dot/hot 值
            b.dot_damage
        }).sum()
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

    /// 初始化所有 Buff 定义
    pub fn populate(&mut self) {
        let buffs = vec![
            BuffData {
                id: "attack_up".into(),
                name: "攻+5".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Atk,
                    op: ModifierOp::Add,
                    value: 5.0,
                }],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
            BuffData {
                id: "attack_down".into(),
                name: "攻-5".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Atk,
                    op: ModifierOp::Add,
                    value: -5.0,
                }],
                tags: vec![GameplayTag::DEBUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
            BuffData {
                id: "defense_up".into(),
                name: "防+5".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Def,
                    op: ModifierOp::Add,
                    value: 5.0,
                }],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
            BuffData {
                id: "defense_down".into(),
                name: "防-5".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Def,
                    op: ModifierOp::Add,
                    value: -5.0,
                }],
                tags: vec![GameplayTag::DEBUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
            BuffData {
                id: "movement_up".into(),
                name: "移+2".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Mov,
                    op: ModifierOp::Add,
                    value: 2.0,
                }],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
            BuffData {
                id: "movement_down".into(),
                name: "移-2".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Mov,
                    op: ModifierOp::Add,
                    value: -2.0,
                }],
                tags: vec![GameplayTag::DEBUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
            BuffData {
                id: "range_up".into(),
                name: "距+1".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::AttackRange,
                    op: ModifierOp::Add,
                    value: 1.0,
                }],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
            BuffData {
                id: "range_down".into(),
                name: "距-1".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::AttackRange,
                    op: ModifierOp::Add,
                    value: -1.0,
                }],
                tags: vec![GameplayTag::DEBUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
            BuffData {
                id: "poison".into(),
                name: "毒-3".into(),
                default_duration: 2,
                modifiers: vec![],
                tags: vec![GameplayTag::DEBUFF, GameplayTag::POISON],
                dot_damage: 3,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
            BuffData {
                id: "burn".into(),
                name: "灼-2".into(),
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Def,
                    op: ModifierOp::Add,
                    value: -2.0,
                }],
                tags: vec![GameplayTag::DEBUFF, GameplayTag::BURN, GameplayTag::FIRE],
                dot_damage: 2,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
            BuffData {
                id: "regen".into(),
                name: "愈+4".into(),
                default_duration: 2,
                modifiers: vec![],
                tags: vec![GameplayTag::BUFF, GameplayTag::REGEN],
                dot_damage: 0,
                hot_heal: 4,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
            BuffData {
                id: "stun".into(),
                name: "晕".into(),
                default_duration: 1,
                modifiers: vec![],
                tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: true,
                is_cleanse: false,
                is_buff: false,
            },
            BuffData {
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
            },
        ];

        for buff in buffs {
            self.buffs.insert(buff.id.clone(), buff);
        }
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
        let mut registry = BuffRegistry::default();
        registry.populate();
        app.insert_resource(registry);
    }
}
