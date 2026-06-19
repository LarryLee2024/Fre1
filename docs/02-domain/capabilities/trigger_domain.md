---
id: 02-domain.trigger
title: Trigger（触发器）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - trigger
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Trigger | 技能激活的条件描述，定义"什么条件下可以激活什么技能" | 负责：条件→技能的映射关系定义，Trigger 的 LocalizationKey（name_key/desc_key）；不负责：条件的具体评估逻辑 |
| TriggerType | 触发类型枚举，定义触发条件的事件类别 | 负责：触发条件分类；不负责：触发后的行为 |
| TriggerCondition | 触发条件的具体定义，包含触发类型和附加参数 | 负责：触发条件的结构化描述；不负责：触发条件的评估执行 |
| TriggerContainer | 挂载在实体上的触发器容器，管理所有注册的触发器 | 负责：触发器的注册/移除/查询管理；不负责：触发器的实际触发评估 |
| TriggerContext | 触发事件的上下文，当触发条件满足时传递给被触发的 Ability | 负责：携带触发事件的关键数据（来源/目标/伤害量等）；不负责：触发条件的评估过程 |

### Trigger 类型枚举

```
TriggerType
 ├── OnTagAdded       标签被授予时触发（如进入战斗触发"先攻检定"）
 ├── OnTagRemoved     标签被移除时触发（如脱离战斗触发"脱战回血"）
 ├── OnDamaged        受到伤害时触发（如受伤触发"反击"）
 ├── OnHealed         受到治疗时触发（如治疗触发"额外恢复"）
 ├── OnAttack         发动攻击时触发（如攻击触发"连击"）
 ├── OnTurnStart      回合开始时触发（如开局触发"自动回血"）
 ├── OnTurnEnd        回合结束时触发（如回合结束触发"持续伤害"）
 ├── OnDeath          单位死亡时触发（如死亡触发"尸爆"）
 ├── OnMove           移动时触发（如移动触发"借机攻击"）
 ├── OnAbilityUsed    技能被使用时触发（如施法触发"法术反制机会"）
 ├── OnCustom         自定义触发事件（由 Domain 注册的触发类型）
 └── OnConditionMet   特定 Condition 满足时触发（如生命<30%触发"狂暴"）
```

### Trigger 与 Event 的职责分离

```
Trigger：
  ┌─ 职责：检测条件→激活技能
  ├─ 示例：OnDamaged → 激活"反击"技能
  ├─ 消费者：Ability 系统（触发后创建 AbilityInstance）
  ├─ 数据结构：轻量（条件 + 目标能力引用）
  └─ 触发范围：预先注册的特定条件

Event：
  ┌─ 职责：系统间结构化数据传递
  ├─ 示例：DamageDealt → 连锁闪电计算伤害
  ├─ 消费者：全系统（不仅 Ability）
  ├─ 数据结构：重量（GameplayContext 载荷）
  └─ 触发范围：全局发布-订阅

协作关系：
  Event 产生通知 → Trigger 消费通知（选择性过滤）→ 条件满足 → 激活 Ability
  例：Event (DamageDealt) → Trigger (OnDamaged) → 检查条件 → 激活反击技能
```

### 已对齐项目术语

- **Event**：Trigger 监听 Event 领域的 GameplayEvent，作为触发条件的输入源
- **Ability**：Trigger 的消费者，条件满足时通知 Ability 系统创建技能实例
- **Condition**：Trigger 中的 TriggerCondition 引用 Condition 领域做复杂条件评估
- **Tag**：OnTagAdded/OnTagRemoved 触发类型监听 Tag 领域的事件

---

## 2. Trigger 状态机

### 触发器的生命周期

```
Registered（已注册）
   │  [已订阅触发事件]
   ▼
Listening（监听中）
   │  [触发事件到达]
   │  [检查 TriggerCondition]
   ├──→ [条件满足]
   │      │
   │      ▼
   │  Fired（已触发——通知 Ability 系统）
   │      │
   │      ▼
   │  Executing（执行中——Ability 接管）
   │      │
   │      ▼ [能力执行完毕]
   ├──→ Listening（重新回到监听状态）
   │
   │  [Trigger 被移除]
   ▼
Removed（已移除）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Registered → Listening | 监听事件源确认就绪 | 订阅相关事件 |
| Listening → Fired | 触发事件到达且 TriggerCondition 满足 | 创建 TriggerContext，通知 Ability 系统 |
| Fired → Executing | Ability 系统确认接收 | 移交 Ability 领域管理 |
| Executing → Listening | Ability 执行完毕 | 回到监听状态（可再次触发） |
| Listening/Fired → Removed | 触发器被移除 | 取消事件订阅，清理 |
| 禁止 | 同一触发器在 Executing 状态下重新触发 | 默认不允许——除非触发器显式声明"允许叠加触发" |

---

## 3. 不变量（Invariants）

### 3.1 触发器必须有明确的目标技能
- **条件**：任何 Trigger 被注册时
- **不变量**：每个 Trigger 必须关联至少一个可激活的 AbilitySpec 或 AbilityDef
- **违反后果**：无目标技能的触发器注册失败

### 3.2 触发类型与事件源一致
- **条件**：Trigger 注册时
- **不变量**：TriggerType 必须对应一个已有的事件源（即对应 Event 领域的某个 GameplayEvent）
- **违反后果**：无事件源的触发类型不产生任何触发

### 3.3 单回合触发频率限制
- **条件**：Trigger 触发后
- **不变量**：默认情况下，同一触发器在同一回合内不得触发超过其声明次数上限（默认 1 次）
- **违反后果**：超过触发上限的触发被忽略

### 3.4 触发条件的独立性
- **条件**：TriggerCondition 评估时
- **不变量**：TriggerCondition 不得依赖其他 Trigger 的触发状态（禁止触发器间耦合）
- **违反后果**：触发器间耦合导致触发时序依赖，不利于组合和调试

### 3.5 自定义触发类型的可注册性
- **条件**：Domain 注册自定义触发类型时
- **不变量**：自定义触发类型必须继承自 OnCustom，并提供完整的事件订阅逻辑
- **违反后果**：未正确定义事件订阅的自定义触发类型不会触发

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：Trigger 直接执行技能逻辑（触发→激活流程中，Trigger 只做检测，不执行） — 理由：技能执行归 Ability 领域，Trigger 只做条件检测与通知
- 🟥 禁止：用 Trigger 替代 Event 做系统间通信 — 理由：职责分离——Trigger 解决"何时激活技能"，Event 解决"系统间数据传输"
- 🟥 禁止：同一 Trigger 同时关联多个 Activation 技能路径 — 理由：一个 Trigger 对应一个目标 Ability，多路径应注册多个 Trigger 实例
- 🟥 禁止：TriggerCondition 中包含随机判定 — 理由：触发条件的确定性影响游戏回放一致性，随机判定应在 Ability 的 Execution 阶段进行
- 🟥 禁止：触发器监听自身触发的技能所产生的事件 — 理由：防止触发器监听到自己触发的事件而无限循环（如"攻击→触发连击→连击是攻击→再次触发连击"）
- 🟥 禁止：TriggerDef 中直接存储用户可见文本的自然语言文本 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 触发器注册

- **输入**：触发器类型、TriggerCondition、目标 AbilitySpec/Def、可选触发次数上限
- **处理**：
  1. 校验目标能力是否存在且可激活（不变量 3.1）
  2. 校验触发类型的事件源是否已注册（不变量 3.2）
  3. 注册触发器到实体的 TriggerContainer
  4. 订阅对应的事件源
- **输出**：注册确认，Trigger 实例
- **失败处理**：目标能力不存在或事件源未注册时注册失败

### 5.2 触发器条件评估与触发

- **输入**：触发事件到达（GameplayEvent）、触发器实例
- **处理**：
  1. 检查触发器是否在当前回合已达到触发上限（不变量 3.3）— 已达上限则忽略
  2. 检查 TriggerType 是否与事件类型匹配
  3. 评估 TriggerCondition（必要时委托 Condition 领域执行）
  4. 条件满足 → 创建 TriggerContext（携带事件的关键数据）
  5. 发布 TriggerFired 事件通知 Ability 系统
- **输出**：TriggerFired 事件（携带 TriggerContext）
- **失败处理**：条件不满足时静默忽略，不产生任何事件

### 5.3 触发器移除

- **输入**：要移除的触发器标识
- **处理**：
  1. 取消事件源订阅
  2. 从 TriggerContainer 移除
- **输出**：移除确认
- **失败处理**：要移除的触发器不存在时忽略（幂等移除）

### 5.4 触发频率限制管理

- **输入**：每回合/每次战斗的触发计数
- **处理**：
  1. 每个触发器维护当前触发计数
  2. 回合开始时重置计数
  3. 每次触发前检查是否已达上限
  4. 超过上限的触发请求被拒绝（不返回错误，仅静默忽略）
- **输出**：触发允许/限制确认
- **失败处理**：触发上限设置不合理（上限=0）时注册时告警，触发器永远不触发

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| TriggerFired | 触发条件满足，Trigger 被激活时 | entity_id, trigger_id, trigger_type, context（TriggerContext）, target_ability_id | Ability（创建技能实例）、日志（LogCode: TRG001） |
| TriggerRegistered | 触发器注册到实体时 | entity_id, trigger_id, trigger_type, target_ability_id | 日志（LogCode: TRG002）、调试工具 |
| TriggerRemoved | 触发器从实体移除时 | entity_id, trigger_id, reason | Ability（清理关联）、日志（LogCode: TRG003） |
| TriggerSuppressed | 触发器因频率限制被抑制时 | entity_id, trigger_id, current_count, max_count | 日志（LogCode: TRG004）、平衡分析工具 |

### 事件处理流程——从触发到激活的完整链路

```
[触发条件满足]
    │
    ├──→ Trigger 领域：发布 TriggerFired 事件
    │
    ▼
[Ability 系统接收到 TriggerFired]
    │
    ├──→ 检查目标 Ability 是否可激活（Condition 检查）
    ├──→ 检查消耗是否足够（ResourceCheck）
    ├──→ 消耗资源
    ├──→ 创建 AbilityInstance
    ├──→ 执行技能逻辑
    │
    ▼
[技能执行完毕]
    │
    ├──→ Trigger 标记为"可再次触发"
    └──→ 如果技能产生新事件（如 DamageDealt）→ 可能触发其他 Trigger
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Trigger 能力领域位于 `core/capabilities/trigger/`，foundation/ 定义 trigger_type.rs 和 trigger_condition.rs，mechanism/ 定义 components.rs（TriggerContainer）和 trigger_eval_system.rs，符合 C1→C2 分层
- ✅ 术语一致：TriggerType、TriggerCondition、TriggerContainer 与架构文档第六节完全一致
- ✅ 职责明确：Trigger 只做"检测→通知"，不执行技能逻辑（Ability 的职责）、不做条件评估（Condition 的职责，TriggerCondition 可委托 Condition 领域）
- ✅ 与 Event 分离：架构文档第六节中 Trigger vs Event 的详细区分得到严格遵守——Trigger 解决"何时激活技能"、Event 解决"系统间通信"
- ✅ 频率控制：架构文档中反击、连锁等依赖 Trigger 的场景，通过触发频率上限避免无限激活
- ✅ LocalizationKey：Trigger 使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖触发器注册、条件评估、触发、频率限制等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] Trigger 与 Event 的职责分离已明确定义
- [x] 每个操作有完整的流程定义（注册、评估触发、移除、频率限制）
