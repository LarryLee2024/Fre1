---
id: 03-technical.determinism-rules
title: Determinism Rules
status: draft
owner: feature-developer
created: 2026-06-14
updated: 2026-06-14
tags:
  - technical
---

# 确定性执行领域

Version: 1.0
Status: Proposed
> **优化来源**: `docs/01-architecture/determinism_rules.md`（吸收 48.md 内容：预生成随机流、派生种子、LogicalId 排序、整数溢出防护、浮点污染隔离墙、ChaCha8Rng、tick 排序）

确定性执行领域管理 SRPG 战斗的物理定律：相同输入 + 相同种子 → 相同结果。

核心原则（对应宪法第十八部分：测试与确定性宪法 + 第十一部分 11.9 随机数分层管理）：
- 🟩 18.4.1 战斗完全可重现：相同输入 + 相同种子 → 相同结果
- 🟩 11.9.1 多 RNG 流独立：战斗/掉落/世界/AI 各有独立随机数流
- 🟩 11.9.2 统一 RNG 服务：业务逻辑禁止直接调用 rand::random()
- 战斗核心数值只用整数，禁止浮点
- ECS 查询结果必须按 LogicalId 显式排序，禁止依赖默认迭代顺序
- 事件排序使用 tick 计数器，禁止使用时间戳

---

# 术语定义

## 确定性（Determinism）

相同输入 + 相同种子 → 相同结果的执行保证，是回放系统、Bug 复现、自动化测试的基石。

不是固定结果。不是无随机。

关键属性：
- 给定相同种子和输入序列，游戏产出完全相同的状态序列
- 包含随机性（通过确定性 PRNG 管理）
- 包含浮点数（通过整数运算替代）
- 依赖显式排序（而非 Bevy 默认无序迭代）

---

## 战斗随机数（BattleRng）

> 🟩 对应宪法 11.9.1：多 RNG 流独立 — 战斗 RNG 为其中之一
> 🟩 对应宪法 11.9.2：统一 RNG 服务 — BattleRng 是战斗域的统一随机源

统一的确定性 PRNG 资源，所有战斗随机性必须从此获取。

不是系统随机。不是独立随机实例。

> **优化来源**: `docs/01-architecture/determinism_rules.md`

关键属性：
- 全局唯一 Resource，🟥 必须使用 `rand_chacha::ChaCha8Rng`（支持 `Send + Sync`、可 clone 用于并行迭代器、跨平台确定性有标准保证）
- 🟥 禁止使用 `xorshift64*` 或其他非标准 PRNG（无 Send+Sync 保证、跨平台不确定）
- 种子在战斗开始时从 LevelConfig 或 ReplaySeed 读取
- 存储在 World Resource 中，所有战斗系统从此获取随机数
- 包含 rng（ChaCha8Rng 内部状态）和 seed（初始种子，存储在回放文件中）

---

## 随机种子（Random Seed）

> **优化来源**: `docs/01-architecture/determinism_rules.md`

决定 PRNG 序列的初始化值，存储在回放文件中用于复现。

不是随机数。不是存档。

关键属性：
- 战斗开始时从 LevelConfig 或 ReplaySeed 读取 master_seed
- 初始化 BattleRng::from_seed(seed)
- 格式与回放文件兼容（只存储 seed，不存储实际随机值）
- 相同种子 + 相同输入序列 → 相同随机序列

### 派生种子策略（Derived Seed Strategy）

🟥 **所有种子必须从 master_seed 派生**，使用确定性的分层派生函数：

```
master_seed（关卡种子，存入回放文件）
  ├── derive("combat", turn_number)     → 回合级种子（每回合重置）
  ├── derive("unit", entity_logical_id) → 单位级种子（Buff 触发、被动判定）
  └── derive("action", action_index)    → 动作级种子（伤害浮动、暴击判定）
```

- 🟥 禁止在回放文件中存储实际随机值，只存储 seed
- 🟥 派生函数必须是确定性的：相同输入 → 相同派生种子

---

## 迭代排序（Iteration Ordering）

ECS 查询结果的显式排序，确保相同状态下产生相同处理结果。

不是 Bevy 默认无序迭代。不是算法排序。

关键属性：
- 行动队列：Initiative 降序 + LogicalId 升序稳定排序
- 同时触发的 Buff 结算：Buff 注册顺序（InsertionOrder）
- 同回合多个死亡判定：LogicalId 升序
- 属性 Modifier 栈计算：ModifierSource 优先级
- AOE 伤害目标遍历：LogicalId 升序

---

## 状态哈希（State Hash）

TurnEnd 时战斗全局状态的确定性摘要，用于检测执行分歧。

不是快照。不是存档。

关键属性：
- 每回合结束（TurnEnd）必须计算
- 包含：存活单位属性、活跃 Buff 列表、当前回合号、回合阶段
- 回放验证时与参考哈希对比
- 不一致时记录分歧位置和差异详情

---

## 整数精度（Integer Precision）

战斗核心数值只用整数，避免浮点精度差异导致的不确定性。

不是浮点数。不是定点数。

关键属性：
- HP/MP/伤害值/攻击力/防御力必须为 i32 或 u32
- 百分比用整数表示（15 = 15%）
- 公式使用先乘后除：value * multiplier / 100
- 地形防御加成、移动消耗均为整数

---

## 逻辑 ID（LogicalId）

战斗开始时分配的稳定逻辑 ID（u32），替代非确定性 Bevy Entity ID。

不是 Entity ID。不是存档 ID。

关键属性：
- 战斗开始时按加载顺序从 0 递增分配
- 存储在 Component 中，随存档/Replay 保存
- 所有排序（行动队列、死亡判定、AOE 遍历）使用 LogicalId
- 相同输入下 LogicalId 跨运行稳定，Entity ID 不稳定

---

## 随机流（RandomStream）

> **优化来源**: `docs/01-architecture/determinism_rules.md`

预生成的 `Vec<u32>` 随机数序列，解决 `ResMut<BattleRng>` 并行冲突。

不是全局 Rng。不是实时生成。

关键属性：
- 🟥 在 UnitSpawn 阶段（战斗前）从 LevelConfig 或 ReplaySeed 读取 master_seed，为每个 Unit 生成预计算随机流 Vec<u32>，存入 RandomStream Component
- 战斗 System 通过 `Res<RandomStream>`（只读锁）+ ActionIndex 获取随机数
- 实现完美并行，无 `ResMut<BattleRng>` 借用冲突
- 🟥 禁止在回合开始或 Command 生成时才预生成（必须在 UnitSpawn 时一次性完成）

---

## 多流 RNG 架构（Multi-Stream RNG）

> 🟥 对应宪法 11.9.1：必须按用途拆分独立随机数流：战斗 RNG、掉落 RNG、世界 RNG、AI RNG
> 🟥 对应宪法 11.9.2：业务逻辑禁止直接调用 rand::random()，必须通过统一的随机数服务获取

全局 RNG 服务（RngService）管理多个独立随机数流，每个流有独立种子和状态，互不干扰。

不是单一 RNG 实例。不是分散的随机调用。

关键属性：
- 🟥 禁止全局共用单一 RNG 实例（违反宪法 11.9.1）
- RngService 作为 World Resource 统一管理所有 RNG 流
- 每个 RNG 流在战斗开始时从 master_seed 派生独立种子
- 业务逻辑必须通过 RngService 获取随机数，禁止直接调用 rand::random()

### RNG 流分类

| 流名称 | 用途 | 种子来源 | 独立性 |
|--------|------|---------|--------|
| BattleRng | 战斗伤害浮动、暴击判定、Buff 触发 | master_seed 派生 | 与掉落/世界/AI 完全独立 |
| DropRng | 战斗掉落、战利品生成 | master_seed 派生 | 与战斗/世界/AI 完全独立 |
| WorldRng | 世界事件、随机遭遇、天气变化 | master_seed 派生 | 与战斗/掉落/AI 完全独立 |
| AiRng | AI 决策随机性（技能选择、走位偏好） | master_seed 派生 | 与战斗/掉落/世界 完全独立 |

### 种子派生规则

- 所有 RNG 流的种子必须从 master_seed 通过确定性派生函数生成
- 🟥 禁止使用系统时间或非确定性源初始化任何 RNG 流
- 派生函数：`derive_seed(master_seed, stream_name) -> stream_seed`
- 相同 master_seed + 相同 stream_name → 相同 stream_seed

### 确定性保证

- 相同 master_seed + 相同输入序列 → 所有 RNG 流产生完全相同的随机序列
- 战斗回放只需存储 master_seed，所有 RNG 流可从种子重建
- 🟥 禁止在回放文件中存储实际随机值，只存储 seed

---

## 网格坐标（GridCoord）

IVec2 逻辑坐标，战斗计算专用。

不是 Transform Vec3。不是渲染坐标。

关键属性：
- 战斗逻辑（移动、射程、AOE 范围）只使用 GridCoord
- Transform 仅用于表现层插值和 UI 显示
- 禁止使用 `Transform.translation.distance()` 计算射程
- 与渲染坐标通过 ViewModelSet 转换

---

# 领域边界

## 本领域负责

- 确定性 PRNG（BattleRng）的种子管理和随机数生成
- 战斗数值的整数精度约束
- ECS 查询结果的显式排序规则
- 系统执行顺序的 SystemSet 排序约束
- 状态哈希的计算和验证
- 回放验证中的分歧检测

## 本领域不负责

- 战斗数值的具体计算公式（由 Attribute Modifier 领域负责）
- 回放文件的存储格式（由 Replay 领域负责）
- 回合状态机（由 Turn 领域负责）
- Effect Pipeline 的内部执行（由 Attribute Modifier 领域负责）
- 战斗胜负条件检查（由 Battle 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| BattleRng 随机数 | Resource 访问 | 所有战斗系统 |
| state_hash | 函数调用 | Replay 领域 |
| 排序规则 | 函数调用 | Turn / Battle / Buff 领域 |
| 执行顺序约束 | SystemSet 定义 | 所有系统 |

---

# 生命周期

本领域无状态机，为纯函数式约束。

BattleRng 的生命周期：
- 战斗开始：从种子初始化 BattleRng
- 战斗中：所有系统从此 Resource 获取随机数
- 战斗结束：BattleRng 随 World 销毁

state_hash 的生命周期：
- 每回合结束（TurnEnd）计算一次
- 回放验证时与参考哈希对比

---

# 不变量

## 不变量1：相同种子 → 相同随机序列

任意时刻：

给定相同种子和输入序列，BattleRng 产生的随机数序列完全一致。

违反表现：

相同种子下暴击判定不同、伤害浮动不同、Buff 触发概率不同。

---

## 不变量2：所有战斗数值计算产生整数结果

任意时刻：

HP/伤害/攻击/防御等核心战斗数值的计算结果为整数，无浮点精度差异。

违反表现：

相同攻击力和防御力下，伤害值在不同运行中产生微小差异。

---

## 不变量3：相同 ECS 状态 → 相同查询结果

任意时刻：

相同 ECS 状态下，显式排序的查询产生完全相同的结果。

违反表现：

行动队列顺序不同、Buff 结算顺序不同、AOE 目标遍历顺序不同。

---

## 不变量4：每次 TurnEnd 的 state_hash 可复现

回合结算阶段：

每次 TurnEnd 计算的 state_hash 在相同状态下必须一致。

违反表现：

回放验证时 state_hash 不匹配，无法定位分歧位置。

---

## 不变量5：所有 ECS 查询排序使用 LogicalId

任意时刻：

所有战斗 ECS 查询排序使用 LogicalId，不使用 Entity ID。Entity ID 由 Bevy 内部计数器生成，跨运行不稳定。

违反表现：

不同运行中行动队列顺序不同、AOE 目标遍历顺序不同，导致状态分歧。

---

## 不变量6：BattleRng 的 ResMut 冲突通过 RandomStream 预生成解决

任意时刻：

战斗 System 通过 `Res<RandomStream>`（只读锁）获取随机数，禁止通过 `ResMut<BattleRng>` 实时获取。RandomStream 在 UnitSpawn 阶段（战斗前）预生成，与 `docs/01-architecture/determinism_rules.md` 保持一致。

违反表现：

多个 System 独占 BattleRng 写锁，战斗逻辑被迫串行化，丧失 ECS 多线程性能。

---

## 不变量7：战斗逻辑仅使用 GridCoord

任意时刻：

战斗逻辑（移动、射程、AOE 范围计算）仅使用 GridCoord（IVec2），禁止使用 Transform.translation 进行范围计算。

违反表现：

不同运行中 Transform 精度差异导致射程判定不同，触发条件不一致。

---

# 业务规则

## 规则1：确定性随机数管理

> 🟩 对应宪法 11.9.2：统一 RNG 服务

必须：
- 所有战斗随机通过单一 BattleRng Resource
- 种子在战斗开始时确定
- BattleRng 存储在 World Resource 中

禁止：
- 🟥 使用 rand::thread_rng()（违反宪法 11.9.2）
- 🟥 使用 rand::rngs::SmallRng::from_entropy()（违反宪法 11.9.2）
- 🟥 在 System 内创建独立 Rng 实例（违反宪法 11.9.2）

允许：
- 种子从回放文件或关卡配置读取

---

## 规则2：整数精度约束

必须：
- HP/伤害/攻击/防御使用 i32 或 u32
- 百分比用整数表示（15 = 15%）
- 公式使用先乘后除：value * multiplier / 100

禁止：
- 核心战斗数值使用 f32/f64
- 浮点运算用于伤害/治疗计算
- 向上取整用于战斗数值（会膨胀伤害）

允许：
- 世界坐标计算使用浮点数（渲染层）
- 动画插值使用浮点数（视觉平滑）
- UI 布局使用浮点数（像素对齐）
- 强制舍入仅限 UI 显示

---

## 规则3：迭代排序

必须：
- 行动队列按 Initiative 降序 + LogicalId 升序稳定排序
- 同时触发的 Buff 按注册顺序结算
- 同回合多个死亡判定按 LogicalId 升序
- AOE 伤害目标按 LogicalId 升序

禁止：
- 依赖 Bevy 默认迭代顺序
- 同一 Set 内系统依赖隐式执行顺序
- 系统间存在循环 Set 依赖
- 使用 Entity ID 进行任何排序（Entity ID 由 Bevy 内部计数器生成，跨运行不稳定）

允许：
- 非战斗逻辑的 ECS 查询无强制排序要求

---

## 规则4：系统执行顺序

必须：
- InputSet → CommandSet → LogicSet → EffectSet → ViewModelSet → UISet
- 管线三步顺序：effect_generate → effect_modify → effect_execute
- turn_end_cleanup → victory_check
- 所有逻辑系统 → view_model_update

禁止：
- 同一 Set 内的系统依赖隐式执行顺序
- 使用 before()/after() 但不声明在 Set 定义中
- 系统间存在循环依赖（A after B 且 B after A）

允许：
- 在 SystemSet 定义中声明明确的执行顺序

---

## 规则5：状态哈希

必须：
- 每回合结束（TurnEnd）计算 state_hash
- 哈希内容包含：存活单位属性、活跃 Buff、回合号、回合阶段
- 回放验证时与参考哈希对比

禁止：
- 跳过 state_hash 计算
- 哈希内容遗漏影响战斗结果的状态
- 使用 println!/dbg! 影响执行路径（输出是副作用）
- 依赖 query.iter() 默认迭代顺序进行哈希（必须排序后再哈希）

允许：
- 每次伤害结算后可选计算 state_hash（战斗级校验）

---

## 规则6：整数溢出安全

必须：
- 核心公式计算时中间变量提升为 i64：`(attack as i64 * 100 * crit_mult as i64) / defense as i64).max(1) as i32`
- 使用 checked_mul / saturating_mul 保证数值边界安全
- 除零防御：defense 为 0 时的兜底处理（`.max(1)`）

禁止：
- 直接使用 i32 进行高倍率乘法运算（如 `attack * 100 * crit_mult`）
- 依赖 Debug 模式下整数溢出的 panic 行为作为安全机制

允许：
- 在确定数值范围安全的前提下使用 i32 运算
- 使用 i64 中间提升后截断回 i32

---

## 规则7：显式排序规则

必须：
- 行动队列：Initiative 降序 + LogicalId 升序稳定排序
- AOE 目标：LogicalId 升序
- 同回合多个死亡判定：LogicalId 升序
- Buff 结算：按注册顺序（InsertionOrder）
- Modifier 栈计算：ModifierSource 优先级

禁止：
- 依赖 Bevy 默认迭代顺序
- 同一 Set 内系统依赖隐式执行顺序
- 使用 Entity ID 进行任何排序

允许：
- 非战斗逻辑的 ECS 查询无强制排序要求

---

## 规则8：寻路 A* 确定性

必须：
- A* 等代价路径引入确定性 Tie-breaker：优先 X 轴或优先 Y 轴
- 路径选择结果跨运行一致

禁止：
- 等代价路径随机选择
- 依赖物理引擎 Broad-phase 返回顺序

允许：
- Tie-breaker 方向在项目内统一规定（全 X 优先或全 Y 优先）

---

## 规则9：状态哈希排序后哈希

必须：
- 计算 state_hash 前，将查询结果收集到 Vec 中并按 LogicalId 排序
- 排序后再遍历写入 Hasher
- 使用派生宏（如 `#[derive(DeterministicHash)]`）自动为战斗 Component 生成哈希代码

禁止：
- 依赖 query.iter() 默认迭代顺序进行哈希
- 人工维护哈希字段列表（使用宏防止遗漏）

允许：
- 每个版本迭代时验证哈希字段完整性

---

## 规则10：LogicSet 编译期浮点防护

必须：
- LogicSet 内的战斗逻辑系统标注 `#[deny(clippy::float_arithmetic)]`
- 编译期静态扫描战斗逻辑代码中的 f32/f64 使用

禁止：
- f32/f64 出现在 LogicSet 中的战斗数值计算
- ViewModelSet 读取逻辑层数据后的浮点计算结果回写到逻辑层 Component

允许：
- ViewModelSet 中将整数转换为浮点数进行 UI 渲染

---

## 规则11：禁止时间戳排序

> 🟥 对应宪法 18.4.2：禁止业务逻辑依赖系统时间，必须使用 GameTime 服务
> **优化来源**: `docs/01-architecture/determinism_rules.md`

必须：
- 事件排序使用 tick 计数器（逻辑帧号）或 EventOrd(u64) 显式排序键
- Tick 内多事件按 EventOrd 升序排列

禁止：
- 使用 `std::time::SystemTime` / `Instant` 排序事件
- 使用 `Instant::now()` 作为事件排序依据
- 依赖操作系统时钟的任何排序逻辑

原因：操作系统时钟不确定（NTP 校准、闰秒、虚拟机时钟漂移），用时钟排序会导致跨运行结果不一致。

允许：
- 使用 tick 计数器（逻辑帧号）排序
- 使用 EventOrd(u64) 由发送方显式指定排序键

---

# 流程管线

> 🟩 对应宪法 18.4.3：所有战斗相关 Bug 必须通过 Battle Replay 重现并转化为永久测试用例
> 🟥 对应宪法 18.4.2：禁止业务逻辑依赖系统时间，必须使用 GameTime 服务

## 确定性保证链（Determinism Guarantee Chain）

```
单一种子 → 确定性 PRNG → 所有随机可复现
整数运算 → 精度可控 → 数值结果一致
显式排序 → 迭代顺序确定 → 处理结果一致
状态哈希 → 分歧可检测 → 问题可定位
```

### Step1：种子初始化

输入：LevelConfig 或 ReplaySeed 中的种子值
处理：BattleRng::from_seed(seed)，插入 World Resource
输出：BattleRng Resource 就绪
禁止：使用非确定性种子（如系统时间）

### Step2：随机数获取

输入：BattleRng Resource
处理：所有战斗系统通过 BattleRng 获取随机数
输出：确定性随机序列
禁止：使用 rand::thread_rng() 或创建独立 Rng

### Step3：整数运算

输入：战斗数值（整数）
处理：先乘后除的公式计算
输出：整数结果
禁止：使用浮点数计算战斗数值

### Step4：显式排序

输入：ECS 查询结果
处理：按排序键稳定排序
输出：确定顺序的结果列表
禁止：依赖默认迭代顺序

### Step5：状态哈希

输入：回合结束时的全局状态
处理：计算确定性哈希
输出：state_hash 值
禁止：跳过哈希计算

---

# 数据结构

## BattleRng（战斗随机数）

> **优化来源**: `docs/01-architecture/determinism_rules.md`

职责：统一的确定性 PRNG 资源

结构：
- rng：ChaCha8Rng — 内部状态（🟥 必须使用 `rand_chacha::ChaCha8Rng`，支持 Send + Sync + 可 clone）
- seed：u64 — 初始种子（存储在回放文件中）

要求：
- 全局唯一 Resource
- 种子在战斗开始时确定
- 所有战斗系统从此获取随机数
- 禁止使用 rand::thread_rng()
- 禁止使用 xorshift64* 或其他非标准 PRNG
- ChaCha8Rng 支持 clone，可安全用于并行迭代器

---

## StateHash（状态哈希）

职责：回合结束时战斗状态的确定性摘要

结构：
- 哈希值：u64
- 包含内容：存活单位属性、活跃 Buff、回合号、回合阶段

要求：
- 每回合结束（TurnEnd）必须计算
- 回放验证时与参考哈希对比
- 不一致时记录分歧位置

---

## IterationSortKey（迭代排序键）

职责：定义 ECS 查询结果的排序依据

结构：
- Initiative（行动力）：降序（高先行动）
- LogicalId（u32）：升序（稳定排序）
- InsertionOrder（Buff 注册顺序）：升序

要求：
- 行动队列：Initiative 降序 + LogicalId 升序
- Buff 结算：InsertionOrder 升序
- 死亡判定：LogicalId 升序
- AOE 目标：LogicalId 升序

---

## SystemSetOrder（系统执行顺序）

职责：定义系统组的执行顺序约束

结构：
- InputSet → CommandSet → LogicSet → EffectSet → ViewModelSet → UISet

要求：
- 同一 Set 内无隐式执行顺序依赖
- 系统间无循环依赖
- 管线三步严格顺序：generate → modify → execute

---

# 禁止事项

禁止：使用 rand::thread_rng()

原因：系统随机不可复现，破坏确定性保证。

违反后果：回放失败，测试不可靠，Bug 无法稳定复现。

---

禁止：核心战斗数值使用 f32/f64

原因：浮点精度不可控，不同平台/编译器可能产生微小差异。

违反后果：伤害值在不同运行中不一致，回放验证失败。

---

禁止：依赖 Bevy 默认迭代顺序

原因：Bevy ECS 查询默认不保证迭代顺序，导致处理结果不确定。

违反后果：行动队列顺序不同、Buff 结算顺序不同、伤害计算结果不一致。

---

禁止：同一 Set 内系统依赖隐式执行顺序

原因：隐式顺序依赖导致竞态条件，系统执行结果不确定。

违反后果：系统间数据竞争，游戏状态不一致。

---

禁止：系统间存在循环 Set 依赖

原因：循环依赖导致调度器死锁。

违反后果：游戏卡死，无法继续执行。

---

禁止：跳过状态哈希计算

原因：分歧不可检测，Bug 隐藏在确定性执行中。

违反后果：回放验证无法发现问题，Bug 长期隐藏。

---

禁止：使用 println!/dbg! 影响执行路径

原因：输出是副作用，可能影响执行顺序或引入非确定性。

违反后果：执行路径不确定，回放结果不一致。

---

禁止：在 System 内创建独立 Rng 实例

原因：独立 Rng 破坏全局状态一致性，随机序列不可控。

违反后果：随机数序列不可复现，战斗结果不一致。

---

禁止：使用 Entity ID 进行任何排序

原因：Bevy Entity ID 由内部计数器生成，跨运行不稳定。不同加载时序导致 Entity ID 不同，排序结果不一致。

违反后果：行动队列顺序不同、AOE 目标遍历顺序不同，导致状态分歧。

---

禁止：Transform.translation 用于距离计算

原因：Transform 坐标受渲染精度、插值、浮点误差影响，战斗逻辑中使用会导致射程判定不一致。

违反后果：不同运行中射程判定不同，触发条件不一致。

---

禁止：f32/f64 出现在 LogicSet

原因：LogicSet 是战斗逻辑系统组，浮点运算会引入跨平台/跨编译器的精度差异。

违反后果：伤害值在不同运行中不一致，回放验证失败。

---

# AI 修改规则

## 如果新增战斗随机性需求

允许：
- 通过 BattleRng Resource 获取随机数
- 使用 BattleRng 的 gen_range 方法
- 定义新的随机判定（如暴击率、闪避率）

禁止：
- 使用 rand::thread_rng()
- 在 System 内创建独立 Rng
- 使用非确定性种子

优先检查：
- 随机数是否从 BattleRng 获取
- 种子是否在战斗开始时确定
- 随机判定是否影响战斗结果（需要 state_hash 覆盖）

---

## 如果修改数值计算公式

允许：
- 修改 Derived Stat 的计算函数
- 添加新的 Derived Stat
- 调整百分比修饰的整数表示

禁止：
- 使用浮点数计算战斗数值
- 修改先乘后除的运算顺序
- 使用向上取整（会膨胀伤害）

优先检查：
- 计算结果是否为整数
- 公式是否使用先乘后除
- 所有引用该属性的系统是否兼容新公式
- state_hash 是否覆盖新增的计算结果

---

## 如果新增 ECS 查询排序

允许：
- 在 Query 结果上添加显式排序
- 定义新的排序键（Entity ID、Initiative 等）
- 使用稳定排序

禁止：
- 依赖 Bevy 默认迭代顺序
- 排序键不稳定（如使用 hash）
- 同一查询使用不同的排序键

优先检查：
- 排序键是否确定性（Entity ID、Initiative 等）
- 排序方向是否一致（降序/升序）
- 所有使用该查询的系统是否兼容新排序

---

## 如果测试失败

排查顺序：
1. 检查 BattleRng 种子是否一致（不同种子导致不同随机序列）
2. 检查随机数是否从 BattleRng 获取（非系统随机）
3. 检查数值计算是否使用整数（非浮点）
4. 检查 ECS 查询是否有显式排序（非默认迭代）
5. 检查系统执行顺序是否正确（SystemSet 约束）
6. 检查 state_hash 是否覆盖所有影响战斗结果的状态
