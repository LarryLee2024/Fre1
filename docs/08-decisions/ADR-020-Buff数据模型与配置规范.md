# ADR-020: Buff 数据模型与配置规范

## 状态

Proposed

## 背景

Buff/Debuff 系统是 SRPG 战斗的核心机制之一。当前项目已有 8 个 Buff（attack_up、attack_down、defense_up、defense_down、burn、poison、regen、stun）通过 RON 配置驱动，但缺少正式的数据模型架构决策。现有 `BuffData` 结构为扁平设计，缺少 Trigger、Duration 策略、Stack 策略等核心抽象，与 `docs/01-architecture/skill-buff-abstraction.md` §2.4 定义的 Buff 抽象模型存在差距。

本 ADR 定义 Buff 的 Definition/Instance 分离策略、RON 配置契约、字段规范和定义态重构路径。

## 引用的领域规则

- `docs/02-domain/skill/skill-rules.md` — 规则7：Buff 容器化管理、Buff 的效果列表走 Effect Pipeline
- `docs/01-architecture/skill-buff-abstraction.md` — §2.4 Buff = Trigger[] + Effect[] 模型、§2.2 Skill = Selector + Effect[]、§4.6 Duration 策略、§4.7 StackPolicy 叠层系统、§4.8 Trigger 系统
- `docs/01-architecture/README.md` — Definition/Instance 分离原则、Bug 修复规则

## 决策

### 1. 双类型模式（BuffDef/BuffData 分离）

遵循 Definition/Instance 分离和 Rule/Content 分离：

```
BuffDef（RON 反序列化用）
  ├─ 使用 TagName 字符串（RON 友好）
  ├─ 包含 version 字段（配置版本管理）
  ├─ 包含 effects: Vec<EffectDef>（替代扁平 dot_damage/hot_heal）
  ├─ 包含 duration: DurationDef（替代 u32）
  ├─ 包含 stack: StackDef（替代无叠层策略）
  └─ impl From<BuffDef> for BuffData

BuffData（运行时用）
  ├─ 使用 GameplayTag 位掩码
  ├─ 从 BuffDef 转换后注册到 BuffRegistry
  ├─ effects → Vec<EffectDef>（Effect Pipeline 可执行）
  └─ duration → DurationPolicy（运行时判断过期）
```

### 2. BuffDef RON 格式契约（目标态）

```ron
(
    id: "burn",
    version: 1,
    name: "灼烧",
    description: "每回合受到火焰伤害",
    effects: [
        Damage(multiplier: 0.3, ignore_def_percent: 0.0),
    ],
    duration: Turns(2),
    stack: NoStack,
    conditions: [],
    tags: [DEBUFF, BURN, FIRE],
    is_buff: false,
)
```

### 3. 过渡期兼容策略

当前 `BuffData` 使用 `dot_damage`/`hot_heal`/`is_stun`/`is_cleanse` 等扁平字段表示特殊效果。迁移分三阶段：

| Phase | 变更 | 说明 |
|-------|------|------|
| Phase 1（当前） | 保留扁平字段 + 新增 effects 字段共存 | 过渡期，两套系统并行 |
| Phase 2 | 新增 Trigger/Duration/Stack 字段 | 配置可选项，默认兼容旧行为 |
| Phase 3 | 移除扁平字段 | 仅当所有 RON 配置完成迁移 |

### 4. BuffData 目标数据结构

```rust
pub struct BuffData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: Vec<EffectDef>,        // 替代 dot_damage/hot_heal
    pub duration: DurationPolicy,       // 替代 default_duration: u32
    pub stack: StackPolicy,             // 替代无策略
    pub conditions: Vec<BuffCondition>, // 新增
    pub tags: Vec<GameplayTag>,
    // 过渡期保留字段（Phase 3 移除）
    pub dot_damage: i32,
    pub hot_heal: i32,
    pub is_stun: bool,
    pub is_cleanse: bool,
    pub is_buff: bool,
}
```

### 5. 字段规范（目标态）

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `id` | String | 是 | — | 全局唯一标识 |
| `version` | u32 | 是 | 0 | 配置版本号 |
| `name` | String | 是 | — | 显示名称 |
| `description` | String | 否 | `""` | 描述文本 |
| `effects` | Vec\<EffectDef\> | 是 | `[]` | 效果列表（替代扁平字段） |
| `duration` | DurationDef | 是 | Turns(1) | 持续策略 |
| `stack` | StackDef | 是 | NoStack | 叠层策略 |
| `conditions` | Vec\<BuffConditionDef\> | 否 | `[]` | 触发条件 |
| `tags` | Vec\<TagName\> | 是 | — | 分类标签 |
| `is_buff` | bool | 是 | true | Buff/Debuff 标识（过渡期） |

### 6. 版本管理

- 所有 BuffDef 必须包含 `version` 字段
- `#[serde(default)]` 保证旧配置（无 version）兼容
- 版本号递增规则：小版本 +1（新增可选字段），大版本 +1（删除/类型变更）
- 当前 8 个 RON Buff 文件需要添加 `version: 1`

## Module Design

### 目标文件组织

```
src/core/buff/
├── mod.rs                    # BuffPlugin 入口
├── domain/
│   ├── mod.rs               # BuffRegistry + RegistryLoader 实现
│   ├── types.rs             # BuffData, BuffDef, DurationPolicy, StackPolicy
│   └── buff_error.rs        # BuffError 枚举
├── apply.rs                 # apply_buff / remove_buff 逻辑
├── instance.rs              # BuffInstance, ActiveBuffs 组件
├── resolve.rs               # 持续效果结算 + tick + rebuild_tags
├── trigger.rs               # TriggerRegistry + BuffTriggerHandler（新增）
└── id.rs                    # BuffId（从 shared/ids/ 引用）
```

### 当前结构（无需变更）

```
src/core/buff/               ← 当前结构已接近目标，仅需：
├── domain/                  ← 新增 types.rs 抽取 BuffData/BuffDef 类型定义
│   ├── mod.rs               ← 保留（注册表 + 默认 Buff）
│   └── buff_error.rs
├── apply.rs                 ← 保留
├── instance.rs              ← 保留
├── resolve.rs               ← 保留
├── trigger.rs               ← 新增（Trigger 系统）
└── id.rs                    ← 保留（已存在）
```

**迁移说明**：`domain/mod.rs` 当前约 415 行，包含 BuffData/BuffDef 类型定义 + BuffRegistry + 默认注册 + 测试。建议将类型定义（BuffData、BuffDef、DurationPolicy、StackPolicy）抽取到 `domain/types.rs`，保持 `domain/mod.rs` 只负责注册表。

## Communication Design

### Message

| Message（共享事件类型） | 发送方 | 接收方 | 用途 |
|------------------------|--------|--------|------|
| `shared::event::buff::BuffApplied` | apply_buff（调用者） | UI、日志、回放 | Buff 已施加通知 |
| `shared::event::buff::BuffRemoved` | resolve.rs | UI、日志、回放 | Buff 已移除通知 |

### 函数调用

- `apply_buff()` — 纯函数（修改 Component），72+ 调用点
- `remove_buff()` — 纯函数（清理修饰符 + 标签）
- `resolve_status_effects()` — ECS System，每回合结算
- `tick_buffs()` — 持续时间递减
- `rebuild_tags()` — 三层标签重建

### 未来 Trigger 通信

Buff 触发行为（OnTurnStart 扣血、OnAfterDamaged 反击）应通过 `TriggerRegistry` + `ExecutionStack` 实现，详情见 ADR-022。

## 边界定义

- 允许：`core/buff/` 依赖 `core/attribute/`（AttributeModifierDef）、`core/tag/`（GameplayTag）、`core/effect/`（EffectDef）
- 允许：`core/buff/` 依赖 `shared/ids/`（BuffId、UnitId）、`shared/event/buff/`（BuffApplied、BuffRemoved）
- 允许：`core/buff/` 被 `core/character/`、`core/battle/`、`core/effect/` 依赖
- 禁止：`core/buff/` 依赖 `ui/`、`infrastructure/`、`debug/`
- 禁止：`core/buff/` 直接执行伤害（必须通过 Effect Pipeline）

## Forbidden（禁止事项）

- 🟥 禁止：在 Buff 定义中使用扁平字段（dot_damage/hot_heal）执行效果 — 理由：违反 Effect Pipeline 统一执行原则
- 🟥 禁止：新增 Buff 类型而不更新所有 RON 配置的 version 字段 — 理由：配置版本管理是长期维护的基础
- 🟥 禁止：运行时修改 BuffDef — 理由：Definition/Instance 分离
- 🟥 禁止：apply_buff() 直接修改 HP — 理由：必须通过 Effect Pipeline 执行效果
- 🟥 禁止：Buff 无来源（source_entity = None 时应有默认值）— 理由：不可审计
- 🟥 禁止：为新增 Buff 修改 Rust 代码 — 理由：新增内容 = 新增 RON 文件
- 🟥 禁止：Buff 永不过期 — 理由：必须有 DurationPolicy 兜底

## Definition / Instance Design

- Definition：BuffDef（RON 反序列化）、BuffData（运行时查询）、DurationPolicy（持续策略）
- Instance：BuffInstance（实例状态）、ActiveBuffs（组件）

## 后果

### 正面
- effects 字段统一走 Effect Pipeline，消除扁平字段的冗余逻辑
- Duration/Stack/Condition 三个正交子系统的引入大幅提升 Buff 表现力
- 版本管理支持长期内容迭代
- 迁移路径平滑，兼容现有 8 个 Buff

### 负面
- BuffDef → BuffData 转换增加少量初始化开销
- 迁移期间两套系统并行增加维护复杂度
- is_buff/is_stun/is_cleanse 等字段需在 Phase 3 才能彻底移除

## 替代方案

| 方案 | 优点 | 缺点 | 为何放弃 |
|------|------|------|----------|
| 保持扁平字段 | 完全兼容 | 违反 Effect Pipeline 统一执行 | 架构不兼容 |
| 一次性迁移所有 Buff | 整洁 | 8 个 RON + apply.rs 全改，风险高 | 分阶段更安全 |
| 废弃 BuffRegistry 改用 SkillData | 统一数据模型 | Buff 语义不同（Trigger vs Selector） | 不合适 |
