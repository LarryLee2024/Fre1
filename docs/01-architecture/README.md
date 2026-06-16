---
id: 01-architecture.README
title: Architecture Overview — Seven-Layer Architecture
status: stable
owner: architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - architecture
  - governance
  - module-design
  - ecs
---

# Architecture Overview — Fre SRPG 七层架构总纲

> **版本**: 4.0 | **角色**: @architect | **宪法优先级**: 🟥 **最高**

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

## 2. 七层架构总图

Fre 项目按照**垂直分层 + 水平 Feature** 的矩阵式结构组织。七层从底向上堆叠，**禁止上层依赖下层，绝对禁止反向依赖**。

```
┌──────────────────────────────────────────────────────────────────────┐
│   Layer 7: Infrastructure & Cross-cutting                            │
│   Registry / Replay / Save / Input / Logging / Error                 │
│   ──── 依赖: 所有下层 ────────────────────────────────────────────── │
├──────────────────────────────────────────────────────────────────────┤
│   Layer 6: Narrative & Content                                       │
│   Story / Dialogue / Quest / WorldState                              │
│   ──── 依赖: Layer 2, 4, 5 ──────────────────────────────────────── │
├──────────────────────────────────────────────────────────────────────┤
│   Layer 5: Party & Camp                                              │
│   PartyManagement / Formation / CampRest / Relationship              │
│   ──── 依赖: Layer 2, 4 ─────────────────────────────────────────── │
├──────────────────────────────────────────────────────────────────────┤
│   Layer 4: Progression & Economy                                     │
│   Leveling / SkillUnlock / Inventory / Economy / Crafting            │
│   ──── 依赖: Layer 2 ────────────────────────────────────────────── │
├──────────────────────────────────────────────────────────────────────┤
│   Layer 3: Combat Execution                                          │
│   CombatIntent Pipeline / SpellCasting / Reactions / BuffLifecycle   │
│   ──── 依赖: Layer 2 ────────────────────────────────────────────── │
├──────────────────────────────────────────────────────────────────────┤
│   Layer 2: Capability System                                         │
│   Tag / Attribute / Modifier / Effect / Ability / Cue / Stacking     │
│   ──── 依赖: Layer 1 ────────────────────────────────────────────── │
├──────────────────────────────────────────────────────────────────────┤
│   Layer 1: Tactical Foundation                                       │
│   Grid/Hex Map / Movement / TurnStateMachine / Faction / Terrain     │
│   ──── 依赖: 无 ─────────────────────────────────────────────────── │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.1 层间依赖铁则

| 规则 | 说明 |
|------|------|
| 🟥 禁止 | 下层依赖上层（如 Layer 2 引用 Layer 3 的类型） |
| 🟥 禁止 | 跨层跳过（如 Layer 1 直接导入 Layer 4） |
| 🟩 允许 | 上层依赖下层（Layer 5 使用 Layer 2 的 Effect 系统） |
| 🟩 允许 | 同层跨 Feature 通信（仅通过 Event/Message，禁止直接引用类型） |
| 🟨 例外 | Infrastructure Layer（Layer 7）对所有层可见，但业务层不依赖其内部实现 |

### 2.2 与宪法三层的关系

宪法（`.trae/rules/架构规则.md`）定义的三层架构（Domain / Application / Presentation）是**运行时视角**，七层架构是**模块组织视角**。两者共存不矛盾：

```
七层视角（模块组织）         三层视角（运行时职责）
─────────────────            ─────────────────
Layer 6, 5, 4                ┌─ Presentation (UI, VFX, SFX, Camera)
  └─ UI 层                  │
Layer 3, 4, 5, 6 的 ECS      ├─ Application (ECS Systems, Schedules, States)
  └─ System 层              │
Layer 1, 2 的领域逻辑         └─ Domain (Pure Functions, Formulas, Rules)
  └─ 计算层
Layer 7                      跨层基础设施
```

> **结论**：Feature 按七层组织，每层内部按 Domain / Application / Presentation 分离。

---

## 3. Feature 模块映射

### 3.1 地基层 (Layer 1) — Tactical Foundation

| Feature 模块 | 对应领域文档 | 对应 Schema |
|-------------|-------------|-------------|
| `grid_map` | `tactical_domain.md` | `tactical_schema.md` |
| `terrain` | `terrain_domain.md` | `terrain_schema.md` |
| `faction` | `faction_domain.md` | `faction_schema.md` |
| `turn_phase` | `tactical_domain.md` | `tactical_schema.md` |

**核心职责**：网格/六边形地图、寻路、移动、地形效果、阵营关系、回合阶段状态机。

**ECS 表达**：
- `GridMap` Resource / `Tile` Entity + Component
- `TurnPhase` State / `TurnQueue` Resource
- `Faction` Component (Tag) / `FactionRelation` Resource

### 3.2 能力系统层 (Layer 2) — Capability System

| Feature 模块 | 对应领域文档 | 对应 Schema |
|-------------|-------------|-------------|
| `tag` | `tag_domain.md` | `tag_schema.md` |
| `attribute` | `attribute_domain.md` | `attribute_schema.md` |
| `modifier` | `modifier_domain.md` | `modifier_schema.md` |
| `aggregator` | `aggregator_domain.md` | `aggregator_schema.md` |
| `gameplay_context` | `gameplay_context_domain.md` | `gameplay_context_schema.md` |
| `spec` | `spec_domain.md` | `spec_schema.md` |
| `condition` | `condition_domain.md` | `condition_schema.md` |
| `trigger` | `trigger_domain.md` | `trigger_schema.md` |
| `ability` | `ability_domain.md` | `ability_schema.md` |
| `targeting` | `targeting_domain.md` | `targeting_schema.md` |
| `execution` | `execution_domain.md` | `execution_schema.md` |
| `effect` | `effect_domain.md` | `effect_schema.md` |
| `stacking` | `stacking_domain.md` | `stacking_schema.md` |
| `event` | `event_domain.md` | `event_schema.md` |
| `cue` | `cue_domain.md` | `cue_schema.md` |

**核心职责**：贯穿所有游戏机制的"魔法圈" — 从 Tag 分类到 Effect 执行到 Cue 表现的完整链路。

**ECS 表达**：15 个 Feature 均映射为独立 Plugin + Components + Systems。

### 3.3 战斗执行层 (Layer 3) — Combat Execution

| Feature 模块 | 对应领域文档 | 对应 Schema |
|-------------|-------------|-------------|
| `combat` | `combat_domain.md` | `combat_schema.md` |
| `spell` | `spell_domain.md` | `spell_schema.md` |
| `reaction` | `reaction_domain.md` | `reaction_schema.md` |

**核心职责**：伤害/治疗计算、法术施放流程、反击/护盾/吸血等反应机制、Buff 生命周期。

### 3.4 成长与经济层 (Layer 4) — Progression & Economy

| Feature 模块 | 对应领域文档 | 对应 Schema |
|-------------|-------------|-------------|
| `progression` | `progression_domain.md` | `progression_schema.md` |
| `inventory` | `inventory_domain.md` | `inventory_schema.md` |
| `economy` | `economy_domain.md` | `economy_schema.md` |
| `crafting` | `crafting_domain.md` | `crafting_schema.md` |
| `summon` | `summon_domain.md` | `summon_schema.md` |

**核心职责**：经验升级、技能解锁、物品装备、商店经济、制造系统、召唤物管理。

### 3.5 队伍与休整层 (Layer 5) — Party & Camp

| Feature 模块 | 对应领域文档 | 对应 Schema |
|-------------|-------------|-------------|
| `party` | `party_domain.md` | `party_schema.md` |
| `camp_rest` | `camp_rest_domain.md` | `camp_rest_schema.md` |

**核心职责**：队伍编成、阵型系统、休整/露营、角色间关系。

### 3.6 叙事与内容层 (Layer 6) — Narrative & Content

| Feature 模块 | 对应领域文档 | 对应 Schema |
|-------------|-------------|-------------|
| `narrative` | `narrative_domain.md` | `narrative_schema.md` |
| `quest` | `quest_domain.md` | `quest_schema.md` |

**核心职责**：故事状态机、对话分支、任务追踪与奖励、世界状态标记。

### 3.7 基础设施与横切层 (Layer 7) — Infrastructure & Cross-cutting

| Feature 模块 | 对应领域文档 | 对应 Schema |
|-------------|-------------|-------------|
| `registry` | — | `registry_schema.md` |
| `pipeline` | — | `pipeline_schema.md` |
| `replay` | — | `replay_schema.md` |
| `save` | — | `save_architecture.md` |
| `input` | — | — |
| `common` | — | — |

**核心职责**：ID 注册与热重载、管线执行引擎、回放系统、存档系统、命令层与输入抽象、通用工具。

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
// 1. Infrastructure (Layer 7)
.add_plugins(RegistryPlugin)
.add_plugins(PipelinePlugin)
.add_plugins(ReplayPlugin)
.add_plugins(SavePlugin)
.add_plugins(InputPlugin)

// 2. Tactical Foundation (Layer 1)
.add_plugins(GridMapPlugin)
.add_plugins(TerrainPlugin)
.add_plugins(FactionPlugin)
.add_plugins(TurnPhasePlugin)

// 3. Capability System (Layer 2)
.add_plugins(TagPlugin)
.add_plugins(AttributePlugin)
.add_plugins(ModifierPlugin)
.add_plugins(AggregatorPlugin)
.add_plugins(GameplayContextPlugin)
.add_plugins(SpecPlugin)
.add_plugins(ConditionPlugin)
.add_plugins(TriggerPlugin)
.add_plugins(AbilityPlugin)
.add_plugins(TargetingPlugin)
.add_plugins(ExecutionPlugin)
.add_plugins(EffectPlugin)
.add_plugins(StackingPlugin)
.add_plugins(EventPlugin)
.add_plugins(CuePlugin)

// 4. Combat Execution (Layer 3)
.add_plugins(CombatPlugin)
.add_plugins(SpellPlugin)
.add_plugins(ReactionPlugin)

// 5. Progression & Economy (Layer 4)
.add_plugins(ProgressionPlugin)
.add_plugins(InventoryPlugin)
.add_plugins(EconomyPlugin)
.add_plugins(CraftingPlugin)
.add_plugins(SummonPlugin)

// 6. Party & Camp (Layer 5)
.add_plugins(PartyPlugin)
.add_plugins(CampRestPlugin)

// 7. Narrative & Content (Layer 6)
.add_plugins(NarrativePlugin)
.add_plugins(QuestPlugin)
```

### 6.2 Plugin 内部结构

每个 Feature Plugin 遵循统一结构：

```
src/<feature>/
├── mod.rs              # pub mod 声明 + re-export
├── plugin.rs           # Plugin impl (唯一对外入口)
├── components.rs       # ECS Components
├── systems.rs          # Systems (内部调度)
├── events.rs           # Feature 内事件
├── resources.rs        # Resources（如果适用）
├── api.rs              # 公开接口/类型（跨 Feature 可见）
└── internal/
    ├── mod.rs
    └── ...             # 内部实现
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
├── common/                      # Layer 7 — 通用工具（纯工具，无业务逻辑）
│   ├── mod.rs
│   └── math.rs
│
├── input/                       # Layer 7 — 输入抽象
│   ├── plugin.rs
│   ├── components.rs
│   ├── events.rs
│   └── api.rs
│
├── registry/                    # Layer 7 — ID 注册与热重载
│   ├── plugin.rs
│   ├── components.rs
│   ├── systems.rs
│   └── api.rs
│
├── pipeline/                    # Layer 7 — 管线执行引擎
│   ├── plugin.rs
│   ├── components.rs
│   ├── systems.rs
│   └── api.rs
│
├── replay/                      # Layer 7 — 回放系统
│   ├── plugin.rs
│   ├── components.rs
│   ├── systems.rs
│   └── api.rs
│
├── save/                        # Layer 7 — 存档系统
│   ├── plugin.rs
│   ├── systems.rs
│   └── api.rs
│
├── grid_map/                    # Layer 1 — 网格地图
├── terrain/                     # Layer 1 — 地形
├── faction/                     # Layer 1 — 阵营
├── turn_phase/                  # Layer 1 — 回合阶段
│
├── tag/                         # Layer 2 — Tag
├── attribute/                   # Layer 2 — 属性
├── modifier/                    # Layer 2 — Modifier
├── aggregator/                  # Layer 2 — 聚合器
├── gameplay_context/            # Layer 2 — 上下文
├── spec/                        # Layer 2 — Spec
├── condition/                   # Layer 2 — 条件
├── trigger/                     # Layer 2 — 触发
├── ability/                     # Layer 2 — 能力
├── targeting/                   # Layer 2 — 目标选择
├── execution/                   # Layer 2 — 执行
├── effect/                      # Layer 2 — 效果
├── stacking/                    # Layer 2 — 堆叠
├── event/                       # Layer 2 — 事件
├── cue/                         # Layer 2 — 表现信号
│
├── combat/                      # Layer 3 — 战斗
├── spell/                       # Layer 3 — 法术
├── reaction/                    # Layer 3 — 反应
│
├── progression/                 # Layer 4 — 成长
├── inventory/                   # Layer 4 — 物品
├── economy/                     # Layer 4 — 经济
├── crafting/                    # Layer 4 — 制造
├── summon/                      # Layer 4 — 召唤
│
├── party/                       # Layer 5 — 队伍
├── camp_rest/                   # Layer 5 — 休整
│
├── narrative/                   # Layer 6 — 叙事
├── quest/                       # Layer 6 — 任务
│
├── ui/                          # Presentation — UI 层
│   ├── hud/
│   ├── battle/
│   ├── menu/
│   └── common/
│
└── lib.rs                       # App 构建 + Plugin 注册
```

> 🟥 **绝对禁止**在 `src/` 下创建 `components.rs`、`systems.rs`、`events.rs` 等全局技术文件。

---

## 9. 架构决策索引

所有 Architecture Decision Record (ADR) 保存在 `docs/08-decisions/`。

| 编号 | 标题 | 状态 | 所属领域 |
|------|------|------|---------|
| ADR-000 | Feature 模块划分总图 | ✅ Proposed | Foundation |
| ADR-001 | Plugin 组合与注册顺序 | ✅ Proposed | Foundation |
| ADR-002 | ECS 四级通信机制选型 | ✅ Proposed | Foundation |
| ADR-010 | Ability → Effect 管线架构 | ⬜ Pending | Capability |
| ADR-011 | Modifier → Attribute 管线架构 | ⬜ Pending | Capability |
| ADR-012 | Stacking / Trigger / Cue 分离 | ⬜ Pending | Capability |
| ADR-013 | Registry 与热重载架构 | ⬜ Pending | Capability |
| ADR-020 | CombatIntent Pipeline 设计 | ⬜ Pending | Combat |
| ADR-021 | 回合状态机设计 | ⬜ Pending | Combat |
| ADR-022 | 网格 / 地形 / 阵营系统设计 | ⬜ Pending | Combat |
| ADR-023 | 法术与反应机制设计 | ⬜ Pending | Combat |
| ADR-030 | 成长 / 物品系统设计 | ⬜ Pending | Progression |
| ADR-031 | 队伍与休整系统设计 | ⬜ Pending | Progression |
| ADR-032 | 经济与制造系统设计 | ⬜ Pending | Progression |
| ADR-033 | 叙事 / 任务 / 召唤设计 | ⬜ Pending | Progression |
| ADR-040 | 数据流与所属权策略 | ⬜ Pending | Cross-cutting |
| ADR-041 | 回放确定性与架构 | ⬜ Pending | Cross-cutting |
| ADR-042 | 存档持久化策略 | ⬜ Pending | Cross-cutting |
| ADR-043 | 命令层与输入抽象 | ⬜ Pending | Cross-cutting |

---

## 10. 架构合规性检查清单

每个 ADR 提交时必须逐项检查：

- [ ] 符合 Feature First 原则（禁止全局技术目录）
- [ ] 符合三层运行时分离（Domain / Application / Presentation）
- [ ] 符合七层层间依赖方向（禁止反向依赖）
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
| `00-foundation/ADR-000-feature-module-map.md` | ⬜ pending | architect | — |
| `00-foundation/ADR-001-plugin-composition.md` | ⬜ pending | architect | — |
| `00-foundation/ADR-002-ecs-communication.md` | ⬜ pending | architect | — |
| `10-capability-system/ADR-010-ability-pipeline.md` | ⬜ pending | architect | — |
| `10-capability-system/ADR-011-modifier-pipeline.md` | ⬜ pending | architect | — |
| `10-capability-system/ADR-012-stacking-trigger-cue.md` | ⬜ pending | architect | — |
| `10-capability-system/ADR-013-registry-hotreload.md` | ⬜ pending | architect | — |
| `20-tactical-combat/ADR-020-combat-pipeline.md` | ⬜ pending | architect | — |
| `20-tactical-combat/ADR-021-turn-state-machine.md` | ⬜ pending | architect | — |
| `20-tactical-combat/ADR-022-grid-terrain-faction.md` | ⬜ pending | architect | — |
| `20-tactical-combat/ADR-023-spell-reaction.md` | ⬜ pending | architect | — |
| `30-progression-narrative/ADR-030-progression-inventory.md` | ⬜ pending | architect | — |
| `30-progression-narrative/ADR-031-party-camp-rest.md` | ⬜ pending | architect | — |
| `30-progression-narrative/ADR-032-economy-crafting.md` | ⬜ pending | architect | — |
| `30-progression-narrative/ADR-033-narrative-quest.md` | ⬜ pending | architect | — |
| `40-cross-cutting/ADR-040-data-flow-ownership.md` | ⬜ pending | architect | — |
| `40-cross-cutting/ADR-041-replay-determinism.md` | ⬜ pending | architect | — |
| `40-cross-cutting/ADR-042-save-persistence.md` | ⬜ pending | architect | — |
| `40-cross-cutting/ADR-043-command-input.md` | ⬜ pending | architect | — |

---

*本文档是 Fre 项目架构的最高准则，所有 Feature 开发、数据设计、代码审查以此为最终依据。修改本文档需经过 @architect 审核。*
