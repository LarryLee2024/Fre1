# Tactical RPG

基于 Bevy 0.18.1 的回合制战棋游戏（SRPG），采用七层 ECS 架构与数据驱动设计，灵感来自 GAS（Game Ability System）。

## 核心功能

- **回合制战斗**：基于回合阶段状态机（选择 → 行动 → 结算），支持移动、攻击、技能、道具等操作
- **数据驱动配置**：单位、技能、Buff、地形、特质、AI 行为、关卡均通过 RON 文件配置，无需修改代码即可扩展内容
- **关卡与胜负条件**：关卡配置包含可组合的胜利/失败条件（全灭、存活回合、击败 Boss、超时等），支持多条件 OR 组合
- **Effect Pipeline**：战斗效果通过生成 → 修饰 → 执行的管线处理（GAS 风格），支持 Execution/Modifier/Cue 组合扩展
- **AI 系统**：数据驱动的 AI 行为配置，支持多种策略模板，与玩家共用 Effect Pipeline
- **装备与背包**：装备穿脱、物品使用、容器管理、负重系统
- **MOD 支持**：MOD 加载、沙箱隔离、兼容性校验
- **国际化**：基于 Fluent 的多语言支持，字体回退链
- **调试面板**：基于 egui 的运行时调试工具，支持 World Inspector 和状态查看

## 安装指南

### 环境要求

- Rust 1.96+（edition 2024）
- Cargo

### 构建与运行

```bash
# 克隆项目
git clone <repository-url>
cd Fre

# 编译运行
cargo run

# 开发模式（启用文件热重载和调试工具）
cargo run --features dev

# 运行测试
cargo test
```

## 项目结构

```
src/
  app/          # Layer 1: 应用装配与 Plugin 编排
  core/         # Layer 2: 业务逻辑 — 战斗、技能、Buff、地图、角色、AI、回合、装备、背包、战役
    battle/     # 战斗效果管线（generate → modify → execute）
    buff/       # Buff/Debuff 系统（⚠️ 正在迁移为 Effect + Duration）
    skill/      # 技能系统（⚠️ 正在迁移为 Ability）
    ability/    # 能力系统（新增，取代 Skill）
    effect/     # 效果系统一级领域
    execution/  # 执行算式层（trait-based 公式执行）
    cue/        # 表现信号总线（GameplayCue）
    attribute/  # 属性系统（Primary/Derived 双分层）
    character/  # 单位组件与 Trait 扩展体系
    map/        # 地图、寻路、关卡配置加载
    turn/       # 回合状态机、行动顺序、胜负条件检查
    ai/         # AI 行为系统
    equipment/  # 装备系统
    inventory/  # 背包系统
    campaign/   # 战役编排与关卡序列
    targeting/  # 目标选择（取代 Selector）
    trigger/    # 触发器系统
    tag/        # 标签系统（GameplayTag 位掩码）
    movement/   # 移动系统
  shared/       # Layer 3: 跨层共享类型（强类型 ID、Registry、错误工具、事件、验证）
  infrastructure/ # Layer 4: 基础设施（资源加载、日志、本地化、配置、回放、持久化、热重载、审计）
  content/      # Layer 5: Content 层统一入口（RON 加载模块）
  modding/      # Layer 6: MOD 支持（加载器、沙箱、校验器）
  ui/           # 表现层 — 用户界面面板与组件
  debug/        # 调试面板与查看器
  tools/        # Layer 7: 开发工具（永不发布）
  input/        # 输入处理

content/       # RON 游戏配置（数据驱动）
  characters/  # 单位模板
  skills/      # 技能定义
  buffs/       # Buff 定义
  effects/     # 效果定义（含 Duration）
  executions/  # 执行算式定义
  cues/        # 表现信号配置
  attributes/  # 属性定义
  tags/        # 标签定义
  classes/     # 职业与特质
  terrains/    # 地形类型
  ai_behaviors/ # AI 行为模板
  stages/      # 关卡配置
  campaigns/   # 战役定义
  modifiers/   # 修饰规则（元素交互等）
  formulas/    # 公式配置

assets/        # 二进制资源（字体、数据）
  fonts/       # 字体文件
  data/        # 运行时数据

docs/
  00-governance/     # 治理规则（AI 开发宪法、编码规范、Bevy 参考）
  01-architecture/   # 架构设计（README.md 为最高优先级，v4.2）
    00-overview/           # 架构总纲（8 文件）
    01-battle-gas/         # 战斗/GAS（4 文件）
    02-ecs-patterns/       # 组件/ECS 模式（3 文件）
    03-data-config-asset/  # 数据/配置/资产（9 文件）
    04-events-logging-error/ # 事件/日志/错误（3 文件）
    05-ui/                 # UI 架构（1 文件）
    06-map-pathfinding/    # 地图/寻路（1 文件）
    07-tools-testing-quality/ # 工具/测试/质量（5 文件）
    08-i18n-modding-collaboration/ # 国际化/MOD/协作（3 文件）
    09-infrastructure-migration/   # 基础设施/迁移（1 文件）
  02-domain/         # 领域规则文档（20 个子领域，39 个规则文件）
    GAS/                   # GAS 领域全景（14 子领域 + 概览文档）
  03-technical/      # 技术实现文档（ECS、通信、性能，14 个文件）
  04-data/           # 数据与配置文档（7 个规则 + 3 个参考数据目录）
  05-testing/        # 测试规范（测试宪法 v3.1）
  06-ai/             # AI 协作流程（7-Agent 角色定义）
  07-operations/     # 运维文档（待填充）
  08-decisions/      # 架构决策记录（35 个 ADR）
  09-planning/       # 执行计划（4 个文件）
  10-reviews/        # 代码审查记录
  11-refactor/       # 技术债扫描记录
  98-roadmap/        # 项目路线图
  99-history/        # 历史归档

tests/         # 集成测试、场景测试、快照测试
```

## 文档详细说明

### 顶层核心文档（最高优先级）

| 文件 | 说明 | 优先级 |
|------|------|--------|
| `docs/01-architecture/README.md` | 七层架构总纲（v4.2），Feature 边界、ECS 规则、Effect/Modifier 管线 | 🟥 **最高** |
| `docs/00-governance/ai-constitution-complete.md` | AI 开发宪法 v1.6 完整版（20 部分），覆盖架构/ECS/代码/测试/日志/工程质量 | 🟥 **最高** |
| `docs/00-governance/coding-rules.md` | 编码执行规范 v1.0，AI 编码自检清单，Effect/Modifier 管线保护 | 🟩 必须遵守 |
| `docs/02-domain/README.md` | 领域规则汇总索引，39 个领域文件的速查入口 | 🟩 必须遵守 |
| `docs/04-data/README.md` | 数据架构规范，Schema 设计指南、Save/Replay 兼容规则 | 🟩 必须遵守 |
| `docs/05-testing/test-spec.md` | 测试宪法 v3.1，测试分层/回放测试/覆盖率策略 | 🟩 必须遵守 |

### `docs/02-domain/` — 领域规则（39 文件 + GAS 目录）

按领域子目录分组，开发对应功能时直接查阅：

**GAS/ 目录 — SRPG-GAS 领域全景（14 子领域 + 概览）**

| 文件 | 关键词 |
|------|--------|
| `GAS/GAS_domain_overview.md` | GAS 领域全景：统一术语、架构图、数据流、业务流程、依赖矩阵 |
| `GAS/ability/ability-rules.md` | Ability 定义、冷却、五阶段释放管线 |
| `GAS/effect/effect-rules.md` | Effect Pipeline 三步管线（Generate→Modify→Execute） |
| `GAS/attribute-modifier/attribute-modifier-rules.md` | 属性修饰管线、Modifier Chain、修饰器栈 |
| `GAS/execution/execution-rules.md` | 执行算式层（trait-based 公式执行） |
| `GAS/formula/formula-rules.md` | 公式系统（10 种纯函数公式） |
| `GAS/trigger/trigger-rules.md` | 触发器系统（16 种触发时机、ExecutionStack） |
| `GAS/tag/tag-rules.md` | GameplayTag 位掩码（37/64 bits、三层标签管理） |
| `GAS/cue/cue-rules.md` | 表现信号总线（Logic/Presentation 分离） |
| `GAS/condition/condition-rules.md` | 条件系统（ConditionalEffect、纯函数评估） |
| `GAS/cost/cost-rules.md` | 消耗系统（8 种消耗类型、先校验后扣除） |
| `GAS/requirement/requirement-rules.md` | 释放前提（9 种前提类型、纯函数检查） |
| `GAS/targeting/targeting-rules.md` | 目标选择（8 种目标类型、纯函数解析） |
| `GAS/duration/duration-rules.md` | 持续策略（7 种 DurationPolicy） |
| `GAS/stack-policy/stack-policy-rules.md` | 叠层策略（4 种 StackingRule）✅ 已实现 |

**Core Domain — 核心业务规则**

| 文件 | 关键词 |
|------|--------|
| `battle/battle-rules.md` | 战斗状态机、Effect Pipeline、伤害计算 |
| `character/character-rules.md` | 角色属性、Faction、UnitSnapshot |
| `ability/ability-rules.md` | Ability 定义、冷却、五阶段释放管线（取代 Skill） |
| `effect/effect-rules.md` | 效果系统一级领域（Damage/Heal/ApplyBuff 原子操作） |
| `execution/execution-rules.md` | 执行算式层（trait-based Damage/Heal/Shield 独立 Executor） |
| `cue/cue-rules.md` | 表现信号总线（GameplayCue 统一表现事件） |
| `attribute/attribute-rules.md` | 属性一级领域（Primary/Derived 双分层） |
| `attribute-modifier/attribute-modifier-rules.md` | Modifier 管线、属性修饰、叠加规则 |
| `turn/turn-rules.md` | TurnPhase、回合阶段、行动队列 |
| `trigger/trigger-rules.md` | 触发器、事件链（ExecutionStack、TriggerRegistry） |
| `condition/condition-rules.md` | 条件系统、效果判断、运行时条件 |
| `formula/formula-rules.md` | 公式系统、数值计算、表达式求值 |
| `targeting/targeting-rules.md` | 目标选择、AOE、空地选择（取代 Selector） |
| `duration/duration-rules.md` | 持续时间（回合/真实时间/永久） |
| `cost/cost-rules.md` | 消耗系统、资源扣除 |
| `stack-policy/stack-policy-rules.md` | 堆叠策略、Buff叠加/替换 |
| `requirement/requirement-rules.md` | 释放前提、技能可用性检查 |
| `tag/tag-rules.md` | 标签系统（GameplayTag 位掩码、三层标签管理） |
| `input/input-rules.md` | 输入处理、UiCommand |
| `skill/skill-rules.md` *(已过时)* | ⚠️ 已被 ability-rules.md 取代 |
| `buff/buff-rules.md` *(已废弃)* | ⚠️ 被吸收为 Effect + Duration（ADR-026） |

**Infrastructure（7 文件）— 基础设施规则**

| 文件 | 关键词 |
|------|--------|
| `error-system-rules.md` | 错误处理、Result 传播、分级 |
| `logging-rules.md` | 日志分级、格式、调试日志 |
| `persistence-rules.md` | 存档格式、版本迁移 |
| `hot-reload-rules.md` | Definition 热更新、战斗中禁止 |
| `determinism-rules.md` | 确定性、多 RNG 流独立 |
| `replay-rules.md` | 战斗回放、Command Stream |
| `testing-rules.md` | 测试金字塔、回放测试 |

**Content/Data（6 文件）— 数据与内容规则**

| 文件 | 关键词 |
|------|--------|
| `content-system-rules.md` | RON 加载、Registry、Definition 不可变 |
| `config-system-rules.md` | 运行时配置、热重载 |
| `content-migration-rules.md` | 版本兼容、字段兼容 |
| `asset-lifecycle-rules.md` | 资源生命周期、Handle 类型、内存预算 |
| `asset-organization-rules.md` | 三树分离、命名空间 |
| `feature-flag-rules.md` | Feature Flag、灰度发布 |

**Cross-cutting（12 文件）— 横切关注点**

| 文件 | 关键词 |
|------|--------|
| `layer-architecture-rules.md` | 分层架构、层间依赖方向 |
| `ecs-communication-rules.md` | Hook/Observer/Message/Trigger |
| `command-bus-rules.md` | UiCommand、命令总线 |
| `shared-layer-rules.md` | Shared 层、公共类型 |
| `modding-system-rules.md` | MOD 加载、资源隔离 |
| `ui-architecture-rules.md` | ViewModel、UiCommand、UI 渲染 |
| `localization-rules.md` | 多语言、Fluent |
| `map-terrain-rules.md` | 地图地形、寻路、视野 |
| `ai-rules.md` | AI 行为、策略模板、决策管线 |
| `performance-budget-rules.md` | 帧率目标、内存限制 |
| `validation-rules.md` | 数据完整性、配置校验 |
| `event-audit-rules.md` | 事件审计、双轨制日志 |

### `docs/01-architecture/` — 架构设计（38 文件，10 个子目录）

七层架构各领域的详细设计文档（v4.2），按类别分组：

| 子目录 | 文件数 | 覆盖主题 |
|--------|--------|----------|
| `00-overview/` | 8 | 架构总纲、七层边界、项目结构、Plugin 设计、Schedule、迁移路线图 |
| `01-battle-gas/` | 4 | 战斗 FSM、GAS 架构、Skill/Effect 抽象、命令总线 |
| `02-ecs-patterns/` | 3 | 组件设计、System 设计、确定性规则 |
| `03-data-config-asset/` | 9 | 内容管线、配置系统、资产组织/生命周期/命名空间、ID 设计、存档迁移 |
| `04-events-logging-error/` | 3 | 事件审计、日志架构、错误架构 |
| `05-ui/` | 1 | UI 领域边界、单向数据流 |
| `06-map-pathfinding/` | 1 | 寻路算法、范围计算 |
| `07-tools-testing-quality/` | 5 | 工具链、测试金字塔、校验规则、性能预算、Feature Flag |
| `08-i18n-modding-collaboration/` | 3 | 国际化、MOD 支持、AI 协作模型 |
| `09-infrastructure-migration/` | 2 | 基础设施模块设计、迁移路线图 |

### `docs/08-decisions/` — 架构决策记录（35 ADR）

| 文件 | 主题 |
|------|------|
| `ADR-001-migration-plan.md` | 迁移总计划 |
| `ADR-002-技术债修复方案.md` | 技术债治理策略 |
| `ADR-003-分层契约与依赖迁移.md` | 七层架构落地 |
| `ADR-004-内容与数据迁移方案.md` | 配置数据迁移 |
| `ADR-005-插件与通信迁移方案.md` | 插件体系与通信 |
| `ADR-006-验证与测试迁移方案.md` | 测试体系迁移 |
| `ADR-007-目录结构迁移映射.md` | 源码/资产/内容目录 |
| `ADR-008-核心机制与工程质量迁移.md` | 核心机制与质量门禁 |
| `ADR-009-迁移合规修正与架构决策.md` | 迁移合规修正 |
| `ADR-010-测试迁移与重整方案.md` | 测试迁移方案 |
| `ADR-011-错误模块实施.md` | 错误模块实施 |
| `ADR-012-日志模块与统一事件目录.md` | 日志模块设计 |
| `ADR-013-技能数据模型与配置规范.md` | 技能数据模型 |
| `ADR-014-技能释放管线设计.md` | 技能释放管线 |
| `ADR-015-技能标签与分类体系.md` | 技能标签体系 |
| `ADR-016-技能系统扩展点设计.md` | 技能扩展点 |
| `ADR-017-国际化架构决策.md` | 国际化架构 |
| `ADR-018-国际化迁移方案.md` | 国际化迁移 |
| `ADR-020-Buff数据模型与配置规范.md` | Buff 数据模型 |
| `ADR-021-Buff生命周期与持续策略.md` | Buff 生命周期 |
| `ADR-022-Buff触发系统与事件架构.md` | Buff 触发系统 |
| `ADR-023-标签系统架构重整.md` | 标签系统架构 |
| `ADR-024-标签系统迁移方案.md` | 标签系统迁移 |
| `ADR-025-七领域模块化架构设计.md` | 七领域模块化 |
| `ADR-026-SRPG-Lite-GAS-架构对齐.md` | GAS 架构对齐 |
| `ADR-027-业务模块执行计划结果.md` | 业务模块执行结果 |
| `ADR-028-logging-error-architecture-review.md` | 日志与错误架构审查 |
| `ADR-029-数据模型完全重构总纲.md` | 数据模型完全重构总纲 |
| `ADR-030-ID系统与Registry基础设施重构.md` | ID 系统与 Registry 重构 |
| `ADR-031-核心属性与标签系统重构.md` | 属性与标签系统重构 |
| `ADR-032-Effect管线全链路重构.md` | Effect 管线全链路重构 |
| `ADR-033-Ability与Trigger系统重构.md` | Ability 与 Trigger 重构 |
| `ADR-034-Cue与Replay与I18n系统实现.md` | Cue/Replay/I18n 实现 |
| `ADR-035-模块清理与迁移执行计划.md` | 模块清理与迁移执行 |

### `docs/09-planning/` — 执行计划

| 文件 | 说明 |
|------|------|
| `adr-026-gap-analysis-and-action-plan.md` | ADR-026 Gap 分析与行动计划 |
| `business-module-execution-plan.md` | 业务模块执行计划 |
| `adr-029-035-domain-validation.md` | ADR-029~035 领域验证计划 |
| `adr-029-035-data-architecture.md` | ADR-029~035 数据架构执行计划 |

### `docs/10-reviews/` — 代码审查记录

| 文件 | 说明 |
|------|------|
| `adr-026-implementation-review.md` | ADR-026 实现审查报告 |

### `docs/11-refactor/` — 技术债扫描记录

> 待首次技术债扫描后填充。

## 架构原则

项目遵循以下核心架构原则（详见 `docs/01-architecture/README.md`）：

1. **Definition / Instance 分离**：配置数据（如 UnitTemplate）不可变，运行时实例（如 Unit）可变
2. **Rule / Content 分离**：检查逻辑是规则，RON 配置是内容
3. **Logic / Presentation 分离**：业务逻辑在 System 中，UI 层只读取状态
4. **数据驱动**：游戏内容通过 RON 文件配置，禁止硬编码
5. **Effect Pipeline**：战斗效果通过 Generate → Modify → Execute 管线处理
6. **七层架构**：App → Core → Shared → Infrastructure → Content → Modding → Tools

## AI 辅助开发

项目配备 7 个专用 AI Agent（详见 `AGENTS.md` 和 `.qoder/agents/`），遵循严格的协作流程：

```
需求
  │
  ├─→ @domain-designer（领域建模）→ @data-architect（数据架构）
  │
  └─→ @architect（架构设计）→ ADR
          │
          ↓
  @feature-developer（代码实现）
          │
          ├─→ @test-guardian（测试审查）
          │
          └─→ @code-reviewer（代码审查）
                      │
                      ↓
          @refactor-guardian（技术债扫描）
```

## 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 1.96+ | 编程语言（edition 2024） |
| Bevy | 0.18.1 | 游戏引擎（ECS） |
| RON | 0.12.1 | 配置文件格式 |
| Fluent | 0.17 | 国际化 |
| tracing | 0.1 | 结构化日志 |
| thiserror | 2 | 错误派生宏 |
| bevy-inspector-egui | 0.36 | 调试检查器 |
| proptest | — | 属性测试（开发依赖） |
| insta | — | 快照测试（开发依赖） |

## 注意事项

- 配置路径使用编译时绝对路径（`CARGO_MANIFEST_DIR`），发布构建时需确保 content/ 和 assets/ 目录与可执行文件相对位置正确
- 关卡配置中 `victory_condition` 为 `Option` 类型，`None` 时回退到默认的全灭胜利条件
- 胜负条件检查仅在 TurnEnd 阶段执行，全灭玩家即失败为绝对不变量（不可被配置覆盖）
- 胜负同时满足时优先判定失败（失败优先原则）
