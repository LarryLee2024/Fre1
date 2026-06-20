//! 队伍管理 Systems
//!
//! 包括成员管理、换人、羁绊评估等 System。
//! 不变量 3.4：羁绊条件必须在队伍成员变化时实时重新评估。

use bevy::prelude::*;

use crate::core::domains::party::components::{BondState, Party};
use crate::core::domains::party::events::{
    BondActivated, BondDeactivated, MemberJoined, MemberRemoved, MemberSwapped,
};
use crate::core::domains::party::rules::{
    add_member_to_party, check_bond_activation, remove_member_from_party, swap_active_with_reserve,
};

/// 响应成员加入事件：评估羁绊状态。
///
/// 不变量 3.4：成员变化时实时重新评估羁绊。
pub fn on_member_joined(
    _trigger: On<MemberJoined>,
    party: Res<Party>,
    mut bond_state: ResMut<BondState>,
    mut commands: Commands,
) {
    // 先收集需要激活的羁绊，避免在迭代时同时进行可变访问
    let active_ids: Vec<_> = bond_state
        .active_bonds
        .iter()
        .map(|b| b.bond_id.clone())
        .collect();
    let mut to_activate = Vec::new();

    for (bond_id, bond_def) in bond_state.defs.iter() {
        if active_ids.contains(bond_id) {
            continue;
        }

        if let Some(participants) = check_bond_activation(bond_def, &party.members) {
            to_activate.push((bond_id.clone(), participants, bond_def.name_key.clone()));
        }
    }

    // 统一激活
    for (bond_id, participants, name_key) in to_activate {
        bond_state
            .active_bonds
            .push(crate::core::domains::party::components::ActiveBond {
                bond_id: bond_id.clone(),
                level: 1,
                participants: participants.clone(),
                accumulated_battles: 0,
            });

        commands.trigger(BondActivated {
            party_id: None,
            bond_id,
            members: participants,
            effect_description: name_key,
        });
    }
}

/// 响应成员移除事件：重新评估羁绊。
///
/// 不变量 3.4：成员变化时实时重新评估羁绊。
pub fn on_member_removed(
    _trigger: On<MemberRemoved>,
    party: Res<Party>,
    mut bond_state: ResMut<BondState>,
    mut commands: Commands,
) {
    let removed_entity = _trigger.event().entity;

    // 检查是否有涉及该成员的羁绊需要解除
    let mut to_remove = Vec::new();
    for (idx, bond) in bond_state.active_bonds.iter().enumerate() {
        if bond.participants.contains(&removed_entity) {
            // 检查移除后是否仍满足条件
            let bond_def = bond_state.defs.get(&bond.bond_id);
            let still_valid =
                bond_def.is_some_and(|def| check_bond_activation(def, &party.members).is_some());

            if !still_valid {
                to_remove.push(idx);
            }
        }
    }

    // 逆序移除
    for &idx in to_remove.iter().rev() {
        let bond = bond_state.active_bonds.remove(idx);
        commands.trigger(BondDeactivated {
            party_id: None,
            bond_id: bond.bond_id,
            reason: "换人后羁绊条件已不满足".to_string(),
        });
    }
}

/// 响应换人事件：重新评估羁绊。
pub fn on_member_swapped(
    _trigger: On<MemberSwapped>,
    party: Res<Party>,
    mut bond_state: ResMut<BondState>,
    mut commands: Commands,
) {
    // 重新评估所有羁绊
    let mut still_valid: Vec<bool> = Vec::new();
    for bond in &bond_state.active_bonds {
        if let Some(def) = bond_state.defs.get(&bond.bond_id) {
            still_valid.push(check_bond_activation(def, &party.members).is_some());
        } else {
            still_valid.push(false);
        }
    }

    // 逆序移除失效的羁绊
    for idx in (0..bond_state.active_bonds.len()).rev() {
        if !still_valid[idx] {
            let bond = bond_state.active_bonds.remove(idx);
            commands.trigger(BondDeactivated {
                party_id: None,
                bond_id: bond.bond_id,
                reason: "换人后羁绊条件已不满足".to_string(),
            });
        }
    }

    // 先收集需要激活的新羁绊，避免在迭代时同时进行可变访问
    let active_ids: Vec<_> = bond_state
        .active_bonds
        .iter()
        .map(|b| b.bond_id.clone())
        .collect();
    let mut to_activate = Vec::new();

    for (bond_id, bond_def) in bond_state.defs.iter() {
        if active_ids.contains(bond_id) {
            continue;
        }

        if let Some(participants) = check_bond_activation(bond_def, &party.members) {
            to_activate.push((bond_id.clone(), participants, bond_def.name_key.clone()));
        }
    }

    // 统一激活新羁绊
    for (bond_id, participants, name_key) in to_activate {
        bond_state
            .active_bonds
            .push(crate::core::domains::party::components::ActiveBond {
                bond_id: bond_id.clone(),
                level: 1,
                participants: participants.clone(),
                accumulated_battles: 0,
            });

        commands.trigger(BondActivated {
            party_id: None,
            bond_id,
            members: participants,
            effect_description: name_key,
        });
    }
}

/// 添加成员到队伍并触发 MemberJoined 事件。
///
/// 外部调用方应在操作完成后调用此函数。
pub fn handle_add_member(
    mut commands: Commands,
    mut party: ResMut<Party>,
    entity: Entity,
    is_active: bool,
) {
    let role = if is_active {
        "active".to_string()
    } else {
        "reserve".to_string()
    };

    match add_member_to_party((*party).clone(), entity, is_active) {
        Ok((new_party, _slot_index)) => {
            *party = new_party;
            commands.trigger(MemberJoined {
                party_id: None,
                entity,
                role,
            });
        }
        Err(e) => {
            tracing::warn!(target: "party",
                event = "party.add_member.failed",
                entity = ?entity,
                error = %e,
                "添加成员失败：add_member_to_party 返回错误：{}",
                e
            );
        }
    }
}

/// 从队伍中移除成员并触发 MemberRemoved 事件。
pub fn handle_remove_member(
    mut commands: Commands,
    mut party: ResMut<Party>,
    entity: Entity,
    reason: String,
) {
    match remove_member_from_party((*party).clone(), entity) {
        Ok((new_party, _auto_filled)) => {
            *party = new_party;
            commands.trigger(MemberRemoved {
                party_id: None,
                entity,
                reason,
            });
        }
        Err(e) => {
            tracing::warn!(target: "party",
                event = "party.remove_member.failed",
                entity = ?entity,
                error = %e,
                "移除成员失败：remove_member_from_party 返回错误：{}",
                e
            );
        }
    }
}

/// 战斗中换人并触发 MemberSwapped 事件。
pub fn handle_swap_member(
    mut commands: Commands,
    mut party: ResMut<Party>,
    outgoing: Entity,
    incoming: Entity,
) {
    match swap_active_with_reserve((*party).clone(), outgoing, incoming) {
        Ok(new_party) => {
            *party = new_party;
            commands.trigger(MemberSwapped {
                party_id: None,
                outgoing,
                incoming,
            });
        }
        Err(e) => {
            tracing::warn!(target: "party",
                event = "party.swap_member.failed",
                outgoing = ?outgoing,
                incoming = ?incoming,
                error = %e,
                "换人失败：swap_active_with_reserve 返回错误：{}",
                e
            );
        }
    }
}
