use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::domains::combat::{
    ActionPoints, BattlePhase, CombatParticipant, Dead, TeamId, TurnEntry, TurnQueue,
};
use crate::core::domains::party::{ActiveBond, BondState, Party, PartyMember};
use crate::core::domains::progression::{
    ClassId, ClassLevels, Experience, ProgressionMarker, SubclassChoice, SubclassId, TalentId,
    TalentTree,
};

use super::events::{LoadCompleted, LoadRequest, SaveError, SaveOperation};
use super::resources::{EntityRemapper, PersistentEntityId, SaveManager};
use super::save_data::*;

#[derive(Resource)]
pub(crate) struct PendingLoad {
    pub data: WorldSaveData,
    pub path: String,
}

/// Observer: LoadRequest → 从指定路径加载存档并替换当前世界状态。
pub fn on_load_request(
    trigger: On<LoadRequest>,
    _save_manager: ResMut<SaveManager>,
    mut commands: Commands,
) {
    let path = trigger.event().path.clone();
    if !std::path::Path::new(&path).exists() {
        tracing::error!(target: "save", "[LoadSystem] 存档文件未找到：{}", path);
        commands.trigger(SaveError {
            error_context: crate::shared::error::ErrorContext {
                domain: "save",
                source: format!("save file not found: {}", path),
                context: None,
            },
            operation: SaveOperation::Load,
        });
        return;
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(target: "save", "[LoadSystem] 读取存档文件失败：{}", e);
            commands.trigger(SaveError {
                error_context: crate::shared::error::ErrorContext {
                    domain: "save",
                    source: format!("failed to read save file: {}", e),
                    context: None,
                },
                operation: SaveOperation::Load,
            });
            return;
        }
    };

    let world_data: WorldSaveData = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!(target: "save", "[LoadSystem] 反序列化存档失败：{}", e);
            commands.trigger(SaveError {
                error_context: crate::shared::error::ErrorContext {
                    domain: "save",
                    source: format!("failed to deserialize save: {}", e),
                    context: None,
                },
                operation: SaveOperation::Load,
            });
            return;
        }
    };

    tracing::info!(target: "save",
        "[LoadSystem] 反序列化存档：版本={}, 战斗实体数={}, 成长实体数={}",
        world_data.save_version,
        world_data.combat.participants.len(),
        world_data.progression.entities.len()
    );

    commands.insert_resource(PendingLoad {
        data: world_data,
        path,
    });
}

pub(crate) fn process_pending_load(
    mut commands: Commands,
    pending: Option<ResMut<PendingLoad>>,
    mut save_manager: ResMut<SaveManager>,
    mut entity_remapper: ResMut<EntityRemapper>,
    party: Option<ResMut<Party>>,
    bond_state: Option<ResMut<BondState>>,
    turn_queue: Option<ResMut<TurnQueue>>,
    battle_phase: Option<ResMut<State<BattlePhase>>>,
) {
    let Some(pending) = pending else {
        return;
    };

    let data = pending.data.clone();
    let path = pending.path.clone();
    commands.remove_resource::<PendingLoad>();

    entity_remapper.clear();

    let mut combat_entities = Vec::new();
    for combat_entity in &data.combat.participants {
        let entity = commands.spawn_empty().id();
        let pid = entity_remapper.assign(entity);
        combat_entities.push((entity, pid, combat_entity));
    }

    for (entity, _pid, combat_entity) in &combat_entities {
        let team_id = TeamId::new(&combat_entity.team_id);
        commands
            .entity(*entity)
            .insert(CombatParticipant::alive(team_id));

        if combat_entity.is_dead {
            commands.entity(*entity).insert(Dead);
        }

        if let Some(ref ap_data) = combat_entity.action_points {
            commands.entity(*entity).insert(ActionPoints {
                standard_action: ap_data.standard_action,
                bonus_action: ap_data.bonus_action,
                reaction: ap_data.reaction,
                movement: ap_data.movement,
                max_movement: ap_data.max_movement,
            });
        }
    }

    let turn_entries: Vec<TurnEntry> = combat_entities
        .iter()
        .filter_map(|(entity, _pid, combat_entity)| {
            let init = combat_entity.initiative;
            if init > 0 || !combat_entity.team_id.is_empty() {
                Some(TurnEntry::new(
                    *entity,
                    TeamId::new(&combat_entity.team_id),
                    init,
                ))
            } else {
                None
            }
        })
        .collect();

    if let Some(mut tq) = turn_queue {
        *tq = TurnQueue::new(turn_entries);
    }

    let phase = match data.combat.phase.as_str() {
        "Preparation" => BattlePhase::Preparation,
        "Battle" => BattlePhase::Battle,
        "Victory" => BattlePhase::Victory,
        "Defeat" => BattlePhase::Defeat,
        _ => BattlePhase::Preparation,
    };
    if battle_phase.is_some() {
        commands.insert_resource(NextState::Pending(phase));
    }

    let mut active_members = Vec::new();
    for member_data in &data.party.active_members {
        let entity = commands.spawn_empty().id();
        entity_remapper.assign(entity);
        active_members.push(PartyMember {
            entity,
            slot_index: member_data.slot_index,
            formation_offset: Vec2::ZERO,
            is_active: member_data.is_active,
        });
    }

    let mut reserve_entities = Vec::new();
    for _ in &data.party.reserve_members {
        let entity = commands.spawn_empty().id();
        entity_remapper.assign(entity);
        reserve_entities.push(entity);
    }

    if let Some(mut p) = party {
        p.members = active_members;
        p.reserve_members = reserve_entities;
        p.max_active = data.party.max_active;
        p.max_total = data.party.max_total;
    }

    if let Some(mut bs) = bond_state {
        *bs = BondState {
            active_bonds: data
                .party
                .active_bonds
                .iter()
                .map(|bond_data| {
                    let participants: Vec<Entity> = bond_data
                        .participant_ids
                        .iter()
                        .filter_map(|pid| entity_remapper.lookup(PersistentEntityId(*pid)))
                        .collect();
                    ActiveBond {
                        bond_id: crate::shared::ids::BondDefId::new(&bond_data.bond_id),
                        level: bond_data.level,
                        participants,
                        accumulated_battles: bond_data.accumulated_battles,
                    }
                })
                .collect(),
            defs: HashMap::new(),
        };
    }

    for prog_entity in &data.progression.entities {
        let entity = commands.spawn_empty().id();
        entity_remapper.assign(entity);

        let class_id_strs: Vec<String> = prog_entity
            .class_levels
            .entries
            .iter()
            .map(|e| e.class_id.clone())
            .collect();

        let entries: Vec<crate::core::domains::progression::ClassLevelEntry> = prog_entity
            .class_levels
            .entries
            .iter()
            .map(|e| {
                crate::core::domains::progression::ClassLevelEntry::new(
                    ClassId::new(&e.class_id),
                    e.level,
                )
            })
            .collect();

        let subclass_choices: HashMap<ClassId, SubclassId> = prog_entity
            .subclass_choices
            .iter()
            .map(|(k, v)| (ClassId::new(k), crate::shared::ids::SubclassId::new(v)))
            .collect();

        let talents: Vec<TalentId> = prog_entity
            .talent_tree
            .unlocked_talents
            .iter()
            .map(TalentId::new)
            .collect();

        commands.entity(entity).insert((
            ProgressionMarker,
            Experience {
                current_xp: prog_entity.experience.current_xp,
                level: prog_entity.experience.level,
                total_xp_earned: prog_entity.experience.total_xp_earned,
                is_max_level: prog_entity.experience.is_max_level,
            },
            ClassLevels { entries },
            TalentTree {
                unlocked_talents: talents,
                available_points: prog_entity.talent_tree.available_points,
            },
            SubclassChoice {
                choices: subclass_choices,
            },
        ));

        let _ = class_id_strs;
    }

    save_manager.current_save_path = Some(std::path::PathBuf::from(&path));
    save_manager.save_version = data.save_version;
    save_manager.metadata.label = data.metadata.label;
    save_manager.metadata.location = data.metadata.location;
    save_manager.metadata.playtime_seconds = data.metadata.playtime_seconds;
    save_manager.metadata.player_level = data.metadata.player_level;
    save_manager.is_dirty = false;

    let entity_count =
        data.combat.participants.len() as u32 + data.progression.entities.len() as u32;
    tracing::info!(target: "save",
        "[LoadSystem] 加载完成：路径={}, 实体数={}",
        path,
        entity_count
    );
    commands.trigger(LoadCompleted {
        path,
        entity_count,
        save_version: save_manager.save_version,
        success: true,
    });
}
