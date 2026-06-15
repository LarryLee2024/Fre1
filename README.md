# Tactical RPG

基于 Bevy 0.18.1 的回合制战棋游戏，采用 ECS 架构与数据驱动设计。

## 核心功能

- **回合制战斗**：基于回合阶段状态机（选择 → 行动 → 结算），支持移动、攻击、技能、道具等操作
- **数据驱动配置**：单位、技能、Buff、地形、特质、AI 行为、关卡均通过 RON 文件配置，无需修改代码即可扩展内容
- **关卡与胜负条件**：关卡配置包含可组合的胜利/失败条件（全灭、存活回合、击败 Boss、超时等），支持多条件 OR 组合
- **Effect Pipeline**：战斗效果通过生成 → 修饰 → 执行的管线处理，支持 Trait/Modifier 组合扩展
- **AI 系统**：数据驱动的 AI 行为配置，支持多种策略模板
- **调试面板**：基于 egui 的运行时调试工具，支持 World Inspector 和状态查看

## 安装指南

### 环境要求

- Rust 1.96+（edition 2024）
- Cargo

### 构建与运行

```bash
# 克隆项目
git clone <repository-url>
cd a1

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
  core/         # 业务逻辑 — 战斗、技能、Buff、地图、角色、AI、回合、装备、背包、战役
    battle/     # 战斗效果管线（generate → modify → execute）
    buff/       # Buff/Debuff 系统
    skill/      # 技能系统
    character/  # 单位组件与 Trait 扩展体系
    map/        # 地图、寻路、关卡配置加载
    turn/       # 回合状态机、行动顺序、胜负条件检查
    ai/         # AI 行为系统
    equipment/  # 装备系统
    inventory/  # 背包系统
    campaign/   # 战役编排与关卡序列
  shared/       # 跨层共享类型（强类型 ID、错误工具）
  infrastructure/ # 基础设施（资源加载、日志）
  content/      # Content 层统一入口（RON 加载模块）
  ui/           # 用户界面面板与组件
  input/        # 输入处理
  debug/        # 调试面板与查看器
  app/          # 应用装配与 Plugin 编排

content/       # RON 游戏配置（数据驱动）
  characters/  # 单位模板
  skills/      # 技能定义
  buffs/       # Buff 定义
  classes/     # 职业与特质
  terrains/    # 地形类型
  ai_behaviors/ # AI 行为模板
  stages/      # 关卡配置
  campaigns/   # 战役定义
  definitions/ # 属性与标签定义
  modifiers/   # 修饰规则（元素交互等）
  equipments/  # 装备定义
  items/       # 物品定义

assets/        # 二进制资源（字体、数据）
  fonts/       # 字体文件
  data/        # 运行时数据

docs/
  00-governance/     # 治理规则（AI 开发宪法、编码规范）
  01-architecture/   # 架构设计（README.md 为最高优先级）
  02-domain/         # 领域规则文档（按子领域分组）
  03-technical/      # 技术实现文档（ECS、通信、性能）
  04-data/           # 数据与配置文档
  05-testing/        # 测试规范
  06-ai/             # AI 协作流程
  07-operations/     # 运维文档
  08-decisions/      # 架构决策记录（ADR）
  09-planning/       # 执行计划
  10-reviews/        # 代码审查记录
  11-refactor/       # 技术债扫描记录
  98-roadmap/        # 项目路线图
  99-history/        # 历史归档
  其他/              # 临时目录

tests/         # 集成测试、场景测试、快照测试
```

## 文档详细说明

### 顶层核心文档（最高优先级）

| 文件 | 说明 | 优先级 |
|------|------|--------|
| `docs/01-architecture/README.md` | 七层架构总纲（v4.0），Feature 边界、ECS 规则、Effect/Modifier 管线 | 🟥 **最高** |
| `docs/00-governance/ai-constitution-complete.md` | AI 开发宪法 v1.6 完整版（20 部分），覆盖架构/ECS/代码/测试/日志/工程质量 | 🟥 **最高** |
| `docs/00-governance/coding-rules.md` | 编码执行规范 v1.0，AI 编码自检清单，Effect/Modifier 管线保护 | 🟩 必须遵守 |
| `docs/02-domain/README.md` | 领域规则汇总索引，39 个领域文件的速查入口 | 🟩 必须遵守 |
| `docs/05-testing/test-spec.md` | 测试宪法 v3.1，测试分层/回放测试/覆盖率策略 | 🟩 必须遵守 |

### `docs/02-domain/` — 领域规则（39 文件）

按领域子目录分组，开发对应功能时直接查阅：

**Core Domain（14 文件）— 核心业务规则**

| 文件 | 关键词 |
|------|--------|
| `battle/battle-rules.md` | 战斗状态机、Effect Pipeline、伤害计算 |
| `character/character-rules.md` | 角色属性、Faction、UnitSnapshot |
| `skill/skill-rules.md` | 技能定义、冷却、五阶段释放管线 |
| `attribute-modifier/attribute-modifier-rules.md` | Modifier 管线、属性修饰、叠加规则 |
| `turn/turn-rules.md` | TurnPhase、回合阶段、行动队列 |
| `trigger/trigger-rules.md` | 触发器、事件链（伤害→护盾→吸血→反击） |
| `condition/condition-rules.md` | 条件系统、效果判断、运行时条件 |
| `formula/formula-rules.md` | 公式系统、数值计算、表达式求值 |
| `selector/selector-rules.md` | 目标选择、AOE、空地选择 |
| `duration/duration-rules.md` | 持续时间（回合/真实时间/永久） |
| `cost/cost-rules.md` | 消耗系统、资源扣除 |
| `stack-policy/stack-policy-rules.md` | 堆叠策略、Buff叠加/替换 |
| `requirement/requirement-rules.md` | 释放前提、技能可用性检查 |
| `input/input-rules.md` | 输入处理、UiCommand |

**Infrastructure（7 文件）— 基础设施规则（→ `docs/03-technical/`、`docs/05-testing/`）**

| 文件 | 关键词 |
|------|--------|
| `03-technical/error-system-rules.md` | 错误处理、Result 传播、分级 |
| `03-technical/logging-rules.md` | 日志分级、格式、调试日志 |
| `03-technical/persistence-rules.md` | 存档格式、版本迁移 |
| `03-technical/hot-reload-rules.md` | Definition 热更新、战斗中禁止 |
| `03-technical/determinism-rules.md` | 确定性、多 RNG 流独立 |
| `03-technical/replay-rules.md` | 战斗回放、Command Stream |
| `05-testing/testing-rules.md` | 测试金字塔、回放测试 |

**Content/Data（6 文件）— 数据与内容规则（→ `docs/04-data/`）**

| 文件 | 关键词 |
|------|--------|
| `04-data/content-system-rules.md` | RON 加载、Registry、Definition 不可变 |
| `04-data/config-system-rules.md` | 运行时配置、热重载 |
| `04-data/content-migration-rules.md` | 版本兼容、字段兼容 |
| `04-data/asset-lifecycle-rules.md` | 资源生命周期、Handle 类型、内存预算 |
| `04-data/asset-organization-rules.md` | 三树分离、命名空间 |
| `04-data/feature-flag-rules.md` | Feature Flag、灰度发布 |

**Cross-cutting（12 文件）— 横切关注点**

| 文件 | 关键词 |
|------|--------|
| `01-architecture/layer-architecture-rules.md` | 分层架构、层间依赖方向 |
| `03-technical/ecs-communication-rules.md` | Hook/Observer/Message/Trigger |
| `03-technical/command-bus-rules.md` | UiCommand、命令总线 |
| `03-technical/shared-layer-rules.md` | Shared 层、公共类型 |
| `03-technical/modding-system-rules.md` | MOD 加载、资源隔离 |
| `03-technical/ui-architecture-rules.md` | ViewModel、UiCommand、UI 渲染 |
| `03-technical/localization-rules.md` | 多语言、Fluent |
| `02-domain/map/map-terrain-rules.md` | 地图地形、寻路、视野 |
| `02-domain/ai/ai-rules.md` | AI 行为、策略模板、决策管线 |
| `03-technical/performance-budget-rules.md` | 帧率目标、内存限制 |
| `04-data/validation-rules.md` | 数据完整性、配置校验 |
| `03-technical/event-audit-rules.md` | 事件审计、双轨制日志 |

### `docs/01-architecture/` — 架构设计（38 文件）

七层架构各领域的详细设计文档：

| 分组 | 文件 | 核心内容 |
|------|------|----------|
| 🏗️ 架构总纲 | `app-bootstrap.md` | App 装配器、状态机、启动/关闭序列 |
| | `layer-contracts.md` | 七层边界定义、三问判断法 |
| | `layer-architecture-rules.md` | 分层架构、层间依赖方向 |
| | `project-structure.md` | 三棵树分离、完整源码/资产/内容树 |
| | `plugin-design.md` | Plugin 生命周期、声明式注册 |
| | `plugin-contract-rules.md` | 显式依赖、API 最小化、分层禁令 |
| | `schedules-design.md` | 自定义 Schedule、SystemSet 排序 |
| | `infrastructure-design.md` | 20 个 Infrastructure 模块 |
| | `migration-roadmap.md` | 7 Phase 迁移计划 |
| ⚔️ 战斗/技能 | `battle-fsm-design.md` | 战斗 FSM、Guard/Action/Effect 三段式 |
| | `skill-buff-abstraction.md` | Effect Executor 抽象、ActionQueue |
| | `command-bus-design.md` | GameCommand、Memento 撤销 |
| 🧙 角色/属性 | `component-design-rules.md` | 四位一体组件分类、Hook 安全 |
| | `system-design-rules.md` | Query 参数上限、读写分离 |
| | `determinism-rules.md` | ChaCha8Rng、整数精度、状态哈希 |
| 🗺️ 地图/寻路 | `pathfinding-design.md` | PathFinder trait、UnitBlocker |
| 🎨 UI | `ui-domain-boundary-rules.md` | 单向数据流、ViewModel、UiCommand |
| 📦 数据/配置 | `content-pipeline.md` | RON→Def→Data→Registry、LoadingProgress |
| | `content-data-format.md` | RON 契约、两阶段加载 |
| | `content-migration-design.md` | 内容格式迁移链 |
| | `config-system-design.md` | 四层配置、反上帝配置 |
| | `ids-design.md` | Strong ID newtype、define_id! 宏 |
| | `asset-organization.md` | 三树分离、Content Packs、外包工作流 |
| | `asset-lifecycle-rules.md` | Handle 选择、分阶段卸载、内存预算 |
| | `asset-namespace-design.md` | 命名空间前缀、MOD 隔离 |
| | `save-migration-rules.md` | 存档 SemVer、三步删除原则 |
| 📋 事件/日志/错误 | `events-audit-design.md` | 独立 Struct 事件、EventWhitelist |
| | `logging-design.md` | 领域事件驱动日志、LogObserver |
| | `error-architecture.md` | 三层错误、失败分类学 |
| 🔧 工具/调试 | `tools-architecture.md` | Tools 二进制、data_validator |
| | `testing-architecture.md` | 五层测试金字塔、Golden Test |
| | `validation-rules.md` | 校验检查点、全局不变量 |
| | `performance-budget.md` | 60fps 帧预算、模块级预算 |
| | `feature-flag-design.md` | 7 个 Feature Flag、PluginGroup |
| | `i18n-design.md` | Fluent 国际化、字体回退链 |
| | `modding-design.md` | MOD 生命周期、分级权限策略 |
| | `collaboration-model.md` | AI 6-Agent 协作、Handoff 协议 |

### `docs/08-decisions/` — 架构决策记录（27 ADR）

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

### `docs/09-planning/` — 执行计划

| 文件 | 说明 |
|------|------|
| `adr-026-gap-analysis-and-action-plan.md` | ADR-026 Gap 分析与行动计划 |
| `business-module-execution-plan.md` | 业务模块执行计划 |

### `docs/10-reviews/` — 代码审查记录

| 文件 | 说明 |
|------|------|
| `adr-026-implementation-review.md` | ADR-026 实现审查报告 |

### `docs/11-refactor/` — 技术债扫描记录


## 架构原则

项目遵循以下核心架构原则（详见 `docs/architecture.md`）：

1. **Definition / Instance 分离**：配置数据（如 UnitTemplate）不可变，运行时实例（如 Unit）可变
2. **Rule / Content 分离**：检查逻辑是规则，RON 配置是内容
3. **Logic / Presentation 分离**：业务逻辑在 System 中，UI 层只读取状态
4. **数据驱动**：游戏内容通过 RON 文件配置，禁止硬编码

## AI 辅助开发

项目配备 6 个专用 AI Agent（详见 `AGENTS.md`），遵循严格的协作流程：

```
需求 → @domain-designer → @architect → @feature-developer → @test-guardian → @code-reviewer
```

## 注意事项

- 配置路径使用编译时绝对路径（`CARGO_MANIFEST_DIR`），发布构建时需确保 content/ 和 assets/ 目录与可执行文件相对位置正确
- 关卡配置中 `victory_condition` 为 `Option` 类型，`None` 时回退到默认的全灭胜利条件
- 胜负条件检查仅在 TurnEnd 阶段执行，全灭玩家即失败为绝对不变量（不可被配置覆盖）
- 胜负同时满足时优先判定失败（失败优先原则）
