// ViewModel 层：游戏逻辑 → ViewModel → UI
// UI 系统只读 ViewModel，不直接 Query 游戏组件

use bevy::prelude::*;

use crate::battle::CombatIntent;
use crate::buff::ActiveBuffs;
use crate::character::{
    Faction, GridPosition, Selected, TraitCollection, TraitRegistry, Unit, UnitClass, UnitName,
    UnitRace,
};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::skill::{SkillCooldowns, SkillRegistry, SkillSlots};
use crate::turn::{TurnOrder, TurnPhase, TurnState};

// ── ViewModel 定义 ──

/// 核心属性条目
#[derive(Clone, Debug)]
pub struct CoreAttrEntry {
    pub label: String,
    pub value: i32,
}

/// 衍生属性条目（分组显示）
#[derive(Clone, Debug)]
pub struct DerivedAttrEntry {
    pub label: String,
    pub value: i32,
}

/// Buff 条目（带颜色分类）
#[derive(Clone, Debug)]
pub struct BuffEntry {
    pub name: String,
    pub remaining_turns: u32,
    pub is_buff: bool,
}

/// 技能条目（带详细信息）
#[derive(Clone, Debug)]
pub struct SkillEntry {
    pub name: String,
    pub id: String,
    pub cost_mp: i32,
    pub range: u32,
    pub cooldown: u32,
    pub description: String,
}

/// Trait 条目
#[derive(Clone, Debug)]
pub struct TraitEntry {
    pub name: String,
    pub description: String,
}

/// 装备槽条目（含空槽位）
#[derive(Clone, Debug)]
pub struct EquipmentSlotEntry {
    pub slot_label: String,
    pub item_name: Option<String>,
    pub rarity: Option<String>,
}

/// 背包条目
#[derive(Clone, Debug)]
pub struct InventoryEntry {
    pub item_name: String,
    pub rarity: String,
    pub instance_id: u64,
}

/// 最后点击查看的单位实体（不限于 Selected，任何单位都可查看信息）
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct HoveredEntity {
    pub entity: Option<Entity>,
}

/// 选中单位的视图模型
#[derive(Resource, Default, Debug)]
pub struct SelectedUnitView {
    pub name: String,
    pub race: String,
    pub class: String,
    // 生命资源
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub stamina: i32,
    pub max_stamina: i32,
    // 核心属性（8维）
    pub core_attrs: Vec<CoreAttrEntry>,
    // 衍生属性（战斗组）
    pub combat_attrs: Vec<DerivedAttrEntry>,
    // 衍生属性（辅助组）
    pub support_attrs: Vec<DerivedAttrEntry>,
    // 技能
    pub skills: Vec<SkillEntry>,
    // Traits
    pub traits: Vec<TraitEntry>,
    // Buff
    pub buffs: Vec<BuffEntry>,
    // 装备
    pub equipment: Vec<EquipmentSlotEntry>,
    // 背包
    pub inventory: Vec<InventoryEntry>,
    pub is_selected: bool,
}

/// 战斗预览视图模型
#[derive(Resource, Default, Debug)]
pub struct CombatPreviewView {
    pub is_visible: bool,
    pub estimated_damage: i32,
    pub hit_rate: i32,
    pub crit_rate: i32,
    pub is_lethal: bool,
}

/// 回合信息视图模型（AGI驱动，不再分阵营阶段）
#[derive(Resource, Default, Debug)]
pub struct TurnInfoView {
    pub turn_number: u32,
    pub is_player_turn: bool,
    /// 行动顺序列表（当前单位名称 + 阵营标识）
    pub turn_order: Vec<(String, bool)>, // (name, is_player)
    /// 当前行动索引
    pub current_index: usize,
}

/// 胜负状态
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub enum GameOverState {
    #[default]
    Playing,
    Victory,
    Defeat,
}

// ── ViewModel 更新系统 ──

/// 从游戏数据构建 SelectedUnitView（基于 HoveredEntity，任何单位都可查看）
/// 仅在 HoveredEntity 变化时刷新，避免每帧重建
pub fn update_selected_unit_view(
    hovered: Res<HoveredEntity>,
    units: Query<(
        &Unit,
        &UnitName,
        &Attributes,
        &SkillSlots,
        &ActiveBuffs,
        Option<&UnitRace>,
        Option<&UnitClass>,
        Option<&SkillCooldowns>,
        Option<&TraitCollection>,
        Option<&crate::equipment::EquipmentSlots>,
        Option<&crate::equipment::Inventory>,
    )>,
    skill_registry: Res<SkillRegistry>,
    trait_registry: Res<TraitRegistry>,
    equipment_registry: Res<crate::equipment::EquipmentRegistry>,
    mut view: ResMut<SelectedUnitView>,
) {
    // 仅在 HoveredEntity 变化时刷新
    if !hovered.is_changed() && !view.is_added() {
        return;
    }

    if let Some(entity) = hovered.entity {
        if let Ok((
            _unit,
            name,
            attrs,
            skill_slots,
            buffs,
            race,
            class,
            _cooldowns,
            trait_collection,
            equipment_slots,
            inventory,
        )) = units.get(entity)
        {
            view.name = name.0.clone();
            view.race = race.map(|r| r.0.clone()).unwrap_or_default();
            view.class = class.map(|c| c.0.clone()).unwrap_or_default();

            // 生命资源
            view.hp = attrs.get(AttributeKind::Hp) as i32;
            view.max_hp = attrs.get(AttributeKind::MaxHp) as i32;
            view.mp = attrs.get(AttributeKind::Mp) as i32;
            view.max_mp = attrs.get(AttributeKind::MaxMp) as i32;
            view.stamina = attrs.get(AttributeKind::Stamina) as i32;
            view.max_stamina = attrs.get(AttributeKind::MaxStamina) as i32;

            // 核心属性（8维）
            view.core_attrs = [
                AttributeKind::Might,
                AttributeKind::Dexterity,
                AttributeKind::Agility,
                AttributeKind::Vitality,
                AttributeKind::Intelligence,
                AttributeKind::Willpower,
                AttributeKind::Presence,
                AttributeKind::Luck,
            ]
            .iter()
            .map(|kind| CoreAttrEntry {
                label: kind.short_label().to_string(),
                value: attrs.get(*kind) as i32,
            })
            .collect();

            // 衍生属性（战斗组）
            view.combat_attrs = [
                AttributeKind::Attack,
                AttributeKind::Defense,
                AttributeKind::MagicAttack,
                AttributeKind::MagicDefense,
            ]
            .iter()
            .map(|kind| DerivedAttrEntry {
                label: kind.label().to_string(),
                value: attrs.get(*kind) as i32,
            })
            .collect();

            // 衍生属性（辅助组）
            view.support_attrs = [
                AttributeKind::Accuracy,
                AttributeKind::Evasion,
                AttributeKind::CritRate,
                AttributeKind::MoveRange,
                AttributeKind::Initiative,
                AttributeKind::AttackRange,
            ]
            .iter()
            .map(|kind| DerivedAttrEntry {
                label: kind.label().to_string(),
                value: attrs.get(*kind) as i32,
            })
            .collect();

            // 技能（带详细信息）
            view.skills = skill_slots
                .skill_ids
                .iter()
                .filter_map(|id| {
                    skill_registry.get(id).map(|sd| SkillEntry {
                        name: sd.name.clone(),
                        id: id.to_string(),
                        cost_mp: sd.cost_mp,
                        range: sd.range,
                        cooldown: sd.cooldown,
                        description: sd.description.clone(),
                    })
                })
                .collect();

            // Traits
            view.traits = trait_collection
                .as_ref()
                .map(|tc| {
                    tc.trait_ids()
                        .iter()
                        .filter_map(|tid| {
                            trait_registry.get(tid).map(|td| TraitEntry {
                                name: td.name.clone(),
                                description: td.description.clone(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Buff
            view.buffs = buffs
                .iter()
                .map(|inst| BuffEntry {
                    name: inst.name.clone(),
                    remaining_turns: inst.remaining_turns,
                    is_buff: inst.is_buff,
                })
                .collect();

            // 装备（遍历所有槽位，空位也显示）
            view.equipment = equipment_slots
                .map(|slots| {
                    use crate::equipment::EquipmentSlot;
                    let all_slots = [
                        EquipmentSlot::MainHand,
                        EquipmentSlot::OffHand,
                        EquipmentSlot::Head,
                        EquipmentSlot::Body,
                        EquipmentSlot::Feet,
                        EquipmentSlot::Accessory1,
                        EquipmentSlot::Accessory2,
                    ];
                    all_slots
                        .iter()
                        .map(|slot| {
                            let slot_label = slot.label().to_string();
                            if let Some(def_id) = slots.get_def_id(*slot) {
                                if let Some(def) = equipment_registry.get(def_id) {
                                    EquipmentSlotEntry {
                                        slot_label,
                                        item_name: Some(def.name.clone()),
                                        rarity: Some(def.rarity.label().to_string()),
                                    }
                                } else {
                                    EquipmentSlotEntry {
                                        slot_label,
                                        item_name: None,
                                        rarity: None,
                                    }
                                }
                            } else {
                                EquipmentSlotEntry {
                                    slot_label,
                                    item_name: None,
                                    rarity: None,
                                }
                            }
                        })
                        .collect()
                })
                .unwrap_or_else(|| {
                    use crate::equipment::EquipmentSlot;
                    [
                        EquipmentSlot::MainHand,
                        EquipmentSlot::OffHand,
                        EquipmentSlot::Head,
                        EquipmentSlot::Body,
                        EquipmentSlot::Feet,
                        EquipmentSlot::Accessory1,
                        EquipmentSlot::Accessory2,
                    ]
                    .iter()
                    .map(|slot| EquipmentSlotEntry {
                        slot_label: slot.label().to_string(),
                        item_name: None,
                        rarity: None,
                    })
                    .collect()
                });

            // 背包
            view.inventory = inventory
                .map(|inv| {
                    inv.items
                        .iter()
                        .filter_map(|instance| {
                            equipment_registry
                                .get(&instance.def_id)
                                .map(|def| InventoryEntry {
                                    item_name: def.name.clone(),
                                    rarity: def.rarity.label().to_string(),
                                    instance_id: instance.instance_id,
                                })
                        })
                        .collect()
                })
                .unwrap_or_default();

            view.is_selected = true;
        } else {
            // 实体已销毁
            view.is_selected = false;
            view.name.clear();
            view.core_attrs.clear();
            view.combat_attrs.clear();
            view.support_attrs.clear();
            view.skills.clear();
            view.traits.clear();
            view.buffs.clear();
            view.equipment.clear();
            view.inventory.clear();
        }
    } else {
        view.is_selected = false;
        view.name.clear();
        view.race.clear();
        view.class.clear();
        view.core_attrs.clear();
        view.combat_attrs.clear();
        view.support_attrs.clear();
        view.skills.clear();
        view.traits.clear();
        view.buffs.clear();
        view.equipment.clear();
        view.inventory.clear();
    }
}

/// 更新战斗预览（SelectTarget 阶段，鼠标悬停敌方时触发）
pub fn update_combat_preview_view(
    turn_phase: Res<State<TurnPhase>>,
    selected_units: Query<&Attributes, With<Selected>>,
    _enemy_units: Query<(&Attributes, &GridPosition), (With<Unit>, Without<Selected>)>,
    _combat_intent: Res<CombatIntent>,
    mut preview_view: ResMut<CombatPreviewView>,
) {
    // 只在 SelectTarget 阶段显示
    if *turn_phase.get() != TurnPhase::SelectTarget {
        preview_view.is_visible = false;
        return;
    }

    // 获取攻击者属性
    let Ok(_source_attrs) = selected_units.single() else {
        preview_view.is_visible = false;
        return;
    };

    // 获取目标属性（通过 combat_intent.target_coord 或鼠标位置）
    // 简化：暂时不实现鼠标悬停检测，只隐藏预览
    preview_view.is_visible = false;
}

/// 从游戏数据构建 TurnInfoView（AGI驱动，包含行动顺序）
pub fn update_turn_info_view(
    turn_state: Res<TurnState>,
    turn_order: Res<TurnOrder>,
    units: Query<(&Unit, &UnitName)>,
    mut view: ResMut<TurnInfoView>,
) {
    if turn_state.is_changed() || turn_order.is_changed() {
        view.turn_number = turn_state.turn_number;
        view.is_player_turn = turn_state.current_faction == Faction::Player;
        view.current_index = turn_order.current_index;

        // 构建行动顺序列表
        view.turn_order = turn_order
            .queue
            .iter()
            .filter_map(|&entity| {
                units
                    .get(entity)
                    .ok()
                    .map(|(unit, name)| (name.0.clone(), unit.faction == Faction::Player))
            })
            .collect();
    }
}

/// 检查胜负条件，更新 GameOverState
pub fn update_game_over_state(units: Query<&Unit>, mut game_over: ResMut<GameOverState>) {
    if *game_over != GameOverState::Playing {
        return;
    }

    let has_player = units.iter().any(|u| u.faction == Faction::Player);
    let has_enemy = units.iter().any(|u| u.faction == Faction::Enemy);

    if !has_enemy {
        *game_over = GameOverState::Victory;
    } else if !has_player {
        *game_over = GameOverState::Defeat;
    }
}
