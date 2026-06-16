# SRPG 项目架构宪法 v4.0
>
> 融合多轮架构评审共识，适配 Bevy 0.18+ 生态，面向数十万行代码规模的长期运营 SRPG 项目
> 文档元数据：
>
> - id: 01-architecture.layer-contracts
> - version: 4.0
> - status: Proposed
> - owner: architect
> - created: 2026-06-14
> - updated: 2026-06-16
> - tags: architecture, layer, domain, bevy, srpg

本版本整合了分层边界、领域拆分、ECS 适配、治理机制四大维度的全部核心共识，修正了早期版本的依赖矛盾与落地缺陷，既保留 DDD 领域纯度的长期价值，也适配 Bevy 生态的工程现实，可直接作为项目骨架落地。

### 版本升级说明（v3.0 → v4.0）

1. **Core 内部架构升级**：将原5级分层（L0~L4）重构为「**Capabilities 能力层 + Domains 业务域 + Mod API**」双轴结构
2. **领域数量扩展**：Capabilities 从14个核心领域扩展为15个（新增 GameplayContext 上下文载荷），Domains 从14个业务域扩展为15个（新增 Summon 召唤域）
3. **内聚优于分层原则**：Capabilities 内部从5层精简为3层（Foundation/Mechanism/Runtime），同一领域的代码不再跨目录拆分
4. **三层分离原则**：Def（模板）→ Spec（配置）→ Instance（运行时）贯穿能力系统全链路
5. **Mod API 升级**：重构为 Facade + Gateway 模式，每个 Domain 对应一个 Gateway，对外只暴露业务语义接口

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

摒弃传统「SkillSystem / CombatSystem / UISystem」的系统式目录架构，采用「**6 Layer（技术边界）+ Capabilities/Domains 双轴结构 + 横切能力治理**」的三维结构，从根源避免后期目录爆炸、边界混乱、依赖蜘蛛网的崩盘式问题。

### 1.4 设计原则

| 原则 | 说明 |
|------|------|
| **领域优先** | 游戏领域概念（Tag/Attribute/Ability）与架构组件（Container/Manager）严格分离 |
| **组合优于创建** | 新玩法优先组合已有机制（Cost=Effect, Cooldown=Tag+Effect），不造新系统 |
| **三层分离** | Def（模板）→ Spec（配置）→ Instance（运行时），贯穿能力系统全链路 |
| **内聚优于分层** | 同一领域的代码放在一起（内聚），而非按抽象层级拆散到不同目录（分层） |
| **单向依赖** | Domains → Capabilities → Shared，禁止反向；Domain 间仅通过事件通信 |
| **数据驱动** | 玩法规则下沉 Domain/rules/，数值配置归 content/，代码只提供机制 |
| **可换可测** | 每层可独立替换与测试，纯函数规则零 ECS 依赖 |
| **个人开发友好** | 避免团队级复杂流程，优先轻量方案 |

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

项目的心脏，唯一允许定义游戏规则的层级。**内部采用「Capabilities 能力层 + Domains 业务域」双轴结构**，兼顾通用能力复用与业务逻辑内聚。

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
├── testing/         # 单元测试基础工具
├── traits/          # 通用横切抽象（日志、审计、事务等 Trait）
└── prelude/         # 统一导出
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

## 第五章 Core 内部双轴架构（Capabilities + Domains）

> 核心升级：摒弃单纵向5级分层（L0~L4），采用「**纵向 Capabilities 能力复用 + 横向 Domains 业务内聚**」双轴结构。纵向 Capabilities 提供通用机制，横向 Domains 封装业务规则，既保证能力复用，又实现业务内聚。

### 5.1 双轴架构总览

```
src/core/
├── core_plugin.rs              # Core 层总 Plugin，注册所有能力与领域
│
├── capabilities/               # 纵向：15个核心能力领域（通用机制骨架，玩法无关）
│   ├── tag/                    # 标签
│   ├── attribute/              # 属性
│   ├── modifier/               # 修改器
│   ├── aggregator/             # 聚合器
│   ├── gameplay_context/       # 上下文/载荷
│   ├── spec/                   # 规格/配置
│   ├── ability/                # 技能逻辑
│   ├── trigger/                # 触发器
│   ├── condition/              # 条件/限制/免疫
│   ├── targeting/              # 目标选择
│   ├── execution/              # 执行计算
│   ├── effect/                 # 效果
│   ├── stacking/               # 堆叠规则
│   ├── event/                  # 系统通信
│   ├── cue/                    # 表现层信号
│   └── runtime/                # C3：跨领域运行时编排底座
│
├── domains/                    # 横向：15个业务子系统（承载全部玩法复杂度）
│   ├── combat/                 # 战斗
│   ├── spell/                  # 法术
│   ├── reaction/               # 反应/援护
│   ├── progression/            # 成长养成
│   ├── inventory/              # 背包物品
│   ├── quest/                  # 任务
│   ├── narrative/              # 叙事/对话
│   ├── tactical/               # 战术/网格
│   ├── terrain/                # 地形
│   ├── faction/                # 阵营关系
│   ├── party/                  # 队伍
│   ├── camp_rest/              # 营地休息
│   ├── crafting/               # 制作
│   ├── economy/                # 经济
│   └── summon/                 # 召唤
│
└── mod_api/                    # Mod 稳定 API（Facade + Gateway 模式）
```

#### 核心分工原则

| 维度 | 职责定位 | 核心产出 | 稳定性 |
|------|----------|----------|--------|
| Capabilities 纵向 | 通用机制与原子能力 | 可复用的系统骨架、数据结构、执行框架 | 极高，极少变更 |
| Domains 横向 | 特定玩法的规则编排 | 业务逻辑、玩法规则、流程控制 | 中等，随玩法迭代 |

> 一句话总结：**Capabilities 管「机制」，Domains 管「规则」**。机制是通用的，规则是业务专属的。

### 5.2 Capabilities 内部三层架构（C1/C2/C3）

> 设计思路：将原5层架构（atom/rule/runtime/model/frame）精简为3层，
> 核心理由是"内聚优于分层"——同一领域的代码应放在一起，而非按抽象层级拆散。
> 原L0(atom)+L1(rule)的人为分裂导致同一领域代码跨层修改，违反内聚性；
> 原L3(model)与Domain的components.rs职责重叠，造成数据结构双份维护；
> 原L4(frame)与Domain的plugin.rs高度重复，增加"放L4还是放Domain"的决策负担。

#### 三层职责划分

| 层 | 名称 | 职责 | 禁止事项 | 对应原5层 |
|------|------|------|----------|----------|
| **C1** | Foundation | 纯数据定义、类型枚举、值对象 | 禁止包含任何行为逻辑、系统、ECS组件 | L0 atom |
| **C2** | Mechanism | 规则组件、查询系统、生命周期管理、ECS组件与System | 禁止包含具体玩法内容 | L1 rule + L2 runtime(部分) |
| **C3** | Runtime | 跨领域能力的运行时编排底座 | 禁止包含具体业务逻辑 | L2 runtime(部分) + L4 frame(事件总线) |

#### 各能力领域内部结构（遵循 C1→C2 内聚）

```
capabilities/<domain>/
├── plugin.rs              # 领域 Plugin（C2）
├── foundation/            # C1：纯数据定义层
│   ├── mod.rs
│   ├── types.rs           # 基础类型与枚举
│   └── values.rs          # 值对象定义
├── mechanism/             # C2：规则与系统层
│   ├── mod.rs
│   ├── components.rs      # ECS 组件
│   ├── query.rs           # 查询/匹配/条件逻辑
│   ├── lifecycle.rs       # 生命周期管理
│   └── systems/            # Bevy Systems
│       ├── mod.rs
│       └── xxx_system.rs
└── events.rs              # 领域事件（C2）
```

#### C3 Runtime 独立模块

```
capabilities/runtime/       # C3：跨领域运行时编排底座
├── plugin.rs
├── pipeline/              # 通用执行管线框架
├── scheduler/             # 时序调度器
├── registry/              # 统一注册中心
├── command/               # 通用命令模式框架
└── replay/                # 回放基础框架
```

### 5.3 依赖方向规则

```
# 依赖方向：C3 → C2 → C1 → Shared（严格单向）
# Domains → Capabilities → Shared（严格单向）
# Domain 间禁止直接引用，仅通过事件通信
```

#### C2↔C3 交互判定规则

| 场景 | 归属 | 理由 |
|------|------|------|
| 标签同步（Tag→Tag） | C2 tag/systems | 单领域内部逻辑 |
| 属性聚合（Modifier→Aggregator→Attribute） | C2 各领域 + C3 pipeline | 跨多个领域，需 C3 pipeline 编排 |
| 技能激活（Condition→Trigger→Ability→Targeting→Execution→Effect） | C3 pipeline | 跨6个领域的链式执行 |
| 回合调度（TurnScheduler） | C3 scheduler | 跨所有战斗参与者的时序编排 |
| 堆叠规则检查 | C2 stacking/systems | 单领域内部逻辑 |

### 5.4 核心设计原则

- **Capabilities 15领域封顶**：不再增加新能力领域，新机制以子模块形式归入已有领域
- **组合优于创建**：法力消耗=Attribute(Mana)+Effect(Cost)，冷却=Tag(Cooldown.X)+Effect(Duration)，不造新系统
- **三层分离**：Def（模板）→ Spec（配置）→ Instance（运行时），贯穿能力系统全链路
- **Combat 必须是薄层**：战斗不是上帝系统，只是 Ability + Effect + Execution + Pipeline 的组合场景
- **Buff 不独立成领域**：所有 Buff/Debuff/DOT/HOT/护盾/光环，本质都是 Effect 的子类，统一收敛在 effect 领域

### 5.5 15 核心能力领域定义

| # | 领域 | 职责 | 分组 |
|---|------|------|------|
| 1 | Tag | 标签定义、位掩码、层级关系 | 核心基石 |
| 2 | Attribute | 属性定义、基础值、当前值 | 核心基石 |
| 3 | Modifier | 修改器定义（Add/Mul/Override） | 核心基石 |
| 4 | Aggregator | 属性聚合管线（Base→Add→Mul→Override→Clamp→Final） | 核心基石 |
| 5 | GameplayContext | 统一上下文/载荷，贯穿全链路的数据载体 | 核心基石 |
| 6 | Spec | 模板→配置→实例三层分离 | 逻辑骨架 |
| 7 | Ability | 技能逻辑与生命周期 | 逻辑骨架 |
| 8 | Trigger | 技能激活条件 | 逻辑骨架 |
| 9 | Condition | 统一条件检查（含免疫/限制/激活条件） | 逻辑骨架 |
| 10 | Targeting | 目标选择（含 Grid 范围） | 行为表现 |
| 11 | Execution | 执行计算（伤害/治疗/自定义） | 行为表现 |
| 12 | Effect | 效果（含 Period/Duration/Instant） | 行为表现 |
| 13 | Stacking | 堆叠规则 | 行为表现 |
| 14 | Event | 系统间结构化通信 | 行为表现 |
| 15 | Cue | 表现层信号（VFX/SFX/动画触发） | 行为表现 |

### 5.6 跨领域通信规则

- 同领域内：优先使用 Observer 局部响应
- 跨多个能力领域：通过 C3 Runtime pipeline 编排
- 跨 Domain：使用 Event（Message）广播
- 🟥 禁止直接 use 其他领域的内部实现类型
- 🟥 禁止反向依赖（上层领域的类型不得出现在下层领域的接口中）

---

## 第六章 Modding 能力体系

Modding 不是独立层级，而是贯穿多层的扩展能力，按职责拆分到对应层级，保证边界可控。

| 模块 | 归属 | 职责 |
|------|------|------|
| `core/mod_api/` | Core 层 | 对外暴露的稳定 Mod 接口，Facade + Gateway 模式，唯一合法的核心规则访问入口，保证向后兼容 |
| `content/mod_support/` | Content 层 | Mod 内容加载、数据覆盖、冲突处理、注册逻辑 |
| `infrastructure/mod_loader/` | Infrastructure 层 | Mod 文件扫描、沙箱隔离、版本校验、依赖管理 |

### Mod API：Facade + Gateway 模式

```
src/core/mod_api/
├── core_facade.rs         # 统一门面，Mod 唯一入口
├── combat_gateway.rs      # 战斗系统网关
├── character_gateway.rs   # 角色系统网关
├── spell_gateway.rs       # 法术系统网关
├── quest_gateway.rs       # 任务系统网关
├── party_gateway.rs       # 队伍系统网关
├── camp_gateway.rs        # 营地系统网关
├── summon_gateway.rs      # 召唤系统网关
├── terrain_gateway.rs     # 地形系统网关
├── craft_gateway.rs       # 制作系统网关
├── economy_gateway.rs     # 经济系统网关
├── inventory_gateway.rs   # 物品系统网关
├── faction_gateway.rs     # 阵营系统网关
├── progression_gateway.rs # 成长系统网关
├── narrative_gateway.rs   # 叙事系统网关
└── registry_gateway.rs    # 注册中心网关
```

#### 设计原则
- 🟩 **面向业务暴露**：Mod 作者只需要理解「战斗/技能/角色」等业务概念，不需要了解 Capabilities/Domains 内部分层
- 🟩 **稳定隔离**：内部重构时，只要保持网关接口不变，Mod 完全不受影响
- 🟩 **分级授权**：API 分为稳定级、实验级、内部级，Mod 只能调用稳定级与公开实验级
- 🟥 **绝对禁止**：Mod 绕过网关直接访问 Capabilities 或 Domains 的内部实现

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
11. 🟥 禁止 Capabilities 层包含具体业务规则，突破机制与业务的边界
12. 🟥 禁止 Domain 之间直接依赖、直接调用内部实现
13. 🟥 禁止 Domain 重复实现 Capabilities 已有的通用机制

---

## 第十四章 落地治理与检查清单

### 14.1 新增模块检查清单

每次新增模块/文件必须逐项确认：

- [ ] 明确归属层级与领域/Domain
- [ ] 未依赖禁止的层级与模块
- [ ] 未包含不属于当前层的逻辑
- [ ] Capabilities 模块未包含业务规则，Domain 未重复实现通用机制
- [ ] 未突破 Domain 边界直接依赖其他 Domain
- [ ] 错误类型定义在对应领域内部
- [ ] 强类型 ID 放在 `shared/ids/`
- [ ] 未创建 utils/helpers 垃圾桶文件

### 14.2 自动化门禁

- 开发期：自定义 Clippy Lint 实时提示跨层依赖违规、Domain 边界违规
- CI 阶段：运行 `dependcheck` 脚本全量扫描，违规直接阻断 PR
- 规则：架构违规零容忍，不允许「先通过后修复」

---

## 最终总结

本架构是 DDD 领域驱动、GAS 原子化设计、Bevy ECS 工程实践三者的融合方案，既保证了数十万行代码规模下的边界清晰与可维护性，也充分适配了 Bevy 生态的开发范式，避免了过度抽象的落地障碍。

核心价值在于：

- 技术边界与业务边界双层隔离，从根源避免架构腐化
- Capabilities/Domains 双轴结构，兼顾能力复用与业务内聚
- 原子化的 GAS-Lite 设计，支撑 Data Driven 与长期内容迭代
- 完整的治理与门禁机制，保证规则在多人协作下长期生效
- 前置的回放、测试、工具设计，匹配长期运营型项目的核心需求

需要的话，我可以基于这份宪法，输出一份可直接初始化项目的**目录结构脚手架**与**依赖检查脚本初版**。
