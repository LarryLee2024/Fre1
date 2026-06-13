# Trait 领域

Version: 1.1

Trait 领域管理角色能力的统一扩展机制。种族、职业、天赋、装备、Buff 均通过 Trait + Modifier 管线影响角色。

核心原则：
- 🟥 Trait 表示能力，不表示分类
- 🟥 组合优于继承（宪法 1.1.10）
- 🟥 统一扩展机制（宪法 1.1.6）
- 🟥 Handler 分发（宪法 6.0.2）

---

# 术语定义

## TraitData

Trait 的运行时数据，包含触发时机和效果列表。

不是 TraitCollection。TraitData 是定义，TraitCollection 是实例集合。

关键属性：
- id / name / description：标识和展示
- trigger：触发时机
- effects：效果列表

---

## TraitTrigger

Trait 的触发时机，决定"什么时候生效"。

不是 TraitSource。Trigger 标记"何时触发"，Source 标记"从哪来"。

关键属性：
- Passive：始终生效
- OnTurnStart / OnTurnEnd：回合触发
- OnAttack / OnHit / OnKill：战斗触发

---

## TraitEffect

Trait 的效果类型，决定"做什么"。

不是 TraitData。Effect 是 Trait 的组成部分，TraitData 包含多个 Effect。

关键属性：
- GrantTag：授予标签
- ModifyAttribute：属性修饰
- ApplyBuff：触发时施加 Buff

---

## TraitEffectHandler

效果处理器，执行具体效果逻辑。

不是 TraitEffect。Handler 是执行者，Effect 是配置。

关键属性：
- type_name()：分发键
- granted_tags()：提取授予的标签
- attribute_modifiers()：提取属性修饰

---

## TraitSource

Trait 的来源标记，区分内在来源和装备来源。

不是 TraitTrigger。Source 标记"从哪来"，Trigger 标记"何时触发"。

关键属性：
- Intrinsic：种族/职业/天赋
- Equipment { slot }：装备，记录槽位

---

## TraitCollection

单位拥有的 Trait 条目集合。

不是 TraitRegistry。Collection 是实例，Registry 是定义注册表。

关键属性：
- entries：TraitEntry 列表

---

# 领域边界

## 本领域负责

- TraitData 定义和注册表（TraitRegistry）
- TraitEffectHandler 注册表（TraitEffectHandlerRegistry）
- TraitTrigger 和 TraitEffect 的对应关系
- TraitCollection 的增删管理
- apply_passive_traits 被动效果应用
- Trait 重建（rebuild_trait_effects）

## 本领域不负责

- 属性计算和修饰符管线（由 stat_system 领域负责）
- 装备穿脱触发（由 equipment_rules 领域负责）
- Buff 的生命周期（由 buff_rules 领域负责）
- 战斗管线中的 Trait 触发调用（由 battle_rules 领域负责）
- UI 展示（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 被动效果应用 | 返回值（标签+修饰符） | character |
| Trait 重建 | 直接函数调用 | equipment |
| 触发型效果 | 推入 EffectQueue | battle |

---

# 生命周期

本领域无状态机，为纯函数式计算。

Trait 生命周期由外部驱动：
- 生成时：apply_passive_traits 应用被动效果
- 穿脱时：rebuild_trait_effects 重建
- 战斗时：trigger_traits 触发效果

---

# 不变量

## 不变量1：Passive 效果仅 GrantTag 和 ModifyAttribute 🟥

任意时刻：

Passive 触发的 Trait 只产生 GrantTag 和 ModifyAttribute 效果。

违反表现：

Passive Trait 产生 ApplyBuff 效果，无触发时机，永远不会执行。

---

## 不变量2：触发型效果仅 ApplyBuff 🟥

任意时刻：

OnAttack/OnHit/OnKill/OnTurnStart/OnTurnEnd 触发的 Trait 只产生 ApplyBuff 效果。

违反表现：

触发型 Trait 产生 GrantTag/ModifyAttribute，标签和修饰符在触发时临时添加后无法正确移除。

---

## 不变量3：Handler 覆盖所有效果类型 🟥

任意时刻：

TraitEffect 的每个变体都有对应的 TraitEffectHandler 注册。

违反表现：

新增效果类型但未注册 Handler，apply_passive_traits 跳过该效果。

---

## 不变量4：修饰符 Source 区间隔离 🟥

apply_passive_traits 完成后：

Trait 修饰符的 ModifierSource 在 Trait 区间内（u64::MAX ~ u64::MAX - 999）。

违反表现：

Trait 修饰符与 Buff/Equipment 修饰符冲突，无法按来源精确移除。

---

## 不变量5：修饰符必须通过 Modifier 管线 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

Trait 的 ModifyAttribute 效果完成后：

属性修饰必须通过 ModifierSource::trait_source 添加到 Attributes，禁止直接修改属性值。

违反表现：

Trait 直接修改 HP、ATK 等最终属性值。

架构违规检测：

发现 Trait 直接修改属性值时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: Trait 直接修改属性值，违反"属性修改必须通过 Modifier 管线"原则。
```

---

# 业务规则

## 规则1：效果与触发器对应 🟥

禁止：
- 🟥 Passive Trait 使用 ApplyBuff 效果
- 🟥 触发型 Trait 使用 GrantTag / ModifyAttribute 效果

必须：
- Passive → GrantTag + ModifyAttribute
- 触发型 → ApplyBuff

---

## 规则2：Handler 分发 🟥

宪法依据：6.0.2（Trait 用于扩展点）

禁止：
- 🟥 match 分发效果类型
- 🟥 为每种效果类型修改 TraitData 方法

必须：
- 通过 type_name() 查找 Handler
- 新增效果类型只需实现 Handler 并注册

允许：
- 🟩 默认注册三个内置 Handler

---

## 规则3：来源追踪 🟥

禁止：
- 🟥 不区分来源直接增删 Trait
- 🟥 装备穿脱时误删内在 Trait

必须：
- Intrinsic 标记种族/职业/天赋
- Equipment { slot } 标记装备来源
- 脱卸装备时按 source 精确移除

---

## 规则4：Trait 重建 🟥

禁止：
- 🟥 穿脱装备后跳过 Trait 重建
- 🟥 增量更新 Trait 效果

必须：
- 清除所有 Trait 来源修饰符
- 清除 Trait 授予的标签
- 重新应用所有 Passive Trait
- 重建 GameplayTags

---

# 流程管线

## 被动 Trait 应用管线

遍历 entries → 跳过非 Passive → Handler 收集标签 → Handler 收集修饰符 → 分配 Source → 返回

### Step1：遍历 entries

输入：TraitCollection.entries
处理：逐条检查 trigger
输出：Passive 触发的 entries
🟥 禁止：处理非 Passive 触发

### Step2：Handler 收集

输入：TraitEffect + HandlerRegistry
处理：通过 type_name() 查找 Handler，调用 granted_tags / attribute_modifiers
输出：标签集合 + 修饰符列表
🟥 禁止：跳过任何效果

### Step3：分配 Source

输入：修饰符列表 + index
处理：每个 Trait 分配 ModifierSource::trait_source(index)
输出：带 Source 的修饰符实例
🟥 禁止：Source 区间与其他来源冲突

---

## Trait 重建管线

清除修饰符 → 清除标签 → 重新应用 Passive → 重建 GameplayTags

### Step1：清除

输入：Attributes + PersistentTags
处理：remove_trait_modifiers()，重置 from_traits
输出：清除后的状态
🟥 禁止：清除 from_equipment（装备层不受影响）

### Step2：重新应用

输入：TraitCollection + TraitRegistry + HandlerRegistry
处理：apply_passive_traits()
输出：新标签 + 新修饰符
🟥 禁止：跳过任何 Passive Trait

### Step3：重建标签

输入：PersistentTags
处理：from_traits | from_equipment
输出：更新后的 GameplayTags
🟥 禁止：包含 Buff 层标签

---

# 数据结构

## TraitData（Definition）

职责：Trait 的运行时数据

结构：
- id / name / description：标识和展示
- trigger：触发时机
- effects：效果列表

要求：
- 🟩 通过 Handler 分发收集标签和修饰符
- 🟥 RON 配置路径：assets/traits/（宪法 1.1.5）

---

## TraitEffect（值对象）

职责：Trait 的效果类型

结构：
- GrantTag(GameplayTag)：授予标签
- ModifyAttribute(AttributeModifierDef)：属性修饰
- ApplyBuff { buff_id, duration }：触发时施加 Buff

要求：
- 🟥 与 TraitTrigger 严格对应
- 🟩 type_name() 返回分发键

---

## TraitEffectHandler（Trait）

职责：效果处理器接口

结构：
- type_name()：分发键
- granted_tags()：提取授予的标签
- attribute_modifiers()：提取属性修饰

要求：
- 🟥 新增效果类型只需实现并注册（宪法 6.0.2）
- 🟩 内置三个 Handler

---

## TraitCollection（Instance Component）

职责：单位 Trait 条目集合

结构：
- entries：TraitEntry 列表

要求：
- 🟩 add_entry 记录来源
- 🟥 remove_by_source 精确清理
- 🟩 trait_ids() 返回去重列表

---

## TraitSource（值对象）

职责：Trait 来源标记

结构：
- Intrinsic：内在来源
- Equipment { slot }：装备来源

要求：
- 🟥 装备穿脱时使用 Equipment 变体
- 🟥 内在 Trait 使用 Intrinsic 变体

---

# 禁止事项

🟥 禁止：为每种能力来源写独立逻辑

原因：Trait 是统一扩展机制，所有来源走同一管线（宪法 1.1.6）

违反后果：种族/职业/装备各有独立逻辑，维护成本指数增长

---

🟥 禁止：match 分发效果类型

原因：Handler 通过 type_name() 分发，无需 match（宪法 6.0.2）

违反后果：新增效果类型需要修改分发代码

架构违规检测：

```
ARCHITECTURE VIOLATION: match 分发效果类型，违反"Handler 通过 type_name 分发"原则。
```

---

🟥 禁止：Passive Trait 使用 ApplyBuff 效果

原因：Passive 无触发时机，ApplyBuff 永远不会执行

违反后果：效果配置错误，Trait 不生效

---

🟥 禁止：穿脱装备后跳过 Trait 重建

原因：装备提供 Trait，穿脱必须同步 TraitCollection

违反后果：角色拥有已卸下装备的 Trait

---

🟥 禁止：增量更新 Trait 效果

原因：增量更新容易遗漏，完全重建保证一致性

违反后果：标签和修饰符与实际 Trait 状态不一致

---

🟥 禁止：Trait 直接修改属性值

原因：宪法 2.2.1 禁止直接修改最终属性值

违反后果：属性变化无法追踪、无法回滚

架构违规检测：

```
ARCHITECTURE VIOLATION: Trait 直接修改属性值，违反"属性修改必须通过 Modifier 管线"原则。
```

---

# AI 修改规则

## 如果新增 Trait 效果类型

允许：
- 新增 TraitEffect 变体
- 新增 TraitEffectHandler 实现并注册

禁止：
- 🟥 修改 TraitData 方法
- 🟥 修改 apply_passive_traits 流程
- 🟥 修改 rebuild_trait_effects 流程

优先检查：
- TraitEffectHandlerRegistry 注册
- type_name() 是否与变体名一致
- 与 TraitTrigger 的对应关系

---

## 如果新增 Trait 触发时机

允许：
- 新增 TraitTrigger 变体
- 在对应阶段添加触发调用

禁止：
- 🟥 修改现有触发器的效果对应关系
- 🟥 修改 Handler 分发逻辑

优先检查：
- 触发位置（battle / turn）
- EffectQueue 是否可用
- 触发目标

---

## 如果新增 Trait

允许：
- 新增 RON 配置文件

禁止：
- 🟥 修改 TraitData 结构
- 🟥 修改 TraitRegistry 加载流程

优先检查：
- TraitRegistry 注册
- 效果类型是否有对应 Handler
- 标签是否在 GameplayTag 枚举中

---

## 如果测试失败

排查顺序：
1. 检查 TraitTrigger 与 TraitEffect 对应关系
2. 检查 Handler 是否正确注册
3. 检查 ModifierSource 区间是否冲突
4. 检查 Trait 重建是否完整
5. 检查来源追踪是否正确

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证 Handler 分发和效果收集
- 🟩 集成测试：验证 Trait 重建流程
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.6 统一扩展体系 | Trait + Modifier 统一所有能力来源 |
| 1.1.10 组合优于继承 | Trait 组合替代继承树 |
| 1.1.5 数据驱动 | TraitData 从 RON 加载 |
| 2.2.1 禁止直接修改最终属性 | Trait 修饰通过 ModifierSource 添加 |
| 6.0.2 Trait 用于扩展点 | TraitEffectHandler trait |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| match 分发效果类型 | 代码审查 | ARCHITECTURE VIOLATION: match 分发效果类型，违反"Handler 通过 type_name 分发"原则。 |
| Trait 直接修改属性值 | 代码审查 | ARCHITECTURE VIOLATION: Trait 直接修改属性值，违反"属性修改必须通过 Modifier 管线"原则。 |
