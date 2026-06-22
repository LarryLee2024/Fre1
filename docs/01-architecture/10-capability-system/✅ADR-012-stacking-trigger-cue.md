---
id: 01-architecture.ADR-012
title: ADR-012 — Stacking / Trigger / Cue Separation
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-012: Stacking / Trigger / Cue 分离架构

## 状态

**Approved** — 依赖 ADR-010（Ability Pipeline）和 ADR-011（Modifier Pipeline），本架构决策正式生效。

## 背景

Stacking（堆叠）、Trigger（触发）、Cue（表现信号）是能力系统中三个经常被混淆的领域：

- **堆叠**是"同一个 Effect 可以叠加多少份"的问题（Data Law 008 — 堆叠行为归属 Stacking）
- **触发**是"什么条件下发生了什么事"的响应机制
- **Cue** 是"效果发生了，表现层该怎么知道"的信号通道（Data Law 009 — 表现必须经过 Cue）

三个领域必须分离到独立的 Feature 模块，禁止交叉污染。

## 引用的领域规则与数据架构

- `docs/02-domain/capabilities/stacking_domain.md` — 堆叠领域规则
- `docs/02-domain/capabilities/trigger_domain.md` — 触发领域规则
- `docs/02-domain/capabilities/cue_domain.md` — Cue 领域规则
- `docs/04-data/capabilities/stacking_schema.md` — Stacking Schema
- `docs/04-data/capabilities/trigger_schema.md` — Trigger Schema
- `docs/04-data/capabilities/cue_schema.md` — Cue Schema
- `.trae/rules/ECS规则.md` — Trigger 作为 Feature 内事件链载体
- `docs/04-data/README.md` — Data Law 008（堆叠归属 Stacking）、Data Law 009（表现必经 Cue）

## 决策

### 1. 三层责任分离

```
┌──────────────────────────────────────────────────────┐
│                    Effect                             │
│  执行后产生三个独立的输出：                              │
│                                                      │
│  1. Modifier (数值修改) → Stacking 检查 → ModifierSet │
│  2. Trigger (事件通知)  → Observer 链 → 连锁反应     │
│  3. Cue (表现信号)     → CueBus      → UI/VFX/SFX   │
└──────────────────────────────────────────────────────┘
```

**责任边界**：

| 领域 | 责任 | 不负责 |
|------|------|--------|
| **Stacking** | 决定"同一 Effect 能否叠加、叠加上限、刷新策略" | 不决定持续时间、不包含业务逻辑 |
| **Trigger** | "条件满足时触发响应"的机制编排 | 不处理堆叠决策、不直接播放效果 |
| **Cue** | Effect 完成的信号传达给表现层 | 不包含 VFX/SFX 实现、不含业务含义 |

### 2. Stacking 架构

#### 2.1 堆叠规则归属

所有与堆叠相关的配置集中在 Stacking 领域：

```rust
/// 堆叠规则 — 在 EffectDef 中引用，不在 Effect/Modifier 中定义
pub struct StackingRule {
    pub group: StackGroup,           // 堆叠分组（相同 group 才竞争堆叠槽）
    pub max_stacks: StackCount,      // 最大堆叠数
    pub refresh_behavior: RefreshBehavior, // 刷新时重置 Duration / 叠加层数/ 拒绝
    pub merge_source: MergePolicy,   // 同源合并 / 异源叠加
}
```

#### 2.2 堆叠分组策略

| 分组策略 | 示例 | 行为 |
|---------|------|------|
| `ByEffectId` | 同一个 EffectDef | 同 ID 不叠加，刷新 Duration |
| `BySourceType` | 所有"燃烧"类 Effect | 同 source_type 竞争槽位 |
| `ByTag` | Tag("movement_buff") | 相同 Tag 的 Effect 竞争 |
| `Unlimited` | 被动光环 | 不限制，始终添加 |
| `Exclusive` | 石化 / 冻结 | 互斥，新覆盖旧 |

#### 2.3 堆叠时机

堆叠检查发生在 **Effect 即将应用到 Entity 时**：

```
Effect Instance 即将创建
       │
       ▼
StackingRule.evaluate(existing_effects, new_effect)
       │
       ├── Allow → 正常创建 (可能替换旧层)
       ├── Reject → 丢弃新 Effect（显示 "Already active"）
       ├── Refresh → 重置现有 Effect 的 Duration
       └── Stack → 增加层数 (max_stacks 限制)
```

### 3. Trigger 架构

#### 3.1 Trigger 的定义

Trigger 是"在某事件发生时，检查一组条件，如果满足则触发响应"的声明式机制。

```rust
/// TriggerDef — 配置文件中定义，引用 EffectDef
pub struct TriggerDef {
    pub trigger_id: TriggerDefId,
    pub event_type: TriggerEventType,  // on_hit | on_death | on_turn_start | ...
    pub conditions: Vec<ConditionDef>,  // 触发条件
    pub effects: Vec<EffectDefId>,      // 触发后执行的 Effect
    pub priority: i32,                  // 执行优先级
    pub cooldown: Option<CooldownDef>,  // 触发冷却
}
```

#### 3.2 触发器执行流程

```
领域事件发生 (e.g., DamageDealt)
       │
       ▼
TriggerScanner System (每帧运行)
       ├── 扫描所有匹配 event_type 的 TriggerDef
       ├── 检查触发条件（Condition）
       ├── 按 priority 排序
       └── 触发冷却检查
       │
       ▼
commands.trigger(TriggerActivated)
       │
       ▼
on_trigger_activated (Observer)
       └── 执行 TriggerDef.effects (进入 Ability Pipeline)
```

#### 3.3 触发时机与规则冲突

| 冲突场景 | 解决策略 |
|---------|---------|
| 多个 Trigger 同一事件 | 按 priority 排序，同 priority 按注册顺序 |
| Trigger 触发 Trigger | 允许，但设最大递归深度 10 |
| Trigger 触发条件不满足 | 跳过，继续检查下一个 |
| 触发冷却未结束 | 跳过（冷却信息存储在 TriggerCooldown Component） |

### 4. Cue 架构

#### 4.1 Cue 的定义

Cue 是 Effect 完成时发出的"表现信号"，不包含表现实现细节。

```rust
/// CueDef — 配置文件中定义
pub struct CueDef {
    pub cue_id: CueDefId,
    pub cue_type: CueType,        // Vfx | Sfx | ScreenShake | TextPopup | ...
    pub parameters: CueParams,    // cue_type 相关的参数
    pub priority: CuePriority,    // 表现排队优先级
    pub duration: Option<Duration>, // 表现持续时间
}
```

#### 4.2 Cue 信号流

```
EffectResolved
       │
       ▼
CueSignal (Event / Trigger)
       │
       ├──→ CueQueue (Resource) → 按优先级排队
       │         │
       │         ▼
       │    CueDispatcher (System, PostUpdate)
       │         │
       │         ├──→ VFX System (播放粒子/闪屏)
       │         ├──→ SFX System (播放音效)
       │         ├──→ UI System (漂浮文字/HUD 更新)
       │         └──→ Camera System (震动/缩放)
       │
       └──→ 不对齐？Fallback: 表现层轮询 CueQueue
```

#### 4.3 Cue 合并与抑制

```rust
/// Cue 合并规则 — 防止同一帧多个相同 Cue 刷屏
pub enum CueMergeStrategy {
    /// 相同 cue_id 合并为一个（如连续闪避文本）
    MergeIdentical,
    /// 相同 cue_type 合并（如堆叠的漂浮文字）
    MergeByType,
    /// 全部保留（如多个不同音效）
    KeepAll,
    /// 抑制低优先级 Cue（高优先级战斗特效压制低优先级 UI 反馈）
    SuppressLowPriority { threshold: CuePriority },
}
```

## Module Design

```
src/core/capabilities/stacking/
  ├── components.rs      — StackGroup, StackCounter Component
  ├── systems.rs         — on_effect_applied (堆叠检查)
  └── resources.rs       — StackingRuleRegistry

src/core/capabilities/trigger/
  ├── components.rs      — TriggerCooldown Component
  ├── resources.rs       — TriggerDefRegistry
  └── systems.rs         — trigger_scanner, on_trigger_activated

src/core/capabilities/cue/
  ├── components.rs      — CueQueue Resource
  ├── systems.rs         — cue_dispatcher, cue_merger
  └── events.rs          — CueSignal
```

## Communication Design

| 通信 | 机制 | 说明 |
|------|------|------|
| Effect → Stacking | Observer (`on_effect_applied`) | Effect 应用时触发堆叠检查 |
| Domain Event → Trigger | TriggerScanner 轮询 | TriggerScanner 每帧检查 DomainEvent |
| Trigger → Effect | Ability Pipeline | 激活 TriggerDef.effects |
| Effect → Cue | `CueSignal` (Event) | Effect 完成时发送信号 |
| Cue → 表现层 | `CueQueue` Resource → PostUpdate System | Cue 按优先级排队分发 |

## 边界定义

### 允许
- `stacking/` 读取 Effect 的 Duration 信息
- `trigger/` 通过 EventReader 读取领域事件
- `cue/` 在 PostUpdate 中分发表现信号
- 表现层在 PostUpdate 中读取 CueQueue

### 🟥 禁止
- `stacking/` 包含任何业务逻辑（if 条件判断 Effect 类型）
- `trigger/` 直接播放 VFX/SFX（必须通过 Cue）
- `cue/` 包含 VFX/SFX 实现代码（仅发信号）
- Effect 直接调用 `play_sound()` 等表现函数
- Stacking 规则写在 EffectDef 字段中（必须引用 StackingRule）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| Effect 直接播放特效 | 违反 Data Law 009 |
| `max_stack` 字段散落在 Effect/Modifier 中 | 违反 Data Law 008 |
| Trigger 中嵌入硬编码的响应逻辑 | Trigger 应引用 EffectDef，不包含行为 |
| Cue 信号中包含业务数据修改 | Cue 是只读信号通道 |
| 触发链深度 > 10 | 递归安全防护 |

## Definition / Instance Design

- **Definition**: `StackingRule` (config), `TriggerDef` (Asset), `CueDef` (Asset)
- **Instance**: `StackCounter` (Component), `TriggerCooldown` (Component), `CueQueue` (Resource)
- **Spec**: 无独立 Spec 层（Trigger 的冷却覆盖可放在 `TriggerDef` 中）
- **Persistence**: 存档 `StackCounter` 和 `TriggerCooldown`

## 后果

### 正面
- Stacking/Trigger/Cue 三个关注点完全分离
- Cue 作为纯信号通道，表现层可独立替换（VFX 换引擎不影响业务）
- Trigger 系统支持声明式配置——新增一个响应 = 新配置文件
- 堆叠规则集中管理，不散落在各 Effect 定义中

### 负面
- Effect → Stacking → Trigger → Cue 的链路较长，调试时需要跨 4 个模块
- TriggerScanner 每帧轮询可能成为性能隐患（需要 Profile）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| Effect 直接管理堆叠 | 违反 Data Law 008，堆叠规则散落 |
| Trigger 直接播放 Cue | 违反 Data Law 009，业务与表现耦合 |
| Cue 合并到 Effect 模块 | Cue 是跨 Feature 信号，独立模块更清晰 |
| Trigger 使用 Bevy Observer 替代 Scanner | Observer 不适合"定期检查条件"的轮询模式 |

## 评审要点

- [ ] TriggerScanner 的轮询频率和性能影响评估？
- [ ] CueMergeStrategy 是否覆盖了所有常见的抑制场景？
- [ ] StackingGroup 的划分——ByTag 是否足够灵活？
- [ ] Trigger 递归深度 10 是否足够？能否在配置中 Per-trigger 覆盖？
