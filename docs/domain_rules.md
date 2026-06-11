# Domain Rules

Version: 1.0

SRPG 业务规则圣经

修改任何业务逻辑前，必须先检查是否违反本文件。

---

# Character

Character =

Race
Job
Stats
Equipment
Traits
Buffs

允许：

通过 Trait + Modifier 组合能力
通过 UnitTemplate 生成运行时实例
通过 RegistryLoader 加载配置

禁止：

把 Entity 当对象使用
直接修改 UnitTemplate
在运行时创建新的 Definition

---

# Race

Race：

永久属性

允许：

提供 Trait + Modifier 集合

禁止：

直接实现逻辑
硬编码种族效果

---

# Job

Job：

成长率 + 技能池 + Trait 集合

升级：

修改成长

禁止：

修改 Definition
硬编码职业效果

---

# Attributes

三层架构：

Core Stat（8维）
↓
Derived Stat（战斗组 + 辅助组）
↓
Vital Resource（HP / MP / Stamina）

允许：

Derived Stat 实时计算
通过统一 Modifier 管线修改属性
属性公式集中管理

禁止：

直接修改最终属性
到处硬编码属性计算
Derived Stat 缓存无失效条件

必须：

所有属性修改走 Modifier 管线
最终属性来源统一

---

# Modifier

Modifier =

ModifierType（Flat / Percent / Override）
+
ModifierSource（Race / Job / Equipment / Buff / Trait）

允许：

叠加多个 Modifier
按优先级排序

禁止：

绕过 Modifier 管线直接改属性

必须：

Modifier 有 Source 标记
Modifier 有过期条件

---

# Equipment

装备：

本质 = Modifier + Trait

穿脱流程：

检查需求 → 穿戴 → 应用效果 → 重建 Trait

允许：

通过 EquipmentDef 配置装备
通过 EquipmentSlot 管理槽位

禁止：

直接修改角色基础属性
跳过需求检查
跳过 Trait 重建

必须：

穿脱后重建 TraitCollection
记录装备实例 ID

---

# Buff

Buff：

临时 Trait

持续时间 = Turn

必须记录：

Stack 数
Source 来源
Remaining Turns

允许：

Stack 叠加（有上限）
Tick 结算（每回合）

禁止：

Buff 永不过期
Buff 无来源

必须：

回合结束检查过期
过期自动移除 + 重建 Trait

---

# Trait

Trait：

统一能力扩展机制

所有能力来源统一进入 Trait 管线：

Race → Trait
Job → Trait
Talent → Trait
Equipment → Trait
Buff → Trait

允许：

TraitTrigger 触发效果
TraitEffectHandler 分发效果

禁止：

为每种能力来源写独立逻辑
硬编码 Trait 效果

必须：

Trait 变化时重建 TraitCollection
跨模块 Trait 效果走 Message

---

# Skill

Skill =

SkillDef（配置）
+
SkillCooldowns（运行时）

允许：

数据驱动配置技能
技能效果预览

禁止：

硬编码技能效果
跳过冷却检查

必须：

冷却每回合递减
技能范围由 effective_skill_range 计算

---

# Effect Pipeline

三步管线：

Generate（生成效果）
↓
Modify（修饰规则）
↓
Execute（执行效果）

允许：

ModifierRule 标签匹配修饰
ModifierEntry 记录每步修饰

禁止：

跳过管线直接执行
跳过 Modify 阶段

必须：

伤害下限 ≥ 1
治疗下限 ≥ 0
所有修饰记录写入 BattleRecord

---

# Damage Pipeline

伤害顺序：

Hit Check
↓
Dodge / Block
↓
Critical
↓
Shield Absorb
↓
Resistance
↓
ModifierRule
↓
Final Damage
↓
HP Change

禁止：

跳过流程
绕过管线直接扣 HP

---

# Death

HP ≤ 0

不等于死亡。

必须：

添加 Dead Tag Component
由 Hook 处理固有行为（移除移动组件等）
由 Observer 处理局部响应（播放动画、刷新 UI）
由 Message 广播跨模块通知（CharacterDied）

禁止：

直接删除 Entity
跳过 Dead Tag 直接处理死亡逻辑
在 HP 变化时内联死亡处理

---

# Inventory

背包：

Container = Slot + Stack + Weight

允许：

ItemDef / ItemInstance 分离
Container 间转移物品

禁止：

直接修改 Stack 数量绕过转移逻辑
Stack.count = 0 时不清理

必须：

转移校验容量
空 Stack 自动清理
ItemDef 引用校验

---

# Encumbrance

总重量超过限制：

移动力下降

禁止：

直接禁止行动

---

# Map

地图数据：

TerrainGrid = 地形唯一真相源
OccupancyGrid = 单位占用独立存在

允许：

Tile 按需设为 Entity
Chunk 按需引入

禁止：

地图逻辑依赖渲染层
寻路数据硬编码

必须：

地图数据与渲染分离
寻路数据运行时生成
OccupancyGrid 与 TerrainGrid 独立更新

---

# Pathfinding

BFS 寻路

允许：

TerrainCostCalculator trait 扩展
标签解析计算器（SWIMMING > FLYING > MOUNTED > ground）

禁止：

硬编码移动成本

必须：

find_reachable_tiles 返回可达范围
reconstruct_path 返回路径

---

# Turn

回合流程：

AppState（MainMenu / InGame / GameOver）
↓
TurnPhase（SubState，仅 InGame 激活）
↓
TurnOrder（Initiative 降序行动队列）

允许：

队列耗尽自动进入 TurnEnd
ForceEndTurn 强制结束

禁止：

状态机处理业务细节
OnEnter / OnExit 包含重逻辑

必须：

回合结束重置所有单位 acted
NeedsResolve 防止重复结算
TurnStarted / TurnEnded Message 广播

---

# AI

AI 决策：

AiBehavior（数据驱动配置）
+
AiStrategyRegistry（trait 分发）

允许：

新增策略只需实现 trait 并注册
RON 配置不同行为模式

禁止：

硬编码 AI 逻辑
enum + match 分发策略

必须：

strategy_name 与 RON 配置对应
未知策略回退默认
AI 和玩家共用 Effect Pipeline
CombatIntent 是唯一攻击意图通道

---

# ModifierRule

修饰规则：

标签匹配 + Calculator trait 分发

匹配条件：

source_tag（攻击方技能标签）
AND
target_tag（目标标签集合）

允许：

多规则链式叠加
自定义 Calculator 注册

禁止：

match 分发效果类型
绕过 Calculator 直接计算

必须：

伤害 ≥ 1
治疗 ≥ 0
每步修饰记录 ModifierEntry

---

# UI

UI =

ViewModel + UiCommand + UiTheme

允许：

UI 只读 ViewModel
UI 发出 UiCommand Message
主题统一样式

禁止：

UI 直接操作 ECS
UI 保存业务真相
业务逻辑直接操作 UI
UI 绕过 ViewModel 直接 Query 游戏组件

必须：

UI → Logic 只走 UiCommand
Logic → UI 只走 ViewModel
模态面板标记 BlocksGameInput
ViewModel 变化时刷新（非每帧轮询）

---

# Save

存档：

保存 Instance

禁止：

保存 Definition

必须：

Instance 引用 Definition ID
加载时从 Registry 恢复 Definition

---

# Battle Record

战斗记录：

结构化记录所有战斗事件

允许：

用于 Replay / 调试 / AI 分析

必须：

记录每步修饰详情（ModifierEntry）
记录伤害来源和目标
记录技能 ID 和效果类型

---

# 跨模块通信

Hook = 固有行为（组件添加/移除时）
Observer = 局部响应（状态变化时）
Message = 跨 Feature 广播

允许：

模块内部优先函数调用

禁止：

所有逻辑都事件化
跨模块访问内部细节
高频逻辑走 Observer 风暴

必须：

跨模块通过 Message / Observer / Command 通信
模块只暴露公共接口

---

# 数据驱动

配置定义内容，代码解释配置。

允许：

新增内容修改 RON 配置
配置热重载

禁止：

新增内容修改逻辑代码
配置结构频繁变更

必须：

配置引用关系自动校验
配置兼容性优先于配置优雅
配置型数据尽量不可变

---

# 停止条件

发现以下情况，必须停止并报告：

1. 属性修改绕过 Modifier 管线
2. 死亡处理跳过 Dead Tag
3. 效果执行跳过 Pipeline
4. UI 直接修改游戏状态
5. 跨模块直接访问内部实现
6. 新增硬编码替代数据驱动
7. Definition 被运行时修改
8. Buff 无来源或永不过期
9. 穿脱装备跳过 Trait 重建
10. 存档保存 Definition 数据
