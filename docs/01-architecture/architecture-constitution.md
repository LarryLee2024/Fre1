---
id: ARCH-CONSTITUTION
title: 架构体系与双轴架构宪法
status: accepted
stability: stable
layer: architecture
related:
  - ai-constitution-complete.md
  - crosscutting-modding-constitution.md
tags:
  - architecture
  - capabilities
  - domains
  - plugin
---

> **原文来源**：`ai-constitution-complete.md` 第二编（L64-L427）、第三编（L430-L697）、第七编（L843-L865）
>
> **锚定总宪法**：第一编总则、第二编、第三编、第七编

## 第二编 纵向三层 + 横向四层架构体系
### 2.1 架构总览
采用「DDD 纵向三层 + 横切四层」的双轴架构体系，从根源避免目录爆炸、边界混乱、依赖蜘蛛网问题。

```
src/
├── main.rs                   # 程序入口
├── lib.rs                    # 库根
│
│                           ┌─ DDD 纵向三层 ─┐
├── shared/                   # L0：底层原子层。零业务语义的通用工具
├── core/                     # L1：领域规则层。能力机制 + 业务子系统
├── infra/                    # L2：技术实现层。渲染/持久化/网络等"脏活"
│                           └─────────────────┘
│                           ┌─ 横切四层 ─┐
├── app/                      # 横切1：启动装配层（Composition Root）
├── content/                  # 横切2：内容桥接层（数据驱动核心）
├── tools/                    # 横切3：开发工具层（feature-gated）
└── modding/                  # 横切4：Mod 扩展层（跨层聚合）
                            └─────────────┘
```

#### DDD 纵向三层（核心依赖链）

| 层级 | 名称 | 核心定位 | 依赖权限 |
|------|------|----------|----------|
| **L0** | Shared | 全局基础原子能力，零业务语义 | 允许稳定基础库依赖，禁止业务依赖 |
| **L1** | Core | 纯游戏规则与领域逻辑（内部双轴结构） | 仅允许依赖 Shared + bevy_ecs 核心 |
| **L2** | Infrastructure | 技术实现与引擎封装 | 允许依赖 Core + Shared |

依赖方向：**L0 Shared ← L1 Core ← L2 Infrastructure**（高层依赖低层，禁止反向）

#### 横切四层（全局视野）

| 层级 | 名称 | 核心定位 | 依赖权限 |
|------|------|----------|----------|
| **横切1** | App | 游戏启动与全局装配（Composition Root） | 允许依赖所有层（唯一知道全部层的地方） |
| **横切2** | Content | 数据与规则的桥接层（只做加载/校验/注册） | 允许依赖 Core + Infrastructure |
| **横切3** | Tools | 开发期工具链 | 允许依赖 Core + Shared，永不进入发布构建 |
| **横切4** | Modding | Mod 扩展层（跨层聚合，暴露稳定 API） | 通过 Core/mod_api 访问核心功能 |

#### 依赖方向总规则

```
# DDD 纵向三层：L0 Shared ← L1 Core ← L2 Infrastructure（高层依赖低层）
Shared → Core → Infrastructure     # 依赖方向：高层依赖低层

# 横切层与纵向层的关系
App   → 知道所有层（唯一 Composition Root，仅装配不含业务逻辑）
Content → Core + Infrastructure（只做加载/校验/注册）
Tools → Core + Shared（开发期专用）
Modding → Core/mod_api（唯一对外暴露的核心接口）

# 禁止的依赖
🟥 Infrastructure → App（反向依赖）
🟥 Content → Shared（跨层依赖）
🟥 Core → Infrastructure（反向依赖）
🟥 Domain 间直接引用（写操作走事件，读操作走 Query API）
```

### 2.2 横切1：App — 启动装配层
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
├── mod.rs
├── app_plugin.rs               # 主 Plugin，汇总注册所有子 Plugin
├── game_app.rs                 # 游戏模式启动入口（注册游戏专用 Plugin/禁用编辑器 Plugin）
├── editor_app.rs               # 编辑器模式启动入口（注册编辑器 Plugin/启用 dev-tools）
├── server_app.rs               # 服务器模式（预留，无渲染纯逻辑）
├── headless_app.rs             # 无头模式启动入口（战斗模拟/自动化测试）
│
├── state/                      # 全局状态机
│   ├── mod.rs
│   ├── app_state.rs            # AppState 主状态（Loading→MainMenu→Game→Editor）
│   └── game_state.rs           # 游戏内子状态（Exploration→Combat→Dialogue→Camp→Menu）
│
├── bootstrap/                  # 启动流程
│   ├── mod.rs
│   ├── load_core.rs            # 核心资源加载（Tag 树/Attribute 定义/注册中心初始化）
│   └── init_content.rs         # 内容初始化（加载配置→校验→注册到 Core）
│
├── schedule/                   # 调度编排
│   ├── mod.rs
│   ├── schedules.rs            # 自定义 Schedule 定义（PreUpdate/CombatTick/PostCombat）
│   └── sets.rs                 # SystemSet 定义与顺序约束
│
├── shutdown.rs                 # 资源清理与关闭流程
└── plugins.rs                  # 所有子 Plugin 集中注册
```

#### 约束
- 🟩 唯一允许依赖所有层的层级
- 🟥 禁止包含任何业务规则逻辑
- 🟥 禁止直接创建业务 Entity（由各业务模块的 startup 系统负责）
- 🟥 禁止硬编码任何游戏数值

### 2.3 纵向 L1：Core — 游戏规则核心层
项目的心脏，唯一允许定义游戏规则的层级。**内部采用「Capabilities 能力层 + Domains 业务域」双轴结构**，兼顾通用能力复用与业务逻辑内聚。

#### 边界判断（三问法）
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

### 2.4 纵向 L0：Shared — 基础原子层
依赖图的底层，允许稳定基础库依赖，禁止业务依赖。

#### 允许的依赖
- ✅ `serde`：序列化框架（配置/存档的基础）
- ✅ `thiserror`：错误派生宏
- ✅ `smallvec`：栈上小向量（ECS 高频路径性能优化）
- ✅ `bitflags`：位掩码（Tag 系统核心依赖）
- ✅ `rand`：随机数基础接口（确定性 RNG 的底层）
- ✅ 其他经过验证的零业务语义基础库

#### 禁止的依赖
- 🟥 任何业务库（bevy_ecs、游戏逻辑 crate 等）
- 🟥 任何 Infrastructure 层的实现（日志框架、序列化格式等）

#### 边界判断
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
- 🟥 禁止依赖任何业务层（Core / Infrastructure / Content）
- 🟥 禁止放入日志实现、本地化、指标统计等技术实现
- 🟥 禁止定义全局统一 `AppError` 大枚举，错误由各领域自行定义
- 🟥 禁止引入带业务语义的第三方库

### 2.5 纵向 L2：Infrastructure — 技术实现层
游戏规则不变、但实现方式可替换的技术能力全部收敛于此。

#### 边界判断
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
   - `logging/`：日志系统（日志管理器/格式化器/写入器/配置，基于 tracing 结构化日志）
   - `save/`：存档读写、版本迁移
   - `localization/`：多语言框架
   - `map/`：地图管线（MapAsset 类型定义 + MapLoader + 自研 MapRenderer，ADR-065）
   - `physics/`：物理碰撞
   - `pathfinding/`：寻路算法实现
   - `navmesh/`：导航网格
   - `input/`：输入处理、按键映射
   - `analytics/`：数据埋点
   - `hot_reload/`：内容热重载
   - `encryption/`：加密能力
   - `anti_cheat/`：反作弊
   - `networking/`：网络通信（预留，单机暂不实现）
   - `cloud/`：云服务对接（预留，单机暂不实现）
   - `mod_loader/`：Mod 文件扫描、沙箱隔离、版本校验、依赖管理
3. **Presentation 表现层**
   - `rendering/`：渲染管线、材质、Shader
   - `camera/`：镜头控制、姿态插值、状态机、边界约束（ADR-064）
   - `ui/`：UI 系统（view/widget/screen/navigation/hud/binding）
   - `audio/`：音频播放、音效管理
   - `animation/`：动画系统
   - `particles/`：粒子特效
   - `platform/`：平台适配（Steam/Epic 等）

#### 约束
- 🟥 禁止包含任何游戏规则逻辑
- 🟥 禁止直接修改 Core 领域的内部状态，只能通过事件/接口交互
- 🟥 表现层模块不得被业务服务层依赖
- 🟥 Camera 禁止依赖 `core::domains::*` 的任何类型（ADR-064），Camera 是表现层基础设施，不应感知业务领域

### 2.6 横切2：Content — 内容桥接层
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
├── mod.rs
├── content_plugin.rs           # 内容层总 Plugin，注册所有加载器与校验器
├── mod_support/                # Mod 内容加载与覆盖
│   └── mod_content_loader.rs   # Mod 内容加载器（扫描 Mod 目录→合并/覆盖基础配置）
├── schema/                     # 配置数据结构定义（Def 类型）
├── attributes/                 # 属性配置加载
├── tags/                       # 标签配置加载
├── modifiers/                  # 修改器配置加载
├── effects/                    # 效果配置加载
├── abilities/                  # 技能配置加载
├── triggers/                   # 触发器配置加载
├── characters/                 # 角色模板配置
├── enemies/                    # 敌人模板配置
├── classes/                    # 职业配置
├── factions/                   # 势力配置
├── maps/                       # 地图配置
├── encounters/                 # 遭遇配置
├── items/                      # 物品配置
├── equipments/                 # 装备配置
├── shops/                      # 商店配置
├── loot_tables/                # 战利品配置
├── progression/                # 进展配置
├── quests/                     # 任务配置
├── stories/                    # 故事配置
├── dialogues/                  # 对话配置
├── localization/               # 多语言文本数据
├── balance/                    # 数值平衡配置
├── migration/                  # 配置版本迁移
└── validation/                 # 内容校验规则
```

#### 双类型协作模式
```
core/capabilities/ability/foundation/ability_def.rs    配置反序列化结构
core/capabilities/ability/foundation/ability_instance.rs  运行时结构
content/abilities/                                       RON → Def → Instance → 注册
```

#### 约束
- 🟥 禁止包含任何游戏规则逻辑，只做「加载 → 校验 → 注册」三件事
- 🟥 禁止硬编码数值内容
- 🟩 测试可用 Mock Registry 替代真实 Registry

### 2.7 横切3：Tools — 开发工具层
开发期专用工具链，永不进入发布构建。

#### 价值预判
大型 SRPG 项目的工具代码通常占总工程量的 20%~40%，是长期迭代的生产力核心。

#### 目录结构
```
src/tools/
├── mod.rs                      # [cfg(feature = "dev-tools")]
├── tools_plugin.rs
├── replay_viewer/              # 回放查看器
├── battle_simulator/           # 战斗模拟器
├── balance_analyzer/           # 数值平衡分析器
├── content_validator/          # 内容合法性校验器
├── localization_tool/          # 多语言工具
├── save_editor/                # 存档编辑器
├── schema_generator/           # 配置 Schema 生成器
├── graph_viewer/               # 依赖/效果管线可视化
├── profiling_tool/             # 性能剖析工具
├── ai_debugger/                # AI 决策调试器
├── map_editor/                 # 地图编辑器
├── ability_editor/             # 技能编辑器
├── effect_editor/              # 效果编辑器
├── quest_editor/               # 任务编辑器
├── pipeline_inspector/         # 执行管线检查器
├── log_analyzer/               # 日志分析器（过滤/聚合/对比/报告）
├── data_browser/               # 数据查询器（查询 ECS World 中 Entity/Component/Resource 状态）
├── replay_diff/                # 回放对比（检测非确定性）
├── migration_tool/             # 数据迁移（旧版本存档/配置→新版本自动转换）
├── dependency_checker/         # 架构检查（静态分析 use 语句，检测架构违规）
└── test_runner/                # 自动测试入口
```

#### 约束
- 🟥 永不进入 release 构建
- 🟥 禁止包含业务规则逻辑
- 🟩 开发期可直接操作 Registry 与 World，权限放宽

### 2.8 横切4：Modding — Mod 扩展层
跨层聚合，暴露稳定的 Modding API。Mod 只能通过 `core/mod_api/` 的 Gateway 访问核心功能，禁止直接操作 ECS World。

#### 目录结构
```
src/modding/
├── api/                        # Mod 开发 API
│   ├── mod_trait.rs              # Mod 生命周期 Trait
│   ├── hook_points.rs            # 扩展点定义
│   └── event_api.rs              # Mod 事件 API
├── registry/                   # Mod 注册中心
│   └── mod_registry.rs           # Mod 注册表
├── loader/                     # Mod 加载器
│   └── mod_loader.rs             # Mod 加载流程
├── sandbox/                    # Mod 沙箱
│   └── mod_sandbox.rs            # Mod 运行沙箱
├── compatibility/              # Mod 兼容性管理
│   ├── version_check.rs          # 版本兼容性检查
│   └── conflict_resolver.rs      # 冲突解决
├── documentation/              # Mod 开发文档生成
└── examples/                   # Mod 示例
```

#### 约束
- 🟥 Mod 禁止绕过稳定 API 直接访问 Core 内部实现
- 🟩 Mod 能力与原生内容走同一套 Registry 与执行管线
- 🟩 API 分级：稳定 API / 实验性 API / 内部 API（Mod 禁止调用）

### 2.9 依赖方向总规则
```
# DDD 纵向三层：L0 Shared ← L1 Core ← L2 Infrastructure
Shared → Core → Infrastructure

# 横切层与纵向层的关系
App   → 知道所有层（仅装配，不含业务逻辑）
Content → Core + Infrastructure（只做加载/校验/注册）
Tools → Core + Shared（开发期专用）
Modding → Core/mod_api（唯一对外暴露的核心接口）
```
🟥 严格禁止反向依赖：Core 不得依赖 Infrastructure/Content/UI，Shared 不得依赖任何业务层。
- 🟩 **类型不可见性**：下层对上层的类型完全不可见——Shared 不知道 Core 的任何类型，Core 不知道 Infrastructure 的任何类型。这不仅是依赖方向约束，更是**编译期类型隔离**：下层的 `Cargo.toml` 中不得出现上层的 crate 依赖。

---

## 第三编 Core 层双轴架构（Capabilities 能力层 + Domains 业务域）
> 核心升级：摒弃单纵向5级分层（L0~L4）的思路，采用「**纵向 Capabilities 通用机制复用 + 横向 Domains 业务内聚**」双轴结构。纵向 Capabilities 提供通用机制，横向 Domains 封装业务规则，既保证能力复用，又实现业务内聚，从根源解决大项目下层内膨胀、调用路径网状化的问题。

### 3.1 双轴架构总览
```
src/core/
├── core_plugin.rs              # Core 层总 Plugin
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

### 3.2 Capabilities 内部三层架构（C1/C2/C3）

> 设计思路：将原5层架构（atom/rule/runtime/model/frame）精简为3层，
> 核心理由是"内聚优于分层"——同一领域的代码应放在一起，而非按抽象层级拆散。

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

### 3.3 15 核心能力领域定义

| # | 领域 | 职责 | 分组 |
|---|------|------|------|
| 1 | Tag | 标签定义、位掩码、层级关系 | 核心基石 |
| 2 | Attribute | 属性定义、基础值、当前值 | 核心基石 |
| 3 | Modifier | 修改器定义（Add/Mul/Override） | 核心基石 |
| 4 | Aggregator | 属性聚合管线 | 核心基石 |
| 5 | GameplayContext | 统一上下文/载荷 | 核心基石 |
| 6 | Spec | 模板→配置→实例三层分离 | 逻辑骨架 |
| 7 | Ability | 技能逻辑与生命周期 | 逻辑骨架 |
| 8 | Trigger | 技能激活条件 | 逻辑骨架 |
| 9 | Condition | 统一条件检查（含免疫） | 逻辑骨架 |
| 10 | Targeting | 目标选择 | 行为表现 |
| 11 | Execution | 执行计算 | 行为表现 |
| 12 | Effect | 效果系统 | 行为表现 |
| 13 | Stacking | 堆叠规则 | 行为表现 |
| 14 | Event | 系统间结构化通信 | 行为表现 |
| 15 | Cue | 表现层信号 | 行为表现 |

### 3.4 横向：Domains 业务域（内聚核心）
每个 Domain 是一个**垂直自包含的业务模块**，封装完整的玩法规则，对外只暴露稳定接口。所有复杂度爆炸的业务逻辑全部下沉到对应 Domain，禁止散落在各层中。

#### 标准结构（所有 Domain 必须统一遵循）
```
domain_name/
├── mod.rs              # 模块声明 + pub use（可选，视复杂度决定）
├── plugin.rs           # 唯一对外入口，注册组件、系统、事件
├── components/         # ECS 组件（按子模块拆分，或 components.rs 单文件）
│   ├── mod.rs
│   └── ...
├── systems/            # 本系统业务系统（按子模块拆分）
│   ├── mod.rs
│   └── ...
├── events/             # 领域事件定义（按子模块拆分，或 events.rs 单文件）
│   ├── mod.rs
│   └── ...
├── error/              # 领域错误枚举（按子模块拆分，或 error.rs 单文件）
│   ├── mod.rs
│   └── ...
├── resources/          # 全局 Resource（按子模块拆分，或 resources.rs 单文件，如有）
│   ├── mod.rs
│   └── ...
├── rules/              # 纯业务规则（优先纯函数，无 ECS 依赖）
│   ├── mod.rs
│   ├── formulas.rs
│   └── rules.rs
├── integration/        # 集成层：唯一调用 Capabilities 能力的入口（Facade + SystemParam）
│   ├── mod.rs
│   └── <capability>/
│       ├── facade.rs
│       ├── types.rs
│       └── system_param.rs
└── tests/              # 四层测试（unit/integration/invariant/fixtures）
    ├── mod.rs
    ├── unit/
    ├── integration/
    ├── invariant/
    └── fixtures/
```

#### 文件 vs 目录决策指南

| 模块 | 初始形态 | 升级为目录的阈值 | 拆分策略 |
|------|----------|-----------------|----------|
| **mod.rs** | 文件 | 不升级 | 入口点，保持单一文件 |
| **plugin.rs** | 文件 | 不升级 | 唯一对外入口，保持单一文件 |
| **components/** | `components.rs` | 组件数 ≥ 5 个 | 按子领域拆分（如 `health.rs`, `unit.rs`, `equipment.rs`） |
| **events/** | `events.rs` | 事件数 ≥ 5 个 | 按事件类型拆分（如 `combat_events.rs`, `movement_events.rs`） |
| **error/** | `error.rs` | 错误变体数 ≥ 10 个 | 按错误来源拆分（如 `combat_error.rs`, `inventory_error.rs`） |
| **resources/** | `resources.rs` | 资源数 ≥ 3 个 | 按资源职责拆分（如 `combat_state.rs`, `turn_order.rs`） |
| **rules/** | 目录 | 初始即为目录 | 按规则类型拆分（`formulas.rs`, `rules.rs`, `validators.rs`） |
| **systems/** | 目录 | 初始即为目录 | 按系统职责拆分（`movement_system.rs`, `combat_system.rs`） |
| **integration/** | 目录 | 初始即为目录 | 按 Capability 拆分（`movement/`, `targeting/`, `effect/`） |
| **tests/** | 目录 | 初始即为目录 | 四层测试结构（unit/integration/invariant/fixtures） |

#### 拆分原则
- 🟩 **单一职责**：每个子模块只负责一个子领域（如 `health.rs` 只管生命值相关组件）
- 🟩 **渐进式拆分**：初期保持简单（单文件），复杂度增长时再拆分
- 🟩 **命名一致性**：子模块文件名与内容对应（如 `combat_events.rs` 包含战斗相关事件）
- 🟩 **mod.rs 聚合**：目录下的 `mod.rs` 负责 re-export 所有子模块的公开类型

#### 核心业务域清单（15个）
| 业务域 | 职责 | 核心封装内容 |
|--------|------|--------------|
| combat | 战斗玩法全域 | 伤害公式、回合规则、胜负判定、战斗流程编排 |
| spell | 法术系统 | 法术位、专注、施法组件、法术豁免、升环 |
| reaction | 反应/援护 | 机会攻击、法术反制、护盾反应、援护格挡 |
| progression | 成长养成 | 经验等级、职业成长、子职、天赋、属性提升 |
| inventory | 背包物品 | 背包管理、物品使用、装备穿戴、掉落规则 |
| quest | 任务系统 | 任务流程、条件判定、奖励发放、任务状态 |
| narrative | 叙事/对话 | 剧情对话、分支选项、条件对话、故事标记 |
| tactical | 战术/网格 | 网格移动、高地优势、掩体、背刺、夹击 |
| terrain | 地形效果 | 毒池、冰面、灼烧地面、高地加成、掩体 |
| faction | 阵营关系 | 声望、友好/敌对关系、势力影响 |
| party | 队伍管理 | 战前编队、换人、小队羁绊、队伍buff |
| camp_rest | 营地休息 | 长休短休、资源恢复、营地交互、法术刷新 |
| crafting | 制作系统 | 装备锻造、附魔、武器改造、分解、材料合成 |
| economy | 经济系统 | 货币、商店买卖、物价波动、交易折扣 |
| summon | 召唤系统 | 召唤物生命周期、AI、占位规则 |

#### 设计原则
- **垂直自治**：每个 Domain 内部包含完整的组件、系统、逻辑、事件
- **薄集成层**：`integration/` 是唯一调用 Capabilities 能力的地方，采用 Facade + SystemParam 模式
- **业务内聚**：同一玩法的所有逻辑全部在一个 Domain 内

### 3.5 边界铁则（不可突破）
#### 3.5.1 Capabilities 与 Domains 的边界
- 🟩 **Capabilities 只提供机制，不实现业务规则**：所有通用能力必须做到「玩法无关」
- 🟩 **Domains 只编排规则，不重复造机制**：所有原子能力必须复用 Capabilities
- 🟩 **唯一合法交互**：Domain 通过 Capabilities 的公开 API 调用通用能力，注入业务规则，完成玩法编排
- 🟥 **绝对禁止**：Capabilities 中出现任何具体业务玩法的硬编码逻辑
- 🟥 **绝对禁止**：Domains 绕过 Capabilities 直接实现底层通用机制

#### 3.5.2 Domain 之间的边界
- 🟥 **绝对禁止直接依赖**：Domain 之间禁止直接 `use` 对方的内部组件、系统、数据结构
- 🟩 **双轨通信**：写操作走事件（Message），读操作走 Query API（如 `is_quest_completed()`）
- 🟩 **能力复用路径**：如果多个 Domain 需要复用同一能力，必须下沉到 Capabilities 层
- ⚠️ **警觉阈值**：单个 Domain 内部文件超过 80 个时，必须评估是否拆分

#### 3.5.3 依赖方向约束
- Domains → Capabilities：✅ 允许，且是唯一合法的调用方向
- Capabilities → Domains：❌ 绝对禁止，能力层不能感知具体业务域
- Domain A → Domain B：❌ 写操作禁止直接调用，只能通过事件；读操作可通过 Query API 查询公开状态
- Domain → Shared：✅ 允许，可使用基础工具与强类型ID

### 3.6 交互规范
#### 3.6.1 Domain 调用 Capabilities 的规范
- 必须通过 Capabilities 各模块的公开 API 调用，禁止直接访问内部实现
- 业务规则通过「配置注入 + 回调注册」的方式接入通用框架
- 每个 Domain 必须有且仅有一个集成层（`integration/` 模块）作为与 Capabilities 的唯一交互入口
- 🟥 集成层必须采用 **Facade + SystemParam** 模式：定义 View Types（Domain 自己的类型）+ SystemParam（封装 Capabilities 查询），Systems 不得直接 import Capabilities 组件类型进行字段访问
- 🟥 集成层按能力域拆分子模块（如 `integration/movement/`, `integration/terrain/`），禁止单文件膨胀为 God File

#### 3.6.2 Domain 间通信规范（双轨制）
- **写操作 → Event**：状态变更必须通过全局领域事件（Message）广播，禁止直接调用对方方法
- **读操作 → Query API**：查询对方公开状态可通过 Query API（如 `is_quest_completed()`），避免事件风暴
- 所有跨 Domain 的事件必须纳入领域事件白名单统一管理
- 禁止为了 Domain 间调用创建临时事件，禁止事件滥用
- 🟥 禁止通过事件传递查询请求（Request-Response 反模式），查询必须走 Query API

### 3.7 Mod API：Facade + Gateway 模式
将 Mod API 重构为统一的门面 + 网关模式，对外只暴露业务语义接口，隐藏内部分层结构。

#### 目录结构
```
src/core/mod_api/
├── mod.rs
├── core_facade.rs            # 核心门面，统一入口
├── combat_gateway.rs         # 战斗 API（查询战斗状态/触发反应/强制结束战斗）
├── character_gateway.rs     # 角色 API（查询属性/标签/修改 Spec 等级）
├── spell_gateway.rs         # 法术 API（查询法术位/施放法术/打断专注）
├── quest_gateway.rs         # 任务 API（接受/完成/查询任务状态）
├── party_gateway.rs         # 队伍 API（换人/查询羁绊/修改队伍配置）
├── camp_gateway.rs          # 营地 API（触发长休/短休/查询营地事件）
├── summon_gateway.rs        # 召唤 API（创建/移除召唤物/查询召唤状态）
├── terrain_gateway.rs       # 地形 API（修改地形/查询地形效果/触发地形交互）
├── craft_gateway.rs         # 制作 API（注册配方/执行制作/查询材料）
├── economy_gateway.rs       # 经济 API（修改物价/查询钱包/执行交易）
├── inventory_gateway.rs     # 物品 API（添加/移除物品/装备穿戴）
├── faction_gateway.rs       # 阵营 API（修改声望/查询关系/注册阵营）
├── progression_gateway.rs   # 成长 API（给予经验/升级/解锁天赋）
└── narrative_gateway.rs     # 叙事 API（设置故事标记/触发对话/查询剧情状态）
```

#### 设计原则
- 🟩 **面向业务暴露**：Mod 作者只需理解业务概念，不需要了解 Capabilities/Domains 内部分层
- 🟩 **稳定隔离**：内部重构时，只要保持网关接口不变，Mod 完全不受影响
- 🟩 **分级授权**：API 分为稳定级、实验级、内部级，Mod 只能调用稳定级与公开实验级
- 🟥 **绝对禁止**：Mod 绕过网关直接访问 Capabilities 或 Domains 的内部实现

---

## 第七编 模块化与 Plugin 边界宪法
- 🟩 **接口最小化原则**
  - 每个 Feature 只暴露必要的公共接口，所有内部实现必须设为私有
  - 🟥 绝对禁止外部模块直接访问其他 Feature 的 internal 子模块
- 🟩 **Plugin 是唯一对外入口**
  - 每个 Feature 必须通过 Plugin 对外暴露能力
  - 推荐每个 Feature 根目录包含 `plugin.rs`（入口）与 `integration/`（跨域访问 ACL，ADR-046）
  - 🟥 禁止新增 `api.rs`，跨域访问统一通过 `integration/`
- 🟩 **边界优先于目录**：模块边界的清晰度比目录结构更重要
- 🟩 **跨模块交互规范**
  - 跨模块写操作必须通过 Message、Command、Trigger 三种方式
  - 跨模块读操作允许通过公开的 Query、Resource、State 读取
  - 🟥 绝对禁止跨模块直接修改其他 Feature 的内部组件与状态
- 🟩 **Plugin 拆分原则**：Plugin 职责过大时按业务领域拆分，而非按代码数量拆分
- 🟩 **路径命名规范**：文件路径必须清晰表达业务含义
  - 合法：`battle/damage/`、`inventory/equipment/`
  - 非法：`base/`、`core/`
- 🟨 **通用代码规范**
  - 优先不创建通用顶层目录
  - 如确需创建 `common/`，只能存放与业务无关的纯工具代码
  - 🟥 绝对禁止在 `common/` 中放入任何业务逻辑
