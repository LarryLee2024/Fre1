# Data Architecture Proposal V2 — BG3 执行与管控数据层提取

> 来源：`docs/其他/79博德3.md` 第5节（执行计算层）、第6节（堆叠规则）、第9节（表现与交互）
> 提取角色：Data Architect
> 提取日期：2026-06-15
> V2变更：内嵌国际化架构（ADR-017），所有文本字段替换为本地化Key
> 国际化依据：`docs/08-decisions/ADR-017-国际化架构决策.md`

---

## Domain Ownership

| 领域 | 数据类别 | 来源章节 |
|------|----------|----------|
| **Execution** | 独立骰式算式（攻击检定/豁免检定/伤害结算） | 第5节 |
| **Stacking** | DND式严格分层不叠加 | 第6节 |
| **Cue** | 玩法-叙事-环境联动表现 | 第9节 |
| **Registry** | 配置注册与查找 | 贯穿全文 |
| **Pipeline** | 效果执行管线 | 贯穿全文 |
| **Replay** | 确定性重放 | 第5节 + 第10节 |

---

## 国际化架构约束（ADR-017）

| 约束 | 规范 |
|------|------|
| Content数据只存Key | `name_key`/`desc_key` 替代 `name`/`description` |
| Key格式 | `namespace.id.suffix` |
| 禁止语义化Key | ❌ `execution.attack_check.name` |
| 禁止硬编码文本 | ❌ `name: "攻击检定"` |

### Key命名空间与ID格式

| 领域 | 命名空间 | ID前缀 | 示例Key |
|------|---------|--------|---------|
| Execution | `execution` | `ex_` | `execution.ex_1001.name` |
| Stacking | `stacking` | `sk_` | `stacking.sk_1001.name` |
| Cue | `cue` | `c_` | `cue.c_1001.name` |
| Formula | `formula` | `f_` | `formula.f_1001.name` |

---

## Schema Design

### 1. Execution Domain — 独立执行算式

#### 1.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `execution_type` | Definition | `Enum` | 执行算式类型 | 攻击检定/豁免检定/伤害结算 |
| `execution_formula` | Definition | `FormulaId` | 计算公式引用 | 各算式的计算规则 |
| `attack_vs` | Definition | `AttributeId` | 攻击检定对比属性 | AC |
| `save_vs` | Definition | `AttributeId` | 豁免检定对比属性 | DC |
| `damage_dice` | Definition | `DiceConfig` | 伤害骰配置 | 如1d8+3 |
| `resistance_rule` | Definition | `ResistanceRule` | 抗性结算规则 | 抗性减半/免疫归零/易伤翻倍 |
| `execution_result` | Runtime | `ExecutionResult` | 执行结果 | 命中/未命中/伤害值 |

#### 1.2 BG3三类执行算式 → Lite-GAS Execution映射

| BG3执行算式 | 公式 | Lite-GAS映射 | 关键差异 |
|-----------|------|-------------|---------|
| 攻击检定 | 1d20 + 修正 vs AC | `AttackExecution` | d20→确定性命中率公式 |
| 豁免检定 | 1d20 + 修正 vs DC | `SaveExecution` | d20→确定性抵抗率公式 |
| 伤害结算 | 伤害骰 + 修正 | `DamageExecution` | 骰子→确定性伤害公式 |

#### 1.3 骰子→确定性公式映射策略

| BG3骰子机制 | 确定性替代 | 公式示例 |
|-----------|-----------|---------|
| d20攻击检定 | 命中率计算 | `hit_rate = atk / (atk + def) * 100` |
| d20豁免检定 | 抵抗率计算 | `save_rate = will / (will + dc) * 100` |
| 伤害骰(1d8) | 固定伤害值 | `damage = base + modifier` |
| 优势(2d20取大) | 命中率加成 | `hit_rate += 25%` |
| 劣势(2d20取小) | 命中率惩罚 | `hit_rate -= 25%` |
| 暴击(自然20) | 暴击率 | `crit_rate = 5%` (可配置) |

#### 1.4 Schema草案

```rust
// === Definition Layer ===

/// 执行算式定义（配置，运行时不可变）
///
/// 不变量：
/// - Execution不拥有业务逻辑，只做数值计算
/// - 公式通过FormulaId引用，不内联（Law 002）
/// - 所有算式完全独立，新增能力只需挂载已有算式
/// - name_key 必须符合 ADR-017 Key命名规范
struct ExecutionDefinition {
    id: ExecutionId,                 // 如 "ex_1001"

    // 国际化字段（ADR-017）
    name_key: String,                // "execution.ex_1001.name"
    desc_key: String,                // "execution.ex_1001.desc"

    execution_type: ExecutionType,
    formula: FormulaId,

    // 攻击检定专用
    attack_attribute: Option<AttributeId>,
    defense_attribute: Option<AttributeId>,

    // 豁免检定专用
    save_attribute: Option<AttributeId>,
    dc_formula: Option<FormulaId>,

    // 伤害结算专用
    damage_type: Option<TagId>,
    resistance_rule: Option<ResistanceRule>,
}

/// 执行算式类型
enum ExecutionType {
    Attack,       // 攻击检定
    Save,         // 豁免检定
    Damage,       // 伤害结算
    Heal,         // 治疗结算
    Shield,       // 护盾结算
    Dispel,       // 驱散结算
    ApplyStatus,  // 施加状态
}

/// 抗性结算规则
///
/// 基于标签匹配，不需要硬编码判断。
struct ResistanceRule {
    resist_tag: TagId,
    resist_effect: ResistEffect,
    immune_tag: Option<TagId>,
    vulnerable_tag: Option<TagId>,
}

/// 抗性效果
enum ResistEffect {
    Half,      // 抗性减半
    Immune,    // 免疫归零
    Double,    // 易伤翻倍
    None,
}

// === Runtime Layer ===

/// 执行结果（临时计算结果，不持久化）
struct ExecutionResult {
    execution_type: ExecutionType,
    hit: Option<bool>,
    damage: Option<u32>,         // 保证≥1
    critical: Option<bool>,
    saved: Option<bool>,
}

// === Persistence Layer ===

/// 执行结果存档（仅Replay使用）
struct ExecutionResultRecord {
    execution_id: ExecutionId,
    source: Entity,
    target: Entity,
    result: ExecutionResult,
    turn: u32,
    sequence: u32,
}
```

---

### 2. Stacking Domain — 分层不叠加策略

#### 2.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `stacking_policy` | Definition | `Enum` | 堆叠策略类型 | 叠加/替换/刷新/取最高 |
| `stacking_scope` | Definition | `Enum` | 堆叠作用域 | 同名/同类型/同来源 |
| `max_stack` | Definition | `Option<u32>` | 最大叠加层数 | 特殊Debuff可叠加 |
| `refresh_duration` | Definition | `bool` | 是否刷新持续时间 | 同名效果再次施加时 |
| `stack_category` | Definition | `ModifierCategory` | 加值类型分类 | 附魔/洞察/环境/士气/幸运 |

#### 2.2 BG3堆叠铁则 → Lite-GAS Stacking策略映射

| BG3堆叠铁则 | Lite-GAS Stacking策略 |
|-----------|---------------------|
| 同名效应不叠加，取最高值 | `StackingPolicy::TakeHighest` |
| 加值按类型分层，同类型不叠加 | `StackingScope::SameCategory` |
| 异类型可叠加 | `StackingPolicy::Stack` |
| 优势/劣势不叠加 | `StackingPolicy::TakeHighest` |
| 同名效果刷新持续时间 | `StackingPolicy::RefreshDuration` |
| 特殊Debuff可叠加层数 | `StackingPolicy::StackUpTo(max)` |

#### 2.3 Schema草案

```rust
// === Definition Layer ===

/// 堆叠策略定义（配置，运行时不可变）
///
/// 不变量：
/// - 所有堆叠行为归属Stacking（Law 008）
/// - max_stack只在Stacking中定义，不散落其他领域
/// - name_key 必须符合 ADR-017 Key命名规范
struct StackingDefinition {
    id: StackingId,                  // 如 "sk_1001"

    // 国际化字段（ADR-017）
    name_key: String,                // "stacking.sk_1001.name"
    desc_key: String,                // "stacking.sk_1001.desc"

    policy: StackingPolicy,
    scope: StackingScope,
    max_stack: Option<u32>,
    refresh_duration: bool,
    category: Option<ModifierCategory>,
}

/// 堆叠策略
enum StackingPolicy {
    TakeHighest,           // 取最高值
    Stack,                 // 自由叠加
    StackUpTo(u32),        // 叠加到上限
    RefreshDuration,       // 刷新持续时间
    Replace,               // 替换旧效果
    Ignore,                // 忽略新效果
}

/// 堆叠作用域
enum StackingScope {
    SameName,       // 同名效果
    SameCategory,   // 同类型加值
    SameSource,     // 同来源
    Global,         // 全局
}

// === Instance Layer ===

/// 堆叠状态
struct StackingState {
    definition_id: StackingId,
    current_stack: u32,
    active_effects: Vec<Entity>,
}
```

---

### 3. Cue Domain — 三层表现信号

#### 3.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `cue_type` | Definition | `Enum` | Cue类型 | 战斗表现/环境交互/叙事反馈 |
| `cue_trigger` | Definition | `CueTrigger` | 触发时机 | OnApply/OnRemove/OnTick |
| `cue_vfx` | Definition | `Option<AssetPath>` | 视觉特效资源路径 | 特效动画 |
| `cue_sfx` | Definition | `Option<AssetPath>` | 音效资源路径 | 音效文件 |
| `cue_ui` | Definition | `Option<CueUiConfig>` | UI反馈配置 | 飘字/状态图标 |
| `cue_environment` | Definition | `Option<CueEnvConfig>` | 环境交互配置 | 火焰点油/冰霜冻结 |
| `cue_data` | Instance | `CueData` | Cue携带的纯数据 | 伤害值/治疗值/状态名 |

#### 3.2 BG3三层Cue → Lite-GAS Cue映射

| BG3 Cue层 | 说明 | Lite-GAS映射 | 吸收策略 |
|----------|------|-------------|---------|
| 战斗表现层 | 伤害飘字、施法动画、命中特效 | `CueType::Battle` | ✅ 直接吸收 |
| 环境交互层 | 火焰点燃油面、冰霜冻结水面 | `CueType::Environment` | ⚠️ 按需吸收 |
| 叙事反馈层 | 角色台词、队友评论 | — | ❌ 不吸收 |

#### 3.3 Schema草案

```rust
// === Definition Layer ===

/// Cue定义（配置，运行时不可变）
///
/// 不变量：
/// - Cue仅携带纯数据事件，不携带资源引用（Law 009扩展）
/// - 表现层订阅Cue事件，反向零依赖战斗逻辑
/// - Effect → Cue → VFX/SFX/UI，禁止跳过
/// - name_key 必须符合 ADR-017 Key命名规范
struct CueDefinition {
    id: CueId,                       // 如 "c_1001"

    // 国际化字段（ADR-017）
    name_key: String,                // "cue.c_1001.name"

    cue_type: CueType,
    trigger: CueTrigger,

    // 表现配置（引用ID，不直接引用资源）
    vfx_id: Option<VfxId>,
    sfx_id: Option<SfxId>,
    ui_config: Option<CueUiConfig>,
    env_config: Option<CueEnvConfig>,
}

/// Cue类型
enum CueType {
    Battle,       // 战斗表现
    Environment,  // 环境交互
    Status,       // 状态变化
}

/// Cue触发时机
enum CueTrigger {
    OnApply, OnRemove, OnTick,
    OnHit, OnMiss, OnCrit,
    OnDeath, OnHeal,
}

/// UI反馈配置
struct CueUiConfig {
    floating_text: Option<FloatingTextConfig>,
    status_icon: Option<StatusIconConfig>,
    screen_effect: Option<ScreenEffectConfig>,
}

/// 环境交互配置
///
/// 本质是效果触发的次级Effect，由标签匹配驱动。
struct CueEnvConfig {
    trigger_tag: TagId,
    result_tag: TagId,
    result_effect: Option<EffectId>,
}

// === Runtime Layer ===

/// Cue事件（运行时，不持久化）
struct CueEvent {
    cue_id: CueId,
    source: Entity,
    target: Entity,
    data: CueData,
}

/// Cue携带的纯数据
///
/// 国际化注意：CueData中的文本（如伤害飘字）不硬编码，
/// 由表现层根据damage_type标签查找本地化Key。
enum CueData {
    Damage { amount: u32, damage_type: TagId, critical: bool },
    Heal { amount: u32 },
    StatusApplied { status: TagId },
    StatusRemoved { status: TagId },
    Miss,
    Death,
}
```

**Cue国际化补充说明**：CueData中的`damage_type: TagId`用于表现层查找本地化Key（如 `tag.tag_0001.name` → "火焰伤害"），飘字内容通过Key动态解析，不硬编码。

---

### 4. Registry Domain — 配置注册与查找

#### 4.1 Schema草案

```rust
/// 领域注册表（Infrastructure层）
///
/// 不变量：
/// - 每个领域一个独立注册表
/// - ID全局唯一
/// - Definition运行时不可变
/// - 加载时校验，失败使用默认值
/// - 所有name_key/desc_key在注册时校验格式

struct AttributeRegistry { entries: HashMap<AttributeId, AttributeDefinition> }
struct TagRegistry { entries: HashMap<TagId, TagDefinition> }
struct ModifierRegistry { entries: HashMap<ModifierId, ModifierDefinition> }
struct EffectRegistry { entries: HashMap<EffectId, EffectDefinition> }
struct AbilityRegistry { entries: HashMap<AbilityId, AbilityDefinition> }
struct TriggerRegistry { entries: HashMap<TriggerId, TriggerDefinition> }
struct TargetingRegistry { entries: HashMap<TargetingId, TargetingDefinition> }
struct ExecutionRegistry { entries: HashMap<ExecutionId, ExecutionDefinition> }
struct StackingRegistry { entries: HashMap<StackingId, StackingDefinition> }
struct CueRegistry { entries: HashMap<CueId, CueDefinition> }
struct FormulaRegistry { entries: HashMap<FormulaId, FormulaDefinition> }
struct RequirementRegistry { entries: HashMap<RequirementId, RequirementDefinition> }
struct ConditionRegistry { entries: HashMap<ConditionId, ConditionDefinition> }

/// 注册表加载时国际化校验
///
/// 对每个Definition的name_key/desc_key执行格式校验。
/// 校验失败时ERROR级别报错，阻止加载。
pub fn validate_registry_i18n<T>(registry: &HashMap<String, T>) -> Result<(), Vec<LocalizationError>>
where
    T: I18nDefinition,
{
    let mut errors = Vec::new();
    for (id, def) in registry {
        if let Err(e) = validate_i18n_fields(def) {
            errors.push(e);
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

---

### 5. Pipeline Domain — 效果执行管线

#### 5.1 BG3效果执行流程 → Lite-GAS Pipeline映射

```
BG3效果执行:                     Lite-GAS Pipeline:
┌──────────────┐               ┌──────────────┐
│ 效果生成      │      →        │ Generate     │
├──────────────┤               ├──────────────┤
│ 修正叠加      │      →        │ Modify       │
├──────────────┤               ├──────────────┤
│ 执行结算      │      →        │ Execute      │
├──────────────┤               ├──────────────┤
│ 表现反馈      │      →        │ Cue          │
└──────────────┘               └──────────────┘
```

#### 5.2 Schema草案

```rust
/// 效果执行管线阶段
///
/// 不变量：
/// - 严格按 Generate → Modify → Execute → Cue 顺序执行
/// - base_amount 必须在 Modify 阶段设置，不在 Generate 阶段
/// - 伤害值必须 ≥ 1（Generate 和 Modify 阶段之后）
enum PipelineStage {
    Generate,    // 生成基础值
    Modify,      // Modifier管线修正
    Execute,     // Execution执行算式
    Cue,         // 表现信号下发
}
```

---

### 6. Replay Domain — 确定性重放

#### 6.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `replay_seed` | Persistence | `u64` | RNG种子 | 骰子确定性 |
| `replay_events` | Persistence | `Vec<ReplayEvent>` | 重放事件流 | 所有战斗操作 |
| `replay_version` | Persistence | `u32` | 重放版本号 | 兼容性 |

#### 6.2 BG3骰子确定性 → Lite-GAS Replay映射

| BG3机制 | Replay挑战 | Lite-GAS处理 |
|---------|-----------|-------------|
| d20骰子 | 必须记录每次投掷结果 | 替换为确定性公式，无需记录 |
| 伤害骰 | 必须记录每次投掷结果 | 替换为确定性公式，无需记录 |
| 优势/劣势 | 必须记录两次投掷 | 替换为数值修正，无需记录 |
| 触发器响应 | 必须记录AI反应决策 | 记录决策结果，非决策过程 |
| name_key/desc_key | 不影响Replay | Key是确定性字符串 |

#### 6.3 Schema草案

```rust
/// Replay事件（持久化）
///
/// 不变量：
/// - 同输入 → 同结果
/// - 不依赖系统随机数、当前时间、外部状态
/// - 国际化Key不影响Replay（Key是确定性字符串）
struct ReplayEvent {
    turn: u32,
    sequence: u32,
    event_type: ReplayEventType,
    data: ReplayEventData,
}

/// Replay事件类型
enum ReplayEventType {
    AbilityUsed, EffectApplied, DamageDealt, Healed,
    StatusGained, StatusLost, TriggerFired,
    TurnStarted, TurnEnded, UnitMoved, UnitDied,
}

/// Replay事件数据
///
/// 国际化注意：Replay不记录翻译后的文本，
/// 只记录ID/Key，回放时由当前语言实时解析。
enum ReplayEventData {
    AbilityUsed { ability: AbilityId, source: Entity, targets: Vec<Entity> },
    DamageDealt { source: Entity, target: Entity, amount: u32, damage_type: TagId },
    Healed { source: Entity, target: Entity, amount: u32 },
    StatusGained { target: Entity, status: TagId, duration: Option<u32> },
    StatusLost { target: Entity, status: TagId },
    TriggerFired { trigger: TriggerId, source: Entity, target: Entity },
    TurnStarted { entity: Entity },
    TurnEnded { entity: Entity },
    UnitMoved { entity: Entity, from: GridPos, to: GridPos },
    UnitDied { entity: Entity },
}

/// Replay存档
struct ReplaySaveState {
    version: u32,
    seed: u64,
    battle_config_id: String,
    events: Vec<ReplayEvent>,
}
```

**Replay国际化说明**：Replay只记录ID（AbilityId, TagId等），不记录翻译后的文本。回放时由当前语言的FTL文件实时解析Key，确保不同语言下Replay结果一致。

---

## Dependency Analysis

### 领域间依赖关系

```
Execution ──→ Formula (公式引用)
   │
   └──→ Tag (伤害类型标签匹配抗性)

Stacking ──→ ModifierCategory (加值类型决定叠加规则)
   │
   └──→ Effect (堆叠策略应用于效果实例)

Cue ──→ Effect (Cue由Effect触发，Law 009)
   │
   ├──→ Tag (环境交互标签匹配 + 国际化Key查找)
   │
   └──→ Asset (VFX/SFX资源，通过ID间接引用)

Registry ──→ All Domains (所有领域定义的注册与查找 + i18n校验)

Pipeline ──→ Effect (管线驱动效果执行)

Replay ──→ All Domains (记录所有领域事件，只记录ID不记录文本)
```

### 管线数据流

```
完整效果执行管线:

Ability/Trigger
  │
  ↓ (引用EffectId)
Effect
  │
  ├──→ Generate: 生成base_amount
  │
  ├──→ Stacking: 判定叠加行为
  │       │
  │       └──→ ModifierCategory: 同类型不叠加
  │
  ├──→ Modify: Modifier管线修正
  │       │
  │       └──→ ModifierInstance: 应用修正值
  │
  ├──→ Execute: Execution算式结算
  │       │
  │       └──→ Formula: 确定性计算
  │
  ├──→ Cue: 表现信号下发
  │       │
  │       ├──→ VFX: 视觉特效
  │       ├──→ SFX: 音效
  │       └──→ UI: 飘字/图标 (通过TagId查找i18n Key)
  │
  └──→ Record: Replay事件记录 (只记录ID，不记录文本)
```

---

## Validation Rules

### Execution校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| formula_id必须已注册 | 加载时 | ERROR |
| Attack类型必须有attack_attribute和defense_attribute | 加载时 | ERROR |
| Save类型必须有save_attribute和dc_formula | 加载时 | ERROR |
| Damage类型必须有damage_type标签 | 加载时 | ERROR |
| 伤害值≥1（Generate+Modify后） | 运行时 | ERROR |
| name_key格式必须符合`execution.ex_XXXX.suffix` | 加载时 | ERROR |

### Stacking校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| StackUpTo策略必须有max_stack | 加载时 | ERROR |
| max_stack > 0 | 加载时 | ERROR |
| SameCategory scope必须有category | 加载时 | ERROR |
| category必须引用已注册ModifierCategory | 加载时 | ERROR |
| name_key格式必须符合`stacking.sk_XXXX.suffix` | 加载时 | ERROR |

### Cue校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| vfx_id/sfx_id/ui_config至少有一个 | 加载时 | WARN |
| env_config的trigger_tag和result_tag必须已注册 | 加载时 | ERROR |
| env_config的result_effect必须已注册 | 加载时 | ERROR |
| name_key格式必须符合`cue.c_XXXX.suffix` | 加载时 | ERROR |

### Replay校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| version必须匹配当前版本或可迁移 | 加载时 | ERROR |
| seed必须存在（如使用确定性随机） | 加载时 | ERROR |
| events不能为空 | 加载时 | WARN |
| event sequence必须递增 | 加载时 | ERROR |

### 国际化校验（ADR-017，适用于所有领域）

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| name_key必选，不能为空 | 加载时 | ERROR |
| Key格式必须为`namespace.id.suffix` | 加载时 | ERROR |
| namespace必须是已注册领域 | 加载时 | ERROR |
| id必须是永久唯一ID格式 | 加载时 | ERROR |
| suffix必须是name/desc/short_desc/tooltip之一 | 加载时 | ERROR |
| 禁止语义化Key | 加载时 | ERROR |
| 禁止无意义编号Key | 加载时 | ERROR |
| Key不能包含中文/特殊字符 | 加载时 | ERROR |

---

## Replay Compatibility

| 数据元素 | Replay影响 | 处理策略 |
|---------|-----------|---------|
| Execution公式 | 必须确定性 | 公式引用ID，不内联（Law 002） |
| Stacking判定 | 必须确定性 | 按ModifierCategory优先级排序 |
| Cue事件 | 不影响Replay | Cue是表现层，不记录到Replay |
| Pipeline顺序 | 必须确定性 | 严格阶段顺序 |
| RNG种子 | 必须确定性 | 记录seed到Replay |
| 执行结果 | 必须可重放 | 记录到ReplayEvent |
| name_key/desc_key | 不影响Replay | Key是确定性字符串，翻译在表现层 |

**Law 010检查**：所有计算确定性。Cue不影响Replay。国际化Key不影响Replay。✅ 通过。

---

## Save Compatibility

| 数据元素 | Save版本策略 | 迁移考虑 |
|---------|-------------|---------|
| ExecutionDefinition | 大版本+1才可删除 | 新算式不影响旧存档 |
| StackingDefinition | 小版本可新增 | 新策略默认不存在于旧存档 |
| CueDefinition | 小版本可新增 | 新Cue不影响旧存档 |
| ReplaySaveState | 版本号迁移 | 新版本Replay事件向后兼容 |
| StackingState.current_stack | 新字段需默认值 | 缺失时默认1 |
| name_key/desc_key | 新字段需默认值 | 旧存档硬编码文本需迁移为Key |

---

## Migration Strategy

### 从BG3模型迁移到Lite-GAS

| 迁移项 | BG3模型 | Lite-GAS目标 | 迁移路径 |
|--------|---------|-------------|---------|
| d20攻击检定 | 1d20 + 修正 vs AC | 确定性命中率公式 | 完全替换 |
| d20豁免检定 | 1d20 + 修正 vs DC | 确定性抵抗率公式 | 完全替换 |
| 伤害骰 | XdY + 修正 | 确定性伤害公式 | 完全替换 |
| 抗性/免疫/易伤 | 标签匹配 | 保留标签匹配机制 | 直接迁移 |
| 加值分类不叠加 | DND五类加值 | ModifierCategory枚举 | 扩展映射 |
| 三层Cue | 战斗+环境+叙事 | 战斗+环境（排除叙事） | 过滤迁移 |
| 硬编码文本 | `name: "攻击检定"` | `name_key: "execution.ex_1001.name"` | Key映射表 |

---

## Future Extension

| 扩展点 | 当前设计 | 未来可能 |
|--------|---------|---------|
| ExecutionType | 7种 | 可新增Summon、Teleport、Transform |
| StackingPolicy | 6种 | 可新增Merge(合并效果) |
| CueType | 3种 | 可新增Cutscene(过场) |
| CueTrigger | 8种 | 可新增OnEquip、OnLevelUp |
| ReplayEventType | 11种 | 可新增OnChat、OnTrade |
| PipelineStage | 4阶段 | 可新增PreCheck(前置检查) |
| 确定性随机 | 当前无 | 可引入seed-based RNG流 |
| Key后缀 | 4种 | 可新增`.flavor`、`.lore` |

---

## Risks

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 骰子→确定性公式影响玩法体验 | 玩家期望随机性 | 可引入seed-based确定性随机 |
| Stacking规则复杂度 | 叠加判定逻辑复杂 | 限制ModifierCategory数量 |
| 环境Cue交互链过长 | 性能和确定性风险 | 限制环境交互深度≤2 |
| Replay版本兼容 | 旧Replay无法播放 | 版本迁移+降级策略 |
| Cue与逻辑层耦合 | 违反Law 009 | 严格Event驱动，反向零依赖 |
| Key映射表维护成本 | 旧存档迁移复杂 | 自动化迁移工具 |

---

## Constitution Check

| Data Law / 规范 | 检查结果 | 说明 |
|----------------|---------|------|
| Law 001 | ✅ 通过 | ExecutionDefinition/ExecutionResult分离；StackingDefinition/StackingState分离；CueDefinition/CueEvent分离 |
| Law 002 | ✅ 通过 | 执行公式通过FormulaId引用，不内联 |
| Law 003 | ✅ 通过 | Cue引用VfxId/SfxId，不重复定义资源 |
| Law 005 | ✅ 通过 | Effect→Execution→Modifier，不跳过管线 |
| Law 006 | ✅ 通过 | Modifier只改变数值，Execution只做计算 |
| Law 008 | ✅ 通过 | max_stack只在StackingDefinition中定义 |
| Law 009 | ✅ 通过 | 所有表现经过Cue，Effect不直接播放特效 |
| Law 010 | ✅ 通过 | 所有计算确定性，骰子机制已替换 |
| ADR-017 | ✅ 通过 | 所有文本字段使用name_key/desc_key，Key格式符合规范 |
| 宪法§17.2.2 | ✅ 通过 | 禁止硬编码玩家可见文本 |

**[Data Exemption]**：无。

---

## 数据清单汇总

### Execution Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| EX-01 | ExecutionId | Definition | String | ✅ | BG3§5 |
| EX-02 | ExecutionType | Definition | Enum(7种) | ✅ | BG3§5 |
| EX-03 | ExecutionDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§5 + ADR-017 |
| EX-04 | ExecutionDefinition.desc_key | Definition | String(LocalizedKey) | ❌ | BG3§5 + ADR-017 |
| EX-05 | ExecutionDefinition.formula | Definition | FormulaId | ✅ | BG3§5 |
| EX-06 | ExecutionDefinition.attack_attribute | Definition | Option<AttributeId> | ❌ | BG3§5 |
| EX-07 | ExecutionDefinition.defense_attribute | Definition | Option<AttributeId> | ❌ | BG3§5 |
| EX-08 | ExecutionDefinition.save_attribute | Definition | Option<AttributeId> | ❌ | BG3§5 |
| EX-09 | ExecutionDefinition.dc_formula | Definition | Option<FormulaId> | ❌ | BG3§5 |
| EX-10 | ExecutionDefinition.damage_type | Definition | Option<TagId> | ❌ | BG3§5 |
| EX-11 | ResistanceRule | Definition | Struct | ❌ | BG3§5 |
| EX-12 | ResistEffect | Definition | Enum(4种) | ❌ | BG3§5 |
| EX-13 | ExecutionResult | Runtime | Struct | — | BG3§5 |
| EX-14 | ExecutionResultRecord | Persistence | Struct | ✅ | BG3§5 |

### Stacking Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| SK-01 | StackingId | Definition | String | ✅ | BG3§6 |
| SK-02 | StackingPolicy | Definition | Enum(6种) | ✅ | BG3§6 |
| SK-03 | StackingScope | Definition | Enum(4种) | ✅ | BG3§6 |
| SK-04 | StackingDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§6 + ADR-017 |
| SK-05 | StackingDefinition.desc_key | Definition | String(LocalizedKey) | ❌ | BG3§6 + ADR-017 |
| SK-06 | StackingDefinition.max_stack | Definition | Option<u32> | ❌ | BG3§6 |
| SK-07 | StackingDefinition.refresh_duration | Definition | bool | ❌ | BG3§6 |
| SK-08 | StackingDefinition.category | Definition | Option<ModifierCategory> | ❌ | BG3§6 |
| SK-09 | StackingState.current_stack | Instance | u32 | ✅ | BG3§6 |
| SK-10 | StackingState.active_effects | Instance | Vec<Entity> | ✅ | BG3§6 |

### Cue Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| CU-01 | CueId | Definition | String | ✅ | BG3§9 |
| CU-02 | CueType | Definition | Enum(3种) | ✅ | BG3§9 |
| CU-03 | CueTrigger | Definition | Enum(8种) | ✅ | BG3§9 |
| CU-04 | CueDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§9 + ADR-017 |
| CU-05 | CueDefinition.vfx_id | Definition | Option<VfxId> | ❌ | BG3§9 |
| CU-06 | CueDefinition.sfx_id | Definition | Option<SfxId> | ❌ | BG3§9 |
| CU-07 | CueUiConfig | Definition | Struct | ❌ | BG3§9 |
| CU-08 | CueEnvConfig | Definition | Struct | ❌ | BG3§9 |
| CU-09 | CueEvent | Runtime | Struct | — | BG3§9 |
| CU-10 | CueData | Runtime | Enum(6种) | — | BG3§9 |

### Registry Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| RG-01 | 13个领域Registry | Infrastructure | HashMap<ID, Def> | ✅ | 贯穿全文 |
| RG-02 | validate_registry_i18n | Infrastructure | Fn | ✅ | ADR-017 |

### Pipeline Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| PL-01 | PipelineStage | Infrastructure | Enum(4种) | ✅ | 贯穿全文 |

### Replay Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| RP-01 | ReplayEvent | Persistence | Struct | ✅ | BG3§5+§10 |
| RP-02 | ReplayEventType | Persistence | Enum(11种) | ✅ | BG3§5+§10 |
| RP-03 | ReplayEventData | Persistence | Enum(10种) | ✅ | BG3§5+§10 |
| RP-04 | ReplaySaveState | Persistence | Struct | ✅ | BG3§5+§10 |
| RP-05 | ReplaySaveState.seed | Persistence | u64 | ✅ | BG3§5 |
| RP-06 | ReplaySaveState.version | Persistence | u32 | ✅ | BG3§5 |
