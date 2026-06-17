---
id: 01-architecture.ADR-031
title: ADR-031 — Party & Camp/Rest Architecture
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-031: 队伍与休整系统架构

## 状态

**Approved** — 依赖 ADR-030（Progression/Inventory）和 ADR-021（Turn State Machine），本架构决策正式生效。

## 背景

队伍（Party）管理角色的编成和阵型。休整（Camp/Rest）是 SRPG 中战斗之间的恢复和交互环节。两者属于 Layer 5（Party & Camp），依赖 Layer 4 的 Progression/Inventory 和 Layer 2 的能力系统。

## 引用的领域规则与数据架构

- `docs/02-domain/domains/party_domain.md` — Party 领域规则
- `docs/02-domain/domains/camp_rest_domain.md` — Camp/Rest 领域规则
- `docs/04-data/domains/party_schema.md` — Party Schema
- `docs/04-data/domains/camp_rest_schema.md` — Camp/Rest Schema

## 决策

### 1. Party 架构

#### 1.1 队伍组成

```rust
/// Party — 全局队伍资源
#[derive(Resource)]
pub struct Party {
    pub members: Vec<PartyMember>,
    pub formation: FormationType,
    pub active_member: Option<usize>,   // 当前选中成员索引
    pub max_members: u32,
}

pub struct PartyMember {
    pub entity: Entity,          // 角色 Entity
    pub slot_index: u32,         // 队伍槽位
    pub formation_offset: Vec2,  // 阵型偏移
    pub is_active: bool,         // 是否出战
}

pub enum FormationType {
    Line,          // 一字排开
    Vanguard,      // 前 3 后 2
    Defensive,     // 前后各 2
    Custom(Vec<Vec2>), // 自定义偏移
}
```

#### 1.2 队伍管理的领域规则

```rust
/// Party 操作 API
impl Party {
    /// 添加成员（检查是否满员）
    pub fn add_member(&mut self, entity: Entity) -> Result<(), PartyError>;

    /// 移除成员
    pub fn remove_member(&mut self, entity: Entity) -> Result<PartyMember, PartyError>;

    /// 交换两个成员的位置
    pub fn swap_members(&mut self, a: usize, b: usize);

    /// 设置战斗中的阵型
    pub fn set_formation(&mut self, formation: FormationType);

    /// 获取战斗中的阵型位置偏移
    pub fn get_formation_offsets(&self) -> Vec<Vec2>;
}
```

#### 1.3 战斗中的队伍

战斗开始时，`Party` Resource 中的数据用于创建战场 Entity：

```rust
fn deploy_party_on_battle_start(
    party: Res<Party>,
    mut commands: Commands,
) {
    for member in &party.members {
        if member.is_active {
            commands.entity(member.entity)
                .insert((
                    InBattle::new(),
                    GridPos::from_offset(member.formation_offset),
                ));
        }
    }
}
```

### 2. Camp/Rest 架构

#### 2.1 休整状态

```rust
/// CampState — 休整期间的全局状态
#[derive(Resource)]
pub struct CampState {
    pub phase: CampPhase,
    pub rest_bonus: RestBonus,      // 休整效果加成
    pub available_interactions: Vec<CampInteraction>,
}

pub enum CampPhase {
    /// 进入营地
    Entering,
    /// 自由行动（对话、商店、合成）
    FreeRoam,
    /// 休整（恢复 HP/MP）
    Resting { progress: f32 },
    /// 离开营地
    Leaving,
}

pub struct RestBonus {
    pub hp_recovery_pct: f32,       // HP 恢复百分比
    pub mp_recovery_pct: f32,       // MP 恢复百分比
    pub status_heal: bool,          // 是否移除负面状态
    pub buff_retention: bool,       // 是否保留 Buff
}
```

#### 2.2 休整效果执行

```rust
fn apply_rest_effects(
    mut party: ResMut<Party>,
    mut health_query: Query<&mut Health>,
    mut mana_query: Query<&mut ManaPool>,
    camp: Res<CampState>,
) {
    for member in &party.members {
        // HP 恢复
        if let Ok(mut health) = health_query.get_mut(member.entity) {
            let recovery = health.max * camp.rest_bonus.hp_recovery_pct;
            health.current = (health.current + recovery).min(health.max);
        }
        // MP 恢复
        if let Ok(mut mana) = mana_query.get_mut(member.entity) {
            let recovery = mana.max * camp.rest_bonus.mp_recovery_pct;
            mana.current = (mana.current + recovery).min(mana.max);
        }
        // 状态清除（如果需要）
        if camp.rest_bonus.status_heal {
            commands.entity(member.entity)
                .remove::<Poisoned>()
                .remove::<Stunned>();
        }
    }
}
```

#### 2.3 营地交互

```
CampPhase::FreeRoam
       │
       ├── Party Management (调整队伍编成)
       ├── Equipment Management (更换装备)
       ├── Skill Management (学习/装备技能)
       ├── Shop (购买物品)
       ├── Crafting (合成制造)
       ├── Conversation (角色对话)
       └── Quest Progress Check (任务推进)
              │
              ▼
CampPhase::Resting (确认休整)
       │
       ▼
CampPhase::Leaving (离开营地，进入下一战)
```

### 3. 战斗 → 营地 → 战斗 闭环

```
战斗胜利
       │
       ▼
BattleEndEvent
       │
       ├── Party 返回
       ├── BattleState → CampState
       └── 经验结算 →
              │
              ▼
CampPhase::Entering
       │
       ▼
...[营地活动]...
       │
       ▼
CampPhase::Leaving
       │
       ▼
下一场战斗加载
```

## Module Design

```
src/core/domains/party/
  ├── plugin.rs              — PartyPlugin
  ├── components.rs          — PartyMember (Component, if needed per-entity)
  ├── resources.rs           — Party Resource
  ├── systems.rs             — add/remove/swap/formation
  └── api.rs                 — Party, FormationType

src/core/domains/camp_rest/
  ├── plugin.rs              — CampRestPlugin
  ├── resources.rs           — CampState
  ├── systems.rs             — camp_enter, camp_free_roam, apply_rest, camp_leave
  └── events.rs              — CampEnter, CampLeave, RestComplete
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| 战斗结束 → 营地 | Event (`CampEnter`) | combat → camp |
| 营地活动 | 直接 API 调用 | camp 内部 |
| 营地离开 → 下一战 | Event (`CampLeave`) → NextState | camp → game |
| 队伍编成变更 | `Party` Resource 直接修改 | camp/party 内部 |

## 边界定义

### 允许
- Camp 中调用 Inventory/Economy/Crafting 的公开 API
- Camp 中修改 Party 成员列表
- Rest 时直接恢复 HP/MP（资源型属性直接修改例外）
- Camp 状态使用独立的 State

### 🟥 禁止
- Camp 中触发战斗逻辑（伤害计算、Buff 冲突）
- Party 成员在非战斗状态下拥有 GridPos
- Rest 移除永久的 Buff/Passive（只移除临时状态）
- Camp 阶段处理实时输入（Camp 是 UI 驱动的）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| Party 管理中直接修改角色属性 | 必须通过 Modifier Pipeline |
| Camp 中执行战斗 Effect | Camp 非战斗状态 |
| 队员在 Party 和战场中数据不一致 | 序列化/反序列化必须一致 |
| Camp 依赖 UI 模块 | Camp 是业务逻辑，UI 是表现 |

## Definition / Instance Design

- **Definition**: `FormationDef` (config), `RestBonusDef` (config)
- **Instance**: `Party` (Resource), `CampState` (Resource), `PartyMember` (struct)
- **Persistence**: `Party.members`（包含 Entity 引用需重映射）、`CampState`（无需持久化，每次新进入）

## 后果

### 正面
- Party 管理独立于战斗，编成在休整期变更
- Camp 状态机覆盖了完整的休整流程
- HP/MP 恢复走直接修改（允许的例外路径）

### 负面
- Party 的 `Entity` 引用在存档读档时需要重映射
- Camp 状态机的阶段数量可能随着玩法增加而膨胀

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| Party 作为 ECS 中的 Entity Group | Bevy Relationship 可以处理但初期不需要 |
| 无 Camp，只有自动恢复 | 缺少战略决策深度 |
| Camp 中所有操作走 Event | 内部操作直接调用更简洁 |

## 评审要点

- [ ] Formation 的偏移量——是否支持运行时自定义阵型？
- [ ] Camp 中是否可以保存（中途退出）？
- [ ] 队员在战斗中死亡，回到营地后如何恢复？
- [ ] Camp 交互是否需要"行动点数"限制（每次休整可做的事有限）？
