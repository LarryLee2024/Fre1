---
id: 02-domain.README
title: Domain Rules
status: stable
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
---

# Domain Rules

Version: 1.0
Created: 2026-06-14

本文档是 `docs/02-domain/` 文件夹所有领域规则文件的汇总索引和速查参考。

---

## 总览

| 分组 | 文件数 | 覆盖主题 |
|------|--------|----------|
| Core Domain | 14 | 战斗、角色、技能、属性修饰、回合、触发、条件、公式、目标选择、持续时间、消耗、堆叠策略、释放前提、输入 |
| Infrastructure | 7 | 错误系统、日志、持久化、热重载、确定性、回放、测试 |
| Content/Data | 6 | 内容系统、配置系统、内容迁移、资源生命周期、资源组织、Feature Flag |
| Cross-cutting | 12 | 分层架构、ECS通信、命令总线、共享层、Mod系统、UI架构、本地化、地图地形、AI、性能预算、校验、事件审计 |

---

## Core Domain（核心业务规则）

| # | 文件 | 架构对应 | 关键词 |
|---|------|----------|--------|
| 1 | battle_rules.md | battle_fsm_design.md | 战斗状态机、Effect Pipeline、伤害计算 |
| 2 | character_rules.md | component_design_rules.md | 角色属性、Faction、UnitSnapshot |
| 3 | skill_rules.md | skill-buff-abstraction.md | 技能定义、冷却、释放管线 |
| 4 | attribute_modifier_rules.md | component_design_rules.md | 属性修饰、Modifier管线、叠加规则 |
| 5 | turn_rules.md | battle_fsm_design.md | 回合阶段、TurnPhase、行动队列 |
| 6 | trigger_rules.md | events_audit_design.md | 触发器、事件链、Trigger机制 |
| 7 | condition_rules.md | skill-buff-abstraction.md | 条件系统、效果判断、运行时条件 |
| 8 | formula_rules.md | content_data_format.md | 公式系统、数值计算、表达式求值 |
| 9 | selector_rules.md | skill-buff-abstraction.md | 目标选择、AOE、空地选择 |
| 10 | duration_rules.md | skill-buff-abstraction.md | 持续时间、回合/真实时间、永久效果 |
| 11 | cost_rules.md | skill-buff-abstraction.md | 消耗系统、资源扣除、消耗类型 |
| 12 | stack_policy_rules.md | skill-buff-abstraction.md | 堆叠策略、Buff叠加、替换规则 |
| 13 | requirement_rules.md | skill-buff-abstraction.md | 释放前提、技能可用性、前提检查 |
| 14 | input_rules.md | infrastructure-design.md | 输入处理、UiCommand、触摸/键盘 |

**battle_rules.md**: 战斗状态机领域，管理战斗流程的完整生命周期，包括初始化、回合循环、效果执行、战斗结算。
- 🟥 Effect Pipeline 必须严格按 Generate → Modify → Execute 三步执行
- 🟥 战斗数值修改必须通过 Modifier 管线，禁止直接修改
- 🟩 BattleRecord 记录所有战斗事件用于回放

**character_rules.md**: 角色属性领域，管理角色的属性定义、Faction阵营、UnitSnapshot快照。
- 🟥 角色属性必须通过 Attribute 系统管理
- 🟩 Faction 区分玩家/敌人/中立；UnitSnapshot 用于AI决策快照

**skill_rules.md**: 技能系统领域，管理技能定义、冷却机制、五阶段释放管线。
- 🟥 技能释放必须经过 Requirement → Cost → Selector → Effect 四阶段
- 🟩 技能配置通过 RON 文件数据驱动

**attribute_modifier_rules.md**: 属性修饰器领域，管理 Modifier 管线的生成、修改、执行三阶段。
- 🟥 Modifier 是修改属性的唯一通道，禁止绕过
- 🟩 支持加法、乘法、覆盖三种操作

**turn_rules.md**: 回合制领域，管理 TurnPhase 阶段转换和行动队列编排。
- 🟥 回合阶段转换只通过 NextState 驱动
- 🟩 TurnPhase：SelectUnit/MoveUnit/SelectTarget/ExecuteAction/WaitAction

**trigger_rules.md**: 触发器领域，管理战斗事件链的触发和传播。
- 🟥 Trigger 用于同Feature内事件链（伤害→护盾→吸血→反击）
- 🟥 不跨模块通信，跨模块用Message

**condition_rules.md**: 条件系统领域，管理效果执行时的条件判断。
- 🟥 Condition 判断"效果是否生效"（执行时），不是"技能能不能放"
- 🟩 条件类型通过注册表分发

**formula_rules.md**: 公式系统领域，管理数值计算的表达式求值。
- 🟥 公式配置通过 RON 文件定义，求值为纯函数
- 🟩 支持自定义公式扩展

**selector_rules.md**: 目标选择器领域，管理技能的目标选择规则和AOE范围。
- 🟥 Selector 只负责目标选择，不负责效果执行（纯函数）
- 🟩 支持 EnemySingle/EnemyAOE/AllySingle/SelfOnly/EmptyTile 等

**duration_rules.md**: 持续时间领域，管理效果的持续时间类型和过期处理。
- 🟥 持续时间类型：回合数、真实时间、永久
- 🟩 回合结束时自动递减，到期自动移除

**cost_rules.md**: 消耗系统领域，管理技能释放的资源消耗校验和扣除。
- 🟥 消耗检查在 Requirement 检查之后执行
- 🟩 支持 HP/MP/道具 等多种消耗类型

**stack_policy_rules.md**: 堆叠策略领域，管理Buff/Debuff的叠加、替换、刷新规则。
- 🟥 堆叠策略决定同类型效果的叠加行为
- 🟩 策略包括：叠加层数、替换旧效果、刷新持续时间

**requirement_rules.md**: 释放前提系统领域，管理技能释放前必须满足的条件。
- 🟥 Requirement 判断"技能能不能放"，前提检查为纯函数
- 🟩 9种内置前提类型，任一不满足时技能不可用

**input_rules.md**: 输入处理领域，管理键盘、鼠标、触摸输入到UiCommand的转换。
- 🟥 输入处理在 InputSchedule 中执行
- 🟩 输入转换为 UiCommand Message 传递给Core层

---

## Infrastructure（基础设施规则）

| # | 文件 | 架构对应 | 关键词 |
|---|------|----------|--------|
| 15 | error_system_rules.md | error-architecture.md | 错误处理、Result传播、错误分级 |
| 16 | logging_rules.md | logging_design.md | 日志分级、日志格式、调试日志 |
| 17 | persistence_rules.md | save_migration_rules.md | 存档、存档格式、版本迁移 |
| 18 | hot_reload_rules.md | infrastructure-design.md | 热重载、Definition热更新 |
| 19 | determinism_rules.md | determinism_rules.md | 确定性、随机种子、RNG流 |
| 20 | replay_rules.md | events_audit_design.md | 战斗回放、审计事件、双轨制日志 |
| 21 | testing_rules.md | testing_architecture.md | 测试规范、测试金字塔、回放测试 |

**error_system_rules.md**: 错误系统领域，管理错误处理的分级、传播和恢复。
- 🟥 可恢复错误必须返回 Result，禁止 panic
- 🟩 错误分级：Debug/Info/Warn/Error/Fatal

**logging_rules.md**: 日志系统领域，管理日志分级、格式和输出规范。
- 🟥 生产构建禁用 Debug/Trace 级别日志
- 🟩 关键操作必须记录 Info 级别日志

**persistence_rules.md**: 持久化领域，管理存档的生成、加载和版本迁移。
- 🟥 存档格式必须支持版本迁移，损坏时有降级策略
- 🟩 存档数据与 Definition 数据物理隔离

**hot_reload_rules.md**: 热重载领域，管理Definition配置的运行时热更新。
- 🟥 热重载只更新 Definition，不修改 Instance；战斗中禁止热重载
- 🟩 失败时回退到上次有效状态

**determinism_rules.md**: 确定性领域，管理战斗的确定性执行保证。
- 🟥 相同初始条件+相同事件流+相同RNG种子→相同结果
- 🟩 多RNG流独立（战斗/掉落/世界/AI）

**replay_rules.md**: 战斗回放领域，管理基于审计事件流的确定性重放。
- 🟥 回放消费Command Stream（Track A），不消费Audit Trail
- 🟩 所有战斗Bug必须通过Battle Replay重现并转化为测试用例

**testing_rules.md**: 测试规范领域，管理测试金字塔和测试标准。
- 🟥 核心领域逻辑必须有单元测试覆盖
- 🟩 回放测试纳入CI回归测试套件

---

## Content/Data（数据与内容规则）

| # | 文件 | 架构对应 | 关键词 |
|---|------|----------|--------|
| 22 | content_system_rules.md | content-pipeline.md | 内容管线、RON加载、Registry |
| 23 | config_system_rules.md | config_system_design.md | 配置系统、热重载配置 |
| 24 | content_migration_rules.md | content_migration_design.md | 内容迁移、版本兼容 |
| 25 | asset_lifecycle_rules.md | asset_lifecycle_rules.md | 资源生命周期、Handle类型、内存预算 |
| 26 | asset_organization_rules.md | asset-organization.md | 资源组织、三树分离、Content Packs |
| 27 | feature_flag_rules.md | feature_flag_design.md | Feature Flag、灰度发布 |

**content_system_rules.md**: 内容系统领域，管理RON配置文件的加载、校验和Registry注册。
- 🟥 所有配置通过AssetServer加载，Definition运行时不可变
- 🟩 Registry统一管理配置数据的加载和查找

**config_system_rules.md**: 配置系统领域，管理运行时配置的加载和热重载。
- 🟥 配置热重载只更新Definition，不修改Instance
- 🟩 配置校验失败时使用默认值

**content_migration_rules.md**: 内容迁移领域，管理配置数据的版本兼容。
- 🟥 版本号递增：小版本+1（新增字段），大版本+1（删除/类型变更）
- 🟩 新字段必须有默认值保证向后兼容

**asset_lifecycle_rules.md**: 资源生命周期领域，管理资源的加载、引用追踪、卸载和降级。
- 🟥 每帧卸载≤4MB；Strong Handle生命周期与宿主一致
- 🟩 场景切换分阶段卸载；加载失败降级（重试→Fallback→继续）
- 🟩 内存预算：Menu 64MB / Battle 256MB / Cutscene 128MB

**asset_organization_rules.md**: 资产组织领域，管理美术资源的物理组织和命名规范。
- 🟥 三树分离：assets/（美术）、content/（配置）、src/（代码）
- 🟩 Content Packs按功能单元组织；资源命名空间：namespace + category + name

**feature_flag_rules.md**: Feature Flag领域，管理实验性功能的开关和灰度发布。
- 🟥 Feature Flag通过配置文件管理，禁止硬编码
- 🟩 支持按设备/地区/用户群灰度发布

---

## Cross-cutting（横切关注点）

| # | 文件 | 架构对应 | 关键词 |
|---|------|----------|--------|
| 28 | layer_architecture_rules.md | layer-contracts.md | 分层架构、层间依赖 |
| 29 | ecs_communication_rules.md | schedules_design.md | ECS通信、Message/Observer/Hook/Trigger |
| 30 | command_bus_rules.md | command_bus_design.md | 命令总线、UiCommand |
| 31 | shared_layer_rules.md | infrastructure-design.md | 共享层、公共类型 |
| 32 | modding_system_rules.md | modding-design.md | MOD系统、资源命名空间 |
| 33 | ui_architecture_rules.md | ui_domain_boundary_rules.md | UI架构、ViewModel |
| 34 | localization_rules.md | i18n_design.md | 本地化、多语言 |
| 35 | map_terrain_rules.md | pathfinding_design.md | 地图地形、寻路、视野 |
| 36 | ai_rules.md | infrastructure-design.md | AI行为、策略模板、决策管线 |
| 37 | performance_budget_rules.md | performance_budget.md | 性能预算、帧率、内存 |
| 38 | validation_rules.md | validation_rules.md | 校验规则、数据完整性 |
| 39 | event_audit_rules.md | events_audit_design.md | 事件审计、双轨制日志 |

**layer_architecture_rules.md**: 分层架构领域，管理模块间的层间依赖方向。
- 🟥 依赖方向：Shared → Infrastructure → Core → Content → UI
- 🟥 禁止反向依赖；跨层通信必须通过Message/Resource

**ecs_communication_rules.md**: ECS通信领域，管理Hook/Observer/Message/Trigger四种通信方式。
- 🟥 跨模块通信必须通过Message；Hook不包含业务逻辑
- 🟩 Observer用于同Feature内局部响应；Trigger用于事件链

**command_bus_rules.md**: 命令总线领域，管理UiCommand的定义、分发和消费。
- 🟥 UiCommand是UI→Core的唯一通信通道
- 🟩 命令携带完整上下文，接收方无需反向查询

**shared_layer_rules.md**: 共享层领域，管理跨层共享的公共类型。
- 🟥 Shared层类型可被所有层引用，禁止实现业务逻辑

**modding_system_rules.md**: MOD系统领域，管理MOD的加载、资源隔离。
- 🟥 MOD资源必须带命名空间前缀，失效时回退到base
- 🟩 MOD加载通过AssetResolver的Resolution Chain

**ui_architecture_rules.md**: UI架构领域，管理ViewModel、UiCommand和UI渲染。
- 🟥 UI只读ViewModel，不直接Query Core组件
- 🟩 ViewModel在LogicSchedule之后更新

**localization_rules.md**: 本地化领域，管理多语言翻译和本地化资源。
- 🟥 翻译文件通过RON配置，支持热重载
- 🟩 本地化键使用点分隔命名（如 `skill.fireball.name`）

**map_terrain_rules.md**: 地图地形领域，管理地图数据、地形类型和寻路算法。
- 🟥 地图数据通过RON加载；寻路算法必须确定性
- 🟩 大地图使用流式块加载

**ai_rules.md**: AI行为系统领域，管理敌方单位的自动决策和行动执行。
- 🟥 AI通过Intent+Effect Pipeline执行，与玩家共用同一通道
- 🟥 AI不访问玩家不可见信息；行为配置从RON加载
- 🟩 策略通过Trait+注册表分发，支持扩展

**performance_budget_rules.md**: 性能预算领域，管理帧率目标和内存限制。
- 🟥 Battle场景内存上限256MB（PC）/192MB（移动端）
- 🟩 目标帧率：PC 60fps，移动端 30fps

**validation_rules.md**: 校验规则领域，管理数据完整性和配置校验。
- 🟥 配置加载时必须校验，失败时使用默认值
- 🟩 校验规则通过Validator trait实现

**event_audit_rules.md**: 事件审计领域，管理战斗事件的记录和审计日志。
- 🟥 审计事件在白名单中注册；支持序列化
- 🟩 双轨制：Track A（回放）/ Track B（调试统计）

---

## 架构 ↔ 领域 交叉引用

| 架构文件 | 领域文件 | 关系 |
|---------|---------|------|
| battle_fsm_design.md | battle_rules.md | 完全对应 |
| component_design_rules.md | character_rules.md | 部分重叠 |
| skill-buff-abstraction.md | skill_rules.md | 完全对应 |
| component_design_rules.md | attribute_modifier_rules.md | 部分重叠 |
| battle_fsm_design.md | turn_rules.md | 完全对应 |
| events_audit_design.md | trigger_rules.md | 部分重叠 |
| skill-buff-abstraction.md | condition_rules.md | 完全对应 |
| content_data_format.md | formula_rules.md | 部分重叠 |
| skill-buff-abstraction.md | selector_rules.md | 完全对应 |
| skill-buff-abstraction.md | duration_rules.md | 完全对应 |
| skill-buff-abstraction.md | cost_rules.md | 完全对应 |
| skill-buff-abstraction.md | stack_policy_rules.md | 完全对应 |
| skill-buff-abstraction.md | requirement_rules.md | 完全对应 |
| infrastructure-design.md | input_rules.md | 部分重叠 |
| error-architecture.md | error_system_rules.md | 完全对应 |
| logging_design.md | logging_rules.md | 完全对应 |
| save_migration_rules.md | persistence_rules.md | 完全对应 |
| infrastructure-design.md | hot_reload_rules.md | 部分重叠 |
| determinism_rules.md | determinism_rules.md | 完全对应 |
| events_audit_design.md | replay_rules.md | 完全对应 |
| testing_architecture.md | testing_rules.md | 完全对应 |
| content-pipeline.md | content_system_rules.md | 完全对应 |
| config_system_design.md | config_system_rules.md | 完全对应 |
| content_migration_design.md | content_migration_rules.md | 完全对应 |
| asset_lifecycle_rules.md | asset_lifecycle_rules.md | 完全对应 |
| asset-organization.md | asset_organization_rules.md | 完全对应 |
| feature_flag_design.md | feature_flag_rules.md | 完全对应 |
| layer-contracts.md | layer_architecture_rules.md | 完全对应 |
| schedules_design.md | ecs_communication_rules.md | 部分重叠 |
| command_bus_design.md | command_bus_rules.md | 完全对应 |
| infrastructure-design.md | shared_layer_rules.md | 部分重叠 |
| modding-design.md | modding_system_rules.md | 完全对应 |
| ui_domain_boundary_rules.md | ui_architecture_rules.md | 完全对应 |
| i18n_design.md | localization_rules.md | 完全对应 |
| pathfinding_design.md | map_terrain_rules.md | 完全对应 |
| infrastructure-design.md | ai_rules.md | 部分重叠 |
| performance_budget.md | performance_budget_rules.md | 完全对应 |
| validation_rules.md | validation_rules.md | 完全对应 |
| events_audit_design.md | event_audit_rules.md | 完全对应 |

---

## 快速查找

| 功能 | 领域文件 |
|------|---------|
| 技能释放流程 | skill_rules.md, requirement_rules.md, cost_rules.md, selector_rules.md |
| Buff/Debuff | attribute_modifier_rules.md, stack_policy_rules.md, duration_rules.md, condition_rules.md |
| 回合制战斗 | turn_rules.md, battle_rules.md |
| AI决策 | ai_rules.md |
| 数值计算 | formula_rules.md, attribute_modifier_rules.md |
| 地图与寻路 | map_terrain_rules.md |
| 资源管理 | asset_lifecycle_rules.md, asset_organization_rules.md |
| 配置数据 | content_system_rules.md, config_system_rules.md, content_migration_rules.md |
| 存档与回放 | persistence_rules.md, replay_rules.md, determinism_rules.md |
| UI交互 | ui_architecture_rules.md, input_rules.md, command_bus_rules.md |
| 错误处理 | error_system_rules.md, logging_rules.md |
| MOD支持 | modding_system_rules.md |
| 本地化 | localization_rules.md |
| 测试规范 | testing_rules.md |
| 性能优化 | performance_budget_rules.md |
| 校验规则 | validation_rules.md |
| 事件审计 | event_audit_rules.md |
| Feature开关 | feature_flag_rules.md |
