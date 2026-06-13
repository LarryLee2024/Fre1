# Inventory 领域

Version: 2.0

## Purpose

Inventory 领域管理物品的存储、堆叠、使用和转移。采用统一容器模型，背包/仓库/宝箱/商店/尸体/掉落袋均为 Container 实例。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| ItemDef | 物品的静态定义，描述物品"是什么" | ≠ ItemInstance：Def 不可变，Instance 有独立状态 |
| ItemInstance | 物品的运行时实例，拥有独立状态 | ≠ ItemStack：Instance 是身份，Stack 是容器中的存储单元 |
| ItemStack | 物品堆叠，一个 Instance + 数量 | ≠ ItemInstance：Stack 是存储单元，Instance 是物品身份 |
| Container | 统一容器，所有存储空间的通用模型 | ≠ ContainerKind：Container 是实例，Kind 是类型标签 |
| UseEffect | 消耗品的使用效果 | ≠ SkillEffect：UseEffect 是消耗品触发，SkillEffect 是技能触发 |
| BattleInventory | 战斗背包，战斗期间的独立物品副本 | ≠ 角色背包：BattleInventory 是副本，战斗结束才合并 |

---

## Responsibilities

### Owns

- ItemDef 定义和注册表
- ItemInstance 创建和管理
- ItemStack 合并和拆分
- Container 的增删查改
- 消耗品使用
- 容器间转移
- 战场背包
- 资源系统

### Does Not Own

- 装备穿脱逻辑 → equipment_rules
- 属性计算和修饰符管线 → stat_system
- Buff 的生命周期 → buff_rules
- UI 展示 → ui_rules

---

## Invariants

### INV-INV-01：InstanceId 全局唯一 🟥

每个 ItemInstance 的 instance_id 在全局范围内唯一。

违反：两个物品共享同一 instance_id，查找和移除操作冲突。

### INV-INV-02：堆叠数量不超过上限 🟥

ItemStack.count ≤ ItemDef.stack_size。

违反：堆叠数量超过上限。

### INV-INV-03：容器容量和重量约束 🟩

容器内 stacks 数量 ≤ capacity（capacity > 0 时），总重量 ≤ max_weight（max_weight > 0 时）。

违反：容器超载。

### INV-INV-04：空堆叠自动清理 🟥

reduce_stack 完成后，count = 0 的 ItemStack 必须从容器中移除。

违反：空堆叠残留，占用容器空间。

### INV-INV-05：消耗品仅 Consumable 可使用 🟥

只有 item_type == Consumable 的物品被执行使用效果。

违反：装备被当作消耗品使用。

### INV-INV-06：属性修改必须通过 Modifier 管线 🟥

宪法：2.2.1

RestoreVital 效果必须通过 Modifier 管线修改属性，禁止直接修改最终属性值。

违反：消耗品直接修改 HP、MP 等最终属性值。

### INV-INV-07：ItemDef 不可变 🟥

宪法：1.1.2

ItemDef 加载后不可修改，多个实例共享同一定义。

违反：修改定义影响所有实例。

---

## State Machine

### 物品实例状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| InContainer | 在某个容器中 | InContainer, Used, Destroyed |
| Used | 消耗品已使用 | — |
| Destroyed | 已销毁 | — |

```
InContainer → InContainer（转移）
            → Used（消耗品使用）
            → Destroyed（丢弃）
```

| 从 | 到 | 条件 |
|----|-----|------|
| InContainer | InContainer | TransferItem Message |
| InContainer | Used | UseItem Message + Consumable 类型 |
| InContainer | Destroyed | reduce_stack 归零 |

---

## Business Rules

### BR-INV-01：堆叠合并

- def_id 相同 + count + other.count ≤ stack_size + 双方 bind == None + enhance_level 相同 + enchantments 相同
- 部分合并：超出上限的部分作为新堆叠

### BR-INV-02：容器操作

- add_stack 逐步检查容量和重量
- 部分成功返回已添加数量
- reduce_stack 归零自动移除
- 转移时 from_entity != to_entity

### BR-INV-03：消耗品使用

- 类型检查：仅 Consumable
- 应用 use_effects（RestoreVital / ApplyBuff / GrantTempTrait / CastSkill）
- reduce_stack(instance_id, 1)
- 发送 ItemUsed Message

### BR-INV-04：容器间转移

- 检查目标容器容量和重量
- 从源容器 reduce_stack → 添加到目标容器 add_stack
- 发送 ItemTransferred Message
- 部分转移允许

### BR-INV-05：战场背包

- 战斗开始复制源背包物品到 BattleInventory
- 战斗结束 drain 转移回源背包
- 战场背包容量 8，重量 30
- 战斗中禁止直接操作角色背包

---

## Pipelines

### 添加物品管线

查找定义 → 尝试合并 → 检查容量 → 检查重量 → 添加新堆叠

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 查找定义 | def_id + 注册表 | ItemDef | def_id 不存在时静默跳过 |
| 尝试合并 | ItemStack + 容器中已有堆叠 | 合并后剩余数量 | 禁止跳过合并条件检查 |
| 检查容量和重量 | 剩余数量 + 容器限制 | 可添加数量 | 禁止绕过容量/重量检查 |
| 添加新堆叠 | 剩余物品 | 成功添加数量 | 禁止添加超过容量的物品 |

### 消耗品使用管线

查找物品 → 类型检查 → 应用效果 → 减少数量 → 发送 Message

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 查找物品 | instance_id + Container | ItemStack | 物品不存在时停止 |
| 类型检查 | ItemDef.item_type | 是/否 | 非 Consumable 停止（INV-INV-05） |
| 应用效果 | use_effects | 属性/Buff 变化 | 禁止跳过任何效果 |
| 减少数量 | instance_id + Container | 堆叠数量变化 | 归零时必须清理（INV-INV-04） |

---

## Data Model

### ItemDef（Definition）

物品的静态定义，不可变。

- 标识：id / name / description
- 类型：item_type（7 种）
- 稀有度 / 标签 / 堆叠上限 / 重量
- 装备相关：modifiers / traits / requirements / slot
- 消耗品相关：use_effects
- 容器相关：container_capacity / container_max_weight
- 配置来源：RON（assets/items/）

### ItemInstance（Instance）

物品的运行时实例。

- instance_id：全局唯一 ID
- def_id：指向定义 ID
- durability / max_durability / enhance_level / enchantments / bind

### ItemStack（值对象）

容器中的存储单元。

- instance：ItemInstance
- count：数量
- total_weight = def.weight × count

### Container（Instance Component）

统一容器。

- kind：容器类型（9 种）
- stacks：ItemStack 列表
- capacity / max_weight：容量和重量限制
- owner：拥有者实体

### Resources（Instance Component）

货币/声望等资源。

- stacks：ResourceStack 列表（resource_id → amount）

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 消耗品使用 | UseItem Message | inventory |
| 物品转移 | TransferItem Message | inventory |
| 使用完成 | ItemUsed Message | ui |
| 转移完成 | ItemTransferred Message | ui |
| 恢复属性 | 通过 ModifierSource 修改 Attributes | stat_system |
| 施加 Buff | 通过 apply_buff 接口 | buff |

---

## Change Rules

### 新增物品类型

- 允许：新增 ItemType 变体 + 新增 ItemDef 字段 + 新增 RON 配置
- 禁止：修改 Container 核心操作、修改堆叠合并条件、修改容量/重量检查逻辑
- 检查：ItemRegistry 注册、stack_size 和 weight 默认值、与装备系统的交互

### 新增使用效果

- 允许：新增 UseEffect 变体 + 在 UseItem 处理中添加效果分支
- 禁止：修改现有 UseEffect 的逻辑、修改 UseItem Message 结构
- 检查：UseItem 处理流程、效果应用是否需要跨领域通信

### 新增容器类型

- 允许：新增 ContainerKind 变体 + 新增预设工厂方法
- 禁止：修改 Container 核心操作、修改容量/重量检查逻辑
- 检查：容器预设参数、与战场背包的交互、UI 展示适配

---

## Architecture Violations

发现架构违规时统一输出：

```
ARCHITECTURE VIOLATION:
Rule: <RuleID>
Reason: <Why>
Fix: <How>
```

| RuleID | 违规行为 | Reason | Fix |
|--------|----------|--------|-----|
| INV-INV-06 | 消耗品直接修改属性值 | 属性修改必须通过 Modifier 管线 | 通过 ModifierSource 添加修饰符 |
| INV-INV-07 | 运行时修改 ItemDef | Definition/Instance 分离 | 改为修改运行时实例 |
| INV-INV-05 | 非 Consumable 使用 UseItem | 只有消耗品有使用效果 | 添加类型检查 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证堆叠合并/容器操作
- 集成测试：验证完整物品生命周期
- Bug 修复必须先编写重现测试

排查顺序：
1. 堆叠合并条件（5 个条件）
2. 容量和重量约束
3. instance_id 全局唯一性
4. 空堆叠是否自动清理
5. ItemDef 引用是否合法
