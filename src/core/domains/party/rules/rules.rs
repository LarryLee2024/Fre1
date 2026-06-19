//! 队伍业务规则 — 纯函数
//!
//! 包括成员管理规则、换人规则、羁绊激活规则等。
//! 详见 docs/02-domain/domains/party_domain.md §3, §5

use bevy::prelude::*;

use crate::core::domains::party::components::{ActiveBond, BondDef, BondMatchMode, PartyMember};

use super::super::Party;

// ─── 成员管理规则 ──────────────────────────────────────────────────

/// 检查是否可以添加新成员（不变量 3.2：总人数上限）。
///
/// # 参数
/// - `party`: 队伍资源
///
/// # 返回值
/// `Ok(())` 或包含当前人数和上限的 `Err`。
pub fn can_add_member(party: &Party) -> Result<(), String> {
    let total = party.members.len() + party.reserve_members.len();
    if total >= party.max_total as usize {
        return Err(format!("队伍已满（{}/{})", total, party.max_total));
    }
    Ok(())
}

/// 检查是否可以激活更多成员（不变量 3.1：战斗人数上限）。
///
/// # 参数
/// - `party`: 队伍资源
///
/// # 返回值
/// `Ok(())` 或包含当前人数和上限的 `Err`。
pub fn can_activate_member(party: &Party) -> Result<(), String> {
    let active = party.members.iter().filter(|m| m.is_active).count();
    if active >= party.max_active as usize {
        return Err(format!("活跃成员已满（{}/{})", active, party.max_active));
    }
    Ok(())
}

/// 检查战斗中是否可以换人（不变量 3.3：每回合最多 1 次，消耗 1 行动力）。
///
/// # 参数
/// - `has_swapped_this_turn`: 本回合是否已换人
/// - `action_points`: 当前行动力
///
/// # 返回值
/// `Ok(())` 或错误描述。
pub fn can_swap_in_battle(has_swapped_this_turn: bool, action_points: u32) -> Result<(), String> {
    if has_swapped_this_turn {
        return Err("本回合已进行过换人操作".to_string());
    }
    if action_points < 1 {
        return Err("行动力不足，需要至少 1 点行动力".to_string());
    }
    Ok(())
}

/// 添加成员到队伍（纯函数）。
///
/// # 参数
/// - `party`: 当前队伍
/// - `entity`: 要添加的实体
/// - `is_active`: 是否作为活跃成员加入
///
/// # 返回值
/// 更新后的队伍和自动分配的 slot_index。
pub fn add_member_to_party(
    mut party: Party,
    entity: Entity,
    is_active: bool,
) -> Result<(Party, u32), String> {
    can_add_member(&party)?;

    // 检查是否已在队伍中
    let already_in =
        party.members.iter().any(|m| m.entity == entity) || party.reserve_members.contains(&entity);
    if already_in {
        return Err("该成员已在队伍中".to_string());
    }

    if is_active {
        can_activate_member(&party)?;
        let slot_index = party.members.len() as u32;
        party.members.push(PartyMember::new(entity, slot_index));
        Ok((party, slot_index))
    } else {
        let slot_index = (party.members.len() + party.reserve_members.len()) as u32;
        party.reserve_members.push(entity);
        Ok((party, slot_index))
    }
}

/// 从队伍中移除成员（纯函数）。
///
/// 如果移除的是活跃成员且有预备成员，自动从预备补充。
///
/// # 参数
/// - `party`: 当前队伍
/// - `entity`: 要移除的实体
///
/// # 返回值
/// 更新后的队伍和是否自动补充了成员。
pub fn remove_member_from_party(mut party: Party, entity: Entity) -> Result<(Party, bool), String> {
    // 尝试从活跃成员中移除
    if let Some(pos) = party.members.iter().position(|m| m.entity == entity) {
        party.members.remove(pos);

        // 如果有预备成员，自动补充（不变量流程 §5.1 第 4 步）
        let auto_filled = if !party.reserve_members.is_empty() {
            let reserve_entity = party.reserve_members.remove(0);
            let slot_index = party.members.len() as u32;
            party
                .members
                .push(PartyMember::new(reserve_entity, slot_index));
            true
        } else {
            false
        };

        Ok((party, auto_filled))
    } else if let Some(pos) = party.reserve_members.iter().position(|e| *e == entity) {
        // 从预备中移除
        party.reserve_members.remove(pos);
        Ok((party, false))
    } else {
        Err("未找到该成员".to_string())
    }
}

/// 交换两个活跃成员的位置。
pub fn swap_members_in_party(mut party: Party, a: usize, b: usize) -> Result<Party, String> {
    if a >= party.members.len() || b >= party.members.len() {
        return Err("成员索引越界".to_string());
    }
    party.members.swap(a, b);
    // 交换后更新 slot_index
    party.members[a].slot_index = a as u32;
    party.members[b].slot_index = b as u32;
    Ok(party)
}

/// 战斗中换人（活跃 ↔ 预备）。
pub fn swap_active_with_reserve(
    mut party: Party,
    active_entity: Entity,
    reserve_entity: Entity,
) -> Result<Party, String> {
    let active_pos = party.members.iter().position(|m| m.entity == active_entity);
    let reserve_pos = party
        .reserve_members
        .iter()
        .position(|e| *e == reserve_entity);

    match (active_pos, reserve_pos) {
        (Some(active_idx), Some(reserve_idx)) => {
            let reserve_entity = party.reserve_members.remove(reserve_idx);
            let old_member = &mut party.members[active_idx];
            let slot_index = old_member.slot_index;
            let formation_offset = old_member.formation_offset;

            // 活跃成员变为预备
            party.reserve_members.push(active_entity);

            // 预备成员变为活跃
            party.members[active_idx] = PartyMember {
                entity: reserve_entity,
                slot_index,
                formation_offset,
                is_active: true,
            };

            Ok(party)
        }
        (None, _) => Err("活跃成员未找到".to_string()),
        (_, None) => Err("预备成员未找到".to_string()),
    }
}

// ─── 羁绊规则 ──────────────────────────────────────────────────────

/// 检查羁绊是否满足激活条件（不变量 3.4：实时评估）。
///
/// # 参数
/// - `bond_def`: 羁绊模板定义
/// - `active_members`: 当前活跃成员列表
///
/// # 返回值
/// 满足条件时返回参与的角色列表，否则返回 `None`。
pub fn check_bond_activation(
    bond_def: &BondDef,
    active_members: &[PartyMember],
) -> Option<Vec<Entity>> {
    let mut participants = Vec::new();

    for requirement in &bond_def.required_members {
        let matched: Vec<&PartyMember> = active_members
            .iter()
            .filter(|m| {
                // 检查特定实体匹配
                if let Some(specific) = requirement.specific_entity
                    && m.entity == specific
                {
                    return true;
                }
                false
            })
            .collect();

        match requirement.match_mode {
            BondMatchMode::All => {
                if matched.is_empty() {
                    return None;
                }
            }
            BondMatchMode::Any => {
                if !matched.is_empty() {
                    // Any 模式：有一个匹配即可
                }
            }
        }

        for m in matched {
            if !participants.contains(&m.entity) {
                participants.push(m.entity);
            }
        }
    }

    if participants.is_empty() {
        None
    } else {
        Some(participants)
    }
}

/// 获取指定等级的经验需求（用于羁绊升级所需的战斗次数）。
pub fn battles_for_bond_level(target_level: u32) -> u32 {
    match target_level {
        1 => 0,
        2 => 3,
        3 => 8,
        _ => u32::MAX,
    }
}

/// 检查羁绊是否可以升级。
pub fn can_upgrade_bond(bond: &ActiveBond) -> bool {
    bond.level < 3 && bond.accumulated_battles >= battles_for_bond_level(bond.level + 1)
}
