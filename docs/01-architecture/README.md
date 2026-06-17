---
id: 01-architecture.README
title: Architecture Overview — DDD Three-Layer + Four Cross-Cutting Layers
status: stable
owner: architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - architecture
  - governance
  - module-design
  - ecs
  - ddd
---

# Architecture Overview — Fre SRPG DDD三层+横切四层 架构总纲

> **版本**: 5.0 | **角色**: @architect | **宪法优先级**: 🟥 **最高**
> **架构依据**: `docs/00-governance/Fre项目架构设计.md`（DDD三层 + 横切四层模型）

本文档是 Fre 项目架构的最高准则。所有 Feature 边界、ECS 规则、Effect/Modifier 管线、模块间通信以本文档为最终依据。

---

## 1. 架构哲学

### 1.1 三条基石原则

| 原则 | 含义 | 违反后果 |
|------|------|----------|
| **Feature First** | 顶层模块按业务领域拆分，禁止 `components/`、`systems/`、`events/` 全局目录 | 架构评审不通过 |
| **三层架构 + 四层数据** | 运行时三层（Domain / Application / Presentation），数据四层（Def / Spec / Instance / Persistence） | 编译期拦截 |
| **Effect Pipeline 唯一入口** | 所有战斗数值变更必须经过 Effect Pipeline，禁止直接扣血、加 Buff | 运行时断言失败 |

### 1.2 复杂度治理

- 🟩 **只解决当前复杂度**：禁止为未明确的未来需求提前设计完整架构
- 🟩 **复杂度预算 > 性能预算**：每新增一个抽象层必须证明收益大于维护成本
- 🟩 **三次才抽象**：代码重复 3 次以上再提取公共抽象
- 🟩 **AI 可读性优先**：直白线性逻辑 > 宏套宏 > 深度泛型 > 类型体操

---

## 2. DDD三层+横切四层 架构总图

Fre 项目采用 **DDD 纵向三层 + 横切四层** 的矩阵式结构，源自 `docs/00-governance/Fre项目架构设计.md`。模块组织遵循**内聚优于分层**原则：同一领域的代码放在一起（内聚），而非按抽象层级拆散到不同目录。

### 2.1 DDD 纵向三层

```
┌──────────────────────────────────────────────────────────────┐
│  L0: Shared（原子层）                                         │
│  IDs / Error / Math / Random / Time / Collections / Hashing  │
│  Validation / Testing / Traits / Path                        │
│  ──── 零业务语义、零技术语义、零框架语义的通用编程原子工具 ──── │
│  依赖: 无（最底层）                                            │
├──────────────────────────────────────────────────────────────┤
│  L1: Core（领域规则层）                                        │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  capabilities/ — 15个核心能力领域（通用机制骨架）        │   │
│  │  Tag / Attribute / Modifier / Aggregator / GameplayCtx  │   │
│  │  Spec / Condition / Trigger / Ability / Targeting       │   │
│  │  Execution / Effect / Stacking / Event / Cue            │   │
│  ├────────────────────────────────────────────────────────┤   │
│  │  domains/ — 15个业务子系统（全部玩法复杂度）              │   │
│  │  Tactical / Terrain / Faction / Combat / Spell         │   │
│  │  Reaction / Progression / Inventory / Party / CampRest  │   │
│  │  Narrative / Quest / Economy / Crafting / Summon        │   │
│  └────────────────────────────────────────────────────────┘   │
│  依赖: Shared（L0）                                           │
├──────────────────────────────────────────────────────────────┤
│  L2: Infra（技术实现层）                                       │
│  Registry / Pipeline / Replay / Save / Input                 │
│  ──── 渲染/持久化/网络等"脏活" ────────────────────────────── │
│  依赖: Core（L1）+ Shared（L0）                               │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 横切四层

横切层跨越所有纵向三层，提供跨领域的基础设施：

```
┌──────────────────────────────────────────────────────────────┐
│  横切1: App（启动装配层 / Composition Root）                   │
│  唯一知道所有层的入口点。Feature-gated 启动 game/editor/headless │
│  依赖: 知道所有层                                              │
├──────────────────────────────────────────────────────────────┤
│  横切2: Content（内容桥接层 / 数据驱动核心）                    │
│  从 assets/config/ 加载配置 → 校验 → 注册到 Registry           │
│  依赖: Core + Infra（只做加载/校验/注册）                      │
├──────────────────────────────────────────────────────────────┤
│  横切3: Tools（开发工具层，feature-gated）                      │
│  Debug 面板 / 性能分析 / 热重载控制台                           │
│  依赖: 所有层（仅 dev 构建）                                    │
├──────────────────────────────────────────────────────────────┤
│  横切4: Modding（Mod 扩展层，跨层聚合）                        │
│  Mod 加载沙箱 / Mod API 稳定层 / 版本兼容检查                   │
│  依赖: Mod API（src/core/mod_api/）                           │
└──────────────────────────────────────────────────────────────┘
```

### 2.3 依赖方向（严格单向，禁止反向）

```
Shared ──→ Core ──→ Infra       # 纵向依赖：低层→高层（单向）
                                        #
Capabilities ──→ Domains          # Domain 引用 Capabilities（禁止反向）
  │              │                #
  │              └──→ 事件通信     # Domains 之间仅通过 Event 通信
  │                              #
Domains ──→ Domains               # 禁止直接引用，仅 Event
  (仅 Event)                      #
                                        #
App      ──→ 所有层（唯一 Composition Root）
Content  ──→ Core + Infra（只做加载/校验/注册）
Tools    ──→ 所有层（仅 dev）
Modding  ──→ mod_api（稳定 API 层）
```

### 2.4 与运行时三层（宪法）的关系

宪法（`.trae/rules/架构规则.md`）定义的 Domain / Application / Presentation 三层是**运行时职责视角**，DDD三层+横切四层是**模块组织视角**。两者共存不矛盾：

```
DDD三层+横切四层（模块组织）      宪法三层（运行时职责）
─────────────────────            ─────────────────
L2 Infra + 横切 Content           ┌─ Presentation (UI, VFX, SFX, Camera)
L1 Core/Domains 的叙事层         │
  └─ 数据/UI 层                 │
L1 Core 的 ECS Systems           ├─ Application (ECS Systems, Schedules, States)
  └─ System 层                  │
L1 Core 的 rules/                └─ Domain (Pure Functions, Formulas, Rules)
L0 Shared                         └─ 计算层
  └─ 计算层
```

> **结论**：模块按 DDD三层+横切四层 组织，每层内部按宪法三层的运行时职责分离代码。

---

## 3. 模块映射

### 3.1 L0: Shared — 原子层

| 模块 | 路径 | 对应领域文档 | 对应 Schema |
|------|------|-------------|-------------|
| IDs | `src/shared/ids/` | — | `foundation/id_strategy.md` |
| Error | `src/shared/error/` | — | — |
| Math | `src/shared/math/` | — | — |
| Random | `src/shared/random/` | — | — |
| Time | `src/shared/time/` | — | — |
| Collections | `src/shared/collections/` | — | — |
| Hashing | `src/shared/hashing/` | — | — |
| Validation | `src/shared/validation/` | — | — |
| Testing | `src/shared/testing/` | — | — |
| Traits | `src/shared/traits/` | — | — |
| Prelude | `src/shared/prelude/` | — | — |
| Path | `src/shared/path/` | — | — |

**核心职责**：零业务语义、零技术语义的通用编程原子工具。

### 3.2 L1: Core — Capabilities（15个核心能力领域）

所有能力领域位于 `src/core/capabilities/<domain>/`，每个领域内部按 C1 Foundation → C2 Mechanism → C3 Runtime 自包含组织。

| 模块 | 领域文档 | 数据 Schema | C1 Foundation | C2 Mechanism |
|------|---------|------------|--------------|--------------|
| `tag` | `tag_domain.md` | `capabilities/tag_schema.md` | tag_id, tag_set, tag_hierarchy | components, query, systems |
| `attribute` | `attribute_domain.md` | `capabilities/attribute_schema.md` | attribute_id, value, category | components, systems |
| `modifier` | `modifier_domain.md` | `capabilities/modifier_schema.md` | modifier_op, data, scalable_value | components, systems |
| `aggregator` | `aggregator_domain.md` | `capabilities/aggregator_schema.md` | calc_stage, snapshot | components, calc_pipeline, systems |
| `gameplay_context` | `gameplay_context_domain.md` | `capabilities/gameplay_context_schema.md` | context_data | components, context_builder, context_chain, systems |
| `spec` | `spec_domain.md` | `capabilities/spec_schema.md` | ability_spec, effect_spec | components, spec_registry, systems |
| `condition` | `condition_domain.md` | `capabilities/condition_schema.md` | condition_type, tag_requirement, attribute_check, resource_check | components, systems |
| `trigger` | `trigger_domain.md` | `capabilities/trigger_schema.md` | trigger_type, trigger_condition | components, systems |
| `ability` | `ability_domain.md` | `capabilities/ability_schema.md` | ability_state, instance, cost, cooldown | components, ability_task, systems |
| `targeting` | `targeting_domain.md` | `capabilities/targeting_schema.md` | target_type, target_data | components, selector, grid_targeting, systems |
| `execution` | `execution_domain.md` | `capabilities/execution_schema.md` | execution_type, context, custom_execution | components, damage/heal_execution, systems |
| `effect` | `effect_domain.md` | `capabilities/effect_schema.md` | effect_duration, period, modifiers, tags | components, effect_lifecycle, systems |
| `stacking` | `stacking_domain.md` | `capabilities/stacking_schema.md` | stacking_type, rule, limit | components, systems |
| `event` | `event_domain.md` | `capabilities/event_schema.md` | gameplay_event, event_type | event_bus, subscription, systems |
| `cue` | `cue_domain.md` | `capabilities/cue_schema.md` | cue_type, cue_data, cue_tag | components, systems |

**C3 Runtime**（跨领域编排底座）: `src/core/capabilities/runtime/` — pipeline, scheduler, registry, command, replay

**核心职责**：贯穿所有游戏机制的"魔法圈" — 从 Tag 分类到 Effect 执行到 Cue 表现的完整链路。

### 3.3 L1: Core — Business Domains（15个业务子系统）

所有业务领域位于 `src/core/domains/<domain>/`，标准内部结构：`plugin.rs + components.rs + systems/ + events.rs + error.rs + rules/ + integration.rs`

| 模块 | 领域文档 | 数据 Schema | 核心职责 |
|------|---------|------------|---------|
| `tactical` | `tactical_domain.md` | `domains/tactical_schema.md` | 网格位置、移动、掩体、夹击 |
| `terrain` | `terrain_domain.md` | `domains/terrain_schema.md` | Tile、表面类型、陷阱、通行性 |
| `faction` | `faction_domain.md` | `domains/faction_schema.md` | 阵营关系、声望、关系判定 |
| `combat` | `combat_domain.md` | `domains/combat_schema.md` | 回合流程、先攻、伤害结算、胜负 |
| `spell` | `spell_domain.md` | `domains/spell_schema.md` | 法术位、专注、豁免、升环 |
| `reaction` | `reaction_domain.md` | `domains/reaction_schema.md` | 机会攻击、法术反制、护盾、援护 |
| `progression` | `progression_domain.md` | `domains/progression_schema.md` | 经验、等级、职业、天赋、ASI |
| `inventory` | `inventory_domain.md` | `domains/inventory_schema.md` | 物品、装备槽位、消耗品、战利品 |
| `party` | `party_domain.md` | `domains/party_schema.md` | 成员名册、羁绊、阵型、换人 |
| `camp_rest` | `camp_rest_domain.md` | `domains/camp_rest_schema.md` | 短休、长休、生命骰、营地事件 |
| `narrative` | `narrative_domain.md` | `domains/narrative_schema.md` | 对话树、StoryFlag、演出 |
| `quest` | `quest_domain.md` | `domains/quest_schema.md` | 目标追踪、奖励、前置条件 |
| `economy` | `economy_domain.md` | `domains/economy_schema.md` | 货币、商店、价格、交易 |
| `crafting` | `crafting_domain.md` | `domains/crafting_schema.md` | 配方、附魔、装备升级 |
| `summon` | `summon_domain.md` | `domains/summon_schema.md` | 召唤物模板、专注绑定、消失 |

### 3.4 L2: Infra — 技术实现层

基础设施位于 `src/infra/<domain>/`：

| 模块 | 数据 Schema | 核心职责 |
|------|------------|---------|
| `registry` | `infrastructure/registry_schema.md` | ID 注册、冲突检测、热重载 |
| `pipeline` | `infrastructure/pipeline_schema.md` | 通用执行管线引擎 |
| `replay` | `infrastructure/replay_schema.md` | 命令录制、确定性回放 |
| `save` | `foundation/save_architecture.md` | 存档序列化、版本迁移 |
| `input` | — | 输入抽象、命令层 |

### 3.5 横切四层

| 层 | 路径 | 核心职责 |
|----|------|---------|
| App | `src/app/` | Composition Root，根据 feature flag 启动 game/editor/headless |
| Content | `src/content/` | 配置加载、校验、注册（assets/config/） |
| Tools | `src/tools/` (dev feature-gated) | Debug 面板、性能分析、热重载控制台 |
| Modding | `src/modding/` | Mod 加载沙箱、Mod API 层 |

---

## 4. 运行时 ECS 架构

### 4.1 数据四层映射到 ECS

| 数据层 | ECS 映射 | 位置 | 可变性 |
|--------|---------|------|--------|
| **Definition** | `Asset<…Def>` (Bevy Asset) | `assets/config/` | 只读，热重载 |
| **Spec** | ECS Component | `src/*/components.rs` | 运行时可变 |
| **Instance** | ECS Component | `src/*/components.rs` | 每 Entity 可变 |
| **Persistence** | Serialized Schema | Save/Replay Files | 序列化时生成 |

详见 `docs/04-data/README.md` 第 2 节。

### 4.2 四级通信机制

| 机制 | 用途 | 生命周期 | 示例 |
|------|------|---------|------|
| **Hook** | Component 添加/移除的轻量副作用 | `#[component(on_add, on_remove)]` | `Dead` Tag 添加时移除移动能力 |
| **Trigger** | 同 Feature 内事件链响应 | `commands.trigger()` + Observer | 伤害 → 护盾 → 吸血 |
| **Observer** | Component 状态变化响应 | `on_event::<T>()` | 血量变化刷新 UI |
| **Message** | 跨 Feature 全局广播 | `EventWriter<T>` / `EventReader<T>` | 回合结束 → 任务检查 |

> 🟥 **禁止**将模块内普通逻辑全部事件化，禁止用 Event 模拟函数调用。

### 4.3 Schedule 权责划分

| Schedule | 职责 | 典型 System |
|----------|------|-------------|
| `PreUpdate` | 输入处理、命令入队、状态同步 | `input_collector`, `command_executor` |
| `Update` | 核心业务逻辑、管线执行 | `combat_pipeline`, `effect_applier`, `movement_solver` |
| `PostUpdate` | 事件响应、表现更新、UI 刷新 | `hp_bar_updater`, `animation_player` |
| `OnEnter(X)` | 进入状态 X 时的初始化 | `on_enter_battle`, `on_enter_turn` |

---

## 5. 管线架构

### 5.1 Effect Pipeline — 战斗效果唯一入口

```
Ability ──→ Execution ──→ Effect
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
               Modifier    Cue       Trigger
                    │         │         │
                    ▼         ▼         ▼
              Attribute    Visual    Observer
              Resolver     System    Chain
```

- 🟥 **禁止**绕过 Effect Pipeline 直接修改战斗数值
- 🟥 **禁止**Ability 直接生成 Modifier（必须经过 Effect）
- 🟩 Effect 是唯一的业务执行入口（Data Law 005）

### 5.2 Modifier Pipeline — 属性修改唯一入口

```
ModifierAdded ──→ ModifierCollector ──→ Aggregator ──→ AttributeResolver ──→ FinalStat
                                                              │
                                                         Changed Filter
                                                              │
                                                              ▼
                                                     Observer Chain
```

- 🟥 **禁止**直接修改最终属性值（必须通过添加/移除 Modifier）
- 🟩 HP/MP 等资源型数值允许专用系统直接修改（SRPG 规则例外）

### 5.3 Combat Pipeline — 伤害/治疗计算流程

```
CombatIntent ──→ Generate ──→ Modify ──→ Execute ──→ Resolve
     │              │            │           │           │
     │              │            │           │           └──→ Damage/Heal
     │              │            │           │           └──→ BuffApply
     │              │            │           │           └──→ ReactionTrigger
     │              │            │           │
     │              │            │           └──→ Cue
     │              │            │
     │              │            └──→ Modifier Overrides
     │              │
     │              └──→ Randomness Roll
     │
     └──→ Target Validation
```

---

## 6. Plugin 组合与注册顺序

### 6.1 Plugin 注册顺序

Plugin 按层从底向上注册，确保下层 Asset/Resource 在上层 System 执行前就绪：

```rust
// ════════════════════════════════════════════
// Phase 0: Core Bevy + Shared (L0)
// ════════════════════════════════════════════
.add_plugins(DefaultPlugins)
.add_plugins(shared::SharedPlugin)

// ════════════════════════════════════════════
// Phase 1: Capabilities — Foundation (L1 Core)
// ════════════════════════════════════════════
.add_plugins(core::capabilities::tag::TagPlugin)
.add_plugins(core::capabilities::attribute::AttributePlugin)
.add_plugins(core::capabilities::modifier::ModifierPlugin)
.add_plugins(core::capabilities::aggregator::AggregatorPlugin)
.add_plugins(core::capabilities::gameplay_context::GameplayContextPlugin)

// ════════════════════════════════════════════
// Phase 2: Capabilities — Logic Skeleton (L1 Core)
// ════════════════════════════════════════════
.add_plugins(core::capabilities::spec::SpecPlugin)
.add_plugins(core::capabilities::condition::ConditionPlugin)
.add_plugins(core::capabilities::trigger::TriggerPlugin)
.add_plugins(core::capabilities::event::EventPlugin)

// ════════════════════════════════════════════
// Phase 3: Capabilities — Behavior (L1 Core)
// ════════════════════════════════════════════
.add_plugins(core::capabilities::ability::AbilityPlugin)
.add_plugins(core::capabilities::targeting::TargetingPlugin)
.add_plugins(core::capabilities::execution::ExecutionPlugin)
.add_plugins(core::capabilities::effect::EffectPlugin)
.add_plugins(core::capabilities::stacking::StackingPlugin)
.add_plugins(core::capabilities::cue::CuePlugin)

// ════════════════════════════════════════════
// Phase 4: Capabilities — Runtime (L1 Core)
// ════════════════════════════════════════════
.add_plugins(core::capabilities::runtime::RuntimePlugin)

// ════════════════════════════════════════════
// Phase 5: Business Domains — Foundation (L1 Core)
// ════════════════════════════════════════════
.add_plugins(core::domains::tactical::TacticalPlugin)
.add_plugins(core::domains::terrain::TerrainPlugin)
.add_plugins(core::domains::faction::FactionPlugin)

// ════════════════════════════════════════════
// Phase 6: Business Domains — Core (L1 Core)
// ════════════════════════════════════════════
.add_plugins(core::domains::combat::CombatPlugin)
.add_plugins(core::domains::spell::SpellPlugin)
.add_plugins(core::domains::reaction::ReactionPlugin)
.add_plugins(core::domains::progression::ProgressionPlugin)
.add_plugins(core::domains::inventory::InventoryPlugin)
.add_plugins(core::domains::party::PartyPlugin)
.add_plugins(core::domains::camp_rest::CampRestPlugin)

// ════════════════════════════════════════════
// Phase 7: Business Domains — Narrative & Economy (L1 Core)
// ════════════════════════════════════════════
.add_plugins(core::domains::narrative::NarrativePlugin)
.add_plugins(core::domains::quest::QuestPlugin)
.add_plugins(core::domains::economy::EconomyPlugin)
.add_plugins(core::domains::crafting::CraftingPlugin)
.add_plugins(core::domains::summon::SummonPlugin)

// ════════════════════════════════════════════
// Phase 8: Infrastructure (L2)
// ════════════════════════════════════════════
.add_plugins(infra::registry::RegistryPlugin)
.add_plugins(infra::pipeline::PipelinePlugin)
.add_plugins(infra::replay::ReplayPlugin)
.add_plugins(infra::save::SavePlugin)
.add_plugins(infra::input::InputPlugin)

// ════════════════════════════════════════════
// Phase 9: Cross-cutting
// ════════════════════════════════════════════
.add_plugins(app::AppPlugin)          // Composition Root
.add_plugins(content::ContentPlugin)  // Data Bridge
#[cfg(feature = "dev")]
.add_plugins(tools::DevToolsPlugin)
.add_plugins(modding::ModdingPlugin)
```

### 6.2 Plugin 内部结构

每个领域模块遵循统一内部结构。Capabilities 使用 C1→C2→C3 三层内聚结构，Business Domains 使用标准 7 文件结构。

**Capabilities 领域结构**（位于 `src/core/capabilities/<domain>/`）：

```
capabilities/<domain>/
├── plugin.rs              # 领域 Plugin（唯一对外入口）
├── foundation/            # C1：纯数据定义层（无行为逻辑）
│   ├── mod.rs
│   ├── types.rs           # 基础类型与枚举
│   └── values.rs          # 值对象定义
├── mechanism/             # C2：规则与系统层
│   ├── mod.rs
│   ├── components.rs      # ECS 组件
│   ├── query.rs           # 查询/匹配/条件逻辑
│   ├── lifecycle.rs       # 生命周期管理
│   └── systems/           # Bevy Systems
│       ├── mod.rs
│       └── xxx_system.rs
└── events.rs              # 领域事件
```

**Business Domains 结构**（位于 `src/core/domains/<domain>/`）：

```
domains/<domain>/
├── plugin.rs          # 唯一对外入口
├── components.rs      # ECS Components
├── systems/           # 业务系统
│   ├── mod.rs
│   ├── xxx_system.rs
│   └── yyy_system.rs
├── events.rs          # 对外发布的领域事件
├── error.rs           # 专属错误枚举
├── rules/             # 纯业务规则（纯函数，零 ECS 依赖）
│   ├── formulas.rs
│   └── rules.rs
└── integration.rs     # 唯一调用 Capabilities 的入口
```

- 🟥 Plugin 是唯一对外入口，禁止外部直接访问 `internal/`
- 🟩 对外暴露的 API 集中在 `api.rs`

---

## 7. 跨 Feature 通信规范

### 7.1 通信矩阵

| 场景 | 推荐机制 | 理由 |
|------|---------|------|
| 同 Feature 内事件链 | Trigger + Observer | 轻量，绑定 Entity |
| 同 Feature 组件变化 | Changed Filter | 零开销 |
| 跨 Feature 状态变更 | Message (Event) | 解耦 |
| Component 生命周期 | Hook | 声明式 |
| 跨 Feature 读操作 | 公开 Query API | 不修改状态 |

### 7.2 域间禁止直接数据引用

- 🟥 **Data Law 012**：Domain 之间禁止直接引用对方的数据结构
- 🟩 跨域通信仅通过 Event/Message
- 🟩 只读查询允许通过 `api.rs` 暴露的公开函数

### 7.3 事件白名单管理

所有跨 Feature Event 必须在目标 Feature 的 `events.rs` 中声明并登记到事件白名单：

```rust
// events.rs — 白名单登记
/// 跨 Feature 事件清单（用于审计和回放）
pub enum WhitelistedEvent {
    TurnPhaseChanged(TurnPhase),
    CombatExecuted(CombatResult),
    UnitDied(Entity),
    QuestProgressed(QuestId),
    ItemAcquired(ItemId),
    GoldChanged(i64),
    LeveledUp(Entity),
}
```

- 🟩 所有领域事件是业务事实源，日志、回放、UI 均为下游消费者

---

## 8. 目录结构总览

```
src/
├── main.rs                   # 程序入口，根据 feature 启动 game/editor/headless
├── lib.rs                    # 库根，导出各层公共接口
│
│                           ┌─ DDD 纵向三层 ─┐
├── shared/                   # L0：零业务语义的通用工具
│   ├── mod.rs
│   ├── ids/                  # 强类型 ID（UnitId, SkillId, BuffId, ItemId, QuestId）
│   ├── error/                # 错误上下文工具
│   ├── math/                 # 纯数学工具（距离/插值/网格坐标）
│   ├── random/               # 确定性随机数 Trait
│   ├── time/                 # GameTime, TurnCount
│   ├── collections/          # 通用集合扩展
│   ├── hashing/              # 非加密高速哈希
│   ├── validation/           # 链式校验器
│   ├── testing/              # 测试构建工具
│   ├── traits/               # 横切能力抽象（日志/审计/事务）
│   └── prelude/              # 统一导出
│
├── core/                     # L1：领域规则层
│   ├── mod.rs
│   ├── core_plugin.rs        # Core 层总 Plugin
│   │
│   ├── capabilities/         # 15个核心能力领域
│   │   ├── tag/              # 标签体系
│   │   ├── attribute/        # 属性体系
│   │   ├── modifier/         # 数值修改器
│   │   ├── aggregator/       # 属性聚合管线
│   │   ├── gameplay_context/ # 统一上下文/载荷
│   │   ├── spec/             # 配置槽位（Def→Instance 桥梁）
│   │   ├── condition/        # 条件检查（免疫/限制/激活）
│   │   ├── trigger/          # 技能激活条件
│   │   ├── ability/          # 技能逻辑与生命周期
│   │   ├── targeting/        # 目标选择（含 Grid 范围）
│   │   ├── execution/        # 执行计算（伤害/治疗/自定义）
│   │   ├── effect/           # 效果（Period/Duration/Instant）
│   │   ├── stacking/         # 堆叠规则
│   │   ├── event/            # 系统间结构化通信
│   │   ├── cue/              # 表现层信号（VFX/SFX/Anim）
│   │   └── runtime/          # C3：跨领域运行时编排
│   │       ├── pipeline/
│   │       ├── scheduler/
│   │       ├── registry/
│   │       ├── command/
│   │       └── replay/
│   │
│   ├── domains/              # 15个业务子系统
│   │   ├── tactical/         # 战术空间（网格/移动/掩体）
│   │   ├── terrain/          # 地形（毒池/冰面/高地）
│   │   ├── faction/          # 阵营关系
│   │   ├── combat/           # 战斗全流程
│   │   ├── spell/            # 法术系统
│   │   ├── reaction/         # 反应/援护
│   │   ├── progression/      # 经验/等级/职业
│   │   ├── inventory/        # 背包/物品
│   │   ├── party/            # 队伍/羁绊
│   │   ├── camp_rest/        # 营地/休息
│   │   ├── narrative/        # 叙事/对话
│   │   ├── quest/            # 任务系统
│   │   ├── economy/          # 经济/商店
│   │   ├── crafting/         # 制造/附魔
│   │   └── summon/           # 召唤物管理
│   │
│   └── mod_api/              # Mod 稳定 API
│
├── infra/                    # L2：技术实现层
│   ├── mod.rs
│   ├── registry/             # ID 注册与热重载
│   ├── pipeline/             # 管线执行引擎
│   ├── replay/               # 回放系统
│   ├── save/                 # 存档系统
│   └── input/                # 输入抽象
│                           └─────────────────┘
│                           ┌─ 横切四层 ─┐
├── app/                      # 横切1：启动装配层（Composition Root）
├── content/                  # 横切2：内容桥接层（数据驱动核心）
├── tools/                    # 横切3：开发工具层（feature-gated）
└── modding/                  # 横切4：Mod 扩展层（跨层聚合）
                            └─────────────┘
```

> 🟥 **绝对禁止**在 `src/` 下创建 `components.rs`、`systems.rs`、`events.rs` 等全局技术文件。

---

## 9. 架构决策索引

所有 Architecture Decision Record (ADR) 保存在 `docs/01-architecture/` 的子目录中。

| 编号 | 标题 | 状态 | 所属领域 |
|------|------|------|---------|
| ADR-000 | Feature 模块划分总图 | ✅ Approved | Foundation |
| ADR-001 | Plugin 组合与注册顺序 | ✅ Approved | Foundation |
| ADR-002 | ECS 四级通信机制选型 | ✅ Approved | Foundation |
| ADR-010 | Ability → Effect 管线架构 | ✅ Approved | Capability |
| ADR-011 | Modifier → Attribute 管线架构 | ✅ Approved | Capability |
| ADR-012 | Stacking / Trigger / Cue 分离 | ✅ Approved | Capability |
| ADR-013 | Registry 与热重载架构 | ✅ Approved | Capability |
| ADR-020 | CombatIntent Pipeline 设计 | ✅ Approved | Combat |
| ADR-021 | 回合状态机设计 | ✅ Approved | Combat |
| ADR-022 | 网格 / 地形 / 阵营系统设计 | ✅ Approved | Combat |
| ADR-023 | 法术与反应机制设计 | ✅ Approved | Combat |
| ADR-030 | 成长 / 物品系统设计 | ✅ Approved | Progression |
| ADR-031 | 队伍与休整系统设计 | ✅ Approved | Progression |
| ADR-032 | 经济与制造系统设计 | ✅ Approved | Progression |
| ADR-033 | 叙事 / 任务 / 召唤设计 | ✅ Approved | Progression |
| ADR-040 | 数据流与所属权策略 | ✅ Approved | Cross-cutting |
| ADR-041 | 回放确定性与架构 | ✅ Approved | Cross-cutting |
| ADR-042 | 存档持久化策略 | ✅ Approved | Cross-cutting |
| ADR-043 | 命令层与输入抽象 | ✅ Approved | Cross-cutting |
| ADR-044 | Pipeline 引擎架构 | ✅ Approved | Cross-cutting |
| ADR-045 | 模块可见性策略（可见性宪法） | 📋 Proposed | Foundation |

---

## 10. 架构合规性检查清单

每个 ADR 提交时必须逐项检查：

- [ ] 符合 Feature First 原则（禁止全局技术目录）
- [ ] 符合三层运行时分离（Domain / Application / Presentation）
- [ ] 符合 DDD三层+横切四层层间依赖方向（禁止反向依赖：Shared ← Core ← Infra）
- [ ] Effect Pipeline 没有被绕过
- [ ] Modifier Pipeline 没有被绕过
- [ ] 定义了明确的 Forbidden 事项
- [ ] 引用了上游领域规则和数据 Schema
- [ ] Plugin 注册顺序符合层次要求
- [ ] 通信机制选择符合四级通信规范
- [ ] 符合 Data Law（`docs/04-data/README.md` 第 5 节）

---

## 附录 A：与上游文档的追溯

| 上游输入 | 位置 | 在本架构中的使用 |
|---------|------|----------------|
| 🟥 **架构设计最高依据** | `docs/00-governance/Fre项目架构设计.md` | DDD三层+横切四层模型、目录结构、依赖方向 |
| 领域规则 | `docs/02-domain/` (30 文件) | 每个 Feature 模块对应一个或多个领域文档 |
| 数据 Schema | `docs/04-data/` (33+ 文件) | 每个 Feature 的数据结构定义依据 |
| 架构规则 | `.trae/rules/架构规则.md` | 宪法的 Feature First、三层架构、依赖方向 |
| ECS 规则 | `.trae/rules/ECS规则.md` | 四级通信机制、Schedule 权责 |
| SRPG 规则 | `.trae/rules/SRPG专项规则.md` | 战斗管线、回合状态机、Buff 生命周期 |
| 编码规则 | `.trae/rules/编码规则.md` | AI 行为规范、自检清单 |

## 附录 B：文件状态追踪

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `README.md` | ✅ stable | architect | 2026-06-16 |
| `00-foundation/ADR-000-feature-module-map.md` | ✅ stable | architect | 2026-06-16 |
| `00-foundation/ADR-001-plugin-composition.md` | ✅ stable | architect | 2026-06-16 |
| `00-foundation/ADR-002-ecs-communication.md` | ✅ stable | architect | 2026-06-16 |
| `10-capability-system/ADR-010-ability-pipeline.md` | ✅ stable | architect | 2026-06-16 |
| `10-capability-system/ADR-011-modifier-pipeline.md` | ✅ stable | architect | 2026-06-16 |
| `10-capability-system/ADR-012-stacking-trigger-cue.md` | ✅ stable | architect | 2026-06-16 |
| `10-capability-system/ADR-013-registry-hotreload.md` | ✅ stable | architect | 2026-06-16 |
| `20-tactical-combat/ADR-020-combat-pipeline.md` | ✅ stable | architect | 2026-06-16 |
| `20-tactical-combat/ADR-021-turn-state-machine.md` | ✅ stable | architect | 2026-06-16 |
| `20-tactical-combat/ADR-022-grid-terrain-faction.md` | ✅ stable | architect | 2026-06-16 |
| `20-tactical-combat/ADR-023-spell-reaction.md` | ✅ stable | architect | 2026-06-16 |
| `30-progression-narrative/ADR-030-progression-inventory.md` | ✅ stable | architect | 2026-06-16 |
| `30-progression-narrative/ADR-031-party-camp-rest.md` | ✅ stable | architect | 2026-06-16 |
| `30-progression-narrative/ADR-032-economy-crafting.md` | ✅ stable | architect | 2026-06-16 |
| `30-progression-narrative/ADR-033-narrative-quest-summon.md` | ✅ stable | architect | 2026-06-16 |
| `40-cross-cutting/ADR-040-data-flow-ownership.md` | ✅ stable | architect | 2026-06-16 |
| `40-cross-cutting/ADR-041-replay-determinism.md` | ✅ stable | architect | 2026-06-16 |
| `40-cross-cutting/ADR-042-save-persistence.md` | ✅ stable | architect | 2026-06-16 |
| `40-cross-cutting/ADR-043-command-input.md` | ✅ stable | architect | 2026-06-16 |
| `40-cross-cutting/ADR-044-pipeline-engine.md` | ✅ stable | architect | 2026-06-17 |
| `00-foundation/ADR-045-module-visibility-strategy.md` | 📋 Proposed | architect | 2026-06-17 |

---

*本文档是 Fre 项目架构的最高准则，所有 Feature 开发、数据设计、代码审查以此为最终依据。修改本文档需经过 @architect 审核。*
