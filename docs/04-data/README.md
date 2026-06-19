---
id: 04-data.README
title: Data Architecture
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - data-architecture
  - governance
  - schema
---

# Data Architecture — 数据架构总纲

> **版本**: 1.0 | **角色**: @data-architect | **宪法优先级**: 🟩 必须遵守

本文档定义 Fre 项目的数据宇宙（Data Universe）架构规范，覆盖所有领域的数据设计、Schema 治理、ID 策略、Save/Replay 兼容性规则。所有数据设计文档以此为最高依据。

---

## 1. 设计原则

### 1.1 数据哲学

| 原则 | 说明 |
|------|------|
| **Definition 与 Instance 强制分离** | 禁止单个结构同时承担配置、运行时状态和存档状态 |
| **Rule 与 Content 强制分离** | 规则属于代码，内容属于配置。禁止配置中出现业务代码 |
| **Replay 优先于便利** | 任何数据设计必须回答「Replay 是否兼容？」 |
| **组合优于创建** | 新玩法优先组合已有机制（Cost=Effect, Cooldown=Tag+Effect），不造新数据系统 |
| **三层分离** | Def（模板）→ Spec（配置）→ Instance（运行时），贯穿能力系统全链路 |
| **配置只引用 ID** | 禁止在配置中重复定义结构，Single Source of Truth |
| **Schema 可演化** | 任何超过两年无法演化的数据结构，都是失败的数据结构 |

### 1.2 优先级排序

```
正确性 > Replay 兼容性 > Save 兼容性 > 可扩展性 > 开发便利性 > 性能
```

性能优化必须基于 Profiling 实证数据，禁止凭体感提前优化 Schema 结构。

---

## 2. 四层数据架构

所有数据必须归属以下四层之一，禁止跨层污染：

```
┌─────────────────────────────────────────────────────────────┐
│                     Definition Layer                        │
│  静态定义，运行时不可变。内容团队通过配置文件实例化。          │
│  位置: assets/config/ (RON/JSON)                            │
│  示例: AbilityDef, EffectDef, TagHierarchy, AttributeDef    │
│  特征: 只读、版本化、可热重载、全局唯一                      │
├─────────────────────────────────────────────────────────────┤
│                      Spec Layer                             │
│  配置槽位，Definition → Instance 的桥梁。运行时可变。         │
│  位置: src/core/capabilities/*/mechanism/components.rs      │
│  示例: AbilitySpec, EffectSpec (含等级、冷却覆盖、快照)       │
│  特征: 引用 Definition ID、携带快照值、与 Entity 绑定        │
├─────────────────────────────────────────────────────────────┤
│                     Instance Layer                           │
│  运行时实例状态，每个实体一份。完整的生命周期管理。             │
│  位置: ECS Components                                       │
│  示例: AbilityInstance, ActiveEffect, BuffInstance           │
│  特征: 可修改、Entity 级隔离、ECS Component 存储             │
├─────────────────────────────────────────────────────────────┤
│                   Persistence Layer                          │
│  需要持久化的状态子集。存档时序列化，读档时重建。              │
│  位置: Save Schema (序列化格式)                              │
│  示例: SaveGame (含 Entity 状态、进度、世界快照)             │
│  特征: 版本化、带迁移链、容错、完整性校验                     │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 层间关系

```
[Definition]  ──引用──→  [Spec]  ──实例化──→  [Instance]
     │                       │                       │
     │                       │                       │
     ▼                       ▼                       ▼
 配置文件变更            运行时选择/升级          存档/读档
 触发热重载             修改 Spec 字段           Persistence
```

### 2.2 禁止模式

| 反模式 | 说明 | 正确做法 |
|--------|------|----------|
| 单结构多用途 | `Ability` 同时承担配置、运行时状态、存档状态 | 拆为 AbilityDef / AbilitySpec / AbilityInstance |
| 配置中嵌逻辑 | `formula: "(atk * 1.5)"` | 规则写在 `rules/` 纯函数中 |
| 运行时修改 Definition | 战斗中改变 AbilityDef 的伤害值 | 在 Spec 层存储快照值 |
| Instance 直接持久化 | 将 ECS Component 原始结构存为存档 | 设计专用的 Persistence Schema |
| Spec 越权 | Spec 包含业务逻辑 | Spec 只存储「选择了什么」，不存储「如何执行」 |

---

## 3. ID 策略与命名规范

### 3.1 ID 体系

所有 Definition 级别数据使用统一 ID 格式：

```
类型前缀 + 编号（6 位十进制，0-padded）
```

| 领域 | 类型前缀 | 示例 | 计数器范围 |
|------|----------|------|-----------|
| Attribute | `attr_` | `attr_000001` | 000000–999999 |
| Tag | `tag_` | `tag_000001` | 000000–999999 |
| Modifier | `mod_` | `mod_000001` | 000000–999999 |
| EffectDef | `eff_` | `eff_000001` | 000000–999999 |
| AbilityDef | `abl_` | `abl_000001` | 000000–999999 |
| TriggerDef | `trg_` | `trg_000001` | 000000–999999 |
| CueDef | `cue_` | `cue_000001` | 000000–999999 |
| ItemDef | `itm_` | `itm_000001` | 000000–999999 |
| QuestDef | `qst_` | `qst_000001` | 000000–999999 |
| SpellDef | `spl_` | `spl_000001` | 000000–999999 |
| BuffDef | `buf_` | `buf_000001` | 000000–999999 |
| FactionDef | `fct_` | `fct_000001` | 000000–999999 |
| TerrainDef | `ter_` | `ter_000001` | 000000–999999 |
| RecipeDef | `rcp_` | `rcp_000001` | 000000–999999 |
| LootTableDef | `oot_` | `oot_000001` | 000000–999999 |

**规则：**
- 禁止语义化 ID（❌ `ability.fireball` → ✅ `abl_000042`）
- 禁止使用无意义编号（❌ `text_001`）
- ID 一旦分配永久有效，删除时标记为 deprecated，不重新分配
- ID 分配通过 `Registry` 领域管理，支持冲突检测

### 3.2 本地化 Key 格式

所有用户可见文本字段在 Definition 中只存储 Key：

```
<命名空间>.<ID>.<后缀>
```

| 后缀 | 用途 | 必选 |
|------|------|------|
| `.name` | 显示名称 | 是 |
| `.desc` | 详细描述 | 是 |
| `.flavor` | 风味文本 | 否 |
| `.tooltip` | 工具提示 | 否 |

示例：`attribute.attr_000001.name`、`ability.abl_000042.desc`

### 3.3 文件命名规范

| 层级 | 格式 | 示例 |
|------|------|------|
| 领域 Schema | `<domain>_schema.md` | `tag_schema.md` |
| 数据提案 | `<domain>_proposal.md` | `combat_proposal.md` |
| 配置文件 | `<prefix>_<id>.ron` | `abl_000042.ron` |

---

## 4. Schema 治理规范

### 4.1 所有 Schema 必须包含的元数据头

```yaml
---
id: <domain>.<schema-type>.<version>
title: <Schema Title>
status: draft | stable | deprecated
owner: <domain-owner>
created: <YYYY-MM-DD>
updated: <YYYY-MM-DD>
layer: definition | spec | instance | persistence
replay-safe: true | false
---
```

### 4.2 每个 Schema 设计的标准章节

```
1. Domain Ownership — 归属领域、涉及的数据类别
2. Problem — 当前数据问题描述
3. Schema Design — 完整的结构定义（含字段类型、约束、默认值）
4. Layer Analysis — Definition/Spec/Instance/Persistence 四层分配
5. Dependency Analysis — 依赖的 ID 和 Schema
6. Validation Rules — 校验规则（构造时 + 运行时）
7. Replay Compatibility — 回放确定性和兼容性分析
8. Save Compatibility — 存档版本兼容性分析（含 Versioning 方案）
9. Migration Strategy — 未来迁移方案
10. Future Extension — 扩展点预留
11. Risks — 潜在风险与缓解措施
12. Constitution Check — 是否符合宪法与 Data Laws
```

### 4.3 Schema 评审 Checklist

每个 Schema 提交时必须逐项检查：

- [ ] Definition 与 Instance 是否强制分离？
- [ ] Rule 与 Content 是否强制分离？
- [ ] 配置是否只引用 ID？
- [ ] Effect 是否是唯一业务执行入口？
- [ ] Modifier 是否不拥有业务逻辑？
- [ ] Duration 是否属于 Effect 而非独立 Buff？
- [ ] 堆叠规则是否全部归属 Stacking 领域？
- [ ] 表现信号是否全部经过 Cue？
- [ ] Replay 兼容性是否满足？
- [ ] 四层分配是否清晰无跨层污染？

---

## 5. Data Laws

以下规则优先级仅低于项目宪法。违反必须标记 `[Data Exemption]` 并附加 ADR。

| # | 规则 | 说明 | 违反后果 |
|---|------|------|---------|
| 001 | **Def-Instance 强制分离** | 单个结构不得同时承担配置、运行时状态、存档状态 | Schema 需重建 |
| 002 | **Rule-Content 强制分离** | 规则属于代码，内容属于配置 | 配置校验拒绝 |
| 003 | **配置只引用 ID** | 禁止在配置中重复定义结构；Single Source of Truth | 编译期禁止 |
| 004 | **Ability 不拥有行为** | Ability 只描述 Cost/Cooldown/Targeting/Effects，不描述 on_hit/on_death 等行为逻辑 | Schema 审核不通过 |
| 005 | **Effect 是唯一业务执行入口** | Ability → Effect, Trigger → Effect；禁止 Ability → Modifier 或 Trigger → Modifier | 运行时断言失败 |
| 006 | **Modifier 不拥有业务逻辑** | Modifier 只改变数值，不含 on_turn_start 等逻辑 | 运行时断言失败 |
| 007 | **Duration 属于 Effect** | Duration 不属于独立 Buff 系统。Effect 的 duration 字段是其固有属性 | Schema 审核不通过 |
| 008 | **堆叠行为归属 Stacking** | 禁止 `max_stack: 5` 散落于 Ability/Effect/Modifier | 重构时自动检测 |
| 009 | **表现必须经过 Cue** | Effect → Cue → VFX/SFX/UI；禁止 Effect 直接播放特效 | 架构评审拦截 |
| 010 | **Replay 优先于便利** | 禁止依赖当前时间、系统随机数、外部状态、非确定性计算 | 回放断裂则回滚 |
| 011 | **Schema 必须版本化** | 所有 Persistence 层 Schema 必须带版本号并支持前向/后向兼容 | 存档兼容失败 |
| 012 | **域间禁止直接数据引用** | Domain 之间禁止直接引用对方的数据结构，仅通过 Event 通信 | 编译期模块边界检查 |

---

## 6. Save 架构

### 6.1 存档分层

```
Save File
├── Header
│   ├── save_version: u32           # 存档版本号
│   ├── game_version: String        # 创建存档的游戏版本
│   ├── timestamp: u64              # 创建时间戳（仅用于显示）
│   ├── checksum: [u8; 32]          # SHA-256 内容校验
│   └── metadata: SaveMetadata      # 玩家可见元数据
│
├── World Snapshot
│   ├── entities: Vec<EntityState>  # 所有持久化 Entity 的状态
│   ├── globals: GlobalState        # 全局状态（时间、故事标记）
│   └── progression: ProgState      # 玩家进度（等级、任务）
│
└── Replay Log (可选)
    └── commands: Vec<ReplayFrame>  # 战斗回放命令序列
```

### 6.2 版本迁移策略

```
旧版本 → [Migration_v1→v2] → [Migration_v2→v3] → 当前版本
```

- 采用链式增量迁移，而非 N² 转换矩阵
- 每个迁移独立可测试
- 禁止跳过中间版本
- 迁移失败时保留原始存档并报错

### 6.3 存档兼容性保证

| 场景 | 保证 |
|------|------|
| 旧存档在新版本加载 | 🟩 保证升级兼容 |
| 新存档在旧版本加载 | 🟦 尽力回滚兼容 |
| 跨大版本升级（如 1.x → 2.x） | 🟨 提供一次性迁移工具 |
| 存档损坏 | 🟩 检测并拒绝加载 |

---

## 7. Replay 架构

### 7.1 Replay 基本原理

```
录制阶段：
  User Input → Command → [Recorder] → ReplayFrame
                              ↓
                        ReplayLog.bin

回放阶段：
  ReplayLog.bin → [Player] → Command → Deterministic World
```

### 7.2 Replay 确定性保证

| 要求 | 实现 |
|------|------|
| 随机数确定 | 种子由 ReplayFrame 提供，使用确定性 PRNG |
| 时间确定 | 使用 GameTime（帧计数），不使用 wall-clock |
| 输入确定 | 所有玩家输入作为 Command 录制 |
| AI 确定 | AI 决策使用与玩家相同的确定性 PRNG 种子 |
| 浮点一致 | 确保跨平台浮点行为一致（`f32` 严格约束） |

### 7.3 Replay Frame 格式

```rust
struct ReplayFrame {
    frame_number: u64,        // 帧序号
    timestamp: GameTime,      // 游戏内时间
    commands: Vec<Command>,   // 本帧的所有命令
    rng_seed: u64,            // 本帧的 RNG 种子
    checksum: u64,            // 关键状态哈希校验
}
```

### 7.4 Replay 兼容的 Schema 要求

- 所有影响回放结果的字段必须显式标记 `#[replay_key]`
- 非确定性数据（如 UUID、wall-clock）不得出现在回放关键路径
- 回放模式下禁止读取外部状态（文件系统、网络）

---

## 8. 数据目录结构

`docs/04-data/` 下的数据架构文档组织方式：

```
docs/04-data/
├── README.md                        # ← 本文档：数据架构总纲
│
├── foundation/                      # 基础架构与治理规则
│   ├── id_strategy.md               # ID 策略详述
│   ├── save_architecture.md         # 存档架构详述
│   ├── replay_architecture.md       # 回放架构详述
│   └── migration_policy.md          # 数据迁移策略
│
├── capabilities/                    # 能力领域 Schema (15 domains)
│   ├── tag_schema.md
│   ├── attribute_schema.md
│   ├── modifier_schema.md
│   ├── aggregator_schema.md
│   ├── gameplay_context_schema.md
│   ├── spec_schema.md
│   ├── condition_schema.md
│   ├── trigger_schema.md
│   ├── ability_schema.md
│   ├── targeting_schema.md
│   ├── execution_schema.md
│   ├── effect_schema.md
│   ├── stacking_schema.md
│   ├── event_schema.md
│   └── cue_schema.md
│
├── infrastructure/                  # 基础设施 Schema (4 domains)
│   ├── registry_schema.md
│   ├── pipeline_schema.md
│   ├── replay_schema.md
│   └── input_schema.md
│
├── domains/                         # 业务领域 Schema (15 domains)
│   ├── tactical_schema.md
│   ├── terrain_schema.md
│   ├── faction_schema.md
│   ├── combat_schema.md
│   ├── spell_schema.md
│   ├── reaction_schema.md
│   ├── progression_schema.md
│   ├── inventory_schema.md
│   ├── party_schema.md
│   ├── camp_rest_schema.md
│   ├── narrative_schema.md
│   ├── quest_schema.md
│   ├── economy_schema.md
│   ├── crafting_schema.md
│   └── summon_schema.md
│
├── bo3/                             # 参考：BG3 数据提取（已有）
│   └── ...
├── ll/                              # 参考：铃兰之剑数据提取（已有）
│   └── ...
└── references/                      # 外部参考与对比分析（可选）
    └── ...
```

---

## 9. 领域 Schema 依赖关系

### 9.1 能力领域依赖链

```
Tag ──────→ Condition ──────→ Trigger
  │                              │
  ├──→ Modifier ──→ Aggregator   │
  │                    │         │
  │                    ▼         ▼
  │              Attribute    Ability
  │                    │         │
  │                    │         ▼
  │                    └──→ Targeting ←── GameplayContext
  │                              │
  │                              ▼
  │                         Execution
  │                              │
  │                              ▼
  └────────────────────────→ Effect ←── Spec
                                   │
                            ┌──────┼──────┐
                            ▼      ▼      ▼
                         Stacking Cue   Event
```

### 9.2 业务领域依赖链

```
Foundation Layer:
  Tactical ←── Terrain ←── Faction
       │                      │
       ▼                      ▼
Core Layer:
  Combat ←── Spell ←── Reaction
     │
     ├──→ Progression ←── Inventory
     │
     └──→ Party ←── CampRest

Narrative Layer:
  Narrative ←── Quest

Economy Layer:
  Economy ←── Crafting ←── Summon
```

### 9.3 Schema 设计顺序

Schema 设计应遵循依赖顺序，确保前置 Schema 稳定后再设计依赖方：

1. **Phase 1**: Tag → Attribute → Modifier → Aggregator → GameplayContext
2. **Phase 2**: Spec → Condition → Trigger → Event
3. **Phase 3**: Ability → Targeting → Execution → Effect → Stacking → Cue
4. **Phase 4**: Registry → Pipeline → Replay
5. **Phase 5**: Tactical → Terrain → Faction
6. **Phase 6**: Combat → Spell → Reaction
7. **Phase 7**: Progression → Inventory
8. **Phase 8**: Party → CampRest
9. **Phase 9**: Narrative → Quest
10. **Phase 10**: Economy → Crafting → Summon

---

## 10. 角色分工与交接

| 角色 | 对数据的职责 |
|------|-------------|
| **Domain Designer** | 定义「规则是什么」— 领域概念、不变量、业务术语 |
| **Data Architect** | 定义「规则如何表达」— Schema、ID、四层分配、兼容性 |
| **Architect** | 定义「系统如何组织」— 模块边界、通信设计、ECS 组件归属 |
| **Feature Developer** | 实现「如何做」— 将 Schema 转为 Rust struct 和 ECS Component |
| **Test Guardian** | 验证「是否正确」— 回放测试、Schema 校验测试 |

### 交接触发器

- 本文档 → 可开始各领域 Schema 设计
- 各领域 Schema 完成 → 通知 @architect 进行模块设计
- Schema/架构完成 → 通知 @feature-developer 开始实现
- 实现过程中 Schema 变更 → 必须经过 @data-architect 审查

---

## 附录 A：术语对照表

| 术语 | 定义 |
|------|------|
| Definition | 静态配置数据，运行时只读，内容团队维护 |
| Spec | Definition → Instance 的桥梁，携带运行时选择与快照 |
| Instance | 运行时实例状态，ECS Component 存储 |
| Persistence | 持久化状态子集，存档 Schema |
| Schema | 数据结构的正式定义，含字段、类型、约束、默认值 |
| Data Layer | Definition / Spec / Instance / Persistence 四层之一 |
| Data Law | 数据架构不可违反的规则，违反需豁免标记 |
| Replay | 基于命令序列的确定性回放系统 |
| Save | 世界状态持久化与版本迁移系统 |
| Hot-reload | 开发期配置文件变更无需重启的实时加载 |

## 附录 B：文件状态追踪

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `README.md` | ✅ stable | data-architect | 2026-06-16 |
| `foundation/id_strategy.md` | ✅ stable | data-architect | 2026-06-16 |
| `foundation/save_architecture.md` | ✅ stable | data-architect | 2026-06-16 |
| `foundation/replay_architecture.md` | ✅ stable | data-architect | 2026-06-16 |
| `foundation/migration_policy.md` | ⬜ pending | data-architect | — |
| `capabilities/tag_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/attribute_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/modifier_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/aggregator_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/gameplay_context_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/spec_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/condition_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/trigger_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/ability_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/targeting_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/execution_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/effect_schema.md` | ✅ stable | data-architect | 2026-06-18 |
| `capabilities/stacking_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/event_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `capabilities/cue_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `infrastructure/registry_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `infrastructure/pipeline_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `infrastructure/replay_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `infrastructure/input_schema.md` | ✅ stable | data-architect | 2026-06-17 |
| `infrastructure/logging_schema.md` | ✅ stable | data-architect | 2026-06-25 |
| `domains/tactical_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/terrain_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/faction_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/combat_schema.md` | ✅ stable | data-architect | 2026-06-18 |
| `domains/spell_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/reaction_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/progression_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/inventory_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/party_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/camp_rest_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/narrative_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/quest_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/economy_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/crafting_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `domains/summon_schema.md` | ✅ stable | data-architect | 2026-06-16 |
