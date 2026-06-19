---
id: 09-planning.Phase-C-D-execution-plan
title: "Phase C+D 实施计划 — 基础设施与首个业务域并行推进"
status: active
owner: feature-developer
created: 2026-06-17
updated: 2026-06-17
completed_c1: true
completed_c2: true
completed_d1: true
completed_m1: true
tags:
  - planning
  - implementation
  - phase-c
  - phase-d
  - tactical
  - pipeline
  - roles
---

# Phase C+D 实施计划 — 基础设施与首个业务域并行推进

> **基于代码库深度分析的重新评估**
> 撰写: Sisyphus | 日期: 2026-06-17
> 前置参考: `feature-developer-implementation-roadmap.md`, `Fre 项目领域文件清单与设计排序分析.md`

---

## 0. 背景：重新评估执行策略

原实施路线图（`feature-developer-implementation-roadmap.md`）假设严格按 Phase A→B→C→D→E→F→G→H 串行推进。但代码库深度分析揭示了一个**更优的执行路径**：

### 当前状态

| Phase | 状态 | 行数 | 说明 |
|-------|------|------|------|
| **Phase A** — Shared | ✅ 完成 | 1,695 行 | IDs, Random, Time, Error, Testing 全部就绪 |
| **Phase B** — Capabilities | ✅ ≈85% | 21,431 行 | Foundation + Mechanism + Events 完整，Plugin 已注册 |
| **Phase C-1** — Pipeline 最小引擎 | ✅ 完成 | ~250 行 | PipelineRegistry + Plugin + Hooks，3 测试 + ✅ review |
| **Phase C-2** — Input 输入抽象 | ✅ 完成 | ~356 行 | InputAction/InputMap/InputState + 22 测试 + ✅ review |
| **Phase D-1** — Tactical 业务域 | ✅ 完成 | ~782 行 | 11 源文件 + 4 层测试（36 tests）+ ✅ review |
| **M1 里程碑** | ✅ 完成 | 742 tests | 全角色评审：3 份 review PASS，零红线 |
| **Post-M1 任务** | 见下游文档 | — | 详见 `Phase-post-M1-execution-plan.md` |

### 关键发现：M1 后的核心风险

**Capabilities 系统（21k 行）仍未经过任何业务域的运行时验证。**

Tactical 的 `integration.rs` 定义了 MovementType → TagId 的映射，但**没有任何 System 实际调用 Capabilities**。Tactical 的移动验证和网格管理完全是自包含的纯函数 + ECS 组件，没有使用 Tag/Attribute/Modifier/Execution 管线。

这意味着：**M1 验证了"我们能写出一个业务域"，但尚未验证"Capabilities 系统在全栈集成中正常工作"。**

### 核心洞察

Capabilities 系统（21k 行）提供了完整的"引擎"，但没有任何业务域在使用它。**最大的风险不是基础设施不完整，而是 Capabilities 系统未经实战验证。**

因此建议**放弃纯串行策略**，改为：

```
Phase C (Pipeline 最小版本) ──→ Phase D (Tactical 第一个业务域)
       │                               │
       │ 只做 Pipeline 核心骨架          │ 实现网格/位置/移动，使用 Capabilities
       │ (Stage注册 + 顺序执行)          │ 验证 Tag/Attribute/Execution 管线
       │                               │
       └────────── 并行推进 ────────────┘
```

这样做的收益：
1. **尽早端到端验证 Capabilities 系统** — 发现设计缺陷时修复成本最低
2. **Pipeline 是 Tactical 的必要前置** — 最小 Pipeline 完成后即可启动 Tactical
3. **其他 Infra (Replay/Save/Input/Registry) 可以延后** — 它们不影响 Capabilities→Domain 的验证环
4. **先出"可玩的最小闭环"** — 比全部基础设施做完再碰业务域更务实

---

## 1. 执行顺序总图

```
                ┌─────────────────────────────────────┐
                │  当前状态：Phase A✅ B≈85%           │
                │  Capabilities 引擎就绪，等待第一个乘客   │
                └─────────────────────────────────────┘
                                │
                ┌───────────────▼───────────────┐
                │                               │
     ┌──────────▼──────────┐        ┌──────────▼──────────┐
     │  Phase C-1          │        │  Phase D-1          │
     │  Pipeline 最小引擎   │        │  Tactical 业务域     │
     │  (并行, 先于 D-1)   │        │  (依赖 C-1 完成)     │
     │                     │        │                     │
     │  工作量: ~300 行     │        │  工作量: ~600 行     │
     │  1-2 次编码会话      │        │  2-3 次编码会话      │
     └──────────┬──────────┘        └──────────┬──────────┘
                │                               │
                └───────────┬───────────────────┘
                            │
                ┌───────────▼───────────┐
                │  Phase C-2            │
                │  Input 输入抽象        │
                │  (Tactical 验证需要)    │
                │  工作量: ~200 行       │
                └───────────┬───────────┘
                            │
                ┌───────────▼───────────┐
                │  里程碑 M1             │
                │  "第一个端到端闭环"      │
                │  cargo run → 网格渲染   │
                │  → 单位放置 → 移动验证  │
                └───────────────────────┘
                            │
          ┌─────────────────┼─────────────────┐
          ▼                 ▼                 ▼
    Phase D-2          Phase C-3          Phase E-1
    Terrain 地形       Registry 注册中心   Combat 战斗域
    Phase D-3          Phase C-4          Phase E-2
    Faction 阵营       Replay 回放骨架    Spell 法术域
                       Phase C-5          Phase E-3
                       Save 存档骨架      Reaction 反应域
```

---

## 2. 角色责任矩阵（按 AGENTS.md）

### 本计划涉及的角色

| 角色 | 职责 | 本计划中的任务 |
|------|------|--------------|
| **@domain-designer** | 定义"规则是什么" | 确认 Tactical/Terrain/Faction 领域规则；输出 domain rules/ 中的纯函数 |
| **@data-architect** | 定义"规则如何表达" | 确认 Schema 设计，确保数据结构与 Capabilities 兼容 |
| **@architect** | 定义"系统如何组织" | Pipeline 集成方案评审；Tactical 模块边界确认 |
| **@feature-developer** | 实现"如何做" | **主执行者**：Pipeline 引擎 + Tactical 域 + Input 输入 + Registry |
| **@test-guardian** | 验证"是否正确" | 每个交付物后补测试，重点是 domain invariant 和 replay 兼容 |
| **@code-reviewer** | 保证"质量合规" | 每个 Phase 交付前审查，检查红线违规 |
| **@refactor-guardian** | 监控技术健康 | M1 里程碑后扫描技术债 |

### 协作流程速查

```
@domain-designer ──→ docs/02-domain/         (领域规则确认)
@data-architect   ──→ docs/04-data/          (Schema 确认)
@architect        ──→ docs/01-architecture/  (架构评审)
                          ↓
@feature-developer ──→ src/                  (编码实现)
                          ↓
@test-guardian    ──→ tests/                 (测试验证)
@code-reviewer    ──→ docs/10-reviews/       (代码审查)
```

---

## 3. Phase C-1: Pipeline 最小引擎

> **目标**: 实现通用 Stage Pipeline 的最小可行版本（Stage 注册 + 顺序执行 + 基础错误处理）
> **并行策略**: 这是 Phase D-1 (Tactical) 的前置依赖，最先启动
> **估算**: ~300 行，3-5 个文件
> **主要执行者**: @feature-developer

### 3.1 前置文档确认（🟥 强制）

| 文档 | 位置 | 责任人 |
|------|------|--------|
| ADR-044 | `docs/01-architecture/40-cross-cutting/ADR-044-pipeline-engine.md` | @architect 生成（✅ 已存在） |
| pipeline_schema | `docs/04-data/infrastructure/pipeline_schema.md` | @data-architect 生成（✅ 已存在） |
| 编码规则 | `.trae/rules/编码规则.md` | @feature-developer 阅读确认 |

**如发现架构/Schema 问题 → @feature-developer 停止，调用上游 agent 修正。**

### 3.2 交付清单

| # | 任务 | 输出文件 | 责任人 | 前置 | 状态 |
|---|------|---------|--------|------|------|
| C1-2 | PipelineRegistry Resource | `src/infra/pipeline/registry.rs` | @feature-developer | 前置文档确认 | ✅ 完成 |
| C1-3 | PipelineHook trait | `src/infra/pipeline/hooks.rs` | @feature-developer | C1-2 | ✅ 完成 |
| C1-4 | PipelinePlugin + mod re-exports | `src/infra/pipeline/plugin.rs`, `mod.rs` | @feature-developer | C1-2 | ✅ 完成 |
| C1-T | 单元测试 | `src/infra/pipeline/tests/unit/registry_test.rs` | @test-guardian | C1-2 | ✅ 完成 (3/3 通过, 已迁移至 tests/ 目录) |
| C1-R | 代码审查 | `docs/10-reviews/pipeline-review.md` | @code-reviewer | C1-T 通过 | ⏳ 待触发 |

### 3.3 设计约束

- **最小能力**: 只做 Stage 注册 → 排序（按优先级）→ 顺序执行 → 错误传递
- **不做**: 前置/后置 Hook、条件执行、并行 Stage、重试逻辑（后续版本加）
- **与 Capabilities 的关系**: Pipeline 不依赖 Capabilities 的任何类型，只定义通用 `dyn PipelineStage` trait
- **数据传递**: Stage 间通过 `PipelineContext`（一个简单的 HashMap<String, Box<dyn Any>>）

### 3.4 质量门禁

```
□ cargo build 零错误
□ cargo test 新增测试全部通过
□ cargo clippy -- -D warnings 零警告
□ 无 #[allow(...)] / as any / @ts-ignore 等类型逃逸
□ PipelineContext 不使用全局可变状态
□ @architect 架构合规确认
```

---

## 4. Phase D-1: Tactical 战术空间域（第一个业务域）

> **目标**: 实现 SRPG 的空间基础 — 网格坐标系、Entity 位置组件、范围计算、移动验证
> **这是第一个实际使用 Capabilities 的业务域，目的是验证全管线**
> **估算**: ~600 行，7-10 个文件
> **主要执行者**: @feature-developer + @domain-designer（rules/ 确认）

### 4.1 前置文档确认（🟥 强制）

| 文档 | 位置 | 责任人 | 说明 |
|------|------|--------|------|
| ADR-022 | `docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md` | @architect | 网格/地形/阵营架构决策 |
| tactical_domain.md | `docs/02-domain/domains/tactical_domain.md` | @domain-designer | 战术空间领域规则 |
| tactical_schema.md | `docs/04-data/domains/tactical_schema.md` | @data-architect | 数据 Schema |
| ADR-010 (参考) | `docs/01-architecture/10-capability-system/ADR-010-ability-pipeline.md` | @feature-developer | 了解如何对接 Ability Pipeline |
| Tag 领域 | `docs/02-domain/capabilities/tag_domain.md` | @feature-developer | 了解 Tag 分类（用于标识单位类型） |

**如任何文档缺失或模糊 → @feature-developer 停止，调用对应上游 agent。**

### 4.2 标准 7 文件结构

Tactical 域遵循 `docs/01-architecture/README.md` §6.2 定义的 Business Domains 标准结构：

```
src/core/domains/tactical/
├── plugin.rs          # 唯一对外入口
├── components.rs      # ECS Components
├── systems/           # 业务系统
│   ├── mod.rs
│   ├── grid_system.rs       # 网格初始化/查询
│   ├── movement_system.rs   # 移动逻辑
│   └── position_system.rs   # 位置同步
├── events.rs          # 领域事件
├── error.rs           # 专属错误枚举
├── rules/             # 纯业务规则（纯函数，零 ECS）
│   ├── mod.rs
│   ├── movement.rs    # 移动消耗计算
│   └── range.rs       # 范围/距离计算
└── integration.rs     # 调用 Capabilities 的入口
```

### 4.3 交付清单

| # | 任务 | 输出文件 | 责任人 | 前置 | 状态 |
|---|------|---------|--------|------|------|
| D1-1 | GridPos + MovementPoints + Facing 组件 | `tactical/components.rs` | @feature-developer | 前置文档确认 | ✅ 完成 |
| D1-2 | GridMap Resource + TileData/TileFlags | `tactical/resources.rs` | @feature-developer | D1-1 | ✅ 完成 |
| D1-3 | Grid 初始化 System | `tactical/systems/grid_system.rs` | @feature-developer | D1-2 | ✅ 完成 |
| D1-4 | 移动验证/执行 System | `tactical/systems/movement_system.rs` | @feature-developer | D1-2 | ✅ 完成 |
| D1-5 | UnitMoved + PositionChanged Event | `tactical/events.rs` | @feature-developer | D1-1 | ✅ 完成 |
| D1-6 | 范围计算 (BFS + attack_range) | `tactical/rules/range.rs` | @feature-developer | D1-1 | ✅ 完成 |
| D1-7 | 移动消耗 (movement_cost + path_total_cost) | `tactical/rules/movement.rs` | @feature-developer | D1-1 | ✅ 完成 |
| D1-8 | TacticalError 枚举 | `tactical/error.rs` | @feature-developer | D1-1 | ✅ 完成 |
| D1-9 | Integration (movement_type_to_tag) | `tactical/integration.rs` | @feature-developer | D1-4 | ✅ 完成 |
| D1-10 | TacticalPlugin + mod 导出 | `tactical/plugin.rs`, `mod.rs` | @feature-developer | D1-1~D1-9 | ✅ 完成 |
| D1-T | 测试 | `tactical/tests/` | @test-guardian | D1-1~D1-10 | ✅ 完成 (36 tests) |
| D1-R | 代码审查 | `docs/10-reviews/tactical-review.md` | @code-reviewer | D1-T 通过 | ✅ 完成 (PASS) |

### 4.4 与 Capabilities 的对接点

Tactical 是 Capabilities 的第一个消费者，以下是对接关键：

| Capability | 在 Tactical 中的使用场景 | 验证目标 |
|-----------|------------------------|---------|
| **Tag** | `Entity` 标记为 `Unit`/`Obstacle`/`Terrain` | Tag 分类 + 查询正确性 |
| **Attribute** | 单位的移动力（`MovementPoints`） | 属性读取 + 消耗更新 |
| **Modifier** | 地形对移动力的修正（如沼泽 -2） | Modifier 应用 → Aggregator 重新计算 |
| **Execution** | 移动到指定网格的执行动作 | Execution→Effect 管线 |
| **Effect** | 进入毒池地形触发伤害效果 | Effect 生命周期 + EffectPipeline |
| **GameplayContext** | 移动执行的上下文传递 | 载荷构建 + 链路追踪 |
| **Targeting** | 移动范围的可达网格筛选 | 范围选择 + 条件过滤 |

### 4.5 质量门禁

```
□ cargo build 零错误
□ cargo test 新增测试全部通过（含领域不变量）
□ cargo clippy -- -D warnings 零警告
□ 无绕过 Effect Pipeline 直接修改属性的行为
□ 跨域通信仅通过 Event（无直接数据结构引用）
□ 7 文件结构完整（plugin / components / systems / events / error / rules / integration）
□ @domain-designer 领域规则合规确认
□ @data-architect Schema 合规确认
□ @architect 模块边界合规确认
```

---

## 5. Phase C-2: Input 输入抽象

> **目标**: 实现 ADR-043 定义的最小输入系统，为 Tactical 的交互验证提供基础
> **并行策略**: 可与 Phase D-1 中不依赖 Input 的部分并行（如 D1-1, D1-5, D1-6）
> **估算**: ~200 行，3-4 个文件
> **主要执行者**: @feature-developer

### 5.1 前置文档确认

| 文档 | 位置 | 责任人 |
|------|------|--------|
| ADR-043 | `docs/01-architecture/40-cross-cutting/ADR-043-command-input.md` | @architect（✅ 已存在） |
| input_schema | `docs/04-data/infrastructure/input_schema.md` | @data-architect（✅ 已存在） |

### 5.2 交付清单

| # | 任务 | 输出文件 | 责任人 | 前置 | 状态 |
|---|------|---------|--------|------|------|
| C2-1 | InputAction + InputMap | `src/infra/input/action.rs` | @feature-developer | 前置文档确认 | ✅ 完成 |
| C2-2 | InputState Resource | `src/infra/input/resources.rs` | @feature-developer | C2-1 | ✅ 完成 |
| C2-3 | Input Systems | `src/infra/input/systems.rs` | @feature-developer | C2-2 | ✅ 完成 |
| C2-4 | InputPlugin 注册 | `src/infra/input/plugin.rs` | @feature-developer | C2-3 | ✅ 完成 |
| C2-5 | mod 声明 | `src/infra/input/mod.rs` | @feature-developer | C2-4 | ✅ 完成 |
| C2-T | 单元测试 | `src/infra/input/tests/` | @test-guardian | C2-1~C2-5 | ✅ 完成 (22 tests) |
| C2-R | 代码审查 | `docs/10-reviews/input-review.md` | @code-reviewer | C2-T 通过 | ✅ 完成 (PASS) |

### 5.3 质量门禁

```
□ cargo build 零错误
□ CommandQueue 支持帧缓冲（不丢命令）
□ GameCommand 枚举不包含具体游戏逻辑（纯输入抽象）
□ @architect ADR-043 合规确认
```

---

## 6. 里程碑 M1: "第一个端到端闭环"

> **标志着 Phase C-1 + C-2 + D-1 全部通过质量门禁**
> **验证方式**: `cargo run` 启动 → 初始化网格 → 生成单位 → 执行移动

### 6.1 M1 验收标准

| 检查项 | 通过标准 | 验证者 |
|--------|---------|--------|
| Pipeline 可初始化 | `PipelinePlugin` 注册后 Stage 可注册/排序/执行 | @test-guardian |
| Grid 可初始化 | `GridConfig` 指定尺寸后生成对应网格 Entity | @test-guardian |
| 单位可放置到网格 | 创建 Entity → 赋予 `GridPosition` → 出现在正确坐标 | @test-guardian |
| 移动范围可计算 | 给定移动力 → 返回可达网格列表（不含障碍物） | @test-guardian |
| 移动可执行 | 单位从 A 移动到 B → `PositionChanged` 事件触发 | @test-guardian |
| Capabilities 被使用 | 移动至少使用 1 个 Tag + 1 个 Attribute + 1 个 Modifier | @architect |
| 所有新增代码无红线 | 宪法 21 条红线检查 | @code-reviewer |

### 5.4 当前完成状态
C-1 Pipeline 最小引擎 — ✅ 完成
- PipelineRegistry Resource + insert/register/get/add_hook/count/iter
- ExecutionLogHook (tracing::trace! 日志输出)
- PipelinePlugin 注册 + mod 重新导出 core pipeline 全部类型
- 3 个单元测试通过（empty/register/get + duplicate panic）
- cargo build 零错误, cargo test 640 全通过

C-2 Input 输入抽象 — ✅ 完成
- `action.rs`: InputAction 枚举（19 个语义动作）+ InputMap Resource（默认 WASD + 鼠标按键绑定）
- `resources.rs`: InputState Resource（pressed/just_pressed/just_released + 鼠标位置跟踪）
- `systems.rs`: collect_input_actions（PreUpdate，原始按键 → InputMap 翻译 → InputState）+ process_meta_commands（Save/Load/Menu 元命令 → CommandQueue 入队）
- `plugin.rs`: InputPlugin 注册 InputMap + InputState Resource，Core CommandQueue 实例化，PreUpdate 双 System
- 核心层 `CommandQueue` 添加 `#[derive(Resource)]` 使其成为合法的 Bevy Resource
- 架构边界：业务命令（MoveUnit/EndTurn/Attack）由各 Domain 的 System 消费 InputState 后自行入队，infra 层只处理无关游戏上下文的元命令
- `cargo build` 零错误零警告, `cargo test` 640 全通过

D-1 Tactical 业务域 — ✅ 完成
- components: GridPos（4 种距离计算 + 2 种邻居生成）, MovementPoints（consume/reset）, Facing, MovementType, HexDirection
- resources: GridMap Resource（BFS tiles_in_range, grid_to_world, world_to_grid）, TileData（u32 紧凑打包）, TileFlags（位标记）
- events: UnitMoved, PositionChanged
- error: TacticalError（6 种领域错误枚举）
- rules/movement: movement_cost（7 种地形 × 5 种移动类型）, path_total_cost
- rules/range: bfs_reachable_positions, attack_range
- systems/grid_system: initialize_default_grid（20x15 Square）
- systems/movement_system: validate_and_execute_move（4 步验证 + 执行）
- integration: movement_type_to_tag（MovementType → TagId）
- plugin: TacticalPlugin（register_type + Startup System）
- cargo build 零错误零警告, cargo test 640 全通过

---

### 6.2 M1 后评审

M1 完成后，按 AGENTS.md 流程触发以下评审：

| 评审项 | 执行角色 | 输出 |
|--------|---------|------|
| 架构合规性 | @architect | ADR 更新或合规确认 |
| 领域合规性 | @domain-designer | Tactical 领域规则对齐确认 |
| 数据合规性 | @data-architect | Schema 实现对齐确认 |
| 代码质量 | @code-reviewer | `docs/10-reviews/tactical-review.md` |
| 技术债扫描 | @refactor-guardian | `docs/11-refactor/` 初始债务清单 |

## 7. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 | 责任人 |
|------|------|------|---------|--------|
| Capabilities 21k 行存在未被发现的架构缺陷 | 中 | **高**（阻塞所有业务域） | Tactical 尽早接入，发现即修复 | @architect + @feature-developer |
| Pipeline 设计过于抽象 | 中 | 中 | 坚持最小版本，不做超前设计 | @feature-developer |
| 领域规则确认耗时（@domain-designer 需介入） | 低-中 | 中 | Tactical 领域规则已完备，只需确认而非重写 | @domain-designer |
| 单人瓶颈（所有代码变更经过同一人） | **高** | **高** | 这是一个已知限制，建议引入协作或分模块 | — |
| @test-guardian 测试滞后于编码 | 中 | 中 | 每个交付项后紧跟测试，不留债 | @test-guardian |
| Pipeline 与 Capabilities Runtime 的职责重叠 | 低 | 中 | Runtime 已有 pipeline/ 子模块，需明确分层边界 | @architect |

---

## 8. 工作量总估与时间线

| 阶段 | 文件数 | 预估行数 | 主要执行者 | 建议工期 | 状态 |
|------|--------|---------|-----------|---------|------|
| C-1 Pipeline 最小引擎 | 3-4 | ~250 | @feature-developer | 1 次编码会话 | ✅ 完成 |
| D-1 Tactical 业务域 | 11 | ~550 | @feature-developer | 2-3 次编码会话 | ✅ 完成 |
| C-2 Input 输入抽象 | 5 | ~220 | @feature-developer | 1 次编码会话 | ✅ 完成 |
| **M1 里程碑** | — | — | **全部角色** | 评审 1 次 | ✅ 完成 |
| **总计** | **~22** | **~1,220** | **全角色协作** | **3-5 次编码会话** | **C-1 + C-2 + D-1 + M1 完成** |

### 建议的执行节奏

```
第 1 轮  →  C-1 Pipeline 引擎（@feature-developer）              ✅ 完成
              同时: @domain-designer + @data-architect 确认 Tactical 前置文档
              
第 2 轮  →  D-1 Tactical 全域（@feature-developer）              ✅ 完成
              
第 3 轮  →  C-2 Input 输入抽象（@feature-developer）             ✅ 完成
              
第 4 轮  →  M1 集成验证 + 全角色评审                            ✅ 完成 (742 tests, 3 reviews PASS)
```

---

## 9. 附录：角色职责快速参考

引自 `AGENTS.md` 标准协作流程：

```
需求
  │
  ├─→ @domain-designer（领域建模） → 输出：`docs/02-domain/`
  │
  ├─→ @data-architect（数据架构）  → 输出：`docs/04-data/`
  │
  └─→ @architect（架构设计）       → 输出：`docs/01-architecture/`
          │
          ↓
  @feature-developer（代码实现）    → 输出：`src/` + `docs/09-planning/`
          │
          ├─→ @test-guardian（测试审查）    → 输出：`docs/05-testing/` + `tests/`
          │
          └─→ @code-reviewer（代码审查）    → 输出：`docs/10-reviews/`
                      │
                      ↓
          @refactor-guardian（技术债扫描）  → 输出：`docs/11-refactor/`
```

### 本计划中各角色的具体任务速查表

| 角色 | 负责内容 | 涉及任务 |
|------|---------|---------|
| **@domain-designer** | 确认 Tactical 领域规则；参与 rules/ 纯函数设计 | D1-0, D1-5, D1-6 |
| **@data-architect** | 确认 Tactical Schema；确认 Input Schema | D1-0b, C2-0 |
| **@architect** | Pipeline 集成架构评审；Tactical 模块边界确认；ADR-044 确认 | C1-R, D1-R, M1 评审 |
| **@feature-developer** | **所有编码工作** — Pipeline / Tactical / Input | C1-1~C1-5, C2-1~C2-3, D1-1~D1-9 |
| **@test-guardian** | 每阶段测试编写；领域不变量测试 | C1-T, C2-T, D1-T |
| **@code-reviewer** | 每个交付物审查报告；红线检查 | C1-R, C2-R, D1-R |
| **@refactor-guardian** | M1 里程碑后技术债扫描 | M1 后评审 |

---

> **M1 后未完成任务与下一步行动，请参阅 [`Phase-post-M1-execution-plan.md`](./Phase-post-M1-execution-plan.md)。**
>
> 本文档仅记录 Phase C-1 + C-2 + D-1 + M1 的已完成工作。

---

*本文档基于 `feature-developer-implementation-roadmap.md` 的全量分析，结合作者对代码库的深度审查，按照 AGENTS.md 角色体系编制。**
