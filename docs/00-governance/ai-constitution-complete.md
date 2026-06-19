# Bevy 0.18+ SRPG 项目总宪法 v5.1（完整版）
> 文档元数据
> - id: 00-governance.project-constitution-complete
> - title: SRPG 项目总宪法（架构 + 开发 + AI 执行）
> - version: 5.1
> - status: Proposed
> - owner: architect
> - created: 2026-06-14
> - updated: 2026-06-19（v5.1 + §1.5 P0铁则第7条 + §21 红线第18条 + §22 Localization 专项规则）
> - tags: governance, constitution, architecture, bevy, srpg
> - 效力说明：本宪法对项目所有架构设计、代码编写、AI生成内容具有最高约束力，优先级高于任何通用编程规范、语言习惯或AI默认输出。条款编号永久固定，违反条款即视为不合格输出。

本版本在 v4.1 基础上完成核心架构对齐：将 Core 层内部结构从「5级纵向分层（L0~L4）」升级为「**Capabilities 能力层 + Domains 业务域 + Mod API**」双轴结构，与项目架构设计文档（Fre项目架构设计）完全对齐。核心变化包括：15个能力领域（含新增 GameplayContext）、内聚优于分层的3层内部架构（Foundation/Mechanism/Runtime）、15个业务域（含新增 Summon）、Facade+Gateway 模式 Mod API。

---

## 第一编 总则
### 1.1 适用范围
- 引擎：Bevy 0.18 及以上版本
- 品类：单机战棋 SRPG
- 规模：50万行+代码量级
- 模式：长期连载式内容迭代
- 核心特性：Replay First、Data Driven、GAS-Lite
- 扩展目标：自动化测试、战斗模拟器、服务器模拟、Mod 生态

### 1.2 架构优先级
所有架构与代码决策严格遵循以下优先级，禁止倒置：
```
正确性 > 可维护性 > 可扩展性 > 开发效率 > 性能
```
性能优化必须基于 Profiling 实证数据，禁止凭体感提前优化。

### 1.3 强制等级说明
| 标记 | 等级 | 说明 |
|------|------|------|
| 🟥 | 绝对禁止 | 任何情况下都不允许出现，不可豁免 |
| 🟩 | 必须遵守 | 无例外强制执行，除非获得明确豁免 |
| 🟨 | 优先选择 | 除非有明确且可验证的技术理由，否则必须采用 |
| 🟦 | 最佳实践 | 推荐但非强制，无技术理由时优先采用 |
| ⚠️ | 警觉阈值 | 达到阈值时必须主动提出重构建议 |

### 1.4 豁免规则
- 所有违反宪法的代码必须标注 `[宪法豁免]` 并说明理由、有效期、审批人
- 豁免代码每3个月必须重新评估
- 性能类豁免必须附带 Profiling 实证数据与 ADR 架构决策记录

### 1.5 P0 级顶层铁则
以下原则具有最高优先级，所有层级与模块必须无条件遵守。
1. **Feature First**：永远按业务领域组织代码，禁止按技术类型拆分全局目录
2. **Data Driven First**：新增内容优先通过配置数据实现，禁止硬编码业务内容
3. **Replay First**：所有核心战斗逻辑必须可确定性重放，禁止不可控随机源
4. **Logic / Presentation Separation**：业务逻辑与视觉表现彻底解耦，禁止混写
5. **Composition Over Inheritance**：所有差异化通过原子能力组合实现，禁止继承式设计
6. **Capabilities/Domains 双轴架构原则**：Core 层采用「纵向 Capabilities 通用机制复用 + 横向 Domains 业务内聚」双轴结构，禁止单维度无限分层
7. **Localization First**：所有用户可见文本禁止直接进入 Rust 代码，必须通过 LocalizationKey 引用；Def 只存 name_key/desc_key 等 Key，不存任何自然语言文本；Replay/Event/BattleLog 只存 Key + 参数，不存最终翻译文本；存档禁止保存翻译结果，只存 ID/Key

---

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
   - `ui/`：UI 系统（view/widget/screen/navigation/hud/binding）
   - `audio/`：音频播放、音效管理
   - `animation/`：动画系统
   - `particles/`：粒子特效
   - `platform/`：平台适配（Steam/Epic 等）

#### 约束
- 🟥 禁止包含任何游戏规则逻辑
- 🟥 禁止直接修改 Core 领域的内部状态，只能通过事件/接口交互
- 🟥 表现层模块不得被业务服务层依赖

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

## 第四编 Modding 能力体系
Modding 不是独立层级，而是贯穿多层的扩展能力，按职责拆分到对应层级，保证边界可控。

| 模块 | 归属 | 职责 |
|------|------|------|
| `core/mod_api/` | Core 层 | 对外暴露的稳定 Mod 接口，Facade + Gateway 模式，唯一合法的核心规则访问入口 |
| `content/mod_support/` | Content 层 | Mod 内容加载、数据覆盖、冲突处理、注册逻辑 |
| `infrastructure/mod_loader/` | Infrastructure 层 | Mod 文件扫描、沙箱隔离、版本校验、依赖管理 |

### 约束
- 🟥 Mod 禁止绕过稳定 API 直接访问 Core 内部实现
- 🟩 Mod 能力与原生内容走同一套 Registry 与执行管线
- 🟩 API 分级：稳定 API / 实验性 API / 内部 API（Mod 禁止调用）

---

## 第五编 横切关注点治理
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

## 第六编 ECS 宪法
### 6.1 核心概念
- 🟩 **Entity 只是 ID**
  - 🟥 绝对禁止：在 Entity 上调用任何方法或将其当作面向对象实例使用
  - 允许将 Entity 作为纯 ID 参数传递
- 🟩 **数据与行为强制分离**
  - Component 只能存储纯数据状态，🟥 绝对禁止包含任何逻辑
  - System 只能包含纯逻辑，🟥 绝对禁止存储任何状态
- 🟩 **Tag Component 优先原则**
  - 实体持久状态必须优先使用空 Tag Component 实现
    - 合法：`Dead`、`Frozen`、`Stunned`
    - 非法：`is_dead: bool`、`is_frozen: bool`
  - 配置字段、临时计算值、非实体级状态允许使用 bool 类型
- 🟨 **从属关系使用官方 Relationship**
  - 实体间从属关系优先使用 Bevy 官方 `Relationship` 机制实现
  - 禁止手动维护父子实体 ID 字段

### 6.2 ECS 使用边界规范
#### Core 层允许使用的 ECS 能力
- `#[derive(Component)]` 定义业务数据组件
- `Query`、`Res`、`EventWriter/Reader`、`Observer`、`Trigger`
- `Schedule`、`SystemSet` 调度编排
- `Resource` 全局业务状态

#### Core 层禁止使用的 ECS/引擎能力
- `AssetServer`、`Handle<Image>` 等资源加载类型
- `RenderDevice`、`Texture` 等渲染类型
- `Input`、`KeyCode` 等输入类型
- `AudioPlayer`、`SpatialAudio` 等音频类型
- 所有 UI 组件与系统

### 6.3 四级通信机制（Hook / Trigger / Observer / Message）
优先级从高到低，优先使用最小粒度的通信方式，禁止事件滥用。
1. **Hook = 组件生命周期固有行为**
   - 组件添加/移除时的轻量固有副作用，通过 `#[component(on_add=..., on_remove=...)]` 声明
   - 仅用于简单状态联动，禁止承载复杂业务逻辑
2. **Trigger = Feature 内事件链载体**
   - 同一 Feature 内的多段响应逻辑、战斗事件链，使用 `commands.trigger()` 机制实现
   - 典型场景：伤害触发护盾、吸血、反击等连锁效果
3. **Observer = 局部状态变化响应**
   - 同一 Feature 内的组件变化、Trigger 触发的响应逻辑，使用 Observer 实现
   - 典型场景：角色死亡播放动画、血量变化刷新 UI
4. **Message = 跨 Feature 全局广播**
   - 跨业务模块、跨 Domain 的通知必须使用全局事件（Event）
   - 典型场景：回合结束、战斗胜利、任务完成

#### 补充规则
- 🟩 模块内部优先函数调用，🟥 绝对禁止将同一模块内的普通逻辑全部事件化
- 🟩 领域事件是唯一业务事实源，日志、战斗回放、UI 履历、成就、任务均为事件的下游消费者
- 🟩 领域事件白名单管理，新增事件必须先更新白名单，🟥 绝对禁止为临时副作用随意新增领域事件

### 6.4 ECS 执行模型
- 🟩 **ECS 是数据流，不是调用链**
  - 🟥 绝对禁止模拟面向对象的调用方式，如 `player.attack(enemy)`
  - 正确方式：创建命令组件，由对应系统统一处理
- 🟩 **组件依赖声明**：组件依赖必须通过 `#[require(Component)]` 属性声明，🟥 绝对禁止手动检查并补全缺失的组件
- 🟩 **状态变更检测**
  - 组件生命周期变化优先使用 Bevy 原生 `Added`、`Changed`、`Removed` 过滤器
  - 🟥 绝对禁止手写 bool 脏标记字段检测组件变化
  - 业务状态机允许使用显式状态组件，自行管理状态流转
- 🟩 **Resource 使用规范**
  - Resource 只能存储真正的全局唯一状态
  - 🟥 绝对禁止将 Resource 当作全局变量仓库使用
- 🟩 **性能约束**
  - 🟥 绝对禁止在高频逻辑中使用 Observer 造成事件风暴
  - 每帧执行 10 次以上的逻辑必须直接使用 System 处理
- 🟩 **状态管理规范**
  - 全局流程状态优先使用 `States`、`SubStates`、`ComputedStates` 实现
  - 禁止手动维护全局状态枚举与切换逻辑
- 🟨 **运行条件优先**
  - 系统的执行前置判断优先使用 `run_if()` 条件表达
  - 避免在系统内部写大段 if 判断是否执行
- 🟩 **Schedule 权责划分**
  - `PreUpdate`：输入处理、命令执行、状态同步
  - `Update`：核心业务逻辑、规则结算
  - `PostUpdate`：事件响应、表现层更新、UI 刷新
  - 禁止跨 Schedule 乱放系统，禁止在错误阶段执行业务逻辑

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

---

## 第八编 SRPG 核心系统专项宪法
### 8.1 角色系统宪法
- 🟩 **分层扩展体系**
  - 角色数值能力：统一通过 Modifier 管线实现
  - 角色行为扩展：优先使用 Trait 定义能力接口，由系统统一调度
  - 特殊机制：召唤、传送、复活等非数值效果，通过独立系统+组件实现
- 🟩 **核心定义**
  - 种族 = Trait 集合 + Modifier 集合
  - 职业 = 成长率表 + 技能池 + Trait 集合
  - 天赋 = 特殊 Trait + 专属 Modifier
  - 装备 = Modifier 集合 + Trait 集合
  - Buff/Debuff = 临时 Modifier + 临时 Trait + 生命周期组件
- 🟩 **状态效果统一**：所有数值类状态效果必须走统一的 Modifier 管线

### 8.2 属性系统宪法
- 🟩 **属性分类强制分离**：基础属性(Primary Stat)与派生属性(Derived Stat)必须完全分离
- 🟩 **派生属性计算策略**
  - 默认采用实时计算，保证数据一致性
  - 经 Profile 证明确实为性能热点后，允许引入缓存组件
- 🟩 **修改规范**
  - 🟥 绝对禁止直接修改基础/派生属性的最终值
  - 基础与派生属性的数值变化必须通过添加/移除 Modifier 实现
  - HP、MP 等资源型数值的增减，可通过专用系统直接修改，不强制走 Modifier
- 🟩 **来源统一**：最终属性值必须只有一个统一的计算来源
- 🟩 **公式集中管理**：所有属性计算公式必须集中在领域层模块中
- 🟨 **属性缓存约定**
  - 缓存组件不是事实源，必须允许随时删除并重新生成
  - 属性源变化时必须立即失效对应缓存

### 8.3 地图系统宪法
- 🟩 地图必须优先被视为 Grid 数据结构
- 🟩 Tile 是否作为 Entity 由具体需求决定，不强制统一
- 🟩 Chunk 按需引入，只有当地图大小超过单块性能阈值时才引入
- 🟩 OccupancyGrid 必须作为独立的数据结构存在
- 🟩 地图逻辑数据必须完全独立于渲染层
- 🟩 寻路数据必须在运行时动态生成
- 🟩 地图逻辑层必须完全独立于渲染层

### 8.4 回合系统规范
- 🟩 **阶段划分标准化**
  - 回合必须划分为明确阶段：回合开始 → 阶段判定 → 单位行动 → 回合结算 → 回合结束
  - 每个阶段对应独立的系统集与触发点
- 🟩 **状态驱动回合流转**
  - 回合切换通过状态机驱动，禁止手动调用回合切换函数

### 8.5 技能执行管线
- 🟩 **五阶段固定管线**
  - 所有技能执行必须遵循：目标校验 → 消耗扣除 → 施法前摇 → 效果执行 → 结果结算
  - 禁止跳过阶段或乱序执行
- 🟩 **预览与执行分离**
  - 技能伤害、范围预览必须走独立的只读路径，绝对禁止产生任何副作用

### 8.6 Buff 生命周期规范
- 🟩 **四阶段标准化**
  - 所有 Buff 必须遵循：Apply（施加）→ Tick（周期触发）→ Expire（到期）→ Remove（移除）四个生命周期阶段
  - 每个阶段有明确的触发时机与 Hook 点

### 8.7 命令层（Command Layer）
- 🟩 **所有操作入口为命令**
  - 玩家输入、AI 决策、回放执行、网络同步，最终都必须转换为标准化业务命令
  - 典型命令：`MoveUnitCommand`、`CastSkillCommand`、`EndTurnCommand`
- 🟩 **命令无差别执行**
  - 执行系统不区分命令来源，只处理命令本身

### 8.8 输入抽象层
- 🟩 输入为独立 Feature，鼠标、键盘、手柄、触摸等输入统一转换为业务命令
- 🟥 绝对禁止业务模块直接读取原始输入按键

### 8.9 读写分离原则（CQRS Lite）
- 🟩 **读路径无副作用**
  - 伤害预览、范围查询、状态展示等读操作，必须使用纯函数或只读查询
  - 绝对禁止在读路径中修改状态、触发事件、消耗资源
- 🟩 **写路径收口**
  - 所有状态修改必须通过命令与执行系统统一处理
  - 禁止零散分布的状态写入逻辑

### 8.10 仿真优先原则
- 🟩 核心规则支持离线仿真，战斗核心规则可以脱离 Bevy 运行时独立执行
- AI 决策、伤害预览、自动测试，均可直接调用同一套规则

### 8.11 随机数分层管理
- 🟩 **多 RNG 流独立**
  - 必须按用途拆分独立随机数流：战斗 RNG、掉落 RNG、世界 RNG、AI RNG
  - 禁止全局共用单一 RNG 实例
- 🟩 **统一 RNG 服务**
  - 业务逻辑禁止直接调用 `rand::random()`，必须通过统一的随机数服务获取

### 8.12 系统职责分类
- 🟨 五类系统划分：Collector（收集输入）、Validator（校验规则）、Executor（执行逻辑）、Projector（投射到表现层）、Observer（监听响应）
- 统一系统定位，生成代码时职责清晰

---

## 第九编 UI 系统宪法
- 🟩 **技术选型**：产品 UI 优先使用 bevy_ui，开发工具 UI 优先使用 bevy_egui
- 🟩 **状态单向流动**
  - UI 只能展示业务状态，🟥 绝对禁止保存业务真相
  - 🟥 绝对禁止业务逻辑直接操作 UI 组件
  - 🟥 绝对禁止在 UI 系统中修改业务状态
  - UI 必须通过监听状态变化自动刷新
- 🟩 **模块化**：UI 必须作为独立的 Feature 模块存在
- 🟩 **临时状态隔离**
  - 选中单位、悬停格子、技能预览等 UI 交互状态，属于表现层临时状态
  - 绝对禁止混入业务事实状态，不参与存档、不进入回放

---

## 第十编 数据驱动与存档宪法
### 10.1 数据驱动核心原则
- 🟩 职责划分：配置定义内容，代码解释配置
- 🟩 配置稳定性优先于配置格式的优雅性
- 🟩 配置的向后兼容性优先于任何其他考虑
- 🟩 所有配置资源必须通过统一的 Asset Pipeline 管理
- 🟩 所有配置必须优先支持热重载
- 🟩 配置之间的引用关系必须实现自动校验
- 🟩 内容生产效率优先于配置格式的争论

### 10.2 配置 Schema 与校验
- 🟩 所有配置必须有 Schema，所有配置结构体对应明确的 Schema 定义
- 🟩 CI 配置 Lint：所有配置必须支持 CI 自动校验，校验不通过禁止合并入主分支

### 10.3 数据所有权原则
- 🟩 每份配置数据必须有唯一的归属 Feature 与维护入口
- 禁止多个 Feature 同时维护同一份核心数据

### 10.4 数值平衡隔离
- 🟩 所有数值平衡参数必须放入配置文件
- 🟥 绝对禁止在代码中硬编码魔法数字

### 10.5 字段删除与兼容性
- 🟩 核心配置字段、存档字段禁止直接删除
- 必须遵循：标记 Deprecated → 新增迁移逻辑 → 下个大版本移除

### 10.6 存档版本与迁移
- 🟩 所有存档文件必须包含显式的 version 字段
- 🟩 新版本必须支持读取上一个大版本的存档，通过自动迁移完成升级
- 🟥 禁止版本升级后旧存档直接无法读取

---

## 第十一编 可观测性宪法
### 11.1 日志框架与核心定位
- 🟩 统一使用 `tracing` 库进行日志记录，🟥 绝对禁止使用 `println!`、`dbg!` 输出运行时信息
- 🟩 日志核心定位：领域事件履历
  - 日志必须记录业务事件事实，🟥 绝对禁止记录技术执行流水
  - 一个完整业务动作最多输出一条 INFO 级别日志；DEBUG / TRACE 级可按需细化

### 11.2 日志分级规范
- 🟥 **ERROR**：理论上不应发生的程序异常，必须附带完整复现上下文
- 🟩 **WARN**：可继续运行的异常情况
- 🟩 **INFO**：核心业务事件边界
- 🟦 **DEBUG**：开发调试辅助信息，发布版默认关闭
- 🟦 **TRACE**：极细粒度算法细节，仅专项调试时临时开启

### 11.3 结构化日志强制要求
- 🟩 所有 INFO 级别日志必须携带 `event` 字段，值与事件名完全一致
- 🟩 日志 `target` 必须与所属 Feature 目录名完全一致，支持精准过滤

### 11.4 日志禁令
- 🟥 绝对禁止记录函数进入、退出、系统执行等技术流水账
- 🟥 Release 版本绝对禁止在每帧执行的系统中输出 INFO / DEBUG 级别日志
- 🟥 绝对禁止在循环、迭代器内部输出 INFO 级别日志
- 🟥 绝对禁止业务代码直接调用 `info!` 输出核心业务事件（必须走领域事件链路）

### 11.5 日志架构规范（领域事件驱动）
- 核心模式：**领域事件触发 → 统一 Log Observer 监听 → 输出 tracing 日志**
- 所有 INFO 级别的核心业务事件，必须通过触发领域事件的方式生成日志
- 日志 Observer 统一放在基础设施层，绝对不侵入业务模块
- Battle Replay、战斗履历 UI、成就系统、任务系统与日志共用同一套领域事件源

### 11.6 错误体系规范
#### 11.6.1 分领域错误原则
- 🟩 每个领域定义独立错误枚举，🟥 绝对禁止使用全局统一的 `AppError` 大枚举、`anyhow::Error`、`Box<dyn Error>` 作为业务层返回错误类型
- 🟨 基础设施层可定义通用错误转换 Trait，不包含任何业务错误变体

#### 11.6.2 失败分类学（强制区分）
1. **规则失败**：业务规则的正常不满足，不属于程序错误，用专门结果枚举表达，禁止用 `Result::Err` 返回
2. **领域错误**：领域内预期内的异常，用对应领域错误枚举的 `Result::Err` 返回
3. **基础设施错误**：底层通用能力异常
4. **程序 Bug**：非法状态、逻辑断言失败，属于代码缺陷

#### 11.6.3 错误强制要求
- 🟩 所有错误必须携带完整上下文信息，🟥 绝对禁止仅返回无上下文的错误变体
- 🟩 推荐使用带编号的错误码，便于快速定位问题

#### 11.6.4 业务层 Panic 禁令
- 🟥 绝对禁止在核心业务领域代码中使用 `unwrap()`、`expect()`、`panic!()`
- 仅允许测试代码、工具代码、编辑器代码、原型验证代码使用

### 11.7 调试工具与可观测性
- 🟩 优先使用 Inspector、Replay、Debug Panel 进行问题排查，🟥 绝对禁止通过堆砌临时日志定位问题
- 🟨 核心战斗系统优先支持单步执行与状态回溯

### 11.8 领域事件与审计轨迹
- 🟩 所有核心业务事实必须通过领域事件表达，所有下游能力共用同一事件源
- 🟩 所有正式领域事件必须收录在白名单文档中
- 🟩 核心战斗流程必须生成结构化的审计轨迹，支撑回放、Bug 复现、自动化测试、数值平衡分析

---

## 第十二编 测试与确定性宪法
### 12.1 测试核心原则
- 🟩 所有功能必须优先编写测试，其次才是手工验证
- 🟩 发现 Bug 后必须先编写重现测试，再修复 Bug

### 12.2 测试架构体系

#### 核心原则

> **测试跟领域走（Feature First），但不写在源码文件内部。**

- 🟥 禁止 `#[cfg(test)] mod tests` 内联测试（对 AI 上下文污染严重，AI 会误改测试、引用测试代码、浪费 token）
- 🟥 禁止将所有测试平铺到根 `tests/unit/`（200+ 文件变成垃圾场）
- 🟩 测试与被测领域同目录放置，形成 Feature Folder 结构
- 🟩 根 `tests/` 仅保留跨领域测试（战斗流程、存档、回归、E2E）

#### 领域内聚测试结构（四层）

```
<domain>/
├── components/
├── systems/
├── events/
├── services/
├── tests/
│   ├── unit/          # 单元测试：验证领域纯函数、核心规则
│   ├── integration/   # 集成测试：验证领域内多组件协作
│   ├── invariant/     # 不变量测试：验证领域不变量（**最核心**）
│   └── fixtures/      # 测试数据（Builder 模式 / RON 文件）
```

#### 四层测试定义

| 层 | 名称 | 职责 | 示例 |
|------|------|------|------|
| **unit** | 单元测试 | 验证单个函数/纯规则的正确性 | HP 计算、Tag 包含检查、Modifier 优先级 |
| **integration** | 集成测试 | 验证领域内多组件协作 | 装备穿戴→Modifier→Attribute 联动 |
| **invariant** | 不变量测试 | 验证领域不变量（**最高价值**） | Tag bit 唯一、Buff 不重复叠加、HP>=0 |
| **fixtures** | 测试数据 | Builder 模式构造的测试数据 | RON 格式角色模板、技能配置 |

#### 不变量测试（最重要）

SRPG 核心架构（Attribute / Tag / Effect / Modifier / Buff / Skill / Turn）有大量领域不变量：

| 不变量 | 说明 |
|--------|------|
| Tag bit 唯一 | 同一 Tag 不能在位掩码中重复设置 |
| Buff 不重复叠加 | 同源同类型 Buff 不会无限堆叠 |
| Effect 不修改不存在属性 | Effect 引用的 AttributeId 必须已注册 |
| HP 永远 >= 0 | HP 计算结果不能为负 |
| Modifier 不改变基础值 | Modifier 只影响聚合后的当前值 |
| 回合先攻排序稳定 | 同先攻值的单位顺序确定 |
| 技能消耗原子性 | 消耗失败时不产生部分效果 |

> 不变量测试的价值远大于普通单元测试，是架构稳定性的最后防线。

#### 跨领域测试（根 tests/）

```
tests/
├── battle_flow/     # 完整战斗流程
├── save_load/       # 存档/读档完整性
├── regression/      # 回归测试（历史 Bug 复现）
├── replay/          # 回放确定性
├── golden/          # 金文件对比
├── simulation/      # 战斗模拟与数值平衡
├── performance/     # 性能回归
└── e2e/             # 端到端测试
```

### 12.3 测试基础设施
- 🟩 核心测试必须使用 Builder 模式构造测试数据，禁止每个测试手动构造大量实体
- 🟩 稳定输出必须使用金文件对比测试，版本升级后输出变化必须显式确认

### 12.4 确定性要求
- 🟩 战斗完全可重现：相同初始状态 + 相同输入序列 + 相同 RNG 种子，必须得到完全一致的战斗结果
- 🟩 禁止业务逻辑依赖系统时间，必须使用统一的 GameTime 服务
- 🟩 所有战斗相关 Bug 必须通过 Battle Replay 重现并转化为永久测试用例
- 🟩 测试必须覆盖所有核心规则，不追求表面的覆盖率数字

---

## 第十三编 资源与内容生产宪法
- 🟩 所有游戏设置必须通过统一的 Settings 体系管理
- 🟩 所有资源加载必须可追踪
- 🟩 所有资源的生命周期必须显式管理
- 🟩 所有资源必须分类统一管理
- 🟩 高频修改的资源必须优先支持热重载
- 🟩 编辑器是正式产品的一部分，不是开发工具
- 🟩 内容生产能力决定项目上限，工具链是长期项目的核心资产

---

## 第十四编 性能宪法
- 🟩 正确性优先：必须先保证代码正确，再考虑性能优化
- 🟩 测量优先：所有性能问题必须通过 Profile 测量确认，🟥 绝对禁止凭直觉优化
- 🟩 优化原则：优先优化热点代码，🟥 绝对禁止为了性能进行全局重构
- 🟩 可读性优先：禁止为了性能牺牲代码可读性，除非有明确的 Profile 数据证明
- 🟩 优先使用 `Changed` 过滤器减少不必要的计算
- 🟩 🟥 绝对禁止在高频计算中使用 Reflect
- 🟩 缓存通用规范：所有缓存必须明确定义失效条件与重建方式，缓存永远不是事实源
- 🟩 不需要的 Feature 必须裁剪，而非无脑开启
- 🟩 大多数独立游戏死于复杂度，而非性能

### 性能例外机制
严格禁止凭感觉优化突破架构边界，申请例外必须同时满足：
1. 有 Profiling 实证数据证明跨层通信产生了可观测的性能瓶颈
2. 仅限核心战斗路径（伤害计算、属性结算、寻路等高频调用）
3. 提交 ADR 架构决策记录，说明原因、影响范围、有效期
4. 明确后续重构计划与到期时间

#### 例外标记规范
```rust
// ARCH_EXCEPTION: 战斗伤害计算性能优化
// ADR-042
// Expires: 2027-01-01
// Approver: Architect
// 说明：跳过事件广播，直接函数调用，降低结算开销
```

---

## 第十五编 生命周期与状态机宪法
- 🟩 `OnEnter` 和 `OnExit` 系统必须保持轻量
- 🟩 重型初始化逻辑必须拆分成多个加载阶段
- 🟩 状态切换时 🟥 绝对禁止隐藏副作用
- 🟩 状态机只负责流程控制，🟥 绝对禁止包含业务细节
- 🟩 初始化过程必须可追踪、可恢复、可中断

---

## 第十六编 代码组织与编写规范
### 16.1 代码组织规则
- 🟩 单一文件单一主题
- 🟩 优先按业务主题拆分文件，🟥 绝对禁止按技术类型拆分全局文件
- 🟥 绝对禁止创建全局顶层的 `systems.rs`、`components.rs` 巨文件
- 🟥 绝对禁止创建 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件
- ⚠️ 文件大小阈值：单文件超过 500 行且内聚性下降时主动提出拆分，超过 1000 行强制评估
- 🟩 AI 可读性优先，优先使用直白的线性逻辑，避免宏套宏、深度泛型、类型体操

### 16.2 函数设计宪法
- 🟩 单一职责：每个函数只能有一个主要职责
- 🟩 函数命名必须描述意图，而非描述实现过程
- 🟩 优先使用 Early Return 减少嵌套
- ⚠️ 超过 3 层嵌套时主动提出重构建议
- 🟩 代码重复出现三次以上时才进行抽象
- 🟩 可读性优先，其次才考虑复用性

### 16.3 Trait 宪法
- 🟩 Trait 只能用于定义对象具备的能力，🟥 绝对禁止用于表示分类
- 🟩 Trait 只能用于定义需要扩展的接口，🟥 绝对禁止用于模拟继承树
- 🟥 绝对禁止为了"代码优雅"而创建无实际价值的 Trait

### 16.4 Feature 成熟度分级
- **Core 级**：核心战斗、角色、属性等基础系统，稳定性要求最高
- **Stable 级**：装备、任务、地图等成熟系统，新增功能不得破坏兼容性
- **Experimental 级**：玩法实验、辅助工具等功能，允许快速迭代

### 16.5 TODO / FIXME / HACK 规范
🟥 禁止无上下文的 TODO/FIXME。结构化注释是 AI 协作项目的核心工程资产。

#### 格式定义
```rust
// TODO[优先级][领域][日期]:
// 原因: [问题说明]
// 完成条件: [如何验证完成]
// 关联ADR: [可选]
// 负责人: [可选]

// FIXME[优先级][领域][日期]:
// 问题: [Bug 描述]
// 复现步骤: [如何复现]

// HACK[关联ADR/原因]:
// [临时绕过说明]
// [何时删除]
```

#### 优先级标准
| 等级 | 含义 | 门禁要求 |
|------|------|----------|
| P0 | 必须修，阻塞发布 | 禁止进入主分支 |
| P1 | 一个迭代内解决 | CI 拦截 |
| P2 | 正常技术债 | 登记跟踪 |
| P3 | 可长期存在 | 无强制要求 |

#### TODO vs FIXME
- **TODO** = 缺功能（未来要做）
- **FIXME** = 有 Bug（已知问题）
- **HACK** = 已知丑陋但暂时无法避免的绕过

#### 合法示例
```rust
// TODO[P2][ATTRIBUTE][2026-06-16]:
// 原因: Aggregator 当前每次全量重算。
// 完成条件: Battle Benchmark 提升 20%+

// FIXME[P1][REPLAY][2026-06-16]:
// 问题: Buff dispel 时触发两次移除事件。
// 复现步骤: 施加 Buff → dispel → 观察事件日志

// HACK[ADR-021]:
// 临时绕过 Trigger 递归问题，待 Runtime v2 重构后删除。
```

#### 非法示例
```rust
// TODO: 优化        ← 无优先级、无领域、无日期、无上下文
// TODO: 重构        ← 不知道什么时候、为什么
// FIXME            ← 无任何信息
```

### 16.6 测试命名规范
- 🟩 测试函数名用英文 snake_case 描述预期行为，使用业务术语如 `damage_applies_armor_reduction`、`buff_removed_on_expiry`、`hp_never_goes_below_zero`
- 🟩 文件名保持英文 snake_case

合法示例：
```rust
#[test]
fn damage_applies_armor_reduction_correctly() { ... }

#[test]
fn buff_removed_on_expiry() { ... }

#[test]
fn hp_never_goes_below_zero() { ... }
```

非法示例：
```rust
#[test]
fn test_damage() { ... }              // 无业务语义
#[test]
fn a() { ... }                        // 无意义命名
```

---

## 第十七编 长期维护与运营宪法
### 17.1 核心维护原则
- 🟩 代码首先是写给人看的，其次才是写给机器执行的
- 🟩 明确优于聪明，简单优于优雅，稳定优于炫技
- 🟩 删除无用代码通常比写新代码更有价值
- 🟩 社区维护的成本通常低于自维护成本
- 🟩 每引入一个自研系统，必须评估未来五年的维护成本
- 🟩 架构必须每 3 个月进行一次复盘和调整，重点清理过度设计和无用代码
- 🟩 工具链与内容生产能力最终决定项目成败

### 17.2 扩展预留规范
- 🟨 Mod 支持预留：核心系统预留轻量扩展点，不提前实现完整 Mod 框架
- 🟩 国际化强制：代码中绝对禁止出现用户可见的硬编码文本，所有用户可见文本必须通过 LocalizationKey + Fluent (.ftl) 文件管理；Def 只存 name_key/desc_key 等 Key 引用，不存直接文本；Replay/BattleLog/Event 只存 Key+参数，不保存最终翻译结果
- 🟨 遥测预留：核心领域事件设计时考虑数据埋点需求，不提前实现完整遥测系统

---

## 第十八编 工程质量与技术债治理
### 18.1 核心原则
- Warning Budget = 0：主分支不允许存在未处理的编译警告
- Bug Budget = 0：P0/P1 级缺陷不允许流入主分支
- Tech Debt Budget = 可控：技术债必须登记跟踪，设定偿还节点

### 18.2 问题分级标准
#### P0 致命级（必须立即修复，禁止提交/合并）
1. 核心业务代码出现 `unwrap()`、`expect()`、`panic!()`、`todo!()`、`unimplemented!()`
2. ECS 系统 Query Borrow 冲突
3. 事件/Observer/Trigger 触发无限循环
4. Entity 泄漏，长期运行资源持续增长
5. 存档兼容性损坏
6. 核心数据损坏、状态机非法跳转
7. 双轴边界严重突破：Capabilities 包含业务规则、Domain 间直接依赖

#### P1 高优先级（一个迭代内必须修复，CI 拦截）
1. Rust 必修编译警告（unused_imports、dead_code、deprecated 等）
2. 已废弃 API 调用
3. Clippy 必修项警告
4. 配置数据校验失败
5. 跨模块边界违规、架构约束被破坏
6. Domain 边界违规、Capabilities 越界实现业务逻辑

#### P2 中优先级（允许短期存在，登记跟踪）
1. 非热点代码存在可优化的性能问题
2. ECS Archetype 频繁抖动
3. 非核心路径存在过度日志、日志噪音
4. Archetype 数量膨胀风险

#### P3 低优先级（技术债，统一偿还）
1. 局部重复代码，未达到抽象阈值
2. 命名风格不统一，不影响功能与可读性
3. 单文件体积偏大但内聚性良好
4. 可补充的文档注释

### 18.3 Rust Warning 与 Clippy 规范
- 必修项警告必须 100% 修复，禁止无理由屏蔽
- 可暂缓项登记跟踪，按需处理
- 所有 `#[allow(...)]` 属性必须附带注释说明豁免理由与有效期

### 18.4 Bevy 专项检查
- 禁止每帧高频生成/销毁大量实体
- 禁止单帧触发大量无意义事件
- Observer/Trigger 递归必须设置深度限制
- 监控 Archetype 数量膨胀

### 18.5 编译治理专项
- 🟩 **依赖形状优化**：通过双轴架构实现 Domain 级平行编译，单个 Domain 修改不触发全量重编译
- 🟩 **Feature 开关**：支持按 Domain 开启/关闭编译，开发期可只编译当前工作域
- 🟩 **CI 增量编译**：配置 sccache 分布式编译缓存，大幅提升 CI 速度

### 18.6 CI 门禁强制标准
主分支准入必须全部满足，任意一项不满足禁止合并：
1. `cargo fmt` 0 格式问题
2. `cargo clippy` 0 必修项警告
3. `cargo test` 全部测试通过
4. 配置数据校验全部通过
5. 架构依赖检查无违规（含层间边界、Capabilities/Domain 边界）
6. Domain 间直接依赖检测无违规

### 18.7 技术债扫描六大维度

refactor-guardian 必须覆盖以下六个扫描维度（来源：50万行级项目实践评估）：

#### 18.7.1 Architecture Drift（架构漂移）
- ADR 定义的依赖方向 vs 实际代码依赖方向的偏差
- 检查：ADR 规定 A→B→C，实际是否出现 C→A 反向依赖
- 级别：方向违反 → Critical

#### 18.7.2 Abstraction Leakage（抽象泄漏）
- 跨域访问内部类型，绕过 integration/ 或 Facade 层
- 检查：`use xxx::mechanism` / `use xxx::internal` / `use xxx::model` 跨域出现
- 级别：跨域 internal 泄漏 → High

#### 18.7.3 AI Maintainability（AI 可维护性）
- 文件/函数/match 过大导致 AI 无法完整理解和修改
- 阈值：文件>1500行=High，>2500行=Critical；函数>100行=High；match>50 arm=High
- 级别：按阈值分级

#### 18.7.4 Test Debt（测试债务）
- 核心 Facade、Observer、Event 链缺乏测试覆盖
- 检查：`integration/facade.rs` 无对应 `tests/`；Observer 无集成测试
- 级别：核心 Facade 无测试 → High

#### 18.7.5 Content Debt（内容债务）
- 业务数值硬编码在代码中，应迁移到 `content/` 配置
- 检查：grep domains/ 中的 `damage=` / `range=` / `cooldown=` 等赋值
- 级别：硬编码业务数值 → Medium

#### 18.7.6 Debt Lifecycle（技术债生命周期）
- 所有 Debt 条目必须包含：状态（Open / Accepted / In Progress / Resolved / WontFix）、发现日期、负责人、关联 ADR
- ID 命名：`Debt-` / `Drift-ADR-` / `Leak-` / `Maintain-` / `TestDebt-` / `Content-`

---

## 第十九编 架构治理与演进
### 19.1 新增模块检查清单
每次新增模块/文件必须逐项确认：
- [ ] 明确归属层级与领域/Domain
- [ ] 未依赖禁止的层级与模块
- [ ] 未包含不属于当前层的逻辑
- [ ] Capabilities 模块未包含业务规则，Domain 未重复实现通用机制
- [ ] 未突破 Domain 边界直接依赖其他 Domain
- [ ] 错误类型定义在对应领域内部
- [ ] 强类型 ID 放在 `shared/ids/`
- [ ] 未创建 utils/helpers 垃圾桶文件

### 19.2 自动化门禁
- 开发期：自定义 Clippy Lint 实时提示跨层依赖、Domain 边界违规
- CI 阶段：运行依赖检查脚本全量扫描，违规直接阻断 PR
- 架构违规零容忍，不允许「先通过后修复」

### 19.3 架构版本管理
采用语义化版本，分级审批：
- **MAJOR**：结构性重构（合并/拆分层级、双轴结构调整）→ 全员评审 + 负责人批准
- **MINOR**：新增领域、新增接口、新增 Domain → Architect 审批
- **PATCH**：规则修正、文档补全 → 自主修改 + 同步周知

### 19.4 演进流程
需求评审 → ADR 记录 → 文档更新 → 全团队同步 → 代码迁移 → 验收确认
迁移期间新旧代码共存，通过 `#[deprecated]` 标记，确保零遗留。

---

## 第二十编 AI 专属执行规范
### 20.1 AI 反模式黑名单（生成前必须对照检查）
违反以下任意一条的代码必须立即重写：
1. 把 Entity 当面向对象实例，模拟 `player.attack(enemy)` 调用
2. 把 Resource 当全局变量仓库
3. 创建全局顶层的 `systems.rs/components.rs/events.rs` 巨文件
4. 滥用事件/Trigger 模拟普通函数调用
5. 业务逻辑直接操作 UI 组件、修改 UI 状态
6. 直接修改基础/派生属性的最终数值
7. 为单个实现创建无价值的 Trait
8. 为未明确的未来需求提前设计复杂架构
9. 手写 bool 脏标记检测组件生命周期变化
10. Release 版本每帧输出 INFO/DEBUG 级别日志
11. 输出技术流水账日志，而非领域事件日志
12. 业务代码直接手写 `info!` 输出核心业务事件
13. 使用全局 AppError、anyhow::Error 作为业务层统一错误类型
14. 核心业务领域代码中使用 unwrap/expect/panic
15. 混淆规则失败与程序错误，将正常规则不满足作为 Err 返回
16. 业务代码提交 todo!/unimplemented!() 占位
17. 新增核心业务系统未附带任何测试用例
18. 新增领域事件未纳入白名单文档直接使用
19. 核心领域逻辑直接依赖 Bevy 表现层类型，不做纯函数抽象
20. 预览/仿真等读路径带有副作用
21. 业务代码直接调用原始随机数 API，不使用统一 RNG 服务
22. 代码中硬编码数值平衡魔法数字
23. 跨模块直接修改其他 Feature 的内部状态
24. 未经授权创建新 Feature、修改公共 API
25. 在 Capabilities 层硬编码业务规则，破坏机制与业务的边界
26. Domain 之间直接 `use` 对方内部类型，绕过双轨通信（写操作走事件，读操作走 Query API）
27. 代码中硬编码用户可见文本字符串（中文/英文/日文等），未使用 LocalizationKey 引用本地化资源

### 20.2 AI 代码自检清单（文档参考，不输出到代码）

> **说明**：此清单仅作为 AI 生成代码时的内部参考，不要求在生成的代码中输出自检结果。
> 真正有效的合规检查依赖 CI 门禁（cargo clippy / dependency_checker / 架构扫描），而非 AI 自检。

AI 生成代码前应内部对照以下要点：

| 检查项 | 说明 |
|--------|------|
| 按业务拆分模块 | 无全局技术型巨文件（systems.rs / components.rs） |
| 配置与运行时分离 | Def（模板）与 Instance（运行时）不混写 |
| 逻辑与表现分离 | 业务规则不依赖渲染/音频/输入 |
| 组合优于继承 | 无子类派生式设计 |
| 不直接操作 UI | 业务逻辑不修改 UI 组件 |
| 属性走 Modifier 管线 | 不直接修改最终属性值 |
| 日志符合领域事件规范 | 不输出技术流水账 |
| 错误分领域定义 | 无全局 AppError 大枚举 |
| 核心业务层无 unwrap/panic | 仅测试/工具代码允许 |
| 双轴边界合规 | Capabilities 无业务规则，Domain 无重复机制 |
| Domain 间无直接依赖 | 写操作走事件，读操作走 Query API |
| 新增系统附带测试 | 测试跟领域走，含 invariant 层 |
| 读路径无副作用 | 预览/仿真不修改状态 |
| 使用统一 RNG 服务 | 不直接调用 rand::random() |
| 无硬编码魔法数字 | 数值配置归 content/ |

### 20.3 AI 权限边界
- 🟥 未经明确授权，禁止创建新的 Feature 模块与 Domain
- 🟥 未经明确授权，禁止修改各 Feature 的公共 API 定义
- 🟥 未经明确授权，禁止修改 Layers 层的核心机制实现

### 20.4 AI 最高优先级执行条款（10条）
超越所有其他条款的最高优先级：
1. 🟥 Feature First：按业务领域拆模块，不按技术类型拆全局目录
2. 🟥 Definition / Instance 强制分离：配置定义与运行时状态完全隔离
3. 🟥 Rule / Content 强制分离：代码只实现通用规则，配置只定义内容
4. 🟥 Logic / Presentation 强制分离：业务逻辑与表现层完全隔离
5. 🟥 四级通信机制：Hook=生命周期，Trigger=事件链，Observer=局部响应，Message=跨域广播
6. 🟥 属性管线统一：基础/派生属性修改必须走 Modifier 体系
7. 🟥 数据驱动绝对优先：成熟扩展点纯配置扩展，新机制允许修改逻辑
8. 🟩 Capabilities/Domains 双轴架构原则：Capabilities 管机制，Domains 管业务，边界不可突破
9. 🟥 测试与确定性优先：Battle Replay + 自动化测试，核心战斗必须可重现
10. 🟥 组合绝对优先：所有差异化通过组件、Trait、Modifier 组合实现
11. 🟥 Localization First：所有用户可见文本必须通过 LocalizationKey 引用，禁止任何用户可见文本硬编码在 Rust 代码中；Def 只存 name_key/desc_key 不存直接文本；存档/Replay/Event 只存 Key+参数，不存翻译结果

---

## 第二十一编 红线禁止事项总览
1. 🟥 禁止创建 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件
2. 🟥 禁止用 `bool` 标志位替代实体级 Tag 系统
3. 🟥 禁止面向对象式实体调用（如 `player.attack(enemy)`）
4. 🟥 禁止非确定性随机源破坏回放
5. 🟥 禁止 UI 层持有业务真相，UI 只能只读展示
6. 🟥 禁止直接修改最终属性值，必须通过 Modifier 管线
7. 🟥 禁止 Core 层引入渲染、音频、资源、输入等引擎表现能力
8. 🟥 禁止 Shared 层引入任何业务逻辑或业务类型
9. 🟥 禁止反向依赖与循环依赖
10. 🟥 禁止硬编码游戏数值与业务内容
11. 🟥 禁止核心业务领域使用 unwrap/expect/panic/todo
12. 🟥 禁止全局统一错误大枚举与 anyhow 滥用
13. 🟥 禁止为临时副作用随意新增领域事件
14. 🟥 禁止凭感觉优化突破架构边界
15. 🟥 禁止 Capabilities 层包含具体业务规则，突破机制与业务的边界
16. 🟥 禁止 Domain 之间直接依赖、直接调用内部实现
17. 🟥 禁止 Domain 重复实现 Capabilities 已有的通用机制
18. 🟥 禁止在 Rust 代码中硬编码任何用户可见文本（技能名称、描述、对话、UI 标签、错误提示等），所有用户可见文本必须通过 LocalizationKey 从外部本地化文件引用

---

## 第二十二编 Localization（国际化）专项规则

### 22.1 核心原则（P0 级）

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.1.1 | **代码中绝对禁止出现用户可见文本** | 🟥 | 代码中只允许出现 LocalizationKey，不允许出现任何中文/英文/日文等用户可见自然语言文本 |
| 22.1.2 | **Def 只存 LocalizationKey** | 🟥 | AbilityDef、EffectDef、ItemDef、QuestDef 等所有 Definition 类型的文本字段必须使用 name_key/desc_key/text_key，禁止直接存储用户可见字符串 |
| 22.1.3 | **Replay/Event 只存 Key+参数** | 🟥 | BattleLog、领域事件、回放帧中禁止保存最终翻译文本，必须使用 Key + 结构化参数，确保语言切换时正确渲染 |
| 22.1.4 | **存档禁止保存翻译文本** | 🟥 | 存档中只能存储 ID/Key，禁止保存任何翻译结果，确保切语言、更新翻译、Mod 覆盖全部安全 |
| 22.1.5 | **Localization 属于 Infrastructure 层** | 🟩 | Localization 是全局基础设施，不属于 UI 层，不属于 Capabilities 能力层。所有用户可见文本的唯一下游 |

### 22.2 LocalizationKey 规范

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.2.1 | **Key 格式** | 🟩 | `LocalizationKey ::= <namespace> "." <scope> "." <id> "." <suffix>` |
| 22.2.2 | **Key 使用无语义 ID** | 🟩 | 优先使用 `ability.abl_000042.name` 而非 `ability.fireball.name`，避免业务重命名导致 Key 失效 |
| 22.2.3 | **Key 必须稳定** | 🟩 | Key 一旦分配永久有效，删除时标记 deprecated，不重新分配 |
| 22.2.4 | **命名空间分层** | 🟩 | L0 Core（系统文本）→ L1 UI（界面文本）→ L2 Gameplay（玩法文本）→ L3 Story（剧情文本），生命周期从稳定到高频变化 |
| 22.2.5 | **必须使用 Fluent (.ftl) 格式** | 🟨 | 优先使用 Fluent (.ftl) 作为本地化文件格式，利用其变量插值、复数规则、性别支持能力 |
| 22.2.6 | **禁止手写复数逻辑** | 🟥 | 复数规则必须交给 Fluent 内置复数系统处理，禁止在代码中手写 if-en/other 等复数判断 |

### 22.3 基础设施与工具

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.3.1 | **LocalizationPlugin** | 🟩 | 必须建立 `LocalizationPlugin` 统一管理本地化生命周期，注册在 Content Plugin 之后、UI Plugin 之前 |
| 22.3.2 | **LocalizationKey 自动生成 Rust 常量** | 🟩 | 必须通过 build.rs 从 .ftl 文件自动生成 Rust 常量模块（如 `loc::ability::abl_000042::NAME`），提供编译期 Key 检查 |
| 22.3.3 | **启动时完整性校验** | 🟩 | 启动时必须对所有已注册的 LocalizationKey 进行完整性检查，缺失 Key 直接阻止启动 |
| 22.3.4 | **Fake Locale (zz-ZZ)** | 🟨 | 必须建立 zz-ZZ 伪语言 locale 用于检测硬编码文本，通过 feature flag 启用 |
| 22.3.5 | **三级回退链** | 🟩 | `{locale}` → `en-US` → `raw_key` 三级回退，禁止直接显示 [Missing Localization] |
| 22.3.6 | **热加载支持** | 🟨 | 修改 .ftl 文件必须热加载生效，无需重启游戏 |
| 22.3.7 | **LocalizedTextCache** | 🟨 | 运行时必须使用缓存避免每帧查询 LocalizationDatabase，语言切换时清空重建 |
| 22.3.8 | **Mod 覆盖链** | 🟩 | 支持 Base Game → DLC → Mod 三级本地化覆盖链 |
| 22.3.9 | **文本长度预算** | 🟨 | UI 设计必须为多语言预留扩展空间（建议 30%~50%），CI 自动检查超长文本 |

### 22.4 CI 与审计

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.4.1 | **CI Localization 检查** | 🟩 | CI 必须包含缺失 Key、重复 Key、未引用 Key、参数不匹配、文本长度超限等本地化检查 |
| 22.4.2 | **翻译覆盖率报告** | 🟩 | 必须定期生成按分类（UI/Gameplay/Quest/Story/Tutorial）的翻译覆盖率报告 |
| 22.4.3 | **废弃 Key 管理** | 🟩 | 支持 deprecated Key 标记，审计输出废弃 Key 列表供清理 |
| 22.4.4 | **术语库（Glossary）** | 🟨 | 必须建立项目术语库，确保术语翻译全项目一致 |

### 22.5 语音预留

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.5.1 | **文本设计预留语音** | 🟨 | 对话数据设计时预留 voice_key/subtitle 字段，即使当前不做配音 |
| 22.5.2 | **Key 体系支持语音扩展** | 🟩 | `story.ch01.dlg_001` 天然支持 text/voice/subtitle 三层扩展 |

---

## 附则
### 修订说明
- 本宪法版本：v5.1（Bevy 0.18+）
- 发布日期：2026-06-19
- 核心升级（v4.1 → v5.0）：
  5. **Localization 国际化专项规则新增**
     - §1.5 P0 顶层铁则新增第7条 Localization First
     - §21 红线禁止事项新增第18条 禁止硬编码用户可见文本
     - §20.1 AI 反模式黑名单新增第27条 硬编码文本违规项
     - §20.4 AI 最高优先级执行条款新增第11条 Localization First
     - §17.2 扩展预留国际化条款从🟨升级为🟩
     - **新增 §22 第二十二编 Localization 专项规则**（22.1~22.5，共 5 节 22 条规则）
- 核心升级（v4.1 → v5.0）：
  1. **Core 内部架构对齐**
     - 将 Layers（L0~L4）5级分层重构为 Capabilities（15个能力领域）+ Domains（15个业务域）+ Mod API 双轴结构
     - Capabilities 内部采用 C1 Foundation / C2 Mechanism / C3 Runtime 三层架构（内聚优于分层）
     - 新增 GameplayContext 上下文载荷领域、Summon 召唤业务域
     - Capabilities 当前15个领域，新增必须通过 ADR 审批
  2. **Mod API 升级**
     - 重构为 Facade + Gateway 模式，每个 Domain 对应一个 Gateway
     - 对外只暴露业务语义接口，隐藏内部分层结构
  3. **治理规则更新**
     - 红线禁止事项第15/17条更新为 Capabilities 相关描述
     - AI 反模式黑名单与自检清单更新为 Capabilities/Domains 术语
     - 问题分级标准更新为双轴边界违规描述
  4. **Shared 层补全**
     - 新增 `traits/` 目录，存放日志/审计/事务等通用横切抽象 Trait
     - 新增 `prelude/` 统一导出
- 修订周期：每半年根据 Bevy 版本更新和项目实践进行一次修订
- 效力期限：永久有效，除非发布新版本宪法明确替代

需要的话，我可以基于 v4.1 双轴架构，输出升级后的 Core 层完整目录脚手架，以及对应的 Domain 边界检查脚本。
