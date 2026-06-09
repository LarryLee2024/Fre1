// ViewModel 层：游戏逻辑 → ViewModel → UI
// UI 系统只读 ViewModel，不直接 Query 游戏组件

use bevy::prelude::*;

use crate::buff::ActiveBuffs;
use crate::character::{Faction, Selected, Unit, UnitName};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::skill::{SkillRegistry, SkillSlots};
use crate::turn::TurnState;

// ── ViewModel 定义 ──

/// 选中单位的视图模型
#[derive(Resource, Default, Debug)]
pub struct SelectedUnitView {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub mov: i32,
    pub magic_attack: i32,
    pub magic_defense: i32,
    pub core_attrs: String,
    pub skills: String,
    pub buffs: String,
    pub is_selected: bool,
}

/// 回合信息的视图模型
#[derive(Resource, Default, Debug)]
pub struct TurnInfoView {
    pub turn_number: u32,
    pub faction_label: String,
    pub is_player_turn: bool,
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

/// 从游戏数据构建 SelectedUnitView
pub fn update_selected_unit_view(
    selected_units: Query<
        (&Unit, &UnitName, &Attributes, &SkillSlots, &ActiveBuffs),
        With<Selected>,
    >,
    skill_registry: Res<SkillRegistry>,
    mut view: ResMut<SelectedUnitView>,
) {
    if let Ok((_unit, name, attrs, skill_slots, buffs)) = selected_units.single() {
        view.name = name.0.clone();
        view.hp = attrs.get(AttributeKind::Hp) as i32;
        view.max_hp = attrs.get(AttributeKind::MaxHp) as i32;
        view.atk = attrs.get(AttributeKind::Attack) as i32;
        view.def = attrs.get(AttributeKind::Defense) as i32;
        view.mov = attrs.get(AttributeKind::MoveRange) as i32;
        view.magic_attack = attrs.get(AttributeKind::MagicAttack) as i32;
        view.magic_defense = attrs.get(AttributeKind::MagicDefense) as i32;
        view.core_attrs = format!(
            "MIG:{} DEX:{} AGI:{} VIT:{} INT:{} WIL:{} PRE:{} LCK:{}",
            attrs.get(AttributeKind::Might) as i32,
            attrs.get(AttributeKind::Dexterity) as i32,
            attrs.get(AttributeKind::Agility) as i32,
            attrs.get(AttributeKind::Vitality) as i32,
            attrs.get(AttributeKind::Intelligence) as i32,
            attrs.get(AttributeKind::Willpower) as i32,
            attrs.get(AttributeKind::Presence) as i32,
            attrs.get(AttributeKind::Luck) as i32,
        );

        // 技能名称列表
        let skill_names: Vec<String> = skill_slots
            .skill_ids
            .iter()
            .filter_map(|id| skill_registry.get(id).map(|sd| sd.name.clone()))
            .collect();
        view.skills = if skill_names.is_empty() {
            "无".to_string()
        } else {
            skill_names.join("/")
        };

        // Buff 列表
        view.buffs = if buffs.is_empty() {
            "无".to_string()
        } else {
            buffs
                .iter()
                .map(|inst| format!("[{}·{}t]", inst.name, inst.remaining_turns))
                .collect::<Vec<_>>()
                .join("")
        };

        view.is_selected = true;
    } else {
        view.is_selected = false;
        view.name.clear();
    }
}

/// 从游戏数据构建 TurnInfoView
pub fn update_turn_info_view(turn_state: Res<TurnState>, mut view: ResMut<TurnInfoView>) {
    if turn_state.is_changed() {
        view.turn_number = turn_state.turn_number;
        view.faction_label = match turn_state.current_faction {
            Faction::Player => "玩家".to_string(),
            Faction::Enemy => "敌方".to_string(),
        };
        view.is_player_turn = turn_state.current_faction == Faction::Player;
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
