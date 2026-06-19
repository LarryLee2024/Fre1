---
id: 02-domain.effect
title: Effect（效果）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-18
tags:
  - domain
  - effect
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Effect | 对目标产生持续性或即时性影响的游戏效果，是技能作用结果的核心载体 | 负责：效果的全生命周期管理；不负责：效果的定义模板 |
| EffectDuration | 效果的持续时间定义，分为瞬时/持续/无限三种类型 | 负责：持续时间分类与计算；不负责：到期后的自动处理 |
| EffectPeriod | 效果的周期 Tick 定义，控制持续效果的触发间隔 | 负责：周期性效果的触发频率；不负责：每次 Tick 的具体效果 |
| EffectModifier | 效果携带的一组 Modifier，在效果应用时注册到目标属性 | 负责：效果对目标属性的影响描述；不负责：Modifier 的实际应用 |
| EffectTags | 效果授予/需要的标签集合，控制效果与目标的标签交互 | 负责：效果的标签需求与标签授予；不负责：标签的存在性检查 |
| ActiveEffectContainer | 目标实体上的活跃效果容器，管理所有当前在生效的效果 | 负责：效果的集合管理（添加/移除/查询）；不负责：单个效果的执行逻辑 |
| EffectLifecycle | 效果生命周期的四阶段描述：施加 → 持续 → 到期 → 移除 | 负责：生命周期阶段的定义与转换；不负责：各阶段的具体逻辑 |

### 持续时间分类

```
EffectDuration
 ├── Instant（瞬时）     → 立即执行，无持续阶段。如：一次性伤害、治疗
 ├── HasDuration（持续） → 有持续时间，持续阶段内可 Tick。如：中毒、灼烧、再生
 │    ├── Duration（总持续时间，单位：回合/秒）
 │    └── Period（Tick 间隔，如每回合一次、每秒一次）
 └── Infinite（无限）    → 无到期时间，需要显式移除。如：光环、永久性 Buff
```

### Effect 的典型分类

```
Effect（按作用方式分类）
 ├── Instant 类：
 │    ├── DamageEffect（即时伤害）
 │    ├── HealEffect（即时治疗）
 │    └── BuffEffect（即时施加 Buff Tag）
 ├── Duration 类：
 │    ├── DoT（Damage over Time，持续伤害）
 │    ├── HoT（Heal over Time，持续治疗）
 │    └── StatBuff（持续属性增减）
 └── Infinite 类：
      ├── AuraEffect（光环效果）
      └── PermanentBuff（永久性增益）
```

### 已对齐项目术语

- **Modifier**：EffectModifier 引用 Modifier 领域定义修改行为
- **Tag**：EffectTags 引用 Tag 领域的 TagId，用于效果的条件与标识
- **Condition**：Effect 应用前检查 Condition（应用条件/免疫条件）
- **Ability**：Ability 执行完毕后产生 Effect 列表
- **Spec**：EffectSpec 是 Effect 的配置实例（由 Spec 领域定义）
- **Stacking**：同一个 Effect 多次作用时由 Stacking 领域管理叠加规则

---

## 2. 效果生命周期状态机

### 四阶段生命周期

```
Applying（施加阶段——检查条件，初始化）
   │  [Condition 检查通过]
   ├──→ [Instant 效果] → 立即执行 → 直接进入 Removed
   │
   └──→ [Duration/Infinite 效果] → 初始化持续时间 → 进入 Active
           │
           ▼
      Active（持续阶段——周期性 Tick）
           │
           │  [Period 到达] → 执行 Tick（如 DoT 跳伤害）
           │  [Duration 耗尽 / 被驱散]
           ▼
      Expiring（到期阶段——执行移除前逻辑）
           │
           │  [Modifier 回退 / Tag 清理]
           ▼
      Removed（已移除）
```

### 状态转换规则

| 转换 | 触发条件 | 动作（Instant） | 动作（Duration） | 动作（Infinite） |
|------|---------|----------------|-----------------|-----------------|
| Applying → Active | 条件通过 | 不适用（跳过） | 注册 Modifier，添加 Tag，开始计时 | 注册 Modifier，添加 Tag |
| Applying → Removed | 条件通过执行完毕（Instant） | 执行效果，直接移除 | 不适用 | 不适用 |
| Active → Expiring | Duration 耗尽 | 不适用 | 开始移除流程 | 不适用 |
| Active → Removed | 被驱散/移除 | 不适用 | 可被驱散 | 需要显式移除 |
| Active → Tick | Period 到达 | 不适用 | 执行 Tick 效果 | 无 Period，不 Tick |
| Expiring → Removed | 移除前逻辑完成 | 不适用 | 回退 Modifier，清理 Tag | 回退 Modifier，清理 Tag |
| 禁止 | 同一 Effect 重复应用时跳过 Stacking 直接注册 | 重复应用必须经过 Stacking 规则处理 |

---

## 3. 不变量（Invariants）

### 3.1 Effect 必须有来源
- **条件**：任何 Effect 被创建时
- **不变量**：每个 Effect 必须有明确的来源（AbilityId / ItemId / TerrainId）和 GameplayContext
- **违反后果**：来源不明的 Effect 不允许注册到 ActiveEffectContainer

### 3.2 Effect 应用前必须检查 Condition
- **条件**：Effect 进入 Applying 阶段时
- **不变量**：必须先检查目标的免疫条件（Tag.Immune.X）和应用条件，通过后方可继续
- **违反后果**：免疫目标被施加了本应免疫的效果

### 3.3 持续时间一致性
- **条件**：Duration 类 Effect 的持续过程中
- **不变量**：Effect 的当前剩余持续时间不得为负值（到期时必须触发 Expiring）
- **违反后果**：持续时间"负值"的效果永远不会被移除

### 3.4 Effect 移除时 Modifier 必须回退
- **条件**：任何 Effect 进入 Expiring/Removed 阶段时
- **不变量**：Effect 注册的所有 Modifier 必须在移除时全部回退（从 ModifierContainer 移除）
- **违反后果**：Effect 移除后 Modifier 残留，属性值永久偏离应有值

### 3.5 同一 Effect 不得重复施加（默认）
- **条件**：Effect 正要对同一目标再次施加时
- **不变量**：默认情况下，同一 Effect（同来源、同 EffectDef）在目标上只能有一个实例。如需多层叠加，归 Stacking 领域处理
- **违反后果**：同一 Effect 重复叠加导致 Modifier 倍增

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：跳过 Condition 检查直接应用 Effect — 理由：免疫/限制检查是强制性的，跳过会导致免疫失效
- 🟥 禁止：Effect 直接修改目标属性值 — 理由：Effect 通过 Modifier 间接修改属性，直接修改 bypass Aggregator 管线
- 🟥 禁止：Duration 类 Effect 永不过期（除非声明为 Infinite 类型） — 理由：Duration 效果必须有明确的到期时间
- 🟥 禁止：Effect 移除后 Modifier 残留 — 理由：移除时必须完全回退所有 Modifier
- 🟥 禁止：Effect 在 Active 阶段被再次应用时绕过 Stacking 规则 — 理由：叠加规则归 Stacking 领域，Effect 领域不处理叠加决策

---

## 5. 流程定义

### 5.1 Effect 施加

- **输入**：目标实体、EffectSpec（含 EffectDef 引用、来源上下文、快照数据）
- **处理**：
  1. 检查目标是否已有同源 Effect（不变量 3.5）— 如有，触发 Stacking 规则处理
  2. 检查目标的免疫条件（Condition）— Tag.Immune.{EffectType} 是否存在
  3. 检查应用条件（Condition）— 目标当前状态是否允许此效果
  4. 条件通过后，执行 Effect 初始化：
     - Instant：立即执行效果，生成 Execution 结果，直接返回
     - Duration：注册 Modifier 到 ModifierContainer，添加 EffectTags，开始持续倒计时
     - Infinite：注册 Modifier，添加 EffectTags（永久，直到被移除）
  5. 注册 Effect 实例到目标的 ActiveEffectContainer
- **输出**：EffectApplied 事件
- **失败处理**：免疫时效果不被施加并发布 ImmunityTriggered 事件；条件不通过时效果推迟或取消

### 5.2 效果持续与 Tick

- **输入**：时间推进（帧更新/回合更新）、ActiveEffectContainer 中的 Duration 类 Effect
- **处理**：
  1. 对所有 Duration 类 Effect 更新倒计时
  2. 当 Period 到达时：
     a. 执行 Tick 效果（如 DoT 再次调用 Execution 计算伤害）
     b. 更新 Tick 计数器
     c. 发布 EffectTicked 事件
  3. 当 Duration 耗尽时，进入 Expiring 阶段
- **输出**：EffectTicked 事件（Period 到达时）
- **失败处理**：单次 Tick 失败不影响后续 Tick

### 5.3 Effect 到期与移除

- **输入**：到期通知（Duration 耗尽）或移除请求（驱散/来源死亡/技能取消）
- **处理**：
  1. 回退所有 Modifier（不变量 3.4）— 从 ModifierContainer 移除
  2. 清理 EffectTags — 从 GameTagContainer 移除
  3. 从 ActiveEffectContainer 中移除 Effect 实例
  4. 发布 EffectRemoved 事件
- **输出**：EffectRemoved 事件
- **失败处理**：Modifier 回退失败时标记 Effect 为"移除失败"状态，触发完整性校验

### 5.4 Effect 到期前保护（某些效果不应提前结束）

- **输入**：移除请求
- **处理**：
  1. 检查 Effect 是否有"不可驱散"标签
  2. 如果有不可驱散标签，移除请求被拒绝（除非来源死亡或强制解除）
- **输出**：移除允许/拒绝确认
- **失败处理**：不可驱散 Effect 被强制移除时记录警告

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| EffectApplied | 效果成功施加时 | entity_id, effect_spec_id, effect_type, duration_type, source_context | UI（显示 Buff 图标）、Modifier（注册修改器）、Cue（特效触发）、日志（LogCode: EFF001） |
| EffectRemoved | 效果移除时 | entity_id, effect_spec_id, reason（expired/dispelled/manual） | UI（移除 Buff 图标）、Modifier（回退修改器）、Cue（移除特效）、日志（LogCode: EFF002） |
| EffectTicked | 周期效果 Tick 时 | entity_id, effect_spec_id, tick_number, total_ticks, tick_result | Execution（结算 Tick 伤害/治疗）、日志（LogCode: EFF003）、Cue（Tick 特效） |
| EffectImmunityTriggered | 效果因免疫被阻止时 | entity_id, effect_spec_id, immune_tag | Cue（显示 IMMUNE 文字）、日志（LogCode: EFF004） |

### 事件订阅关系图

```
EffectApplied
    │
    ├──→ Modifier：注册 Effect 携带的 Modifier 到目标属性
    ├──→ Tag：添加 EffectTags 到目标标签容器
    ├──→ UI：显示 Buff/Debuff 图标
    ├──→ Cue：触发施加特效（如绿色闪光=中毒）
    └──→ Stacking：更新堆叠计数

EffectTicked
    │
    ├──→ Execution：执行 Tick 伤害/治疗计算
    ├──→ Cue：触发 Tick 特效（如毒跳数字）
    └──→ 日志：记录 Tick 数据

EffectRemoved
    │
    ├──→ Modifier：回退所有关联 Modifier
    ├──→ Tag：移除关联 EffectTags
    ├──→ UI：移除 Buff/Debuff 图标
    ├──→ Cue：触发移除特效
    └──→ Stacking：更新堆叠计数
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Effect 能力领域位于 `core/capabilities/effect/`，foundation/ 定义 effect_duration.rs、effect_period.rs、effect_modifiers.rs、effect_tags.rs，mechanism/ 定义 components.rs（ActiveEffectContainer）、effect_lifecycle.rs 和多个 systems/，符合 C1→C2 分层
- ✅ 术语一致：EffectDuration、EffectPeriod、EffectModifier、EffectTags、EffectLifecycle 与架构文档第六节完全一致
- ✅ 不造新系统：光环 = Targeting(Area) + Effect(Infinite)，DoT/HoT = Effect(Duration + Period)，统一复用 Effect 机制
- ✅ 职责明确：Effect 只做"生命周期管理"，不做"叠加规则"（Stacking）、不做"属性修改"（Modifier + Aggregator）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖效果施加、持续/Tick、到期移除、免疫检查等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] 四阶段生命周期定义清晰（Applying→Active→Expiring→Removed）
- [x] 每个操作有完整的流程定义（施加、持续/Tick、到期移除、保护）
