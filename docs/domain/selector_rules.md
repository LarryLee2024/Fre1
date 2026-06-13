# 目标选择器系统领域

Version: 1.0
Status: Proposed

目标选择器系统管理技能的目标选择规则。技能种类 ≈ Selector × Effect，Selector 决定"对谁放"。

核心原则：
- Selector 只负责目标选择，不负责效果执行
- 目标解析基于施法者坐标 + TargetRule 产生目标实体列表
- Selector 与 Skill Range 配合确定影响范围

---

# 术语定义

## 选择器（Selector）

技能的目标选择规则，决定技能可以作用于哪些实体。

不是 Effect。不是 Requirement。不是技能本身。

关键属性：
- 定义态为 SelectorDef（RON 反序列化用），运行态为 SelectorDef 实例
- 每种 Selector 类型有对应的解析逻辑（TargetSelector trait）
- 通过 SelectorRegistry 按类型名查找分发
- 新增 Selector 类型只需实现 trait 并注册

---

## 目标解析（Target Resolution）

Selector 从当前战场坐标 + TargetRule 产生目标实体列表的过程。

不是寻路。不是移动。不是效果执行。

关键属性：
- 输入：施法者坐标 + Selector 参数 + 战场状态
- 输出：目标实体列表（Vec<Entity>）
- 纯函数，不修改战场状态
- 过滤逻辑：敌我判定 / 存活判定 / 可见性判定

---

## 目标规则（TargetRule）

Selector 的具体参数化，定义选择范围和过滤条件。

不是坐标。不是实体列表。不是 Selector 本身。

关键属性：
- 枚举变体：EnemySingle / EnemyAOE十字 / EnemyAOE圆形 / EnemyAll / AllySingle / AllyAOE / SelfOnly / EmptyTile / SummonSlot
- 每种变体携带参数（如 AOE 的范围大小、形状）
- 通过 TargetRule 的解析逻辑生成候选坐标集合

---

## 技能范围（Skill Range）

技能的施法距离和影响范围，与 Selector 配合确定目标。

不是移动力。不是攻击范围。不是 Selector 本身。

关键属性：
- range：施法距离（从施法者到目标的最远距离）
- aoe_size：AOE 影响范围（仅 AOE 类 Selector 有效）
- 与 Selector 配合：Selector 决定选择规则，Range 决定距离限制
- 空地选择（EmptyTile）时 Range 决定可放置位置

---

## 空地选择（EmptyTile Selection）

对空地释放的 Selector 类型，用于传送、陷阱放置等场景。

不是敌人选择。不是友方选择。不是自身选择。

关键属性：
- 目标为无实体的空地图格
- 需要验证目标坐标可行走（非阻挡地形）
- 传送技能：目标坐标为传送目的地
- 陷阱放置：目标坐标为陷阱放置位置
- 召唤技能：目标坐标为召唤物放置位置

---

# 领域边界

## 本领域负责

- Selector 类型定义和目标选择逻辑（TargetSelector trait）
- 目标解析流程（坐标 → 候选 → 过滤 → 实体列表）
- Selector 注册表（SelectorRegistry）
- 空地选择的验证逻辑
- 技能范围（Skill Range）的定义和查询

## 本领域不负责

- 技能释放前的前置检查（由 Requirement 领域负责）
- 技能消耗的校验和扣除（由 Cost 领域负责）
- 效果的实际执行（由 Effect Pipeline 领域负责：Generate → Modify → Execute）
- 条件效果的判断（由 Condition 领域负责）
- 战场地图数据和寻路（由 Map 领域负责）
- 输入处理和 UI 交互（由 Input/UI 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 目标实体列表 | 函数调用（resolve_targets） | Battle/Skill 领域 |
| 战场坐标查询 | 函数调用（GridPosition） | Map 领域 |
| 目标过滤（敌我/存活） | 函数调用（GameplayTag 查询） | Character 领域 |
| 空地验证 | 函数调用（is_walkable） | Map 领域 |

---

# 生命周期

本领域无状态机，为纯函数式计算。

目标解析是无状态的纯函数调用：输入施法者坐标 + Selector 参数，输出目标实体列表。不涉及状态转换。

唯一有状态的是 SelectorRegistry（Resource），其生命周期为：
- 系统启动时注册所有内置 TargetSelector
- 运行时只读查找，不修改

---

# 不变量

## 不变量1：目标解析为纯函数

任意时刻：

TargetSelector::resolve_targets() 不修改战场状态，仅读取坐标和战场数据并返回目标实体列表。

违反表现：

resolve_targets() 内部修改了 Entity 位置、添加/移除单位、改变了战场状态。

---

## 不变量2：目标必须存活

任意时刻：

Selector 解析出的目标实体列表中的所有 Entity 必须是存活状态（不含 Dead 标签）。

违反表现：

对已死亡单位释放技能，效果作用于尸体。

---

## 不变量3：目标过滤基于标签

任意时刻：

敌我判定必须基于 GameplayTag（如 Enemy / Ally 标签），不基于字符串匹配或硬编码 Entity ID。

违反表现：

使用 skill_id == "fireball" 判断目标类型。使用硬编码 Entity ID 列表。

---

## 不变量4：空地选择必须验证可行走

任意时刻：

EmptyTile Selector 的目标坐标必须通过地图可行走验证（is_walkable），不可放置在阻挡地形上。

违反表现：

传送技能将单位传送到墙壁中。陷阱放置在不可行走的地形上。

---

# 业务规则

## 规则1：Selector 与 Effect 解耦

禁止：
- 在 Selector 中执行效果逻辑
- Selector 解析目标时修改 Entity 属性

必须：
- Selector 只负责生成目标实体列表
- 效果执行由 Effect Pipeline 负责

允许：
- Selector 查询战场状态（只读）用于目标过滤

---

## 规则2：Selector 注册表分发

禁止：
- 使用 match 硬编码 Selector 分发
- 未注册的 Selector 类型在解析时静默跳过（应输出 warn 日志）

必须：
- 每种 SelectorDef 变体对应一个 TargetSelector 实现
- 评估器通过 SelectorRegistry 注册
- type_name 与 SelectorDef 的类型标识一致

允许：
- 新增 Selector 类型只需实现 trait + 注册，不修改管线代码

---

## 规则3：AOE 形状定义

禁止：
- AOE 范围硬编码（如十字范围写死为 1 格）
- 不同 AOE 形状使用同一解析逻辑

必须：
- AOE 范围由 SelectorDef 参数配置（如 aoe_size: u32）
- 十字范围和圆形范围有独立的解析逻辑
- AOE 范围从施法者位置或目标位置扩展（由 SelectorDef 配置）

允许：
- AOE 范围在 RON 文件中配置（assets/skills/*.ron）

---

## 规则4：目标数量限制

禁止：
- 无限制地选择目标（如 EnemyAll 无上限）
- 忽略战场上的实体总数

必须：
- SelfOnly 始终只选择施法者自身（数量 = 1）
- EnemySingle / AllySingle 始终只选择一个目标
- AOE 类 Selector 的目标数量由战场实体分布决定

允许：
- EnemyAll / AllyAOE 选择所有符合条件的目标（无数量上限）

---

# 流程管线

## 目标解析管线

```
施法者坐标 → Selector + Range → 候选坐标集合 → 过滤（敌我/存活/可见）→ 目标实体列表
```

### Step1：施法者坐标

输入：施法者 Entity 的 GridPosition 组件
处理：获取施法者当前地图坐标
输出：施法者坐标（x, y）
禁止：在获取坐标时修改 Entity 位置

### Step2：Selector + Range

输入：SelectorDef（目标规则）+ Skill Range（施法距离）
处理：根据 Selector 类型和 Range 生成候选坐标集合
输出：候选坐标集合（Vec<(x, y)>）
禁止：忽略 Range 限制（超出距离的坐标应被排除）

### Step3：候选坐标集合

输入：施法者坐标 + Selector 参数 + Range
处理：根据 Selector 类型生成范围内的坐标（十字/圆形/全图）
输出：候选坐标列表
禁止：生成不可行走的坐标（空地选择时）

### Step4：过滤（敌我/存活/可见）

输入：候选坐标列表 + 战场实体状态
处理：逐坐标查询实体，过滤不符合条件的目标
输出：过滤后的目标实体列表
禁止：跳过存活判定（对死亡单位选择目标）

### Step5：目标实体列表

输入：过滤后的目标列表
处理：组装最终目标实体列表
输出：Vec<Entity>（目标实体列表）
禁止：返回空列表时静默跳过（应输出日志提示无有效目标）

---

# 数据结构

## SelectorDef（选择器定义）

职责：定义技能的目标选择规则（RON 反序列化用）

结构：
- 类型标识：SelectorDef 枚举变体（EnemySingle / EnemyAOE十字 / EnemyAOE圆形 / EnemyAll / AllySingle / AllyAOE / SelfOnly / EmptyTile / SummonSlot）
- 参数：根据变体不同而不同（如 AOE 的范围大小、形状）

要求：
- 每个变体通过 type_name() 返回选择器类型名
- type_name 与 TargetSelector::type_name 一致
- 不包含运行时状态

---

## TargetSelector（目标选择器 trait）

职责：描述如何解析一种选择器类型的 trait 实现

结构：
- type_name()：返回选择器类型名（与 SelectorDef::type_name 对应）
- resolve_targets(&self, ctx: &TargetContext) -> Vec<Entity>：解析目标实体列表

要求：
- 每种 SelectorDef 变体实现一个 TargetSelector
- 通过 SelectorRegistry 注册
- 纯函数，不修改任何游戏状态

---

## SelectorRegistry（选择器注册表）

职责：存储所有 TargetSelector 实现，通过类型名查找分发

结构：
- selectors：类型名到选择器的映射

要求：
- 注册所有内置目标选择器（9 种）
- O(1) HashMap 查找
- 不重复注册（register 时检查 key 是否存在）

---

## TargetContext（目标解析上下文）

职责：封装目标解析所需的全部输入数据

结构：
- source_entity：施法者 Entity
- source_position：施法者坐标（GridPosition）
- selector：SelectorDef（目标规则）
- skill_range：u32（施法距离）
- aoe_size：u32（AOE 范围，仅 AOE 类有效）
- entity_positions：实体坐标映射（Entity → GridPosition）
- entity_tags：实体标签映射（Entity → GameplayTags）
- grid：战场网格数据

要求：
- 纯数据传递，不存储持久状态
- 通过 from_query() 从 ECS 查询构建
- 克隆坐标和标签数据，避免借用冲突

---

## SkillRange（技能范围）

职责：定义技能的施法距离和影响范围

结构：
- range：u32 — 施法距离（从施法者到目标的最远距离，0 表示仅自身）
- aoe_size：u32 — AOE 影响范围（仅 AOE 类 Selector 有效，0 表示非 AOE）
- aoe_shape：AoeShape — AOE 形状（Cross / Circle，仅 AOE 类有效）

要求：
- range = 0 时仅对自身生效（SelfOnly）
- aoe_size = 0 时为单体技能
- aoe_shape 仅在 aoe_size > 0 时有效

---

# 禁止事项

禁止：TargetSelector 解析时修改战场状态

原因：目标解析必须是纯函数，修改状态会导致不可预测的副作用

违反后果：目标解析产生副作用，战场状态不一致，调试困难

---

禁止：对已死亡单位选择目标

原因：死亡单位不应参与技能效果，选择死亡单位违反游戏逻辑

违反后果：技能效果作用于尸体，产生无意义的效果或错误

---

禁止：使用 match 硬编码 Selector 分发

原因：match 分发违反 TargetSelector trait 扩展原则，新增选择器类型需修改分发代码

违反后果：每次新增选择器类型都要修改核心解析管线，违反开闭原则

---

禁止：空地选择不验证可行走

原因：传送/陷阱等技能的目标位置必须可行走，否则破坏游戏逻辑

违反后果：单位被传送到墙壁中，陷阱放置在不可达位置

---

禁止：修改 SelectorDef 定义态

原因：SelectorDef 是不可变配置，运行时通过选择器解析

违反后果：全局选择器配置被污染，多场战斗数据不一致

---

禁止：忽略 Skill Range 限制

原因：Range 限制了技能的施法距离，忽略会导致超距离释放技能

违反后果：远程技能变为全图技能，近程技能变为远程技能

---

禁止：AOE 选择器不检查目标存活

原因：AOE 范围内可能包含死亡单位，选择死亡单位违反游戏逻辑

违反后果：AOE 技能效果作用于死亡单位，产生无意义的效果

---

# AI 修改规则

## 如果新增 Selector 类型

允许：
- 在 SelectorDef 枚举中添加新变体
- 实现对应的 TargetSelector trait
- 注册到 SelectorRegistry

禁止：
- 修改解析管线的调度逻辑
- 在 resolve_targets 方法中使用 match 硬编码分发

优先检查：
- SelectorDef::type_name 与 TargetSelector::type_name 是否一致
- TargetContext 是否包含新选择器所需的上下文字段
- 新选择器的 RON 反序列化是否兼容旧配置

---

## 如果修改目标解析逻辑

允许：
- 调整现有 TargetSelector 的 resolve_targets 实现
- 修改 AOE 范围计算逻辑

禁止：
- 改变目标过滤逻辑（必须包含存活判定）
- 修改 TargetSelector trait 的 resolve_targets 方法签名
- 移除现有选择器类型的解析器

优先检查：
- 修改后的解析逻辑是否影响现有技能的目标选择结果
- AOE 范围计算是否正确（十字/圆形）
- 空地选择是否验证可行走

---

## 如果修改 TargetContext

允许：
- 添加新的上下文字段（如新的选择器类型需要的数据）
- 调整 from_query() 的构建逻辑

禁止：
- 移除现有上下文字段（会影响现有选择器）
- 改变 from_query() 的返回类型

优先检查：
- 新字段是否为 Option 类型（向后兼容）
- 所有现有选择器是否兼容新 Context 结构
- 缺失新字段时选择器是否正确处理

---

## 如果测试失败

排查顺序：
1. 检查 SelectorDef::type_name 与 TargetSelector::type_name 是否匹配
2. 检查 TargetContext 是否包含选择器所需的全部字段
3. 检查目标过滤是否包含存活判定
4. 检查 AOE 范围计算是否正确
5. 检查空地选择是否验证可行走
