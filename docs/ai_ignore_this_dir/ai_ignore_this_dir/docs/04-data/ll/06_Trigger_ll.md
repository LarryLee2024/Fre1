# Trigger 领域 — 铃兰之剑数据提取

> 领域：Trigger | 来源：78铃兰.md §七、补充1、补充5、补充7 | 数据层：Definition + Instance

---

## 一、数据实体清单

### 1.1 TriggerDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | TriggerId | PK | 触发器唯一标识 | §七 |
| `event_type` | EventType | - | 监听的事件类型 | §七.1 |
| `condition` | Option<ConditionDef> | - | 触发条件 | §七.2 |
| `effect` | EffectId | FK | 触发后执行的效果 | §七.2 |
| `priority` | u32 | ≥0 | 触发优先级 | §七.3 |
| `max_trigger_per_turn` | Option<u32> | - | 每回合最大触发次数 | §七.2 |
| `max_trigger_per_battle` | Option<u32> | - | 每场战斗最大触发次数 | §七.2 |
| `chain_depth` | u32 | =0 | 触发链深度限制 | §七.3 |

### 1.2 TriggerInstance（Instance层）

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `entity` | Entity | 挂载实体 | - |
| `trigger_id` | TriggerId | 引用TriggerDefinition | - |
| `triggers_this_turn` | u32 | 本回合已触发次数 | §七.2 |
| `triggers_this_battle` | u32 | 本场战斗已触发次数 | §七.2 |

---

## 二、触发事件分类（5大类）

### 2.1 EventType 枚举

| 事件大类 | 事件ID | 说明 | 来源 |
|----------|--------|------|------|
| **回合事件** | `TurnStart` | 回合开始 | §七.1 |
| | `TurnEnd` | 回合结束 | §七.1 |
| | `ActionStart` | 行动开始 | §七.1 |
| | `ActionEnd` | 行动结束 | §七.1 |
| **战斗事件** | `DealDamage` | 造成伤害 | §七.1 |
| | `TakeDamage` | 受到伤害 | §七.1 |
| | `KillUnit` | 击杀单位 | §七.1 |
| | `UnitDied` | 被击杀 | §七.1 |
| **技能事件** | `BeforeSkillCast` | 释放技能前 | §七.1 |
| | `AfterSkillCast` | 释放技能后 | §七.1 |
| | `SkillHit` | 技能命中 | §七.1 |
| **状态事件** | `BuffApplied` | 获得Buff | §七.1 |
| | `BuffRemoved` | 失去Buff | §七.1 |
| | `ControlApplied` | 控制生效 | §七.1 |
| | `ControlRemoved` | 控制解除 | §七.1 |
| **移动事件** | `MoveStart` | 移动开始 | §七.1 |
| | `MoveEnd` | 移动结束 | §七.1 |
| | `EnterTile` | 进入格子 | §七.1 |
| | `LeaveTile` | 离开格子 | §七.1 |

### 2.2 补充事件（死亡链路）

| 事件ID | 说明 | 来源 |
|--------|------|------|
| `NearDeath` | 濒死窗口 | 补充7 |
| `AboutToDie` | 即将死亡 | 补充7 |
| `UnitDeath` | 单位死亡 | 补充7 |

### 2.3 补充事件（多段伤害）

| 事件ID | 说明 | 触发频率 | 来源 |
|--------|------|----------|------|
| `OnDamageDealt` | 造成伤害时 | 每段触发 | 补充5 |
| `OnSkillHit` | 被技能命中时 | 整个技能触发1次 | 补充5 |

> **关键边界**：「技能命中」和「造成伤害」是两个独立事件，不能合并。

---

## 三、反应技规则

### 3.1 反应技核心约束

| 约束 | 说明 | 来源 |
|------|------|------|
| 触发条件 | 必须满足特定事件+特定条件 | §七.2 |
| 次数限制 | 每回合/每场战斗有最大触发次数 | §七.2 |
| 不消耗行动 | 不占用主动行动次数 | §七.2 |
| 独立伤害计算 | 单独走伤害公式，通常有倍率衰减 | §七.2 |

### 3.2 典型反应技

| 反应技 | 触发事件 | 效果 | 来源 |
|--------|----------|------|------|
| 反击 | TakeDamage | 普攻回击攻击者 | §七.2 |
| 援护 | AllyTakeDamage | 替友军承受伤害 | §七.2 |
| 追击 | AllyDealDamage | 友军攻击时补刀 | §七.2 |

---

## 四、触发优先级与冲突处理

### 4.1 优先级规则

| 规则 | 说明 | 来源 |
|------|------|------|
| 同事件多触发器 | 按优先级顺序执行，高优先级先结算 | §七.3 |
| 减伤类先于伤害类 | 减伤Trigger优先级高于伤害Trigger | §七.3 |
| 同类反应只触发最高级 | 多个反击效果只触发优先级最高的一个 | §七.3 |

### 4.2 触发链深度限制

| 规则 | 说明 | 来源 |
|------|------|------|
| 链深度=0 | 反应技触发的伤害不能再触发新的反应 | §七.3 |
| 典型 | 反击不会触发对方的反击 | §七.3 |

### 4.3 反击类堆叠

| 规则 | 说明 | 来源 |
|------|------|------|
| 多个反击共存 | 只触发优先级最高的一个 | §五.3 |
| 反击次数增加 | 对每个反击单独生效，分别加次数 | §五.3 |
| 示例 | 回击+强力回击+反击次数+3，单回合最多8次反击 | §五.3 |

---

## 五、Schema草案

```yaml
# trigger_config.ron
(
  triggers: [
    # 反击触发器
    (id: "counter_attack", event_type: TakeDamage,
     condition: (attacker_in_range: true, not_controlled: true),
     effect: "counter_damage", priority: 10,
     max_trigger_per_turn: Some(1), chain_depth: 0),
    # 濒死触发器
    (id: "near_death_save", event_type: NearDeath,
     condition: (has_save_buff: true),
     effect: "prevent_death", priority: 99,
     max_trigger_per_battle: Some(1), chain_depth: 0),
  ],
)
```

---

## 六、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Effect | Trigger → Effect | 触发后执行Effect |
| Ability | Trigger → Ability | 反应技是特殊Ability |
| Tag | Trigger ← Tag | Tag作为触发条件 |
| Pipeline | Trigger ← Pipeline | Pipeline定义触发时机 |

---

## 七、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 004 | ✅ | 事件驱动行为归属Trigger领域，Ability不拥有行为 |
| 005 | ✅ | Trigger触发Effect执行，不直接调用Modifier |
| 010 | ✅ | 触发链深度限制保证Replay确定性 |

---

## 八、代码实现映射

| 概念 | Rust 类型 | 源码路径 | 层级 |
|------|-----------|----------|------|
| Trigger | `enum Trigger { TurnStart, TurnEnd, BeforeAttack, AfterAttack, BeforeDamaged, AfterDamaged, BeforeMove, AfterMove, KillTarget, Death, BattleStart, BattleEnd, OnHeal, OnBuffApplied, OnBuffRemoved, OnRevive }`（15+ 变体） | `src/core/trigger/mod.rs` | Definition |
| TriggerContext | `TriggerContext { trigger, source_entity, target_entity, damage_dealt, is_critical, chain_depth }` | `src/core/trigger/mod.rs` | Runtime |

**触发链深度限制**：`chain_depth` 字段防止反应技触发无限递归（反应技造成的伤害不再触发新的反应）

**当前实现**：Trigger 系统通过 `enum Trigger` + `TriggerContext` 实现事件分发，与 Battle 事件系统集成