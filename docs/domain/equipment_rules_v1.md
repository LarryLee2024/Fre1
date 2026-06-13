# Equipment 领域

Version: 1.1

Equipment 领域管理角色装备的穿脱、槽位、需求检查和效果应用。装备本质 = Modifier + Trait。

核心原则：
- 🟥 Definition / Instance 分离（宪法 1.1.2）
- 🟩 装备 = Modifier + Trait
- 🟥 Rule / Content 分离（宪法 1.1.3）
- 🟩 标签三层架构
- 🟥 修饰符必须通过 Modifier 管线（宪法 2.2.1）

---

# 术语定义

## EquipmentDef

装备的静态定义，描述装备"是什么"。

不是 EquipmentInstance。Def 不可变，Instance 有独立状态。

关键属性：
- id / name / description：标识和展示
- slot：装备槽位
- rarity：稀有度
- modifiers：属性修饰符
- traits：授予的 Trait ID
- requirements：需求条件

---

## EquipmentInstance

装备的运行时实例，拥有独立状态。

不是 EquipmentDef。Instance 有耐久/强化/附魔，Def 是共享配置。

关键属性：
- instance_id：唯一实例 ID
- def_id：指向定义 ID
- durability / max_durability：耐久度
- enhance_level：强化等级
- enchantments：附魔 trait

---

## EquipmentSlots

单位上的装备槽位容器，管理各槽位的装备实例。

不是 EquipmentRegistry。Slots 是实例，Registry 是定义。

关键属性：
- slots：槽位 → (实例ID, 定义ID) 映射

---

## EquipmentRequirement

装备需求条件，决定"谁能用"。

不是 SkillCondition。Requirement 是装备前置，SkillCondition 是技能前置。

关键属性：
- RequireTag：需要拥有指定标签
- AttributeMin：属性最低要求

---

## ModifierSource

修饰符来源标识。装备领域使用 equipment_source(slot) 变体。

不是修饰符本身。ModifierSource 是来源标识，Modifier 是属性修饰。

关键属性：
- equipment_source(slot)：装备来源，记录具体槽位

---

# 领域边界

## 本领域负责

- EquipmentDef 定义和注册表（EquipmentRegistry）
- EquipmentInstance 的创建和管理
- EquipmentSlots 槽位管理
- 穿脱流程（EquipItem / UnequipItem Message）
- 需求检查（check_equipment_requirements）
- 装备效果应用（修饰符 + 标签 + Trait）
- Trait 重建（rebuild_trait_effects）

## 本领域不负责

- 属性计算和修饰符管线（由 stat_system 领域负责）
- Trait 定义和效果处理（由 trait_rules 领域负责）
- 背包管理（由 inventory_rules 领域负责）
- Buff 管理（由 buff_rules 领域负责）
- UI 展示（由 ui_rules 领域负责）

## 跨领域通信方式

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

# 生命周期

## 装备实例生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| InInventory | 在背包中 | Equipped |
| Equipped | 装备在槽位上 | InInventory |
| Destroyed | 已销毁 | — |

## 状态转换图

InInventory → Equipped → InInventory
                      ↘ Destroyed

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| InInventory | Equipped | EquipItem Message + 需求满足 |
| Equipped | InInventory | UnequipItem Message |
| Equipped | Equipped | 替换旧装备（先脱后穿） |

---

# 不变量

## 不变量1：穿脱后 Trait 必须重建 🟥

穿脱操作完成后：

TraitCollection 必须反映当前装备状态。

违反表现：

卸下装备后仍拥有该装备的 Trait，属性计算错误。

架构违规检测：

发现穿脱后 TraitCollection 与装备状态不一致时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: 穿脱后 TraitCollection 未重建，装备 Trait 与实际状态不一致。
```

---

## 不变量2：修饰符来源精确追踪 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

穿脱操作完成后：

装备修饰符的 ModifierSource 必须与 instance_id 对应。

违反表现：

脱卸装备时无法精确移除对应修饰符，残留或误删。

---

## 不变量3：标签分层不混淆 🟥

任意时刻：

装备标签存储在 PersistentTags.from_equipment，不与 Trait/Buff 标签混淆。

违反表现：

Buff 过期时误删装备标签，或装备穿脱时误删 Trait 标签。

---

## 不变量4：槽位唯一性 🟥

任意时刻：

每个 EquipmentSlot 最多有一个装备实例。

违反表现：

同一槽位出现多个装备，属性叠加错误。

---

## 不变量5：修饰符必须通过 Modifier 管线 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

穿脱操作完成后：

装备的属性修饰必须通过 ModifierSource::equipment_source 添加到 Attributes，禁止直接修改属性值。

违反表现：

装备直接修改 HP、ATK 等最终属性值。

---

# 业务规则

## 规则1：穿戴 🟥

禁止：
- 🟥 跳过需求检查
- 🟥 跳过 Trait 重建
- 🟥 不处理旧装备替换

必须：
- 需求检查先于穿戴，失败发送 EquipFailed
- 槽位已占用时先脱卸旧装备
- 从背包移除装备
- 应用修饰符 + 标签 + Trait
- 重建 Trait 效果和 GameplayTags
- 发送 ItemEquipped Message

---

## 规则2：脱卸 🟥

禁止：
- 🟥 跳过修饰符清理
- 🟥 跳过 Trait 重建
- 🟥 不放回背包

必须：
- 移除修饰符（按 ModifierSource 精确移除）
- 移除装备授予的标签
- 移除装备授予的 Trait
- 清除槽位
- 创建 ItemStack 放回背包
- 重建 Trait 效果和 GameplayTags
- 发送 ItemUnequipped Message

---

## 规则3：需求检查 🟩

禁止：
- 跳过任何需求条件
- 需求检查后修改角色状态

必须：
- 按定义顺序检查
- 第一个不满足即返回 Failed
- RequireTag 检查 GameplayTags
- AttributeMin 检查属性值

---

## 规则4：装备效果应用 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

禁止：
- 🟥 直接修改角色基础属性
- 🟥 绕过 Modifier 管线

必须：
- 修饰符通过 ModifierSource::equipment_source 添加
- 标签添加到 PersistentTags.from_equipment
- Trait 添加到 TraitCollection（TraitSource::Equipment { slot }）

---

# 流程管线

## 穿戴管线

查找实例 → 查找定义 → 需求检查 → 脱旧装备 → 从背包移除 → 装备到槽位 → 应用效果 → 重建 Trait → 重建标签 → 发送 Message

### Step1：查找

输入：instance_id + EquipmentRegistry + Container
处理：查找装备实例和定义
输出：EquipmentInstance + EquipmentDef
🟩 禁止：实例或定义不存在时静默跳过

### Step2：需求检查

输入：EquipmentDef.requirements + Attributes + GameplayTags
处理：逐条检查需求
输出：Satisfied 或 Failed(reason)
🟥 禁止：检查失败时继续穿戴

### Step3：脱旧装备

输入：EquipmentSlots + 槽位
处理：如果槽位已占用，执行脱卸流程
输出：旧装备放回背包
🟥 禁止：跳过旧装备处理

### Step4：应用效果

输入：EquipmentDef.modifiers + tags + traits
处理：添加修饰符、标签、Trait
输出：属性和标签变化
🟥 禁止：绕过 Modifier 管线（宪法 2.2.1）

### Step5：重建

输入：TraitCollection + PersistentTags
处理：rebuild_trait_effects + rebuild_tags_from_components
输出：更新后的 Trait 和 GameplayTags
🟥 禁止：跳过重建

---

## 脱卸管线

检查槽位 → 移除修饰符 → 移除标签 → 移除 Trait → 清除槽位 → 放回背包 → 重建 Trait → 重建标签 → 发送 Message

### Step1：移除效果

输入：EquipmentDef + ModifierSource
处理：移除修饰符、标签、Trait
输出：属性和标签变化
🟥 禁止：遗漏任何修饰符

### Step2：放回背包

输入：EquipmentInstance
处理：创建 ItemStack 放入 Container
输出：背包更新
🟥 禁止：丢弃装备

### Step3：重建

输入：TraitCollection + PersistentTags
处理：rebuild_trait_effects + rebuild_tags_from_components
输出：更新后的 Trait 和 GameplayTags
🟥 禁止：跳过重建

---

# 数据结构

## EquipmentDef（Definition）

职责：装备的静态定义

结构：
- id / name / description：标识和展示
- slot：装备槽位（8 种）
- rarity：稀有度（5 级）
- tags：装备标签列表
- modifiers：属性修饰符列表
- traits：授予的 Trait ID 列表
- requirements：需求条件列表
- weight：重量

要求：
- 🟥 不可变，加载后不修改（宪法 1.1.2）
- 🟥 RON 配置路径：assets/equipment/（宪法 1.1.5）

---

## EquipmentInstance（Instance）

职责：装备的运行时实例

结构：
- instance_id：唯一实例 ID
- def_id：指向定义 ID
- durability / max_durability：耐久度
- enhance_level：强化等级
- enchantments：附魔 trait 列表

要求：
- 🟥 创建时 durability = max_durability
- 🟩 同一 Def 可有多个 Instance

---

## EquipmentSlots（Instance Component）

职责：单位装备槽位容器

结构：
- slots：槽位 → (实例ID, 定义ID) 映射
- next_instance_id：实例 ID 生成器

要求：
- 🟥 每个槽位最多一个装备
- 🟩 equip 返回被替换的旧装备
- 🟩 unequip 空槽位返回 None

---

# 禁止事项

🟥 禁止：跳过需求检查

原因：需求是装备平衡的核心机制

违反后果：低属性角色装备高级装备，游戏平衡被破坏

---

🟥 禁止：穿脱后跳过 Trait 重建

原因：装备提供 Trait，穿脱必须同步 TraitCollection

违反后果：角色拥有已卸下装备的 Trait，属性计算错误

---

🟥 禁止：直接修改角色基础属性

原因：装备效果必须走 Modifier 管线（宪法 2.2.1）

违反后果：属性变化无法追踪、无法回滚

架构违规检测：

```
ARCHITECTURE VIOLATION: 装备直接修改属性值，违反"属性修改必须通过 Modifier 管线"原则。
```

---

🟥 禁止：脱卸装备不放回背包

原因：装备是玩家资产，脱卸不是销毁

违反后果：装备丢失，玩家体验受损

---

🟥 禁止：装备标签与 Buff 标签混淆

原因：标签三层架构要求分层管理

违反后果：Buff 过期时误删装备标签

---

🟥 禁止：运行时修改 EquipmentDef

原因：宪法 1.1.2 Definition 不可变

违反后果：多个实例共享定义时产生意外副作用

架构违规检测：

```
ARCHITECTURE VIOLATION: 运行时修改 EquipmentDef，违反 Definition/Instance 分离原则。
```

---

# AI 修改规则

## 如果新增装备

允许：
- 新增 RON 配置文件
- 新增 EquipmentSlot 变体

禁止：
- 🟥 修改穿戴/脱卸流程
- 🟥 修改需求检查逻辑
- 🟥 修改 Trait 重建逻辑

优先检查：
- EquipmentRegistry 注册
- 修饰符 Source 区间是否冲突
- Trait ID 是否在 TraitRegistry 中
- 标签是否在 GameplayTag 枚举中

---

## 如果新增装备需求类型

允许：
- 新增 EquipmentRequirement 变体
- 在 check_equipment_requirements 中添加检查

禁止：
- 🟥 修改现有需求的检查逻辑
- 🟥 改变需求检查顺序

优先检查：
- RON 反序列化适配
- 需求检查短路逻辑

---

## 如果新增装备效果

允许：
- 新增修饰符类型
- 新增 Trait 授予

禁止：
- 🟥 绕过 Modifier 管线
- 🟥 跳过 Trait 重建

优先检查：
- ModifierSource 区间
- TraitCollection 来源追踪
- GameplayTags 重建

---

## 如果测试失败

排查顺序：
1. 检查需求检查是否全部通过
2. 检查修饰符是否正确添加/移除
3. 检查 Trait 是否正确重建
4. 检查标签是否分层正确
5. 检查旧装备是否正确替换

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证穿脱流程各步骤
- 🟩 集成测试：验证完整装备生命周期
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.2 Definition/Instance 分离 | EquipmentDef(Definition) vs EquipmentInstance(Instance) |
| 1.1.3 Rule/Content 分离 | 穿脱流程是规则，RON 配置是内容 |
| 1.1.5 数据驱动 | EquipmentDef 从 RON 加载 |
| 2.2.1 禁止直接修改最终属性 | 装备修饰通过 ModifierSource 添加 |
| 5.0 通信三原则 | EquipItem/UnequipItem Message |
| 6.0.2 Trait 用于扩展点 | 装备 = Modifier + Trait |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 穿脱后 Trait 未重建 | 代码审查 | ARCHITECTURE VIOLATION: 穿脱后 TraitCollection 未重建，装备 Trait 与实际状态不一致。 |
| 装备直接修改属性值 | 代码审查 | ARCHITECTURE VIOLATION: 装备直接修改属性值，违反"属性修改必须通过 Modifier 管线"原则。 |
| 运行时修改 EquipmentDef | 代码审查 | ARCHITECTURE VIOLATION: 运行时修改 EquipmentDef，违反 Definition/Instance 分离原则。 |
