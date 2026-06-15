# Pipeline + Replay 领域 — 铃兰之剑数据提取

> 领域：Pipeline + Replay | 来源：78铃兰.md §八、补充9、补充12 | 数据层：Runtime + Persistence

---

## 一、Pipeline 数据实体

### 1.1 回合执行管线（§八）

```
回合开始
  ├─ 回合开始阶段
  │   ├─ 回合开始事件触发
  │   ├─ Buff回合开始Tick
  │   └─ AP/CP回复
  ├─ 行动阶段
  │   ├─ 按速度排序行动
  │   ├─ 每个单位：
  │   │   ├─ 选择行动
  │   │   ├─ 执行行动
  │   │   └─ 行动结束事件
  │   └─ 直到所有单位行动完毕
  └─ 回合结束阶段
      ├─ 回合结束事件触发
      ├─ Buff回合结束Tick
      ├─ 冷却-1
      └─ 检查胜负条件
```

### 1.2 技能执行管线（7步）

```
1. 前置校验 → 消耗检查 + 冷却检查 + 标签检查
2. 目标选择 → 按Targeting规则确定目标
3. 消耗扣除 → 扣AP/CP
4. 效果执行 → 按Effect列表顺序执行
5. 被动触发 → 触发相关Trigger
6. 状态更新 → 更新UI/记录战斗日志
7. 冷却设置 → 设置技能冷却
```

---

## 二、行动调度规则（补充9）

### 2.1 速度排序

| 字段 | 类型 | 说明 | 来源 |
|------|------|------|------|
| `speed` | f32 | 速度属性 | 补充9 |
| `faction_order` | Enum | 速度相同时的阵营排序 | 补充9 |
| `position_order` | u32 | 速度+阵营相同时的站位排序 | 补充9 |

排序规则：速度从高到低 → 速度相同按阵营固定排序 → 再按站位固定排序

### 2.2 插队规则

| 操作 | 效果 | 来源 |
|------|------|------|
| 立即行动 | 插入当前行动位的下一位 | 补充9 |
| 延迟行动 | 移到当前回合队尾 | 补充9 |
| 再行动 | 插入当前回合队尾 | 补充9 |

### 2.3 回合边界

| 规则 | 说明 | 来源 |
|------|------|------|
| 回合结束 | 所有单位行动完毕 | 补充9 |
| 召唤物行动 | 当回合不行动，下回合按速度插入排序 | 补充9 |
| 再行动 | 插入队尾，不是立刻再动 | 补充9 |

---

## 三、Replay 数据实体

### 3.1 ReplayEvent（Persistence层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `turn` | u32 | - | 回合号 | §八 |
| `action_index` | u32 | - | 行动序号 | §八 |
| `actor` | EntityId | - | 行动实体 | §八 |
| `action_type` | ActionType | - | 行动类型 | §八 |
| `target` | Option<EntityId> | - | 目标实体 | §八 |
| `ability_id` | Option<AbilityId> | - | 使用的技能 | §八 |
| `seed` | u64 | - | 随机数种子快照 | 补充12 |
| `result_snapshot` | ResultSnapshot | - | 结果快照 | §八 |

### 3.2 ActionType 枚举

| 类型 | 说明 | 来源 |
|------|------|------|
| `Move` | 移动 | §八 |
| `UseAbility` | 使用技能 | §八 |
| `Wait` | 等待 | §八 |
| `EndTurn` | 结束回合 | §八 |

---

## 四、Replay 确定性约束

### 4.1 确定性保证规则

| 规则 | 说明 | 来源 |
|------|------|------|
| 固定管线 | 技能执行7步管线严格有序 | §八 |
| 确定性排序 | 速度→阵营→站位，无随机 | 补充9 |
| 统一取整 | 所有百分比计算完成后统一取整 | 补充11 |
| 数值边界 | 属性下限/概率上限锁死 | 补充11 |
| 触发链限制 | 链深度=0，防止无限触发 | §七.3 |
| AI共用规则 | AI决策使用同一套执行管线 | 补充12 |

### 4.2 禁止的非确定性因素

| 禁止项 | 原因 | 来源 |
|--------|------|------|
| 当前时间 | Replay必须可重现 | Data Law 010 |
| 系统随机 | 必须使用确定性RNG | Data Law 010 |
| 外部状态 | 不可依赖外部输入 | Data Law 010 |
| 浮点精度差异 | 统一取整消除 | 补充11 |

### 4.3 AI决策对齐（补充12）

| 规则 | 说明 | 来源 |
|------|------|------|
| AI可读取 | 可见单位的位置/血量/属性/Buff/冷却 | 补充12 |
| AI不可读取 | 玩家操作/未暴露单位/随机种子 | 补充12 |
| 决策优先级 | 保命>完成目标>输出>辅助 | 补充12 |
| 仿真对齐 | AI模拟使用同一套核心规则 | 补充12 |

---

## 五、Schema草案

```yaml
# pipeline_config.ron
(
  turn_pipeline: (
    phases: [
      (name: "TurnStart", events: ["TurnStart"], tick_timing: TurnStart),
      (name: "ActionPhase", events: ["ActionStart", "ActionEnd"]),
      (name: "TurnEnd", events: ["TurnEnd"], tick_timing: TurnEnd),
    ],
  ),
  skill_pipeline: (
    steps: [
      (name: "Validate", checks: ["cost", "cooldown", "tags"]),
      (name: "TargetSelect", rule: "targeting"),
      (name: "CostDeduct", resources: ["ap", "cp"]),
      (name: "EffectExecute", order: "sequential"),
      (name: "TriggerFire", events: ["AfterSkillCast", "SkillHit"]),
      (name: "StateUpdate", outputs: ["ui", "battle_log"]),
      (name: "CooldownSet", value: "cooldown"),
    ],
  ),
  action_queue: (
    sort_by: ["speed DESC", "faction ASC", "position ASC"],
    insert_rules: [
      (action: "ImmediateAction", position: "after_current"),
      (action: "DelayAction", position: "end_of_turn"),
      (action: "ExtraAction", position: "end_of_turn"),
    ],
  ),
)

# replay_event.ron
(
  events: [
    (turn: 1, action_index: 0, actor: "unit_001",
     action_type: UseAbility, target: Some("unit_002"),
     ability_id: Some("fireball"), seed: 12345678,
     result_snapshot: (damage: 45, target_hp: 55)),
  ],
)
```

---

## 六、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| 全Core Domain | Pipeline → 全部 | Pipeline定义执行时序 |
| Replay | Pipeline → Replay | Replay依赖Pipeline确定性 |
| Trigger | Pipeline → Trigger | Pipeline定义触发时机 |
| Execution | Pipeline → Execution | Pipeline定义执行顺序 |

---

## 七、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 004 | ✅ | Pipeline中触发行为通过Trigger执行 |
| 005 | ✅ | Pipeline中业务执行通过Effect |
| 010 | ✅ | 固定管线+确定性排序+统一取整+数值边界+触发链限制，全链路保证Replay确定性 |