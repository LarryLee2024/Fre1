# Architecture

Version: 1.0

本文件优先级高于任何代码实现。

当代码与本文件冲突时：
优先认为代码错误，而不是文档错误。

---

# 项目目标

项目类型：

SRPG（战棋RPG）

技术栈：

- Rust
- Bevy 0.18+
- ECS
- Data Driven

核心原则（优先级从高到低）：

1. Feature First
2. Definition / Instance 分离
3. Rule / Content 分离
4. Logic / Presentation 分离
5. Hook = 固有行为，Observer = 局部响应，Message = 跨Feature广播
6. Trait + Modifier 统一扩展体系
7. 数据驱动优于硬编码
8. 小函数、小模块、小依赖
9. Battle Replay + 测试优先于手工验证
10. 组合优于继承

---

# Feature 划分

允许：

```
core/
character/
battle/
buff/
skill/
equipment/
inventory/
map/
turn/
ai/
ui/
debug/
```

禁止：

```
components.rs
systems.rs
events.rs
utils.rs
```

作为顶层业务模块。

发现以上文件作为业务模块时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 发现技术分层模块 [文件名]，违反 Feature First 原则。
```

---

# 模块边界

## Battle

负责：

- 战斗效果管线（generate → modify → execute）
- 伤害计算与修饰
- 战斗记录（BattleRecord + DamageBreakdown）
- 回合内行动执行

禁止：

- 修改背包
- 修改任务
- 修改UI
- 直接修改角色属性

必须通过：

```
DamageApplied Message
HealApplied Message
CharacterDied Message
```

通知其他模块。

## Character

负责：

- 单位生成（spawn from template）
- 单位组件（Unit, Faction, GridPosition, Dead, UnitName, UnitId）
- Trait 扩展体系（TraitCollection, TraitTrigger, TraitEffect）
- 移动动画

禁止：

- 直接计算伤害
- 直接操作 Buff

必须通过：

```
TraitTrigger::OnAttack → battle/pipeline 触发
TraitEffect::ApplyBuff → buff/apply 执行
```

## Buff

负责：

- Buff 定义与注册（BuffData, BuffDef, BuffRegistry）
- Buff 实例管理（BuffInstance, ActiveBuffs）
- Buff 穿戴/移除（修改 Attributes + Tags）
- 持续效果结算（DoT/HoT/晕眩/tick）

禁止：

- 直接扣血
- 直接修改 UI

必须通过：

```
DotApplied Message → battle/record 记录
HotApplied Message → battle/record 记录
StunApplied Message → battle/record 记录
```

## Skill

负责：

- 技能定义与注册（SkillData, SkillDef, SkillRegistry）
- 技能槽位与冷却（SkillSlots, SkillCooldowns）
- 技能效果预览

禁止：

- 直接执行伤害
- 直接施加 Buff

必须通过：

```
CombatIntent → battle/pipeline 执行
EffectDef → EffectHandler 分发
```

## Equipment

负责：

- 装备定义与注册（EquipmentDef, EquipmentRegistry）
- 装备实例与槽位（EquipmentInstance, EquipmentSlots）
- 穿脱逻辑
- 装备需求检查

禁止：

- 直接修改角色属性

必须通过：

```
ModifierSource::Equipment → Attribute Modifier Stack
TraitSource::Equipment → TraitCollection
EquipItem / UnequipItem Message
ItemEquipped / ItemUnequipped Message
```

## Inventory

负责：

- 物品定义与注册（ItemDef, ItemRegistry）
- 物品实例与堆叠（ItemInstance, ItemStack）
- 容器管理（Container）
- 战斗背包（BattleBag）
- 物品转移与使用

禁止：

- 直接修改角色属性
- 直接施加 Buff

必须通过：

```
UseItem Message → inventory/use_item
TransferItem Message → inventory/transfer
ItemUsed / ItemTransferred Message → 通知
```

## Map

负责：

- 地形数据（TerrainGrid）
- 单位占位（OccupancyGrid）
- 寻路（BFS + 地形消耗）
- 坐标转换（GameMap）

禁止：

- 直接移动角色
- 直接修改角色属性

必须通过：

```
GridPosition Component → character/movement 执行
OccupancyGrid → 查询占位状态
```

## Turn

负责：

- 主状态（AppState）
- 回合阶段（TurnPhase SubState）
- 行动队列（TurnOrder）
- 回合 Message

禁止：

- 执行战斗逻辑
- 修改角色状态

必须通过：

```
NextState<TurnPhase> → 驱动阶段转换
TurnStarted / TurnEnded Message → 通知
ForceEndTurn Message → 强制结束
```

## AI

负责：

- AI 行为定义与注册（AiBehavior, AiBehaviorRegistry）
- 策略选择（TargetSelector, MoveSelector, SkillSelector）
- AI 决策系统

禁止：

- 独立执行攻击逻辑
- 独立计算伤害

必须通过：

```
CombatIntent → 进入统一 Effect Pipeline
AiStrategyRegistry → trait 对象分发
```

发现 AI 模块包含独立伤害计算时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: AI 模块包含独立伤害计算，违反"AI 与玩家共享 Effect Pipeline"原则。
```

## UI

负责：

- 用户输入处理（UiCommand）
- 命令分发（command_handler）
- ViewModel 层
- 面板与组件展示

禁止：

- 保存业务真相
- 直接操作 ECS 组件修改业务状态
- 直接查询 ECS World 获取业务数据

必须通过：

```
UiCommand Message → command_handler → 游戏状态变更
ViewModel Resource → 只读展示
```

发现 UI 代码直接修改 Attributes / ActiveBuffs / EquipmentSlots 时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: UI 直接修改业务状态，违反 Logic/Presentation 分离原则。
```

## Core

负责：

- 属性系统（Attributes, AttributeKind, ModifierSource）
- 标签系统（GameplayTag 位掩码）
- 效果管线（EffectDef, PendingEffect, EffectHandler）
- 修饰规则（ModifierRuleRegistry）
- 注册表加载（RegistryLoader）
- 场景快照（Snapshot）

禁止：

- 依赖任何业务模块
- 包含业务逻辑

发现 core 模块 use 了 character/battle/buff 等业务模块时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: core 模块依赖业务模块，违反"核心层无外部依赖"原则。
```

## Debug

负责：

- 调试面板（bevy_egui）
- DebugPanelState 管理
- 可观测性（BattleRecord, DamageBreakdown）

禁止：

- 影响生产逻辑
- 修改业务状态

---

# 插件注册顺序

必须按以下顺序注册：

```
1. 核心层：EffectPlugin, ModifierRulePlugin, AttributeDefPlugin, TagDefPlugin
2. 数据层：SkillPlugin, BuffPlugin, AiBehaviorPlugin, EquipmentPlugin, InventoryPlugin
3. 逻辑层：AssetsPlugin, TurnPlugin, MapPlugin, CharacterPlugin, BattlePlugin, AiPlugin
4. 表现层：UiPlugin, InputPlugin, DebugPlugin
```

禁止：

- 表现层插件在数据层之前注册
- 逻辑层插件在核心层之前注册

发现注册顺序错误时：

必须输出：

```
ARCHITECTURE VIOLATION: 插件注册顺序错误，[插件名] 不应在 [层级] 之前注册。
```

---

# Definition / Instance 分离

Definition：

不可变配置

例如：

```
BuffData / BuffDef
SkillData / SkillDef
EquipmentDef
ItemDef
UnitTemplate
AiBehavior
TerrainDef
```

Instance：

运行时状态

例如：

```
BuffInstance / ActiveBuffs
SkillSlots / SkillCooldowns
EquipmentInstance / EquipmentSlots
ItemInstance / ItemStack
Unit + Attributes + ActiveBuffs
```

禁止：

- 修改 Definition 中的任何字段
- 在 Instance 中硬编码配置数据

发现 `BuffData.xxx = ...` 或 `SkillData.xxx = ...` 赋值时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 运行时修改 Definition 数据，违反 Definition/Instance 分离原则。
```

---

# Rule / Content 分离

代码：

负责规则

配置：

负责内容

新增职业：

允许：

- 新增 RON 配置文件

禁止：

- 修改伤害计算代码

新增技能：

允许：

- 新增 RON 配置文件

禁止：

- 修改 Effect Pipeline 代码

新增装备：

允许：

- 新增 RON 配置文件

禁止：

- 修改 Modifier 规则代码

发现为了新增内容而修改核心规则代码时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 新增内容修改了规则代码 [文件名]，违反 Rule/Content 分离原则。
应通过 RON 配置实现，而非修改代码。
```

---

# Logic / Presentation 分离

Logic：

- 伤害计算
- Buff 施加与结算
- 属性修饰
- 回合管理
- AI 决策

Presentation：

- 动画
- 音效
- UI 面板
- 战斗飘字
- 调试面板

禁止：

- `apply_damage()` 播放动画
- `add_buff()` 刷新 UI
- `execute_effects()` 播放音效

必须通过：

```
DamageApplied Message → combat_vfx_handler 播放飘字
CharacterDied Message → Observer 播放死亡动画
BuffApplied Message → UI 刷新 Buff 列表
```

发现业务函数包含 UI/动画/音效调用时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 业务逻辑 [函数名] 包含表现层调用，违反 Logic/Presentation 分离原则。
```

---

# ECS

## Entity

Entity 仅为 ID。

禁止：

- EntityManager OOP 风格封装
- 在 Entity 上存储行为方法
- 把 Entity 当对象使用

## Component

Component 存数据。

禁止：

- Component 包含复杂业务逻辑
- Component impl 包含超过 3 个方法
- Component 方法修改其他 Component

## System

System 存行为。

## Hook

用于：

组件固有行为

例如：

```
Dead 添加后 → 自动移除 MoveTarget，标记已行动
```

## Observer

用于：

外部响应

例如：

```
死亡动画
任务更新
UI 刷新
```

## Message

用于：

跨 Feature 通信

例如：

```
DamageApplied
CharacterDied
EquipItem
TurnEnded
UiCommand
```

当前 Message 注册表：

| Message | 发送方 | 接收方 |
|---------|--------|--------|
| UiCommand | input | command_handler |
| DamageApplied | battle/execute | ui/combat_vfx, ui/combat_log, battle/record |
| HealApplied | battle/execute | ui/combat_log, battle/record |
| CharacterDied | battle/execute | battle/events, ui/combat_log, battle/record |
| StunApplied | buff/resolve | ui/combat_log, battle/record |
| DotApplied | buff/resolve | ui/combat_log, battle/record |
| HotApplied | buff/resolve | ui/combat_log, battle/record |
| EquipItem | ui | equipment/equip |
| UnequipItem | ui | equipment/equip |
| ItemEquipped | equipment/equip | ui/combat_log |
| ItemUnequipped | equipment/equip | ui/combat_log |
| UseItem | ui | inventory/use_item |
| TransferItem | ui | inventory/transfer |
| TurnStarted | turn | battle/record |
| TurnEnded | turn | battle/record |
| ForceEndTurn | ui/command_handler | turn |

## Required Components

用于：

声明依赖

禁止：

- spawn 后手动补组件

必须通过：

```
#[require(Attributes, ActiveBuffs, SkillSlots, EquipmentSlots)]
struct Unit;
```

## Tag Component

用于：

标记状态

允许：

```
Dead
Selected
Acted
```

禁止：

```
is_dead: bool
is_selected: bool
has_acted: bool
```

发现 `is_xxx: bool` 字段时：

必须输出：

```
ARCHITECTURE WARNING: 发现 bool 字段 [字段名]，建议使用 Tag Component 替代。
```

## Resource

用于：

全局只读状态

允许：

```
SkillRegistry
BuffRegistry
EquipmentRegistry
TurnOrder
GameMap
TerrainGrid
OccupancyGrid
```

禁止：

- Resource 作为全局变量仓库
- 在 Resource 中存储可变业务状态

---

# Effect Pipeline

战斗效果必须走三步管线：

```
generate → modify → execute
```

禁止：

- 跳过 modify 步骤直接执行
- 在 generate 中直接扣血
- 在 execute 中重新计算修饰

扩展点：

- EffectHandler trait：新增效果类型
- ModifierCalculator trait：新增修饰规则
- TraitTrigger 枚举：新增触发时机

新增效果类型时：

允许：

- 实现 EffectHandler trait
- 注册到 EffectHandlerRegistry

禁止：

- 修改管线调度代码
- 修改 generate/modify/execute 的执行顺序

---

# 属性系统

## Primary / Derived 分离

Primary Stat：

```
Might, Agility, Vitality, Intelligence, Luck, Resistance, Dexterity, Willpower
```

Derived Stat：

```
MaxHp = 5 + Vitality * 5
MaxMp = Intelligence * 5
Attack = Might * 2
Defense = Vitality
MagicAttack = Intelligence * 2
Initiative = Agility * 2
MoveRange = Agility / 2
```

禁止：

- 直接修改 Derived Stat
- 缓存 Derived Stat（必须实时计算）

## Modifier Stack

所有属性修改必须通过 ModifierSource：

```
ModifierSource::Base           # 种族/职业基础值
ModifierSource::Buff(id)       # Buff 来源
ModifierSource::Equipment(slot) # 装备来源
ModifierSource::Trait(name)    # Trait 来源
ModifierSource::Temporary(name) # 临时修饰
```

禁止：

- 直接修改 base 属性值
- 绕过 Modifier Stack 修改最终属性

发现 `attributes.base[xxx] = ...` 或 `attributes.current_hp -= ...` 不走管线时：

必须停止。

必须输出：

```
ARCHITECTURE VIOLATION: 直接修改属性绕过 Modifier Stack，违反统一修饰管线原则。
```

---

# Trait + Modifier 统一扩展

所有能力来源统一通过 Trait + Modifier 体系：

| 来源 | TraitSource | 示例 |
|------|-------------|------|
| 种族 | Race | 飞行（忽略地形消耗） |
| 职业 | Class | 战士（近战加成） |
| 装备 | Equipment | 火焰武器（OnAttack 施加燃烧） |
| Buff | Buff | 狂暴（OnTurnStart 增加攻击） |
| 天赋 | Talent | 龙裔（OnKill 恢复HP） |

允许：

- 新增 TraitSource 枚举变体
- 新增 TraitTrigger 触发时机
- 新增 TraitEffect 效果类型

禁止：

- 为不同能力来源创建独立的扩展机制
- 绕过 TraitCollection 直接查询能力

---

# 数据驱动

## 注册表

所有配置通过 RON 文件加载：

```
assets/units/*.ron       → UnitTemplateRegistry
assets/skills/*.ron      → SkillRegistry
assets/buffs/*.ron       → BuffRegistry
assets/equipment/*.ron   → EquipmentRegistry
assets/items/*.ron       → ItemRegistry
assets/terrains/*.ron    → TerrainRegistry
assets/maps/*.ron        → LevelRegistry
assets/ai/*.ron          → AiBehaviorRegistry
assets/modifier_rules/*.ron → ModifierRuleRegistry
assets/traits/*.ron      → TraitRegistry
```

## 双类型模式

每个领域两种类型：

- `XxxDef`：RON 反序列化用，使用 TagName 字符串
- `XxxData`：运行时用，使用 GameplayTag 位掩码

必须实现：

```
impl From<XxxDef> for XxxData
```

禁止：

- 运行时使用字符串查询标签
- RON 文件中使用位掩码

---

# 回合状态机

## 状态层次

```
AppState
├── MainMenu
├── InGame
│   └── TurnPhase (SubState)
│       ├── SelectUnit
│       ├── MoveUnit
│       ├── ActionMenu
│       ├── SelectTarget
│       ├── ExecuteAction
│       ├── WaitAction
│       └── TurnEnd
└── GameOver
```

## 阶段转换

必须通过 `NextState<TurnPhase>` 驱动。

禁止：

- 手动设置 TurnPhase 而不经过 NextState
- 在 OnEnter 中执行跨阶段跳转

## AI 与玩家共享流程

AI 必须设置 CombatIntent 后进入统一 Effect Pipeline。

禁止：

- AI 独立计算伤害
- AI 绕过 Effect Pipeline 直接扣血

---

# 地图系统

## 数据分离

```
TerrainGrid  → 地形数据唯一真相源
OccupancyGrid → 单位占位唯一真相源
GameMap       → 坐标转换
```

禁止：

- Tile 作为 Entity
- 在 Unit Component 上存储地形信息
- 寻路时直接查询 Entity

---

# UI 架构

## 三层分离

```
UiCommand（意图层）→ ViewModel（状态层）→ Panel/Widget（展示层）
```

禁止：

- UI 直接修改业务状态
- UI 保存业务真相
- UI 直接 Query ECS World

必须通过：

```
UiCommand Message → command_handler → 状态变更
ViewModel Resource → 只读展示
```

## 正式 UI vs 开发工具

正式 UI：

```
bevy_ui
```

开发工具：

```
bevy_egui
```

Inspector：

必须保留

---

# AI 架构

## 数据驱动

AI 行为从 RON 加载：

```
AiBehavior
├── target_strategy
├── move_strategy
├── skill_strategy
└── skill_priority
```

## 策略扩展

```
TargetSelector trait
MoveSelector trait
SkillSelector trait
```

允许：

- 新增策略实现
- 注册到 AiStrategyRegistry

禁止：

- 在 decision.rs 中硬编码策略逻辑

---

# Reflect

仅用于：

- 编辑器
- 调试器
- 配置检查

禁止：

- 战斗计算依赖 Reflect
- 高频逻辑使用 Reflect 查询

发现战斗代码中 `reflect_*` 调用时：

必须输出：

```
ARCHITECTURE WARNING: 战斗代码使用 Reflect，违反"Reflect 不参与高频计算"原则。
```

---

# Logging

统一：

```
tracing
```

禁止：

```
println!
dbg!
```

日志必须：

- 结构化
- 记录状态变化，不记录函数进入退出
- Error 包含完整上下文

禁止：

- 每帧日志
- 循环内日志

---

# Testing

允许：

- Unit Test（验证规则：伤害、Buff、属性、寻路）
- Integration Test（验证 Feature：装备、背包、战斗、升级）
- Scenario Test（验证流程：战斗回合、技能释放、胜负结算）
- Battle Replay（验证状态流）

禁止：

- 通过修改业务逻辑让测试通过
- 通过修改测试适配错误逻辑
- 删除测试来消除失败

发现测试与逻辑冲突时：

必须停止。

必须输出：

```
POSSIBLE TEST BUG: [描述冲突]
```

或

```
POSSIBLE LOGIC BUG: [描述冲突]
```

等待确认。

---

# 代码组织

## 文件

- 一个文件一个主题
- 优先按业务拆文件，不按代码类型拆文件
- 500 行开始警觉
- 1000 行必须拆分

## 函数

- 一个函数一个主要职责
- 函数名描述意图，不描述过程
- 优先 Early Return
- 超过 3 层嵌套必须重构
- 超过 100 行开始警觉
- 重复三次以上再抽象

## Trait

- Trait 表示能力，不表示分类
- Trait 用于扩展点，不用于模拟继承树
- 重复出现三次以上再抽象 Trait
- 禁止为了优雅创造 Trait

禁止：

```
trait Character { ... }
trait Monster { ... }
trait Boss { ... }
```

允许：

```
trait DamageSource { ... }
trait Healable { ... }
trait TargetSelector { ... }
```

---

# AI 约束

AI 修改代码时优先级：

```
1. Architecture（本文件）
2. Domain Rules（docs/domain/*.md）
3. Test Spec（docs/testing/*.md）
4. Existing Code
```

禁止：

- 为了通过测试修改业务规则
- 为了通过业务规则删除测试
- 违反本文件中的任何禁止项

必须说明：

- 为什么修改
- 影响范围
- 风险

发现修改违反优先级时：

必须停止。

必须输出：

```
PRIORITY VIOLATION: 修改违反了优先级规则。[描述冲突]
Architecture > Domain Rules > Test Spec > Existing Code
```

---

# 可观测性

必须：

- 关键系统支持单步执行与状态回溯
- DamageBreakdown 记录 generate→modify→execute 全链路
- 系统执行顺序可观察
- 复杂系统拥有可视化观察窗口

调试面板：

```
F1  → Battle Debugger
F2  → Buff Viewer
F3  → Gizmos Overlay
F4  → Damage & Attribute Viewer (Tab 切换)
F5  → Turn Queue Viewer
F6  → Pause
F7  → Step
F12 → World Inspector
```

---

# 性能

原则：

- 先正确，再优化
- 先 Profile，再优化
- 性能问题必须测量

优先：

- Changed 过滤优于全量扫描
- Feature 裁剪优于无脑开启全部功能

禁止：

- Reflect 参与高频计算
- 缓存不定义失效条件
- 未 Profile 就全局重构
