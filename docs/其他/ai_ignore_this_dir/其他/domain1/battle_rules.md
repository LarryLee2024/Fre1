# 战斗领域规则 (Battle Rules)

## 1. 领域概述

战斗系统是 SRPG 的核心玩法循环，负责处理从攻击意图到伤害结算的完整流程。采用**三步效果管线**（Generate → Modify → Execute）实现战斗逻辑，通过 Message 机制实现逻辑与表现分离。

### 核心原则

- **Logic / Presentation 分离**：逻辑产生结果，表现负责展示；伤害计算不依赖 UI 和特效
- **ECS 是数据流，不是调用链**：不模拟 `player.attack(enemy)`
- **Message 负责跨 Feature 广播**：伤害、死亡等事件通过 Message 通知
- **EntityEvent 适合复杂战斗链**：装备→护盾→角色→被动技能

---

## 2. 效果管线（Effect Pipeline）

### 2.1 三步流程

```
Generate → Modify → Execute
  生成       修饰      执行
```

| 步骤 | 系统 | 职责 | 输入 | 输出 |
|------|------|------|------|------|
| Generate | `generate_combat_effects` | 从技能定义 + 属性计算原始效果 | CombatIntent + SkillData | EffectQueue.pending |
| Modify | `modify_effects` | 应用 ModifierRule 修饰规则 | EffectQueue + ModifierRuleRegistry | 修饰后的 EffectQueue |
| Execute | `execute_effects` | 扣血/加Buff/击杀判定 + 发送 Message | 修饰后的 EffectQueue | 属性变化 + Messages |

### 2.2 EffectQueue — 效果队列

```rust
#[derive(Resource, Default)]
pub struct EffectQueue {
    pub pending: Vec<PendingEffect>,
}
```

**规则**：
- 管线三步共享同一个 EffectQueue
- Generate 推入效果，Modify 修改效果，Execute 消费效果
- Execute 使用 `drain(..)` 清空队列

### 2.3 PendingEffect — 待处理效果

```rust
pub struct PendingEffect {
    pub source: Entity,           // 攻击者
    pub target: Entity,           // 目标
    pub data: PendingEffectData,  // 效果数据
    pub source_tags: Vec<GameplayTag>,  // 技能标签
    pub terrain_id: String,       // 地形 ID
}
```

### 2.4 PendingEffectData — 效果数据

| 类型 | 字段 | 说明 |
|------|------|------|
| `Damage` | `amount, is_skill, base_amount, modifiers` | 伤害效果 |
| `Heal` | `amount, base_amount` | 治疗效果 |
| `ApplyBuff` | `buff_id, duration` | 施加 Buff |
| `Cleanse` | — | 净化所有 Debuff |

---

## 3. Generate 阶段详解

### 3.1 攻击者来源

| 来源 | 查找方式 | 场景 |
|------|----------|------|
| 玩家 | `Selected` 组件 | 玩家选择攻击 |
| AI | `CombatIntent.source_entity` | AI 决策攻击 |

### 3.2 前置检查

1. **晕眩检查**：攻击者有 `STUN` 标签时跳过
2. **冷却检查**：技能冷却 > 0 时跳过（玩家需要，AI 已在决策时检查）
3. **目标查找**：匹配 `target_coord` 且不同阵营的单位

### 3.3 效果生成

```rust
for effect_def in &skill_data.effects {
    if let Some(handler) = handler_registry.find(effect_def.type_name()) {
        let ctx = GenerateContext { source_attrs, target_attrs, defense_bonus, ... };
        if let Some(data) = handler.generate(effect_def, &ctx) {
            queue.push(PendingEffect { ... });
        }
    }
}
```

**规则**：
- 通过 `EffectHandlerRegistry` trait 分发，新增效果类型无需修改 generate
- 地形防御加成（`defense_bonus`）在 Generate 阶段传入
- 技能标签（`skill_data.tags`）作为 `source_tags` 传递给后续修饰

### 3.4 Trait 触发

Generate 阶段结束后，触发攻击者的 **OnAttack** Trait：
- 遍历 `TraitCollection`，匹配 `TraitTrigger::OnAttack`
- 将 `ApplyBuff` 效果推入 EffectQueue
- `GrantTag` 和 `ModifyAttribute` 是 Passive 效果，不在触发器中处理

---

## 4. Modify 阶段详解

### 4.1 修饰规则应用

```rust
for effect in &mut queue.pending {
    match &mut effect.data {
        PendingEffectData::Damage { amount, base_amount, modifiers, .. } => {
            if base_amount.is_none() { *base_amount = Some(*amount); }
            let (new_amount, entries) = rules.apply_damage_modifiers_with_breakdown(...);
            *amount = new_amount;
            *modifiers = entries;
        }
        PendingEffectData::Heal { amount, base_amount } => {
            if base_amount.is_none() { *base_amount = Some(*amount); }
            *amount = rules.apply_heal_modifiers(...);
        }
        _ => {} // ApplyBuff / Cleanse 不修饰
    }
}
```

**规则**：
- `base_amount` 在首次 modify 时记录，后续不覆盖
- 伤害修饰记录 `ModifierEntry` 列表，支持伤害明细展示
- ApplyBuff 和 Cleanse 不参与修饰

---

## 5. Execute 阶段详解

### 5.1 伤害执行

```
1. 构建 DamageBreakdown（base_amount → modified_amount → actual_damage）
2. 扣血：new_hp = max(0, hp - amount)
3. 发送 DamageApplied Message
4. 死亡判定：new_hp <= 0 时插入 Dead 组件 + 发送 CharacterDied Message
```

### 5.2 治疗执行

```
1. 回血：new_hp = min(max_hp, hp + amount)
2. 发送 HealApplied Message
```

**规则**：治疗不超过 MaxHp

### 5.3 Buff 执行

```
1. 从 BuffRegistry 查找 BuffData
2. 调用 apply_buff() 施加到目标
3. 未知 buff_id 静默跳过
```

### 5.4 Cleanse 执行

```
1. 调用 remove_all_debuffs() 移除所有减益 Buff
```

---

## 6. CombatIntent — 战斗意图

```rust
#[derive(Resource, Default)]
pub struct CombatIntent {
    pub source_entity: Option<Entity>,  // 攻击者实体
    pub target_coord: Option<IVec2>,    // 目标坐标
    pub skill_id: Option<String>,       // 技能 ID
}
```

**规则**：
- 玩家通过 UI 交互设置 CombatIntent
- AI 通过决策系统设置 CombatIntent
- Generate 阶段读取 CombatIntent 确定攻击者和目标

### 6.1 PrevPosition — 移动前位置

```rust
#[derive(Resource, Default)]
pub struct PrevPosition {
    pub coord: Option<IVec2>,
}
```

用于取消操作时回退到移动前位置。

---

## 7. 行动路由

### 7.1 execute_action_on_enter

攻击/技能行动完成后的统一路由：

1. 清除范围标记和高亮
2. 晕眩单位：标记已行动，移除选中
3. 设置技能冷却
4. 标记已行动，移除选中
5. 调用 `route_after_action()` 前进到下一个单位

### 7.2 wait_action_on_enter

待机行动完成后的统一路由：

1. 清除范围标记和高亮
2. 标记已行动，移除选中
3. 调用 `route_after_action()` 前进到下一个单位

### 7.3 route_after_action

从 TurnOrder 队列前进到下一个存活单位：

```
loop {
    match turn_order.advance() {
        Some(next_entity) => {
            if !is_alive(attrs) { continue; }  // 跳过已死亡单位
            更新 current_faction
            如果是 AI，重置计时器
            切换到 SelectUnit 阶段
            return;
        }
        None => {
            切换到 TurnEnd 阶段  // 队列耗尽
            return;
        }
    }
}
```

**规则**：
- 通过 HP 判断存活，不依赖 Dead 组件（Dead 是 deferred command）
- 队列耗尽时自动进入 TurnEnd

---

## 8. Trait 触发器

### 8.1 触发时机

| 触发器 | 触发位置 | 说明 |
|--------|----------|------|
| `OnAttack` | Generate 阶段末尾 | 攻击者攻击时触发 |
| `OnHit` | Execute 阶段（被攻击时） | 目标被攻击时触发 |
| `OnKill` | Execute 阶段（击杀时） | 攻击者击杀时触发 |

### 8.2 触发规则

- 仅处理 `ApplyBuff` 效果，将 Buff 推入 EffectQueue
- `GrantTag` 和 `ModifyAttribute` 是 Passive 效果，不在触发器中处理
- 多个同类型 Trait 全部触发
- 触发目标是攻击目标（OnAttack/OnKill）或攻击者（OnHit）

---

## 9. 战斗消息（Message）

### 9.1 消息类型

| 消息 | 发送时机 | 包含信息 |
|------|----------|----------|
| `DamageApplied` | 伤害执行后 | 攻击者/目标/伤害量/技能标记/地形/坐标/伤害分解 |
| `HealApplied` | 治疗执行后 | 目标/治疗量 |
| `CharacterDied` | HP ≤ 0 时 | 死亡实体/名称/阵营 |
| `StunApplied` | 晕眩施加时 | 目标/名称 |
| `DotApplied` | DoT 伤害时 | 目标/伤害量/坐标 |
| `HotApplied` | HoT 治疗时 | 目标/治疗量 |

### 9.2 CharacterDied 响应

`on_character_died` 系统响应死亡消息：

1. 从 TurnOrder 队列移除死亡实体
2. 修正 `current_index`（被移除实体在当前索引之前时减 1）
3. Despawn 死亡实体

### 9.3 消息注册

Bevy 0.18 要求在使用 `MessageReader`/`MessageWriter` 前注册消息类型：

```rust
app.add_message::<CharacterDied>()
    .add_message::<DamageApplied>()
    .add_message::<HealApplied>()
    // ...
```

---

## 10. DamageBreakdown — 伤害分解

```rust
pub struct DamageBreakdown {
    pub base_amount: i32,       // 原始效果值（Generate 阶段）
    pub modified_amount: i32,   // 修饰后效果值（Modify 阶段）
    pub modifiers: Vec<ModifierEntry>,  // 修饰步骤列表
    pub actual_damage: i32,     // 实际扣血（Execute 阶段）
}
```

**规则**：
- 匹配实际效果管线三步：Generate → Modify → Execute
- `base_amount` 在 Modify 阶段首次记录
- `modifiers` 记录每步修饰的 before/after/rule_name
- `actual_damage` 目前等于 `modified_amount`（未来可能有护盾等减免）

---

## 11. 战斗记录系统

### 11.1 BattleRecord — 战斗记录资源

```rust
#[derive(Resource, Reflect, Default)]
pub struct BattleRecord {
    pub entries: Vec<BattleEntry>,
    pub turn_number: u32,
}
```

### 11.2 BattleEntry — 记录条目

| 变体 | 记录内容 |
|------|----------|
| `TurnStarted { turn }` | 回合开始 |
| `TurnEnded { turn }` | 回合结束 |
| `DamageApplied { ... }` | 伤害（含 breakdown） |
| `HealApplied { ... }` | 治疗 |
| `DotApplied { ... }` | DoT 伤害 |
| `HotApplied { ... }` | HoT 治疗 |
| `StunApplied { ... }` | 晕眩 |
| `CharacterDied { ... }` | 角色死亡 |

### 11.3 查询接口

| 方法 | 说明 |
|------|------|
| `entries_for(entity)` | 获取指定实体的全部记录 |
| `entries_for_turn(turn)` | 获取指定回合的全部记录 |
| `recent(n)` | 获取最近 N 条记录 |
| `stats_for(entity)` | 计算指定实体的战斗统计 |
| `clear()` | 清空记录 |

### 11.4 EntityBattleStats — 实体战斗统计

```rust
pub struct EntityBattleStats {
    pub damage_dealt: i32,    // 造成总伤害
    pub damage_taken: i32,    // 承受总伤害
    pub heal_received: i32,   // 总治疗量
    pub kills: i32,           // 击杀数
}
```

**击杀数计算**：查找致死伤害的攻击者（CharacterDied 前最后一条 DamageApplied 的 attacker）

### 11.5 录制系统

8 个录制系统通过 `MessageReader` 监听战斗消息，写入 `BattleRecord`：

- `record_turn_started` / `record_turn_ended`
- `record_damage` / `record_heal`
- `record_dot` / `record_hot`
- `record_stun` / `record_character_died`

---

## 12. 战斗日志

### 12.1 CombatLog — 日志资源

```rust
pub struct CombatLog {
    pub entries: VecDeque<Vec<LogSegment>>,  // 最大 50 条
}
```

### 12.2 LogSegment — 日志片段

```rust
pub struct LogSegment {
    pub text: String,
    pub color: Color,
}
```

**规则**：
- 每条日志由多个片段组成，支持多色文本
- 超过 50 条自动截断最旧记录

### 12.3 日志颜色

| 常量 | 颜色 | 用途 |
|------|------|------|
| `NORMAL` | 灰色 | 普通文本 |
| `DAMAGE` | 红色 | 伤害 |
| `HEAL` | 绿色 | 治疗 |
| `KILL` | 粉色 | 击杀 |
| `PLAYER` | 蓝色 | 玩家名 |
| `ENEMY` | 橙色 | 敌方名 |
| `TURN` | 黄色 | 回合标记 |
| `TERRAIN` | 浅绿 | 地形 |

### 12.4 日志面板功能

- 折叠/展开切换（折叠时只显示最新一条）
- 拖拽调整面板大小
- 多色文本横向排列

---

## 13. 距离计算

```rust
pub fn manhattan_distance(a: IVec2, b: IVec2) -> u32
```

使用曼哈顿距离计算格子间距离，用于攻击范围判定。

---

## 14. 关键约束

1. **三步管线严格顺序**：Generate → Modify → Execute，不可跳步或乱序
2. **Logic / Presentation 分离**：Execute 只发送 Message，不调用 UI/VFX
3. **死亡判定在 Execute**：HP ≤ 0 时插入 Dead + 发送 CharacterDied
4. **Dead Hook 保证固有行为**：添加 Dead 时自动标记已行动 + 移除选中
5. **存活判断用 HP**：`route_after_action` 通过 HP 判断存活，不依赖 Dead 组件
6. **伤害最低为 1**：ModifierRule 保证 `result.max(1.0)`
7. **治疗不超过 MaxHp**：`min(max_hp, hp + amount)`
8. **Message 必须注册**：使用前必须 `add_message::<T>()`
9. **EffectQueue 每轮清空**：Execute 使用 `drain(..)` 消费所有待处理效果
10. **Trait 触发仅 ApplyBuff**：OnAttack/OnHit/OnKill 只处理 ApplyBuff 效果
