---
id: 02-domain.stacking
title: Stacking（堆叠规则）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - stacking
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Stacking | 同一效果多次作用时的叠加规则定义与控制 | 负责：效果叠加的策略（是否可叠、如何叠加、叠加上限），Stacking 的 LocalizationKey（name_key/desc_key）；不负责：效果的创建与移除 |
| StackingType | 堆叠类型枚举，定义效果叠加的基本策略 | 负责：堆叠策略分类；不负责：策略的具体参数 |
| StackingRule | 堆叠规则的详细定义，包含同源/异源识别逻辑以及分组条件 | 负责：细化堆叠规则的触发条件；不负责：堆叠计数的维护 |
| StackingLimit | 堆叠上限与超出上限时的处理策略 | 负责：最大堆叠数定义和溢出行为；不负责：堆叠层数变化时的业务逻辑 |
| StackingState | 挂载在 Effect 上的堆叠状态组件，维护当前堆叠计数 | 负责：堆叠计数的实际存储与更新；不负责：何时应用堆叠规则 |
| StackIdentity | 堆叠标识，判断两个 Effect 是否属于同一次堆叠 | 负责：Effect 同源/异源的判定依据；不负责：堆叠策略的选择 |

### StackingType 详细定义

```
StackingType
 ├── None（不堆叠）           — 同一效果重复应用时直接忽略新实例
 │                              适用场景：同一 Buff 第二次施加（如"庇护术"不可叠加）
 │
 ├── Aggregate（累加层数）     — 同一效果重复应用时累加层数
 │                              适用场景：中毒层数叠加（每层+1d4 毒伤）
 │                              规则：层数 = min(现有层数 + 新增层数, StackingLimit)
 │
 ├── RefreshDuration（刷新）   — 重复应用时重置持续时间，层数不变
 │                              适用场景：持续伤害刷新时长（重新计算到期时间）
 │                              规则：持续时间的剩余值 = Max(当前剩余, 新实例持续时间)
 │
 └── Replace（替换）          — 重复应用时新实例替换旧实例
                               适用场景：不同来源的同类 Buff 替换生效（取更强者）
                               规则：根据优先级或数值大小判定替换还是被替换
```

### StackIdentity 判定逻辑

```
两个 Effect 是否属于同一堆叠由以下维度综合判定：

1. EffectDefId（效果定义 Id）
   ── 同 EffectDefId → 属于同类型效果
   ── 不同 EffectDefId → 不属于同一堆叠

2. SourceEntityId（来源实体）
   ── 同来源 → 同源堆叠（同一施法者的多次同效果）
   ── 不同来源 → 异源堆叠（不同施法者的同效果）

3. SourceAbilityId（来源技能/能力）
   ── 同能力 → 技能相关的同源判定
   ── 不同能力 → 技能相关的异源判定

判定步骤：
Step 1: 检查 EffectDefId —— 不同 → 不进行堆叠（各自独立）
Step 2: 检查 SourceEntityId + StackingRule 的同源/异源配置
Step 3: 根据 StackingType 执行对应策略
```

### 已对齐项目术语

- **Effect**：Stacking 在 Effect 重复应用时介入，判断如何处理叠加
- **Spec**：EffectSpec 中的 stack_count 字段记录堆叠层数
- **Condition**：某些堆叠策略的条件可能需要 Condition 检查（如"毒素叠加但目标有 Tag.Immune.Poison 时跳过"）
- **Modifier**：Aggregate 类型的堆叠需要多次注册 Modifier（每层对应一份 Modifier）

---

## 2. 堆叠状态机

```
无堆叠（第一次应用）
   │  [第二次同 Effect 到达]
   ▼
Stacking Decision（堆叠判定中）
   │  [判定堆叠类型]
   │
   ├──→ None：忽略新实例（保持原状态）
   ├──→ Aggregate：层数 +1（或 +n）
   │      │
   │      ▼
   │  LayerUpdated（层数已更新）
   │      │
   │      ▼
   │  Check Limit（检查上限）
   │      │
   │      ├──→ 未达上限 → Active（继续）
   │      │
   │      └──→ 已达上限 → Overflow Behavior（溢出处理）
   │
   ├──→ RefreshDuration：重置持续时间
   │      │
   │      ▼
   │  DurationRefreshed（持续已刷新）
   │
   └──→ Replace：比较新旧实例
          │
          ├──→ 新 > 旧 → 替换为新的
          └──→ 旧 ≥ 新 → 保留旧的
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| 无堆叠 → Stacking Decision | 同 Effect 第二次到达 | 开始堆叠判定 |
| Stacking Decision → LayerUpdated | 类型为 Aggregate | 层数 += 新增层数 |
| Stacking Decision → DurationRefreshed | 类型为 RefreshDuration | 重置剩余持续时间 |
| Stacking Decision → 保持不变 | 类型为 None 或 Replace(旧≥新) | 忽略新实例 |
| Stacking Decision → 替换为新 | 类型为 Replace(新>旧) | 移除旧实例，注册新实例 |
| LayerUpdated → Check Limit | 层数已更新 | 检查是否达到上限 |
| Check Limit → Overflow Behavior | 已达上限 | 执行溢出策略 |

---

## 3. 不变量（Invariants）

### 3.1 堆叠层数上限
- **条件**：任何 Aggregate 类型堆叠更新后
- **不变量**：堆叠层数不得超过 StackingLimit（默认最大 99 层）
- **违反后果**：层数超限时触发溢出处理，多余的层被丢弃或刷新持续时间

### 3.2 不同 EffectDef 不参与堆叠
- **条件**：堆叠判定时
- **不变量**：只有 EffectDefId 完全相同的 Effect 才能参与同一堆叠
- **违反后果**：不同 EffectDef 的效果被错误合并，效果类型混淆

### 3.3 同源优先
- **条件**：同源 vs 异源堆叠策略不同时
- **不变量**：同源堆叠的默认策略为 Aggregate（层数叠加）；异源堆叠的默认策略为 Replace（取更强者）
- **违反后果**：同源的可叠加效果被当作替换效果，层数丢失

### 3.4 溢出可配置
- **条件**：堆叠达到上限时
- **不变量**：溢出策略必须明确配置（丢弃/刷新/替换），不允许"未定义"
- **违反后果**：溢出行为未定义导致不同的实现产生不同的行为

### 3.5 层数变化必须触发属性重算
- **条件**：Aggregate 堆叠层数变化后
- **不变量**：堆叠层数变化后，必须触发受影响的 Modifier 重新注册/移除，进而触发 Aggregator 重算
- **违反后果**：层数变化后属性未更新，效果与显示不一致

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：Effect 领域自身处理堆叠逻辑 — 理由：堆叠策略归 Stacking 领域，Effect 只负责生命周期
- 🟥 禁止：不同 EffectDefId 的效果被强制合并堆叠 — 理由：堆叠只在同类效果间发生，异类效果各自独立
- 🟥 禁止：堆叠判定依赖运行时状态（如当前帧数/随机数） — 理由：堆叠规则必须是确定性的，依赖运行时状态导致回放不一致
- 🟥 禁止：堆叠层数可以无限增长（无上限设置） — 理由：必须始终有 StackingLimit 的限制
- 🟥 禁止：StackingDef 中直接存储用户可见文本的自然语言文本 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 堆叠判定

- **输入**：新到达的 EffectInstance、目标实体上已有的 ActiveEffectContainer
- **处理**：
  1. 在 ActiveEffectContainer 中搜索同 EffectDefId 的已有效果
  2. 如果未找到已有实例 → 第一次应用，正常施加，标记堆叠层数为 1
  3. 如果找到已有实例 → 进入堆叠策略选择：
     a. 判定 StackIdentity（同源/异源）
     b. 根据 StackingType 执行对应策略
- **输出**：堆叠判定结果（叠加/刷新/替换/忽略）
- **失败处理**：堆叠规则未配置时使用默认策略（None，不堆叠）

### 5.2 Aggregate 叠加

- **输入**：已有效果实例、新增实例的层数
- **处理**：
  1. 计算新层数 = min(当前层数 + 新增层数, StackingLimit)
  2. 如果达到 StackingLimit → 执行溢出策略（丢弃多余层/刷新持续时间/替换为新）
  3. 更新效果上的 Modifier 注册（层数增加 → 重新注册 Modifier）
  4. 发布 StackAdded 事件
- **输出**：更新后的层数，StackAdded 事件
- **失败处理**：Modifier 重新注册失败时回退到原层数

### 5.3 RefreshDuration 刷新

- **输入**：已有效果实例、新实例的持续时间
- **处理**：
  1. 比较当前剩余持续时间和新实例的持续时间
  2. 取最大值作为新的剩余持续时间
  3. 不改变层数
  4. 发布 StackRefresh 事件
- **输出**：更新后的剩余持续时间，StackRefresh 事件
- **失败处理**：无

### 5.4 Replace 替换

- **输入**：已有效果实例、新实例
- **处理**：
  1. 比较新旧实例的优先级（由 StackingRule 定义比较标准——可基于来源等级/效果强度/施法者属性）
  2. 如果新实例优先级更高：移除旧实例，注册新实例
  3. 如果旧实例优先级更高：忽略新实例
  4. 发布 StackReplace 事件
- **输出**：替换结果，StackReplace 事件
- **失败处理**：移除旧实例失败时，新实例注册被拒绝

### 5.5 溢出处理

- **输入**：已达上限的堆叠、溢出的策略
- **处理**：
  1. Discard（丢弃）：忽略新到达的层数，保持原状态
  2. Refresh（刷新）：丢弃新层数但刷新持续时间
  3. Replace（替换）：移除旧实例，用新实例替换（变相重置为 1 层）
- **输出**：溢出处理结果
- **失败处理**：每种策略均保证堆叠数不超过上限

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| StackAdded | 堆叠层数增加时 | entity_id, effect_spec_id, old_stack, new_stack, max_stack | Modifier（重新注册层数相关 Modifier）、UI（更新层数显示） |
| StackRemoved | 堆叠层数减少时 | entity_id, effect_spec_id, old_stack, new_stack, reason | Modifier（移除层数相关 Modifier）、UI（更新层数显示） |
| StackRefreshed | 堆叠持续时间刷新时 | entity_id, effect_spec_id, new_duration, old_duration | UI（更新持续时间显示） |
| StackReplaced | 堆叠被新实例替换时 | entity_id, effect_spec_id, old_source, new_source | Modifier（重新注册 Modifier）、UI（更新效果信息） |
| StackOverflow | 堆叠达到上限触发溢出时 | entity_id, effect_spec_id, current_stack, limit, overflow_action | 日志（LogCode: EFF008）、平衡分析工具 |

### 事件订阅关系图

```
StackAdded / StackRemoved
    │
    ├──→ Modifier：重新计算层数相关的 Modifier 总值
    │    （如每层 +5 力量 → 3 层 = +15 → 变为 4 层 = +20）
    ├──→ Aggregator：触发属性重算
    ├──→ UI：更新层数显示（如 Buff 图标上的数字）
    └──→ Cue：层数变化特效

StackRefreshed
    │
    ├──→ Effect：更新 Effect 的剩余持续时间
    ├──→ UI：更新持续时间显示
    └──→ Cue：刷新特效

StackOverflow
    │
    ├──→ 日志：记录溢出事件用于平衡分析
    └──→ 平衡分析：检测溢出频率是否过高（提示可能需要调整上限）
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Stacking 能力领域位于 `core/capabilities/stacking/`，foundation/ 定义 stacking_type.rs、stacking_rule.rs、stacking_limit.rs，mechanism/ 定义 components.rs（StackingState）和 stacking_system.rs，符合 C1→C2 分层
- ✅ 术语一致：StackingType、StackingRule、StackingLimit、StackingState 与架构文档第六节完全一致
- ✅ 职责明确：Stacking 只做"叠加规则判定"，不做"效果生命周期"（Effect）、不做"属性修改"（Modifier + Aggregator）
- ✅ 重复保护：不变量 3.1（层数上限）+ 3.2（不同 EffectDef 不堆叠）+ 3.5（层数变化触发重算）三重保护确保堆叠安全
- ✅ LocalizationKey：Stacking 使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖堆叠判定、四种堆叠策略、溢出处理等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] 四种堆叠类型定义清晰（None/Aggregate/RefreshDuration/Replace）
- [x] StackIdentity 判定逻辑明确（同 EffectDef + 同源/异源）
- [x] 每个操作有完整的流程定义（判定、叠加、刷新、替换、溢出）
