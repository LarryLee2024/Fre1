---
id: 02-domain.ability
title: Ability（技能逻辑）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - ability
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Ability | 可主动或被触发激活的游戏能力，是技能系统的执行核心 | 负责：技能的生命周期管理（激活→执行→完成→冷却）；不负责：技能定义模板 |
| AbilityState | 技能运行时状态枚举，描述技能当前所处的生命周期阶段 | 负责：状态定义与转换规则；不负责：状态转换的业务触发 |
| AbilityInstance | 技能激活后的运行时实例，携带激活时的完整上下文 | 负责：单次技能执行的运行态数据；不负责：技能的配置数据 |
| Cost | 技能激活的资源消耗描述，复用 Modifier 机制实现 | 负责：消耗的计算规则（不造 CostSystem）；不负责：消耗的检查 |
| Cooldown | 技能使用后的冷却规则，复用 Tag+Effect 机制实现 | 负责：冷却状态描述（不造 CooldownSystem）；不负责：冷却计时 |
| ActiveAbilityContainer | 实体持有的活跃技能容器，记录实体所有可激活和正在激活的技能 | 负责：技能的集合管理；不负责：单个技能的执行状态 |

### AbilityState 状态机

```
            ┌──────────────────────────────────────────────┐
            │                                              │
            ▼                                              │
Ready（就绪——可激活）                                       │
   │                                                       │
   │ [激活请求] 检查 Condition → 检查 Cost                 │
   ▼                                                       │
Casting（施法/前摇）——对应需要施法时间的技能                  │
   │                                                       │
   │ [施法完成 / 被打断]                                    │
   ▼                                                       │
Active（活跃/执行中——技能正作用于目标）                       │
   │                                                       │
   │ [Execution 执行完毕]                                   │
   ▼                                                       │
Cooldown（冷却中）                                           │
   │                                                       │
   │ [冷却时间到期 —— Effect(Duration) 到期]                │
   ▼                                                       │
Ready（回到就绪——可再次激活）                                 │
   │                                                       │
   │ [被移除]                                               │
   ▼                                                       │
Removed（已移除）                                           │
                                                            │
Blocked（被封锁——因沉默/眩晕等无法使用，独立于主流程的状态）
   │  [沉默/眩晕解除]                                        │
   └──→ 回到 Blocked 前的状态（Ready/Casting/Active）        │
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Ready → Casting | 激活请求 → Condition 通过 → Cost 足够 | 消耗资源，创建 AbilityInstance，进入施法阶段 |
| Ready → Active | 激活请求 → Condition 通过 → Cost 足够（无施法时间的技能直接进入） | 消耗资源，直接执行 |
| Casting → Active | 施法完成 | 技能效果开始执行 |
| Casting → Ready | 施法被打断/取消 | 回退资源消耗（如适用），回到就绪 |
| Active → Cooldown | Execution 执行完毕 | 设置冷却 Effect（Duration 类型） |
| Cooldown → Ready | 冷却 Effect 到期移除 | 清除冷却 Tag，回到就绪 |
| 任何状态 → Blocked | 沉默/眩晕/石化等控制效果施加 | 暂停当前执行（如为 Casting 则打断） |
| Blocked → 原状态 | 控制效果解除 | 恢复 |
| 任何状态 → Removed | 技能从实体移除 | 级联取消活跃 Instance，清理所有关联 |
| 禁止 | Casting/Active 状态下被同一技能再次激活 | 除非有特殊规则（如"瞬发"标签允许） |

### 已对齐项目术语

- **Condition**：技能激活前检查条件（定义在 Condition 领域）
- **Spec**：AbilitySpec 提供技能的等级/强化/冷却缩减等配置（定义在 Spec 领域）
- **Trigger**：Trigger 监听事件→触发技能激活（定义在 Trigger 领域）
- **Targeting**：技能激活后进行目标选择（定义在 Targeting 领域）
- **Execution**：技能的核心计算执行（定义在 Execution 领域）
- **Effect**：技能产生的持续性效果（定义在 Effect 领域）
- **Attribute**：技能的消耗资源引用（如法力值）

---

## 2. 核心设计原则——组合优于创建

| 常见需求 | 错误做法 | 正确做法（组合已有机制） |
|----------|----------|--------------------------|
| 法力消耗 | ManaSystem | Attribute(Mana) + Cost(复用 Modifier/Effect) |
| 冷却时间 | CooldownSystem | Tag(Cooldown.X) + Effect(Duration) |
| 怒气/体力/行动点 | 多套资源系统 | Attribute + ResourcePipeline |
| 免疫 | ImmunitySystem | Tag(Immune.X) + Condition(Check) |
| 光环 | AuraSystem | Targeting(Area) + Effect(Infinite) |
| DoT/HoT | DoTSystem | Effect(Duration + Period) |
| 暴击/闪避 | CritSystem | Execution(CustomCalc) + Modifier |
| 反击 | CounterAttackSystem | Trigger(OnAttacked) + Ability(Counter) |

---

## 3. 不变量（Invariants）

### 3.1 Condition 检查先于 Cost 消耗
- **条件**：任何技能激活流程中
- **不变量**：必须先完成 Condition 检查（含免疫检查），确认通过后再执行 Cost 消耗
- **违反后果**：条件不满足但资源已被消耗，资源无法回退

### 3.2 唯一激活路径
- **条件**：技能激活时
- **不变量**：所有技能必须通过 Ability 领域的激活流程进入执行，禁止绕过激活路径直接触发技能效果
- **违反后果**：绕过激活路径导致 Condition/Cost/Cooldown 检查被跳过

### 3.3 冷却与激活互斥
- **条件**：技能处于 Cooldown 状态时
- **不变量**：处于 Cooldown 状态的技能不得被激活
- **违反后果**：技能在冷却中被激活，循环全被打破

### 3.4 激活中的技能不得被级联取消（除非来源被移除）
- **条件**：AbilityInstance 处于 Active 状态时
- **不变量**：只有技能的来源实体被销毁或技能 Spec 被移除时，才可级联取消活跃技能
- **违反后果**：活跃技能被随意取消导致效果不一致

### 3.5 消耗的不可逆性（施法完成后）
- **条件**：技能进入 Active 状态后
- **不变量**：Active 状态下的已消耗资源不得回退（Casting 阶段被打断可以回退）
- **违反后果**：技能执行完毕但消耗被回退，产生无成本的技能使用

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：第三方系统直接设置技能的冷却状态 — 理由：冷却统一通过 Effect(Duration) + Tag(Cooldown) 管理，禁止手动操作冷却计时器
- 🟥 禁止：激活流程中跳过 Condition 检查 — 理由：激活条件的检查是强制性的，跳过会导致免疫/限制类效果失效
- 🟥 禁止：在 Ability 领域内部实现"法力消耗"或"冷却"的专用系统 — 理由：必须复用 Effect/Modifier/Attribute/Tag 组合，不造新系统
- 🟥 禁止：一个技能同时处于多个 AbilityInstance 运行状态（除非有瞬发/多重施法标签） — 理由：默认情况下一个技能同时只能有一个活跃实例

---

## 5. 流程定义

### 5.1 技能激活流程

- **输入**：激活请求（AbilitySpec Id、施法者实体、可选目标/位置）
- **处理**：
  1. 检查技能当前状态（不变量 3.3 — Cooldown 状态直接拒绝）
  2. 检查 Condition（不变量 3.1）
  3. 检查 Cost 是否足够
  4. 消耗资源
  5. 更新技能状态为 Casting（无施法时间的技能直接进入 Active）
  6. 创建 AbilityInstance
  7. 调用 Targeting 选择目标（如未预选）
  8. 进入 Active 状态，触发 Execution
- **输出**：AbilityInstance，AbilityActivated 事件
- **失败处理**：Condition 不通过或资源不足时激活失败，不消耗资源，不进入冷却

### 5.2 技能执行流程

- **输入**：AbilityInstance（Active 状态）、TargetData、GameplayContext
- **处理**：
  1. 将 AbilityInstance 委托给 Execution 领域执行计算
  2. Execution 产生 Effect 列表
  3. 逐个检查 Effect 的应用条件（Condition）
  4. 对通过条件的 Effect 调用 Effect 领域进行应用
  5. 等待所有 Effect 处理完毕
- **输出**：技能执行结果，AbilityCompleted 事件
- **失败处理**：Execution 阶段异常时标记技能执行失败，发布 AbilityFailed 事件

### 5.3 技能取消/打断

- **输入**：取消请求、取消原因（用户取消/被打断/来源死亡）
- **处理**：
  1. 如果处于 Casting 阶段：打断施法，回退已消耗资源（如适用），回到 Ready
  2. 如果处于 Active 阶段：立即终止当前 Execution，发布 AbilityCancelled 事件
  3. 不进入冷却
- **输出**：AbilityCancelled 事件
- **失败处理**：技能已完成执行时忽略取消请求

### 5.4 冷却管理

- **输入**：技能执行完毕通知
- **处理**：
  1. 计算冷却时长（基础冷却 × 冷却缩减倍率）
  2. 创建冷却 Effect（Duration）：
     - 添加 Tag(Cooldown.{SkillId}) 标记技能正在冷却
     - Effect(Duration) 到期时自动移除 Tag
  3. Condition 领域对 Tag(Cooldown.{SkillId}) 检查阻止再次激活
- **输出**：CooldownStarted 事件
- **失败处理**：冷却 Effect 创建失败时技能进入"卡死"状态（无法激活，无冷却倒计时），需手动修复

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| AbilityActivated | 技能成功激活时 | entity_id, ability_spec_id, instance_id, context | Trigger（屏蔽自身触发）、UI（技能冷却/施法条显示） |
| AbilityCompleted | 技能执行完毕时 | entity_id, ability_spec_id, instance_id, result | Trigger（释放频率限制位）、Progression（技能使用经验） |
| AbilityCancelled | 技能被取消/打断时 | entity_id, ability_spec_id, instance_id, reason | Trigger（释放频率限制位） |
| AbilityCooldownStarted | 冷却开始时 | entity_id, ability_spec_id, cooldown_duration | UI（显示冷却倒计时） |

### 事件订阅关系图

```
AbilityActivated
    │
    ├──→ Trigger：记录此技能已触发（防止自触发循环）
    ├──→ UI：显示施法条 / 技能图标变灰
    └──→ 日志：记录技能激活

AbilityCompleted
    │
    ├──→ Trigger：释放频率限制位（可再次触发）
    ├──→ Progression：记录技能使用次数（用于升级熟练度）
    ├──→ UI：隐藏施法条
    └──→ 日志：记录技能执行结果

AbilityCancelled
    │
    ├──→ Trigger：释放频率限制位
    ├──→ Effect：级联取消已应用的效果
    └──→ UI：取消动画/施法条消失

AbilityCooldownStarted
    │
    ├──→ UI：技能图标显示冷却转圈
    └──→ 日志：记录冷却开始
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Ability 能力领域位于 `core/capabilities/ability/`，foundation/ 定义 ability_state.rs、ability_instance.rs、cost.rs、cooldown.rs，mechanism/ 定义 components.rs、ability_task.rs 和多个 systems/，符合 C1→C2 分层
- ✅ 术语一致：AbilityState、AbilityInstance、Cost、Cooldown 与架构文档第六节完全一致
- ✅ 组合优于创建：Cost 复用 Effect，Cooldown 复用 Tag+Effect，未造新系统
- ✅ 职责明确：Ability 只做"生命周期编排"，不执行计算（Execution）、不选目标（Targeting）、不管理持续性效果（Effect）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖技能激活、执行、取消、冷却等全生命周期
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] 5 状态技能状态机定义清晰（Ready/Casting/Active/Cooldown/Blocked）
- [x] 每个操作有完整的流程定义（激活、执行、取消、冷却管理）
