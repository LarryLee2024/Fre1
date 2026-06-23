---
id: SRPG-SYSTEMS-CONSTITUTION
title: SRPG 核心系统专项宪法
status: accepted
stability: stable
layer: domain
related:
  - ai-constitution-complete.md
  - ecs-constitution.md
tags:
  - srpg
  - combat
  - attribute
  - skill
  - buff
  - camera
  - command
---

> **原文来源**：`ai-constitution-complete.md` 第八编（L867-L974）
> **锚定总宪法**：第八编

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
- 🟩 **Capability 运行时查询**：应提供 `has::<CanAttack>()` 式的统一查询 API，替代 `query.get::<Component>(entity).is_ok()` 的模式。查询 API 位于 Capabilities 层，所有 Domain 通过统一入口获取实体的能力状态
- 🟩 **Object Safety 分层策略**：热路径（战斗执行、属性计算）必须使用泛型静态分发；冷路径（编辑器、Mod系统、工具）允许使用 dyn 动态分发；设计 trait 时必须考虑 object safety，显式标注是否允许 dyn；架构规则"Registry + Trait Object"限定在冷路径
- 🟩 **编译期能力约束**：关键能力接口（如技能执行、物品使用）应通过 trait bound 在编译期约束，而非仅运行时检查；`fn execute<T: CanCast>(...)` 优于 `if unit.can_cast()`；编译期约束是组合而非继承——每个能力是独立 Marker Trait，实体自由组合，无父子层级

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
- 🟩 **SSOT 唯一实现位置**：DamageFormula 在 combat/rules/ 中有唯一实现位置，UI 伤害预览必须通过 integration 层调用同一公式，禁止在 UI 层复制公式逻辑
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
- 🟩 **Policy 模式要求**：伤害结算、掉落判定、目标选择等策略逻辑必须收敛为独立 Policy 对象（如 DamagePolicy、LootPolicy、TargetPolicy），禁止散落在 System 中的 if 链。参考 economy 域 RestockPolicy 模式：单一 `evaluate()` 入口，返回结构化决策结果。

### 8.7 命令层（Command Layer）
- 🟩 **所有操作入口为命令**
  - 玩家输入、AI 决策、回放执行、网络同步，最终都必须转换为标准化业务命令
  - 典型命令：`MoveUnitCommand`、`CastSkillCommand`、`EndTurnCommand`
- 🟩 **命令无差别执行**
  - 执行系统不区分命令来源，只处理命令本身

### 8.8 输入抽象层
- 🟩 输入为独立 Feature，鼠标、键盘、手柄、触摸等输入统一转换为业务命令
- 🟥 绝对禁止业务模块直接读取原始输入按键

### 8.9 读写分离原则（CQRS）
- 🟩 **读路径无副作用**
  - 伤害预览、范围查询、状态展示等读操作，必须使用纯函数或只读查询
  - 绝对禁止在读路径中修改状态、触发事件、消耗资源
- 🟩 **写路径收口**
  - 所有状态修改必须通过命令与执行系统统一处理
  - 禁止零散分布的状态写入逻辑
- 🟩 **显式 WriteFacade / ReadFacade 分离**
  - Domain integration 层必须明确区分 WriteFacade（命令处理）和 ReadFacade（查询 API）
  - 读模型使用扁平的 View 结构体，写模型使用 Aggregate + Event 模式
  - UI 层的 Projection 模式 = ReadFacade 的 UI 端实现
- 🟩 **现有实现参考**：Combat facade.rs（build_effect_view / request_effect_apply）+ MovementCapabilityView + AggregateDirty 事件

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

### 8.13 Camera 系统宪法（ADR-064）
- 🟩 **Camera 是 Infra 层独立模块**：Camera 位于 `src/infra/camera/`，属于 L2 Infra 表现层，不是业务 Domain，不是 UI 子模块
- 🟥 **Event 驱动，禁止直接修改 Transform**：所有外部镜头操作必须通过 `commands.trigger(CameraRequest::...)`，禁止外部系统直接 Query<&mut Transform, With<Camera>> 修改镜头——违反 Event 驱动原则，导致 Camera 状态机被绕过，不可 Replay
- 🟥 **业务解耦**：Camera 禁止依赖 `core::domains::*` 的任何类型，CameraTarget 使用 Vec2/UnitId/i32 等非领域类型，禁止 Camera 模块中出现 Combat/Dialogue/Unit 等业务词汇
- 🟩 **Replay 兼容**：CameraCommand 必须支持 Serialize/Deserialize，关键帧镜头操作通过 CameraCommand 录制
- 🟩 **边界解耦**：CameraBounds 使用纯 Vec2，不包含 GridPos/TileMap 引用，边界由场景系统在 OnEnter 生命周期设置
- 🟥 绝对禁止 Camera 系统直接读取 GridPos/TileMap/MapConfig 等地图数据——Camera 是表现层，不受业务数据污染
