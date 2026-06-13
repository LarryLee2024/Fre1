# 确定性执行领域

Version: 1.0
Status: Proposed

确定性执行领域管理 SRPG 战斗的物理定律：相同输入 + 相同种子 → 相同结果。

核心原则：
- 单一确定性 PRNG 管理所有战斗随机性
- 战斗核心数值只用整数，禁止浮点
- ECS 查询结果必须显式排序，禁止依赖默认迭代顺序

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

统一的确定性 PRNG 资源，所有战斗随机性必须从此获取。

不是系统随机。不是独立随机实例。

关键属性：
- 全局唯一 Resource，使用确定性算法（如 xorshift64*）
- 种子在战斗开始时从 LevelConfig 或 ReplaySeed 读取
- 存储在 World Resource 中，所有战斗系统从此获取随机数
- 包含 state（内部状态）和 seed（初始种子）

---

## 随机种子（Random Seed）

决定 PRNG 序列的初始化值，存储在回放文件中用于复现。

不是随机数。不是存档。

关键属性：
- 战斗开始时从 LevelConfig 或 ReplaySeed 读取
- 初始化 BattleRng::from_seed(seed)
- 格式与回放文件兼容
- 相同种子 + 相同输入序列 → 相同随机序列

---

## 迭代排序（Iteration Ordering）

ECS 查询结果的显式排序，确保相同状态下产生相同处理结果。

不是 Bevy 默认无序迭代。不是算法排序。

关键属性：
- 行动队列：Initiative 降序 + Entity ID 升序稳定排序
- 同时触发的 Buff 结算：Buff 注册顺序（InsertionOrder）
- 同回合多个死亡判定：Entity ID 升序
- 属性 Modifier 栈计算：ModifierSource 优先级
- AOE 伤害目标遍历：Entity ID 升序

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

# 业务规则

## 规则1：确定性随机数管理

必须：
- 所有战斗随机通过单一 BattleRng Resource
- 种子在战斗开始时确定
- BattleRng 存储在 World Resource 中

禁止：
- 使用 rand::thread_rng()
- 使用 rand::rngs::SmallRng::from_entropy()
- 在 System 内创建独立 Rng 实例

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
- 行动队列按 Initiative 降序 + Entity ID 升序稳定排序
- 同时触发的 Buff 按注册顺序结算
- 同回合多个死亡判定按 Entity ID 升序
- AOE 伤害目标按 Entity ID 升序

禁止：
- 依赖 Bevy 默认迭代顺序
- 同一 Set 内系统依赖隐式执行顺序
- 系统间存在循环 Set 依赖

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

允许：
- 每次伤害结算后可选计算 state_hash（战斗级校验）

---

# 流程管线

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

职责：统一的确定性 PRNG 资源

结构：
- state：u64 — 内部状态（使用确定性算法）
- seed：u64 — 初始种子（存储在回放文件中）

要求：
- 全局唯一 Resource
- 种子在战斗开始时确定
- 所有战斗系统从此获取随机数
- 禁止使用 rand::thread_rng()

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
- Entity ID：升序（稳定排序）
- InsertionOrder（Buff 注册顺序）：升序

要求：
- 行动队列：Initiative 降序 + Entity ID 升序
- Buff 结算：InsertionOrder 升序
- 死亡判定：Entity ID 升序
- AOE 目标：Entity ID 升序

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
