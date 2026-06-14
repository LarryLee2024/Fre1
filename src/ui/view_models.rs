// ViewModel 层：游戏逻辑 → ViewModel → UI
// UI 系统只读 ViewModel，不直接 Query 游戏组件

use bevy::prelude::*;

use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::battle::CombatIntent;
use crate::core::buff::ActiveBuffs;
use crate::core::character::{
    Faction, GridPosition, Selected, TraitCollection, TraitRegistry, Unit, UnitClass, UnitName,
    UnitRace,
};
use crate::core::skill::{SkillCooldowns, SkillRegistry, SkillSlots};
use crate::core::turn::{TurnOrder, TurnPhase, TurnState};

// ── ViewModel 定义 ──

/// 核心属性条目
#[derive(Clone, Debug, Reflect)]
pub struct CoreAttrEntry {
    pub label: String,
    pub value: i32,
}

/// 衍生属性条目（分组显示）
#[derive(Clone, Debug, Reflect)]
pub struct DerivedAttrEntry {
    pub label: String,
    pub value: i32,
}

/// Buff 条目（带颜色分类）
#[derive(Clone, Debug, Reflect)]
pub struct BuffEntry {
    pub name: String,
    pub remaining_turns: u32,
    pub is_buff: bool,
}

/// 技能条目（带详细信息）
#[derive(Clone, Debug, Reflect)]
pub struct SkillEntry {
    pub name: String,
    pub id: String,
    pub cost_mp: i32,
    pub range: u32,
    pub cooldown: u32,
    pub description: String,
}

/// Trait 条目
#[derive(Clone, Debug, Reflect)]
pub struct TraitEntry {
    pub name: String,
    pub description: String,
}

/// 装备槽条目（含空槽位）
#[derive(Clone, Debug, Reflect)]
pub struct EquipmentSlotEntry {
    pub slot_label: String,
    pub item_name: Option<String>,
    pub rarity: Option<String>,
}

/// 背包条目
#[derive(Clone, Debug, Reflect)]
pub struct InventoryEntry {
    pub item_name: String,
    pub rarity: String,
    pub count: u32,
    pub instance_id: u64,
}

use crate::shared::resettable::ResettableResource;

/// 最后点击查看的单位实体（不限于 Selected，任何单位都可查看信息）
#[derive(Resource, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Resource)]
pub struct HoveredEntity {
    pub entity: Option<Entity>,
}

impl ResettableResource for HoveredEntity {}

/// 选中单位的视图模型
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct SelectedUnitView {
    pub name: String,
    pub race: String,
    pub class: String,
    /// 单位格子坐标（用于 UI 定位，如行动菜单）
    pub grid_coord: IVec2,
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

impl SelectedUnitView {
    /// 清空所有视图数据
    fn clear(&mut self) {
        self.is_selected = false;
        self.name.clear();
        self.race.clear();
        self.class.clear();
        self.grid_coord = IVec2::ZERO;
        self.hp = 0;
        self.max_hp = 0;
        self.mp = 0;
        self.max_mp = 0;
        self.stamina = 0;
        self.max_stamina = 0;
        self.core_attrs.clear();
        self.combat_attrs.clear();
        self.support_attrs.clear();
        self.skills.clear();
        self.traits.clear();
        self.buffs.clear();
        self.equipment.clear();
        self.inventory.clear();
    }
}

impl ResettableResource for SelectedUnitView {}

/// 战斗预览视图模型
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct CombatPreviewView {
    pub is_visible: bool,
    pub estimated_damage: i32,
    pub hit_rate: i32,
    pub crit_rate: i32,
    pub is_lethal: bool,
}

impl ResettableResource for CombatPreviewView {}

/// 回合信息视图模型（AGI驱动，不再分阵营阶段）
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct TurnInfoView {
    pub turn_number: u32,
    pub is_player_turn: bool,
    /// 行动顺序列表（当前单位名称 + 阵营标识）
    pub turn_order: Vec<(String, bool)>, // (name, is_player)
    /// 当前行动索引
    pub current_index: usize,
}

// ── 框架 UI ViewModel ──

/// 关卡条目（LevelSelect 屏幕用）
#[derive(Clone, Debug, Reflect)]
pub struct StageEntry {
    pub stage_id: String,
    pub level_name: String,
    pub status: crate::core::campaign::progress::StageStatus,
    pub level_description: String,
}

/// 关卡选择屏幕状态
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct LevelSelectState {
    pub campaign_name: String,
    pub stages: Vec<StageEntry>,
    pub selected_stage: Option<String>,
}

/// 游戏结果类型
#[derive(Clone, Debug, Reflect, PartialEq, Eq)]
pub enum GameOutcome {
    Victory,
    Defeat,
}

impl Default for GameOutcome {
    fn default() -> Self {
        GameOutcome::Victory
    }
}

/// 游戏结果屏幕 ViewModel
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct GameResultView {
    pub result: GameOutcome,
    pub turn_count: u32,
    pub stage_name: String,
    pub has_next_stage: bool,
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
        Option<&crate::core::equipment::EquipmentSlots>,
        Option<&crate::core::inventory::container::Container>,
        &GridPosition,
    )>,
    skill_registry: Res<SkillRegistry>,
    trait_registry: Res<TraitRegistry>,
    equipment_registry: Res<crate::core::equipment::EquipmentRegistry>,
    item_registry: Res<crate::core::inventory::definition::ItemRegistry>,
    mut view: ResMut<SelectedUnitView>,
) {
    if !hovered.is_changed() && !view.is_added() {
        return;
    }

    let Some(entity) = hovered.entity else {
        view.clear();
        return;
    };

    let Ok((
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
        container,
        grid_pos,
    )) = units.get(entity)
    else {
        view.clear();
        return;
    };

    view.name = name.0.clone();
    view.race = race.map(|r| r.0.clone()).unwrap_or_default();
    view.class = class.map(|c| c.0.clone()).unwrap_or_default();
    view.grid_coord = grid_pos.coord;
    view.is_selected = true;

    fill_vital_attrs(&mut view, attrs);
    fill_core_attrs(&mut view, attrs);
    fill_combat_attrs(&mut view, attrs);
    fill_support_attrs(&mut view, attrs);
    fill_skills(&mut view, skill_slots, &skill_registry);
    fill_traits(&mut view, trait_collection, &trait_registry);
    fill_buffs(&mut view, buffs);
    fill_equipment(&mut view, equipment_slots, &equipment_registry);
    fill_inventory(&mut view, container, &item_registry);
}

fn fill_vital_attrs(view: &mut SelectedUnitView, attrs: &Attributes) {
    view.hp = attrs.get(AttributeKind::Hp) as i32;
    view.max_hp = attrs.get(AttributeKind::MaxHp) as i32;
    view.mp = attrs.get(AttributeKind::Mp) as i32;
    view.max_mp = attrs.get(AttributeKind::MaxMp) as i32;
    view.stamina = attrs.get(AttributeKind::Stamina) as i32;
    view.max_stamina = attrs.get(AttributeKind::MaxStamina) as i32;
}

fn fill_core_attrs(view: &mut SelectedUnitView, attrs: &Attributes) {
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
}

fn fill_combat_attrs(view: &mut SelectedUnitView, attrs: &Attributes) {
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
}

fn fill_support_attrs(view: &mut SelectedUnitView, attrs: &Attributes) {
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
}

fn fill_skills(
    view: &mut SelectedUnitView,
    skill_slots: &SkillSlots,
    skill_registry: &SkillRegistry,
) {
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
}

fn fill_traits(
    view: &mut SelectedUnitView,
    trait_collection: Option<&TraitCollection>,
    trait_registry: &TraitRegistry,
) {
    view.traits = trait_collection
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
}

fn fill_buffs(view: &mut SelectedUnitView, buffs: &ActiveBuffs) {
    view.buffs = buffs
        .iter()
        .map(|inst| BuffEntry {
            name: inst.name.clone(),
            remaining_turns: inst.remaining_turns,
            is_buff: inst.is_buff,
        })
        .collect();
}

fn fill_equipment(
    view: &mut SelectedUnitView,
    equipment_slots: Option<&crate::core::equipment::EquipmentSlots>,
    equipment_registry: &crate::core::equipment::EquipmentRegistry,
) {
    use crate::core::equipment::EquipmentSlot;

    let all_slots = [
        EquipmentSlot::MainHand,
        EquipmentSlot::OffHand,
        EquipmentSlot::Head,
        EquipmentSlot::Body,
        EquipmentSlot::Feet,
        EquipmentSlot::Accessory1,
        EquipmentSlot::Accessory2,
    ];

    view.equipment = equipment_slots
        .map(|slots| {
            all_slots
                .iter()
                .map(|slot| {
                    let slot_label = slot.label().to_string();
                    let (item_name, rarity) = slots
                        .get_def_id(*slot)
                        .and_then(|def_id| equipment_registry.get(def_id))
                        .map(|def| (Some(def.name.clone()), Some(def.rarity.label().to_string())))
                        .unwrap_or((None, None));
                    EquipmentSlotEntry {
                        slot_label,
                        item_name,
                        rarity,
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| {
            all_slots
                .iter()
                .map(|slot| EquipmentSlotEntry {
                    slot_label: slot.label().to_string(),
                    item_name: None,
                    rarity: None,
                })
                .collect()
        });
}

fn fill_inventory(
    view: &mut SelectedUnitView,
    container: Option<&crate::core::inventory::container::Container>,
    item_registry: &crate::core::inventory::definition::ItemRegistry,
) {
    view.inventory = container
        .map(|c| {
            c.stacks
                .iter()
                .filter_map(|stack| {
                    item_registry
                        .get(&stack.instance.def_id)
                        .map(|def| InventoryEntry {
                            item_name: def.name.clone(),
                            rarity: def.rarity.label().to_string(),
                            count: stack.count,
                            instance_id: stack.instance.instance_id,
                        })
                })
                .collect()
        })
        .unwrap_or_default();
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

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证 ViewModel 默认值和枚举行为
    // ✅ 符合领域规则：是 — 覆盖 ViewModel 不变量
    // ✅ 确定性：是 — 硬编码默认值
    // ✅ 使用标准数据：是 — 使用标准 Default 实现
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 ViewModel 接口测试
    // ================================================

    use super::*;

    /// Test ID: UI-INV-002
    /// Title: SelectedUnitView 默认值为空
    ///
    /// Given: SelectedUnitView::default()
    /// When: 检查所有字段
    /// Then: 所有字段为空/零/false
    ///
    /// Assertions: name 为空, is_selected 为 false
    #[test]
    fn selected_unit_view_default_is_empty() {
        // Given
        let view = SelectedUnitView::default();

        // When - 无需操作

        // Then
        assert!(view.name.is_empty());
        assert!(view.race.is_empty());
        assert!(view.class.is_empty());
        assert_eq!(view.hp, 0);
        assert_eq!(view.max_hp, 0);
        assert_eq!(view.mp, 0);
        assert_eq!(view.max_mp, 0);
        assert_eq!(view.stamina, 0);
        assert_eq!(view.max_stamina, 0);
        assert!(!view.is_selected);
        assert!(view.core_attrs.is_empty());
        assert!(view.combat_attrs.is_empty());
        assert!(view.support_attrs.is_empty());
        assert!(view.skills.is_empty());
        assert!(view.traits.is_empty());
        assert!(view.buffs.is_empty());
        assert!(view.equipment.is_empty());
        assert!(view.inventory.is_empty());
    }

    /// Test ID: UI-INV-002b
    /// Title: CombatPreviewView 默认值为不可见
    ///
    /// Given: CombatPreviewView::default()
    /// When: 检查所有字段
    /// Then: is_visible 为 false，其他数值为 0
    ///
    /// Assertions: is_visible == false, estimated_damage == 0
    #[test]
    fn combat_preview_view_default_is_hidden() {
        // Given
        let view = CombatPreviewView::default();

        // When - 无需操作

        // Then
        assert!(!view.is_visible);
        assert_eq!(view.estimated_damage, 0);
        assert_eq!(view.hit_rate, 0);
        assert_eq!(view.crit_rate, 0);
        assert!(!view.is_lethal);
    }

    /// Test ID: UI-INV-002c
    /// Title: TurnInfoView 默认值为回合 0
    ///
    /// Given: TurnInfoView::default()
    /// When: 检查所有字段
    /// Then: turn_number 为 0，is_player_turn 为 false
    ///
    /// Assertions: turn_number == 0, is_player_turn == false
    #[test]
    fn turn_info_view_default_is_zero() {
        // Given
        let view = TurnInfoView::default();

        // When - 无需操作

        // Then
        assert_eq!(view.turn_number, 0);
        assert!(!view.is_player_turn);
        assert!(view.turn_order.is_empty());
        assert_eq!(view.current_index, 0);
    }

    /// Test ID: UI-INV-004
    /// Title: HoveredEntity 默认值为 None
    ///
    /// Given: HoveredEntity::default()
    /// When: 检查 entity 字段
    /// Then: entity 为 None
    ///
    /// Assertions: entity == None
    #[test]
    fn hovered_entity_default_is_none() {
        // Given
        let hovered = HoveredEntity::default();

        // When - 无需操作

        // Then
        assert!(hovered.entity.is_none());
    }

    /// Test ID: UI-INV-004b
    /// Title: HoveredEntity 可设置 Entity
    ///
    /// Given: HoveredEntity::default()
    /// When: 设置 entity = Some(Entity::from_bits(42))
    /// Then: entity 等于设置的值
    ///
    /// Assertions: entity == Some(Entity::from_bits(42))
    #[test]
    fn hovered_entity_can_set_entity() {
        // Given
        let mut hovered = HoveredEntity::default();
        let expected = Entity::from_bits(42);

        // When
        hovered.entity = Some(expected);

        // Then
        assert_eq!(hovered.entity, Some(expected));
    }
}
