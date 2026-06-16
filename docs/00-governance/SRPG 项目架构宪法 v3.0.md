# SRPG 项目架构宪法 v3.0
>
> 融合多轮架构评审共识，适配 Bevy 0.18+ 生态，面向数十万行代码规模的长期运营 SRPG 项目
> 文档元数据：
>
> - id: 01-architecture.layer-contracts
> - version: 3.0
> - status: Proposed
> - owner: architect
> - created: 2026-06-14
> - updated: 2026-06-16
> - tags: architecture, layer, domain, bevy, srpg

本版本整合了分层边界、领域拆分、ECS 适配、治理机制四大维度的全部核心共识，修正了早期版本的依赖矛盾与落地缺陷，既保留 DDD 领域纯度的长期价值，也适配 Bevy 生态的工程现实，可直接作为项目骨架落地。

---

## 第一章 架构定位与目标

### 1.1 适用范围

- 引擎：Bevy 0.18+
- 品类：单机战棋 SRPG
- 规模：20万~50万行代码量级
- 模式：长期连载式内容迭代
- 核心特性：Replay First、Data Driven、GAS-Lite
- 扩展目标：自动化测试、战斗模拟器、服务器模拟、Mod 生态

### 1.2 架构优先级

所有架构决策严格遵循以下优先级排序，禁止倒置：

```
正确性 > 可维护性 > 可扩展性 > 开发效率 > 性能
```

性能优化必须基于 Profiling 实证数据，禁止凭体感提前优化。

### 1.3 核心设计思想

摒弃传统「SkillSystem / CombatSystem / UISystem」的系统式目录架构，采用「**6 Layer（技术边界）+ 5级 Domain（业务边界）+ 横切能力治理**」的三维结构，从根源避免后期目录爆炸、边界混乱、依赖蜘蛛网的崩盘式问题。

---

## 第二章 顶层铁则（P0 级）

所有开发活动必须无条件遵守以下原则，无例外空间。

### P0-1 Feature First（领域驱动）

永远按业务领域组织代码，按技术类型拆分是反模式。

- 🟩 允许：`ability/`、`effect/`、`quest/`、`character/`
- 🟥 禁止：`systems/`、`components/`、`events/`、`utils/`、`helpers/`、`common/` 垃圾桶模块

### P0-2 Data Driven First

新增内容优先通过配置数据实现，禁止硬编码业务内容。

- 🟩 允许：通过 RON 配置定义火球术的数值与效果
- 🟥 禁止：在代码中 `match skill_id { Fireball => ... }` 分支写死逻辑

### P0-3 Replay First

所有核心战斗逻辑必须可确定性重放。

- 🟩 允许：使用全局统一的确定性随机源 `BattleRng`
- 🟥 禁止：`rand::thread_rng()` 等不可控随机源破坏回放一致性

### P0-4 逻辑与表现强制分离

业务规则与视觉表现彻底解耦，禁止混写在同一系统中。

- 标准流：Core 计算规则 → 产出 Cue 表现信号 → Infrastructure 层播放动画/音效/特效
- 🟥 禁止：伤害计算与动画播放写在同一个 System 中

### P0-5 组合优于继承

角色、技能、Buff 的差异通过原子能力组合实现，禁止继承式设计。

- 基础能力单元：Tag + Modifier + Effect + Ability
- 🟥 禁止：通过子类派生实现不同角色的行为差异

---

## 第三章 六层架构总览

### 3.1 层级定义（技术边界）

从上层到下层，依赖方向严格单向，下层不得依赖上层。

| 层级 | 名称 | 核心定位 | 依赖权限 |
|------|------|----------|----------|
| Layer 1 | App | 游戏启动与全局装配 | 允许依赖所有层 |
| Layer 2 | Core | 纯游戏规则与领域逻辑 | 仅允许依赖 Shared + bevy_ecs 核心 |
| Layer 3 | Shared | 全局基础原子能力 | 零外部依赖（依赖图叶子节点） |
| Layer 4 | Infrastructure | 技术实现与引擎封装 | 允许依赖 Core + Shared |
| Layer 5 | Content | 数据与规则的桥接层 | 允许依赖 Core + Shared + Infrastructure |
| Layer 6 | Tools | 开发期工具链 | 允许依赖 Core + Shared，永不进入发布构建 |

> 注：原 Modding 独立层已取消，改为**跨层能力**分散到对应层级，避免形成全权限法外之地。

### 3.2 依赖方向总规则

```
App → 所有层（仅装配，不含业务逻辑）
Core → Shared（唯一外部依赖）+ bevy_ecs/bevy_app/bevy_schedule
Shared → 无（纯叶子节点）
Infrastructure → Core + Shared
Content → Core + Shared + Infrastructure
Tools → Core + Shared（开发期专用）
```

🟥 严格禁止反向依赖：Core 不得依赖 Infrastructure/Content/UI，Shared 不得依赖任何业务层。

---

## 第四章 各层详细规范

### Layer 1：App — 启动装配层

#### 职责

唯一职责是组装整个游戏，是全局视野的唯一合法持有者。

- 注册所有 Plugin
- 定义 AppState / TurnPhase 状态机
- 定义 Schedules 和 SystemSets
- 启动资源加载、关闭资源清理

#### 判断标准

这段代码是把游戏「启动起来」的装配逻辑吗？是则归 App 层。

#### 目录结构

```
src/app/
├── app_plugin.rs       # 主 Plugin，汇总注册所有子 Plugin
├── game_state.rs       # AppState 全局状态定义
├── schedules.rs        # Schedule 调度编排
├── sets.rs             # SystemSet 集合定义
├── bootstrap/          # 启动流程
├── shutdown.rs         # 关闭清理逻辑
└── plugins.rs          # 子 Plugin 汇集注册
```

#### 约束

- 🟩 唯一允许依赖所有层的层级
- 🟥 禁止包含任何业务规则逻辑
- 🟥 禁止直接创建业务 Entity（由各业务模块的 startup 系统负责）
- 🟥 禁止硬编码任何游戏数值

### Layer 2：Core — 游戏规则核心层

项目的心脏，唯一允许定义游戏规则的层级。

#### 职责边界判断（三问法）

如果删掉 Bevy 的渲染/音频/资源/输入能力，只保留 ECS 调度，这段逻辑依然成立吗？

- 成立 → Core 层
- 不成立 → Infrastructure 层

#### 允许的引擎依赖

- ✅ `bevy_ecs`：Component、Query、Resource、Event、Observer
- ✅ `bevy_app`：Plugin、App 扩展
- ✅ `bevy_schedule`：Schedule、SystemSet

#### 禁止的引擎依赖

- ❌ `bevy_render`、`bevy_ui`、`bevy_audio`、`bevy_asset`、`bevy_input`
- ❌ 所有平台相关、表现相关、IO 相关的能力

#### 必须在 Core 的内容

战斗规则、属性系统、Buff/Debuff 逻辑、技能规则、装备规则、背包逻辑、寻路算法、AI 决策规则、回合管理、胜负判定、任务规则、对话树、地形规则、职业成长、经济规则、掉落规则

#### 绝对不在 Core 的内容

资源加载、存档序列化、网络通信、UI 渲染、音频播放、输入处理、文件读取、热重载、调试面板、Shader 编译

### Layer 3：Shared — 基础原子层

整个依赖图的根节点，零外部依赖。

#### 职责边界判断

删掉所有游戏逻辑后，这个模块依然有通用价值吗？

- 是 → Shared 层
- 否 → 放回对应业务模块

#### 目录结构（极限瘦身版）

```
src/shared/
├── ids/             # 强类型 ID（UnitId, SkillId, BuffId 等）
├── error/           # 错误上下文 Trait、错误转换工具
├── result/          # 通用 Result 辅助（各领域自行定义错误枚举）
├── math/            # 纯数学工具（距离计算、数值插值等）
├── random/          # 确定性随机数基础接口
├── time/            # 时间工具、帧计数、回合计数基础类型
├── collections/     # 通用集合扩展
├── hashing/         # 哈希工具
├── validation/      # 通用校验工具
└── testing/         # 单元测试基础工具
```

#### 严格准入三问

新增模块必须全部满足：

1. 对所有模块都通用有用吗？
2. 不包含任何业务逻辑吗？
3. 不依赖任何业务类型吗？
任一不满足则不得放入 Shared。

#### 约束

- 🟥 禁止依赖任何其他层
- 🟥 禁止放入序列化、日志实现、本地化、指标统计等技术抽象
- 🟥 禁止定义全局统一 `AppError` 大枚举，错误由各领域自行定义

### Layer 4：Infrastructure — 技术实现层

游戏规则不变、但实现方式可替换的技术能力全部收敛于此。

#### 职责边界判断

游戏规则不变的前提下，能不能换一种技术实现方式？

- 能 → Infrastructure 层
- 不能 → Core 层

#### 内部分三层（避免垃圾桶化）

内部严格单向依赖，禁止反向调用。

1. **Foundation 基础技术层**（被所有 infra 模块依赖）
   - `ecs/`：ECS 通用扩展、工具组件
   - `serialization/`：序列化框架、格式适配
   - `asset/`：资源加载框架、热重载基础
   - `threading/`：线程池、任务调度
   - `memory/`：内存管理、对象池
2. **Services 通用服务层**
   - `save/`：存档读写、版本迁移
   - `localization/`：多语言框架
   - `physics/`：物理碰撞
   - `pathfinding/`：寻路算法实现
   - `navmesh/`：导航网格
   - `input/`：输入处理、按键映射
   - `analytics/`：数据埋点
   - `hot_reload/`：内容热重载
   - `encryption/`：加密能力
   - `anti_cheat/`：反作弊
   - `networking/`：网络通信
   - `cloud/`：云服务对接
3. **Presentation 表现层**
   - `rendering/`：渲染管线、材质、Shader
   - `ui/`：UI 系统（view/widget/screen/navigation/hud/binding）
   - `audio/`：音频播放、音效管理
   - `animation/`：动画系统
   - `particles/`：粒子特效
   - `platform/`：平台适配（Steam/Epic 等）

#### 约束

- 🟥 禁止包含任何游戏规则逻辑
- 🟥 禁止直接修改 Core 领域的内部状态，只能通过事件/接口交互
- 🟥 表现层模块不得被业务服务层依赖

### Layer 5：Content — 内容桥接层

连接外部配置数据与内部规则的桥梁，是 Data Driven 的核心载体。

#### 职责

- 将 RON/JSON/CSV 等配置文件加载为内部 `XxxDef` 类型
- 校验配置数据的完整性、合法性与一致性
- 转换为运行时 `XxxData` 类型，注册到对应 Registry

#### 核心区分

- 规则（Skill 系统）在 Core
- 内容（火球术具体数值）在 Content

#### 目录结构

```
src/content/
├── schema/            # 配置数据结构定义（Def 类型）
├── attributes/
├── tags/
├── modifiers/
├── effects/
├── abilities/
├── triggers/
├── characters/
├── enemies/
├── classes/
├── factions/
├── items/
├── equipments/
├── quests/
├── stories/
├── maps/
├── localization/      # 多语言文本数据
├── balance/           # 数值平衡配置
├── migration/         # 配置版本迁移
└── validation/        # 内容校验规则
```

#### 双类型协作模式

```
Core/skill/skill_def.rs    配置反序列化结构（字符串标识）
Core/skill/skill_data.rs   运行时结构（强类型 ID、位掩码）
Content/skills/            RON → SkillDef → SkillData → 注册到 Registry
```

#### 约束

- 🟥 禁止包含任何游戏规则逻辑，只做「加载 → 校验 → 注册」三件事
- 🟥 禁止硬编码数值内容
- 🟩 测试可用 Mock Registry 替代真实 Registry

### Layer 6：Tools — 开发工具层

开发期专用工具链，永不进入发布构建。

#### 价值预判

大型 SRPG 项目的工具代码通常占总工程量的 20%~40%，是长期迭代的生产力核心。

#### 目录结构

```
src/tools/
├── replay_viewer/       # 回放查看器
├── battle_simulator/    # 战斗模拟器
├── balance_analyzer/    # 数值平衡分析器
├── content_validator/   # 内容合法性校验器
├── localization_tool/   # 多语言工具
├── save_editor/         # 存档编辑器
├── schema_generator/    # 配置 Schema 生成器
├── graph_viewer/        # 依赖/效果管线可视化
├── profiling_tool/      # 性能剖析工具
├── ai_debugger/         # AI 决策调试器
├── map_editor/          # 地图编辑器
├── ability_editor/      # 技能编辑器
├── effect_editor/       # 效果编辑器
├── quest_editor/        # 任务编辑器
└── pipeline_inspector/  # 执行管线检查器
```

#### 约束

- 🟥 永不进入 release 构建
- 🟥 禁止包含业务规则逻辑
- 🟩 开发期可直接操作 Registry 与 World，权限放宽

---

## 第五章 Core 内部五级领域架构

Core 内部同样严格遵守单向依赖，禁止下层依赖上层，禁止同层循环调用，从根源避免「领域蜘蛛网」。

### 5.1 层级与依赖规则

| 层级 | 名称 | 包含领域 | 可依赖范围 |
|------|------|----------|------------|
| L0 | 原子层 | attribute、tag、modifier、stacking | 无（最底层） |
| L1 | 能力基础层 | effect、trigger、targeting、cue | L0 全部领域 |
| L2 | 执行层 | execution、ability、pipeline、registry | L0 + L1 全部领域 |
| L3 | 游戏实体层 | character、equipment、item、combat、map | L0 + L1 + L2 全部领域 |
| L4 | 玩法层 | quest、story、faction、progression、ai、decision、replay | L0~L3 全部领域 |

### 5.2 核心设计原则

- **Combat 必须是薄层**：战斗不是上帝系统，只是 `Ability + Effect + Execution + Pipeline` 的组合场景，不承载核心规则。
- **Buff 不独立成领域**：所有 Buff/Debuff/DOT/HOT/护盾/光环，本质都是 Effect 的子类，统一收敛在 effect 领域。
- **所有能力原子化**：技能、装备、天赋、地形效果，全部复用同一套 Effect + Trigger + Targeting 原子能力。

### 5.3 14 核心领域定义

| 领域 | 职责 | 层级 |
|------|------|------|
| Attribute | 基础属性、派生属性、属性计算公式 | L0 |
| Tag | 统一标签系统，用于分类、条件判断、查询过滤 | L0 |
| Modifier | 属性修改器（加法/乘法/覆盖） | L0 |
| Stacking | 效果堆叠规则（层数、持续时间、刷新机制） | L0 |
| Effect | 效果系统（Buff/Debuff/DOT/HOT/护盾/光环） | L1 |
| Trigger | 触发器（OnHit/OnDeath/OnMove/OnTurnStart 等） | L1 |
| Targeting | 目标选择与范围判定 | L1 |
| Cue | 表现信号系统（连接逻辑与表现的桥梁） | L1 |
| Execution | 公式执行与效果结算 | L2 |
| Ability | 技能系统（主动技/战技/奥义/普攻） | L2 |
| Pipeline | 统一执行管线（技能/效果的流程编排） | L2 |
| Registry | 统一注册中心（所有内容数据的运行时容器） | L2 |
| Replay | 确定性回放系统 | L4 |
| Decision | AI 决策框架 | L4 |

### 5.4 跨领域通信规则

- 同层/上下层领域交互，优先使用 Observer 局部响应
- 跨多个领域的全局通知，使用 Message 广播
- 🟥 禁止直接 use 其他领域的内部实现类型
- 🟥 禁止反向依赖（上层领域的类型不得出现在下层领域的接口中）

---

## 第六章 Modding 能力体系

Modding 不是独立层级，而是贯穿多层的扩展能力，按职责拆分到对应层级，保证边界可控。

| 模块 | 归属 | 职责 |
|------|------|------|
| `core/mod_api/` | Core 层 | 对外暴露的稳定 Mod 接口，是唯一合法的核心规则访问入口，保证向后兼容 |
| `content/mod_support/` | Content 层 | Mod 内容加载、数据覆盖、冲突处理、注册逻辑 |
| `infrastructure/mod_loader/` | Infrastructure 层 | Mod 文件扫描、沙箱隔离、版本校验、依赖管理 |

### 约束

- 🟥 Mod 禁止绕过稳定 API 直接访问 Core 内部实现
- 🟩 Mod 能力与原生内容走同一套 Registry 与执行管线
- 🟩 API 分级：稳定 API / 实验性 API / 内部 API（Mod 禁止调用）

---

## 第七章 横切关注点治理

不单独设立 `crosscutting/` 目录，采用「**抽象定义在 Shared，具体实现在 Infrastructure**」的模式，既保证复用性，又不破坏单向依赖链。

| 横切能力 | 抽象归属 | 实现归属 | 说明 |
|----------|----------|----------|------|
| 日志 Logging | Shared/logging_trait | Infrastructure/logging | Core 仅依赖 Trait，不依赖具体日志框架 |
| 指标 Metrics | Shared/metrics_trait | Infrastructure/analytics | 性能指标、业务指标埋点 |
| 审计 Audit | Shared/audit_trait | Infrastructure/audit | 关键操作审计轨迹 |
| 遥测 Telemetry | - | Infrastructure/analytics | 纯技术层能力，Core 无感知 |
| 事务 Transaction | Shared/transaction_trait | Infrastructure/transaction | 战斗结算原子性保障 |
| 安全 Security | - | Infrastructure/security | 加密、反作弊等纯技术能力 |

---

## 第八章 ECS 使用边界规范

### 核心判定

ECS 是项目的数据模型与执行框架，不是「技术实现」。

- 🟩 业务数据组件（Health、Position、Attribute）允许定义在 Core 层
- 🟥 表现技术组件（Mesh、AudioSource、UiNode）只能定义在 Infrastructure 层

### Core 层允许使用的 ECS 能力

- `#[derive(Component)]` 定义业务数据组件
- `Query`、`Res`、`EventWriter/Reader`、`Observer`、`Trigger`
- `Schedule`、`SystemSet` 调度编排
- `Resource` 全局业务状态

### Core 层禁止使用的 ECS/引擎能力

- `AssetServer`、`Handle<Image>` 等资源加载类型
- `RenderDevice`、`Texture` 等渲染类型
- `Input`、`KeyCode` 等输入类型
- `AudioPlayer`、`SpatialAudio` 等音频类型
- 所有 UI 组件与系统

---

## 第九章 跨层通信规范

三级通信机制，优先级从高到低，优先使用最小粒度的通信方式，禁止事件滥用。

### 1. Hook（固有钩子）

- 场景：组件生命周期的固有行为（添加/移除时的自动处理）
- 范围：单个领域内部使用
- 示例：添加 `Buff` 组件时自动触发 Modifier 计算

### 2. Observer（局部观察者）

- 场景：相邻领域之间的响应式交互
- 范围：跨 1~2 个领域的事件响应
- 示例：`EffectApplied` 触发属性更新

### 3. Message（全局广播）

- 场景：跨多个领域的全局业务事件
- 范围：跨层、跨多个模块的广播通知
- 示例：`BattleEnded`、`QuestCompleted`、`UnitDied`

### 跨层通信标准路径

- **Core → UI**：Core 更新业务组件 → ViewModel 只读查询 → UI 渲染
- **UI → Core**：UI 发出 `UiCommand` → 命令分发器 → Core 业务系统执行
- **Core → Infrastructure**：Core 发布领域事件 → Infrastructure Observer 监听执行技术操作
- **Content → Core**：Content 加载数据 → 调用 Registry 接口注册

---

## 第十章 Replay 体系规范

### 10.1 确定性要求

同一份回放文件，在以下环境中运行结果必须 100% 一致：

- 本地客户端
- Headless 模式
- 服务器模拟
- 自动化测试

### 10.2 记录范围

- ✅ 必须记录：初始种子、所有玩家指令、版本号、关键状态快照
- ❌ 禁止记录：动画、UI 状态、音频、粒子等表现层内容

### 10.3 约束

- 所有随机数必须来自确定性随机源
- 禁止系统时间、线程调度等不可控因素影响核心逻辑
- 回放系统必须支持快进、倒退、逐帧调试

---

## 第十一章 测试架构体系

测试代码与业务代码同等重要，单独作为一级体系管理，严格遵守分层依赖规则。

### 目录结构

```
tests/
├── unit/          # 单领域单元测试，仅依赖被测领域 + Shared
├── integration/   # 跨领域集成测试，验证领域间交互
├── replay/        # 回放一致性测试，验证确定性
├── golden/        # 金档测试，标准输出比对
├── simulation/    # 战斗模拟测试，数值平衡验证
└── performance/   # 性能回归测试
```

### 约束

- 单元测试不得引入 Infrastructure 依赖
- 测试代码同样禁止破坏分层边界
- Mock 工具统一收敛在 Shared/testing

---

## 第十二章 架构例外与演进机制

### 12.1 性能例外申请

严格禁止凭感觉优化突破架构边界，申请例外必须同时满足：

1. 有 Profiling 实证数据证明跨层通信产生了可观测的性能瓶颈
2. 仅限核心战斗路径（伤害计算、属性结算、寻路等高频调用）
3. 提交 ADR 架构决策记录，说明原因、影响范围、有效期
4. 明确后续重构计划与到期时间

### 12.2 例外标记规范

```rust
// ARCH_EXCEPTION: 战斗伤害计算性能优化
// ADR-042
// Expires: 2027-01-01
// Approver: Architect
// 说明：跳过事件广播，直接函数调用，降低结算开销
```

### 12.3 架构版本管理

采用语义化版本，分级审批：

- **MAJOR**：结构性重构（合并/拆分层级）→ 全员评审 + 负责人批准
- **MINOR**：新增领域、新增接口 → Architect 审批
- **PATCH**：规则修正、文档补全 → 自主修改 + 同步周知

### 12.4 演进流程

需求评审 → ADR 记录 → 文档更新 → 全团队同步 → 代码迁移 → 验收确认
迁移期间新旧代码共存，通过 `#[deprecated]` 标记，确保零遗留。

---

## 第十三章 红线禁止事项

1. 🟥 禁止创建 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件
2. 🟥 禁止用 `bool` 标志位替代 Tag 系统
3. 🟥 禁止面向对象式实体调用（如 `player.attack(enemy)`）
4. 🟥 禁止非确定性随机源破坏回放
5. 🟥 禁止 UI 层持有业务真相，UI 只能只读展示
6. 🟥 禁止直接修改最终属性值，必须通过 Modifier 管线
7. 🟥 禁止 Core 层引入渲染、音频、资源、输入等引擎表现能力
8. 🟥 禁止 Shared 层引入任何业务逻辑或业务类型
9. 🟥 禁止反向依赖与循环依赖
10. 🟥 禁止硬编码游戏数值与业务内容

---

## 第十四章 落地治理与检查清单

### 14.1 新增模块检查清单

每次新增模块/文件必须逐项确认：

- [ ] 明确归属层级与领域
- [ ] 未依赖禁止的层级与模块
- [ ] 未包含不属于当前层的逻辑
- [ ] 错误类型定义在对应领域内部
- [ ] 强类型 ID 放在 `shared/ids/`
- [ ] 未创建 utils/helpers 垃圾桶文件

### 14.2 自动化门禁

- 开发期：自定义 Clippy Lint 实时提示跨层依赖违规
- CI 阶段：运行 `dependcheck` 脚本全量扫描，违规直接阻断 PR
- 规则：架构违规零容忍，不允许「先通过后修复」

---

## 最终总结

本架构是 DDD 领域驱动、GAS 原子化设计、Bevy ECS 工程实践三者的融合方案，既保证了数十万行代码规模下的边界清晰与可维护性，也充分适配了 Bevy 生态的开发范式，避免了过度抽象的落地障碍。

核心价值在于：

- 技术边界与业务边界双层隔离，从根源避免架构腐化
- 原子化的 GAS-Lite 设计，支撑 Data Driven 与长期内容迭代
- 完整的治理与门禁机制，保证规则在多人协作下长期生效
- 前置的回放、测试、工具设计，匹配长期运营型项目的核心需求

需要的话，我可以基于这份宪法，输出一份可直接初始化项目的**目录结构脚手架**与**依赖检查脚本初版**。
