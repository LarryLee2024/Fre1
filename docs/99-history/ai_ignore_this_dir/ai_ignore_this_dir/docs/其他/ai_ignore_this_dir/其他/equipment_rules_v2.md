# Equipment 领域

Version: 2.0

## Purpose

Equipment 领域管理角色装备的穿脱、槽位、需求检查和效果应用。装备本质 = Modifier + Trait，穿脱必须同步 TraitCollection 和修饰符。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| EquipmentDef | 装备的静态定义，描述装备"是什么" | ≠ EquipmentInstance：Def 不可变，Instance 有独立状态 |
| EquipmentInstance | 装备的运行时实例，拥有独立状态 | ≠ EquipmentDef：Instance 有耐久/强化/附魔 |
| EquipmentSlots | 单位上的装备槽位容器 | ≠ EquipmentRegistry：Slots 是实例，Registry 是定义 |
| EquipmentRequirement | 装备需求条件，决定"谁能用" | ≠ SkillCondition：Requirement 是装备前置 |
| ModifierSource | 修饰符来源标识 | ≠ 修饰符本身：Source 是来源标识 |

---

## Responsibilities

### Owns

- EquipmentDef 定义和注册表
- EquipmentInstance 的创建和管理
- EquipmentSlots 槽位管理
- 穿脱流程
- 需求检查
- 装备效果应用（修饰符 + 标签 + Trait）
- Trait 重建

### Does Not Own

- 属性计算和修饰符管线 → stat_system
- Trait 定义和效果处理 → trait_rules
- 背包管理 → inventory_rules
- Buff 管理 → buff_rules
- UI 展示 → ui_rules

---

## Invariants

### INV-EQP-01：穿脱后 Trait 必须重建 🟥

穿脱操作完成后，TraitCollection 必须反映当前装备状态。

违反：卸下装备后仍拥有该装备的 Trait。

### INV-EQP-02：修饰符来源精确追踪 🟥

宪法：2.2.1

装备修饰符的 ModifierSource 必须与 instance_id 对应，脱卸时按来源精确移除。

违反：脱卸装备时无法精确移除对应修饰符，残留或误删。

### INV-EQP-03：标签分层不混淆 🟥

装备标签存储在 PersistentTags.from_equipment，不与 Trait/Buff 标签混淆。

违反：Buff 过期时误删装备标签，或装备穿脱时误删 Trait 标签。

### INV-EQP-04：槽位唯一性 🟥

每个 EquipmentSlot 最多有一个装备实例。

违反：同一槽位出现多个装备，属性叠加错误。

### INV-EQP-05：修饰符必须通过 Modifier 管线 🟥

宪法：2.2.1

装备的属性修饰必须通过 ModifierSource 添加到 Attributes，禁止直接修改属性值。

违反：装备直接修改 HP、ATK 等最终属性值。

### INV-EQP-06：EquipmentDef 不可变 🟥

宪法：1.1.2

EquipmentDef 加载后不可修改，多个实例共享同一定义。

违反：修改定义影响所有实例。

---

## State Machine

### 装备实例状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| InInventory | 在背包中 | Equipped |
| Equipped | 装备在槽位上 | InInventory |
| Destroyed | 已销毁 | — |

```
InInventory → Equipped → InInventory
                      ↘ Destroyed
```

| 从 | 到 | 条件 |
|----|-----|------|
| InInventory | Equipped | EquipItem Message + 需求满足 |
| Equipped | InInventory | UnequipItem Message |
| Equipped | Equipped | 替换旧装备（先脱后穿） |

---

## Business Rules

### BR-EQP-01：穿戴

- 需求检查先于穿戴，失败发送 EquipFailed
- 槽位已占用时先脱卸旧装备
- 从背包移除装备
- 应用修饰符 + 标签 + Trait
- 重建 Trait 效果和 GameplayTags
- 发送 ItemEquipped Message

### BR-EQP-02：脱卸

- 移除修饰符（按 ModifierSource 精确移除）
- 移除装备授予的标签
- 移除装备授予的 Trait
- 清除槽位
- 创建 ItemStack 放回背包
- 重建 Trait 效果和 GameplayTags
- 发送 ItemUnequipped Message

### BR-EQP-03：需求检查

- 按定义顺序检查
- 第一个不满足即返回 Failed
- RequireTag 检查 GameplayTags
- AttributeMin 检查属性值

### BR-EQP-04：装备效果应用

- 修饰符通过 ModifierSource 添加
- 标签添加到 PersistentTags.from_equipment
- Trait 添加到 TraitCollection（TraitSource::Equipment { slot }）

---

## Pipelines

### 穿戴管线

查找实例 → 查找定义 → 需求检查 → 脱旧装备 → 从背包移除 → 装备到槽位 → 应用效果 → 重建 Trait → 重建标签

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 查找 | 实例 ID + 注册表 + 背包 | 装备实例 + 定义 | 实例或定义不存在时静默跳过 |
| 需求检查 | 需求条件 + 属性 + 标签 | 满足或不满足 | 检查失败时停止穿戴 |
| 脱旧装备 | 槽位 + 当前装备 | 旧装备放回背包 | 禁止跳过旧装备处理 |
| 应用效果 | 修饰符 + 标签 + Trait | 属性和标签变化 | 禁止绕过 Modifier 管线（INV-EQP-05） |
| 重建 | TraitCollection + PersistentTags | 更新后的 Trait 和标签 | 禁止跳过重建（INV-EQP-01） |

### 脱卸管线

检查槽位 → 移除修饰符 → 移除标签 → 移除 Trait → 清除槽位 → 放回背包 → 重建 Trait → 重建标签

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 移除效果 | 装备定义 + ModifierSource | 属性和标签变化 | 禁止遗漏任何修饰符 |
| 放回背包 | 装备实例 | 背包更新 | 禁止丢弃装备 |
| 重建 | TraitCollection + PersistentTags | 更新后的 Trait 和标签 | 禁止跳过重建（INV-EQP-01） |

---

## Data Model

### EquipmentDef（Definition）

装备的静态定义，不可变。

- 标识：id / name / description
- 槽位（8 种）
- 稀有度（5 级）
- 标签列表
- 属性修饰符列表
- 授予的 Trait ID 列表
- 需求条件列表
- 重量
- 配置来源：RON（assets/equipment/）

### EquipmentInstance（Instance）

装备的运行时实例。

- 唯一实例 ID
- 对应定义 ID
- 耐久度 / 最大耐久度
- 强化等级
- 附魔 trait 列表

### EquipmentSlots（Instance Component）

单位装备槽位容器。

- 槽位 → (实例ID, 定义ID) 映射
- 实例 ID 生成器
- 每个槽位最多一个装备

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 穿戴请求 | EquipItem Message | equipment |
| 脱卸请求 | UnequipItem Message | equipment |
| 穿戴完成 | ItemEquipped Message | ui / inventory |
| 脱卸完成 | ItemUnequipped Message | ui / inventory |
| 穿戴失败 | EquipFailed Message | ui |
| 属性修饰符变化 | 通过 ModifierSource 添加到 Attributes | stat_system |
| Trait 变化 | rebuild_trait_effects | trait |

---

## Change Rules

### 新增装备

- 允许：新增 RON 配置 + 新增 EquipmentSlot 变体
- 禁止：修改穿脱流程、修改需求检查逻辑、修改 Trait 重建逻辑
- 检查：注册表注册、修饰符 Source 区间、Trait ID 存在、标签在枚举中

### 新增装备需求类型

- 允许：新增 EquipmentRequirement 变体 + 添加检查逻辑
- 禁止：修改现有需求检查逻辑、改变检查顺序
- 检查：RON 反序列化适配、需求检查短路逻辑

### 新增装备效果

- 允许：新增修饰符类型 + 新增 Trait 授予
- 禁止：绕过 Modifier 管线（INV-EQP-05）、跳过 Trait 重建（INV-EQP-01）
- 检查：ModifierSource 区间、TraitCollection 来源追踪、GameplayTags 重建

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
| INV-EQP-01 | 穿脱后 Trait 未重建 | 装备 Trait 必须与实际状态一致 | 在穿脱流程末尾调用 rebuild_trait_effects |
| INV-EQP-05 | 装备直接修改属性值 | 属性修改必须通过 Modifier 管线 | 通过 ModifierSource 添加修饰符 |
| INV-EQP-06 | 运行时修改 EquipmentDef | Definition/Instance 分离 | 改为修改运行时实例 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证穿脱流程各步骤
- 集成测试：验证完整装备生命周期
- Bug 修复必须先编写重现测试

排查顺序：
1. 需求检查是否全部通过
2. 修饰符是否正确添加/移除
3. Trait 是否正确重建
4. 标签是否分层正确
5. 旧装备是否正确替换
