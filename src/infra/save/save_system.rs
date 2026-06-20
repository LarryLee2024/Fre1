use bevy::prelude::*;

use crate::core::domains::combat::{ActionPoints, BattlePhase, CombatParticipant, Dead, TurnQueue};
use crate::core::domains::party::{ActiveBond, BondState, Party, PartyMember};
use crate::core::domains::progression::{ClassLevels, Experience, SubclassChoice, TalentTree};

/// Query type for progression entities — factored out to satisfy clippy::type_complexity.
type ProgressionEntityQuery = (
    Entity,
    &'static Experience,
    &'static ClassLevels,
    Option<&'static TalentTree>,
    Option<&'static SubclassChoice>,
);

use super::events::{SaveCompleted, SaveRequest};
use super::resources::{EntityRemapper, SaveManager};
use super::save_data::*;

/// 响应 SaveRequest 事件：将当前世界状态序列化为 JSON 存档。
///
/// 当前序列化 Combat / Party / Progression 三个领域的数据；
/// 其他领域（Inventory、Quest、Narrative 等）将在后续迭代中添加。
/// 使用 EntityRemapper 将运行时 Entity 映射为持久化 ID，确保加载时正确重建引用。
pub fn save_world_system(
    trigger: On<SaveRequest>,
    mut save_manager: ResMut<SaveManager>,
    mut entity_remapper: ResMut<EntityRemapper>,
    combat_participants: Query<(
        Entity,
        &CombatParticipant,
        Option<&Dead>,
        Option<&ActionPoints>,
    )>,
    progression_entities: Query<ProgressionEntityQuery>,
    battle_phase: Option<Res<State<BattlePhase>>>,
    turn_queue: Option<Res<TurnQueue>>,
    party: Option<Res<Party>>,
    bond_state: Option<Res<BondState>>,
    mut commands: Commands,
) {
    let request = trigger.event();
    let path = request
        .path
        .clone()
        .or_else(|| {
            save_manager
                .current_save_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "save_001.fresave".to_string());

    if let Some(label) = &request.label {
        save_manager.metadata.label = label.clone();
        tracing::info!("[SaveSystem] saving with label: {}", label);
    }
    save_manager.current_save_path = Some(std::path::PathBuf::from(&path));
    save_manager.is_dirty = false;

    entity_remapper.clear();

    let mut combat_entities = Vec::new();
    for (entity, participant, dead, action_points) in combat_participants.iter() {
        let pid = entity_remapper.assign(entity);
        let entry = turn_queue
            .as_ref()
            .and_then(|tq| tq.entries().iter().find(|e| e.entity == entity));
        let initiative = entry.map(|e| e.initiative).unwrap_or(0);

        combat_entities.push(CombatEntityData {
            persistent_id: pid.0,
            team_id: participant.team_id.to_string(),
            initiative,
            is_dead: dead.is_some(),
            action_points: action_points.map(|ap| ActionPointsData {
                standard_action: ap.standard_action,
                bonus_action: ap.bonus_action,
                reaction: ap.reaction,
                movement: ap.movement,
                max_movement: ap.max_movement,
            }),
        });
    }

    let phase_str = match battle_phase.as_ref().map(|bp| bp.get()) {
        Some(BattlePhase::Preparation) => "Preparation",
        Some(BattlePhase::Battle) => "Battle",
        Some(BattlePhase::Victory) => "Victory",
        Some(BattlePhase::Defeat) => "Defeat",
        None => "Preparation",
    }
    .to_string();

    let active_members: Vec<PartyMemberSaveData> = party
        .as_ref()
        .map(|p| {
            p.members
                .iter()
                .filter_map(|m| {
                    entity_remapper
                        .lookup_persistent(m.entity)
                        .map(|pid| PartyMemberSaveData {
                            persistent_id: pid.0,
                            slot_index: m.slot_index,
                            is_active: m.is_active,
                        })
                })
                .collect()
        })
        .unwrap_or_default();

    let reserve_members: Vec<u64> = party
        .as_ref()
        .map(|p| {
            p.reserve_members
                .iter()
                .filter_map(|e| entity_remapper.lookup_persistent(*e).map(|pid| pid.0))
                .collect()
        })
        .unwrap_or_default();

    let active_bonds: Vec<ActiveBondSaveData> = bond_state
        .as_ref()
        .map(|bs| {
            bs.active_bonds
                .iter()
                .map(|bond| {
                    let participant_ids: Vec<u64> = bond
                        .participants
                        .iter()
                        .filter_map(|e| entity_remapper.lookup_persistent(*e).map(|pid| pid.0))
                        .collect();
                    ActiveBondSaveData {
                        bond_id: bond.bond_id.to_string(),
                        level: bond.level,
                        participant_ids,
                        accumulated_battles: bond.accumulated_battles,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mut prog_data = Vec::new();
    for (entity, xp, class_levels, talent_tree, subclass_choice) in progression_entities.iter() {
        let pid = if let Some(pid) = entity_remapper.lookup_persistent(entity) {
            pid
        } else {
            entity_remapper.assign(entity)
        };

        let subclass_choices: Vec<(String, String)> = subclass_choice
            .map(|sc| {
                sc.choices
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        prog_data.push(ProgressionEntityData {
            persistent_id: pid.0,
            experience: ExperienceData {
                current_xp: xp.current_xp,
                level: xp.level,
                total_xp_earned: xp.total_xp_earned,
                is_max_level: xp.is_max_level,
            },
            class_levels: ClassLevelsData {
                entries: class_levels
                    .entries
                    .iter()
                    .map(|e| ClassLevelEntryData {
                        class_id: e.class_id.to_string(),
                        level: e.level,
                    })
                    .collect(),
            },
            talent_tree: TalentTreeData {
                unlocked_talents: talent_tree
                    .map(|t| t.unlocked_talents.iter().map(|id| id.to_string()).collect())
                    .unwrap_or_default(),
                available_points: talent_tree.map(|t| t.available_points).unwrap_or(0),
            },
            subclass_choices,
        });
    }

    let world_data = WorldSaveData {
        save_version: save_manager.save_version,
        metadata: SaveMetadataData {
            label: save_manager.metadata.label.clone(),
            location: save_manager.metadata.location.clone(),
            playtime_seconds: save_manager.metadata.playtime_seconds,
            player_level: save_manager.metadata.player_level,
        },
        combat: CombatSaveData {
            phase: phase_str,
            round_number: turn_queue.as_ref().map_or(1, |tq| tq.round_number()),
            current_index: turn_queue.as_ref().map_or(0, |tq| tq.current_index()),
            participants: combat_entities,
        },
        party: PartySaveData {
            formation: party.as_ref().map_or_else(
                || {
                    format!(
                        "{:?}",
                        crate::core::domains::party::FormationType::default()
                    )
                },
                |p| format!("{:?}", p.formation),
            ),
            max_active: party.as_ref().map_or(4, |p| p.max_active),
            max_total: party.as_ref().map_or(12, |p| p.max_total),
            active_members,
            reserve_members,
            active_bonds,
        },
        progression: ProgressionSaveData {
            entities: prog_data,
        },
    };

    match serde_json::to_string_pretty(&world_data) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, &json) {
                tracing::error!("[SaveSystem] failed to write save file: {}", e);
                return;
            }
            let entity_count =
                world_data.combat.participants.len() + world_data.progression.entities.len();
            tracing::info!(
                "[SaveSystem] save completed: path={}, entities={}",
                path,
                entity_count
            );
            commands.trigger(SaveCompleted {
                path,
                entity_count: entity_count as u32,
                success: true,
            });
        }
        Err(e) => {
            tracing::error!("[SaveSystem] serialization failed: {}", e);
        }
    }
}
