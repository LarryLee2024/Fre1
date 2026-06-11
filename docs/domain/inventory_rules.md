# Inventory 领域

Version: 1.0

Inventory 领域管理物品的存储、堆叠、使用和转移。采用统一容器模型，背包/仓库/宝箱/商店/尸体/掉落袋均为 Container 实例。

核心原则：
- 统一容器模型
- Definition / Instance 分离
- 堆叠合并
- 容量与重量双限制

---

# 术语定义

## ItemDef

物品的静态定义，描述物品"是什么"。

不是 ItemInstance。Def 不可变，Instance 有独立状态。

关键属性：
- id / name / description：标识和展示
- item_type：物品类型（Equipment / Consumable / Material / Quest / Ammo / Currency / Container）
- stack_size：堆叠上限
- weight：重量

---

## ItemInstance

物品的运行时实例，拥有独立状态。

不是 ItemDef。Instance 有耐久/强化/附魔/绑定，Def 是共享配置。

关键属性：
- instance_id：全局唯一 ID
- def_id：指向定义 ID
- durability / enhance_level / enchantments / bind

---

## ItemStack

物品堆叠，一个 Instance + 数量。

不是 ItemInstance。Stack 是容器中的存储单元，Instance 是物品身份。

关键属性：
- instance：ItemInstance
- count：数量

---

## Container

统一容器，所有存储空间的通用模型。

不是 ContainerKind。Container 是实例，ContainerKind 是类型标签。

关键属性：
- kind：容器类型
- stacks：ItemStack 列表
- capacity / max_weight：容量和重量限制

---

## UseEffect

消耗品的使用效果。

不是 SkillEffect。UseEffect 是消耗品触发，SkillEffect 是技能触发。

关键属性：
- RestoreVital / ApplyBuff / GrantTempTrait / CastSkill

---

# 领域边界

## 本领域负责

- ItemDef 定义和注册表（ItemRegistry）
- ItemInstance 创建和管理
- ItemStack 合并和拆分
- Container 的增删查改
- 消耗品使用（UseItem Message）
- 容器间转移（TransferItem Message）
- 战场背包（BattleInventory）
- 资源系统（Resources）

## 本领域不负责

- 装备穿脱逻辑（由 equipment_rules 领域负责）
- 属性计算和修饰符管线（由 stat_system 领域负责）
- Buff 的生命周期（由 buff_rules 领域负责）
- UI 展示（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 消耗品使用 | UseItem Message | inventory |
| 物品转移 | TransferItem Message | inventory |
| 使用完成 | ItemUsed Message | ui |
| 转移完成 | ItemTransferred Message | ui |
| 恢复属性 | 直接修改 Attributes | stat_system |
| 施加 Buff | 直接调用 apply_buff | buff |

---

# 生命周期

## 物品实例生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| InContainer | 在某个容器中 | InContainer（转移）, Used, Destroyed |
| Used | 消耗品已使用 | — |
| Destroyed | 已销毁 | — |

## 状态转换图

InContainer → InContainer（转移）
            → Used（消耗品使用）
            → Destroyed（丢弃）

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| InContainer | InContainer | TransferItem Message |
| InContainer | Used | UseItem Message + Consumable 类型 |
| InContainer | Destroyed | reduce_stack 归零 |

---

# 不变量

## 不变量1：InstanceId 全局唯一

任意时刻：

每个 ItemInstance 的 instance_id 在全局范围内唯一。

违反表现：

两个物品共享同一 instance_id，查找和移除操作冲突。

---

## 不变量2：堆叠数量不超过上限

任意时刻：

ItemStack.count ≤ ItemDef.stack_size。

违反表现：

堆叠数量超过上限，UI 显示异常。

---

## 不变量3：容器容量和重量约束

add_stack 完成后：

容器内 stacks 数量 ≤ capacity（capacity > 0 时），总重量 ≤ max_weight（max_weight > 0 时）。

违反表现：

容器超载，物品数量或重量超出限制。

---

## 不变量4：空堆叠自动清理

reduce_stack 完成后：

count = 0 的 ItemStack 必须从容器中移除。

违反表现：

空堆叠残留，占用容器空间。

---

## 不变量5：消耗品仅 Consumable 可使用

UseItem 处理完成后：

只有 item_type == Consumable 的物品被执行使用效果。

违反表现：

装备被当作消耗品使用。

---

# 业务规则

## 规则1：堆叠合并

禁止：
- 不检查合并条件直接合并

必须：
- def_id 相同
- count + other.count ≤ stack_size
- 双方 bind == None
- enhance_level 相同
- enchantments 相同

允许：
- 部分合并（超出上限的部分作为新堆叠）

---

## 规则2：容器操作

禁止：
- 绕过容量和重量检查添加物品
- 空堆叠不清理

必须：
- add_stack 逐步检查容量和重量
- 部分成功返回已添加数量
- reduce_stack 归零自动移除
- 转移时 from_entity != to_entity

---

## 规则3：消耗品使用

禁止：
- 非 Consumable 类型使用 UseItem

必须：
- 应用 use_effects（RestoreVital / ApplyBuff / GrantTempTrait / CastSkill）
- reduce_stack(instance_id, 1)
- 发送 ItemUsed Message

---

## 规则4：容器间转移

禁止：
- 同时可变借用同一 Entity

必须：
- 检查目标容器容量和重量
- 从源容器 reduce_stack
- 添加到目标容器 add_stack
- 发送 ItemTransferred Message

允许：
- 部分转移成功

---

## 规则5：战场背包

禁止：
- 战斗中直接操作角色背包

必须：
- 战斗开始复制源背包物品到 BattleInventory
- 战斗结束 drain 转移回源背包
- 战场背包容量 8，重量 30

---

# 流程管线

## 添加物品管线

查找定义 → 尝试合并 → 检查容量 → 检查重量 → 添加新堆叠

### Step1：查找定义

输入：def_id + ItemRegistry
处理：查找 ItemDef
输出：ItemDef
禁止：def_id 不存在时静默跳过

### Step2：尝试合并

输入：ItemStack + 容器中已有堆叠
处理：逐个检查 can_merge_with，合并
输出：合并后剩余数量
禁止：跳过合并条件检查

### Step3：检查容量和重量

输入：剩余数量 + 容器容量/重量
处理：检查是否可添加新堆叠
输出：可添加数量
禁止：绕过容量/重量检查

### Step4：添加新堆叠

输入：剩余物品
处理：创建新 ItemStack 添加到容器
输出：成功添加数量
禁止：添加超过容量的物品

---

## 消耗品使用管线

查找物品 → 类型检查 → 应用效果 → 减少数量 → 发送 Message

### Step1：查找物品

输入：instance_id + Container
处理：查找 ItemStack
输出：ItemStack
禁止：物品不存在时继续

### Step2：类型检查

输入：ItemDef.item_type
处理：检查是否 Consumable
输出：是/否
禁止：非 Consumable 继续使用

### Step3：应用效果

输入：ItemDef.use_effects
处理：RestoreVital / ApplyBuff / GrantTempTrait / CastSkill
输出：属性/Buff 变化
禁止：跳过任何效果

### Step4：减少数量

输入：instance_id + Container
处理：reduce_stack(instance_id, 1)
输出：堆叠数量变化
禁止：归零时不清理

---

# 数据结构

## ItemDef（Definition）

职责：物品的静态定义

结构：
- id / name / description：标识和展示
- item_type：物品类型（7 种）
- rarity：稀有度
- tags：标签列表
- stack_size：堆叠上限
- weight：重量
- modifiers / traits / requirements / slot：装备相关
- use_effects：消耗品相关
- container_capacity / container_max_weight：容器相关

要求：
- 不可变，加载后不修改
- RON 配置路径：assets/items/

---

## ItemInstance（Instance）

职责：物品的运行时实例

结构：
- instance_id：全局唯一 ID
- def_id：指向定义 ID
- durability / max_durability：耐久度
- enhance_level：强化等级
- enchantments：附魔 trait
- bind：绑定状态
- signature / quest_state：定制标记

要求：
- instance_id 全局唯一
- 装备创建时 durability = 100

---

## ItemStack（值对象）

职责：容器中的存储单元

结构：
- instance：ItemInstance
- count：数量

要求：
- count ≤ def.stack_size
- can_merge_with 检查 5 个条件
- total_weight = def.weight × count

---

## Container（Instance Component）

职责：统一容器

结构：
- kind：容器类型（9 种）
- stacks：ItemStack 列表
- capacity / max_weight：容量和重量限制
- owner：拥有者实体
- container_traits：容器 Trait

要求：
- add_stack 自动合并 + 容量/重量检查
- reduce_stack 归零自动清理
- 预设工厂方法

---

## Resources（Instance Component）

职责：货币/声望等资源，独立于物品堆叠

结构：
- stacks：ResourceStack 列表（resource_id → amount）

要求：
- add 自动累加
- spend 不足返回 false

---

# 禁止事项

禁止：绕过容量和重量检查添加物品

原因：容量和重量是游戏平衡的核心约束

违反后果：容器无限装载，游戏经济崩溃

---

禁止：空堆叠不清理

原因：空堆叠占用容器空间，影响容量计算

违反后果：容器显示空槽但无法添加物品

---

禁止：非 Consumable 使用 UseItem

原因：只有消耗品有使用效果

违反后果：装备被当作消耗品使用，物品丢失

---

禁止：战斗中直接操作角色背包

原因：战斗背包是独立副本，战斗结束才合并

违反后果：战斗中物品变化影响非战斗状态

---

禁止：同时可变借用同一 Entity 的容器

原因：Rust 借用规则禁止

违反后果：运行时 panic

---

# AI 修改规则

## 如果新增物品类型

允许：
- 新增 ItemType 变体
- 新增 ItemDef 字段
- 新增 RON 配置

禁止：
- 修改 Container 核心操作
- 修改堆叠合并条件
- 修改容量/重量检查逻辑

优先检查：
- ItemRegistry 注册
- stack_size 和 weight 默认值
- 与装备系统的交互

---

## 如果新增使用效果

允许：
- 新增 UseEffect 变体
- 在 UseItem 处理中添加效果分支

禁止：
- 修改现有 UseEffect 的逻辑
- 修改 UseItem Message 结构

优先检查：
- UseItem 处理流程
- 效果应用是否需要跨领域通信
- ItemUsed Message 是否需要扩展

---

## 如果新增容器类型

允许：
- 新增 ContainerKind 变体
- 新增预设工厂方法

禁止：
- 修改 Container 核心操作
- 修改容量/重量检查逻辑

优先检查：
- 容器预设参数
- 与战场背包的交互
- UI 展示适配

---

## 如果测试失败

排查顺序：
1. 检查堆叠合并条件（5 个条件）
2. 检查容量和重量约束
3. 检查 instance_id 全局唯一性
4. 检查空堆叠是否自动清理
5. 检查 ItemDef 引用是否合法
