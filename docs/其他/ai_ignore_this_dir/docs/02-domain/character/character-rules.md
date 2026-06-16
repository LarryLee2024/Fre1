---
id: 02-domain.character.character-rules
title: Character Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - character
---

# 角色系统领域

Version: 1.0
Status: Proposed

角色系统领域管理单位的创建、模板驱动生成、种族/职业/特质组合、生命周期标记和单位 Registry。

核心原则：
- 🟩 1.1.3 单位定义由 UnitTemplate 驱动，禁止代码硬编码（Rule/Content 分离）
- 🟩 1.1.2 Definition/Instance 分离：UnitTemplate 是不可变配置，Unit 是运行时实例
- 🟩 1.1.6 单位死亡标记 Dead 组件，禁止立即 despawn Entity（组合优先于继承）

---

# 术语定义

## 角色/单位（Unit/Character）

战场上可控制的实体，由 UnitTemplate 生成并挂载必需组件后生成 Entity。

不是 Entity 本身（Entity 是 ECS 底层概念）。不是玩家。不是 NPC。

关键属性：
- 由 Unit + UnitName + UnitId + GridPosition + Attributes + SkillSlots + TraitCollection 等组件组成
- 通过 Required Components 自动挂载 Attributes、SkillSlots、ActiveBuffs、GameplayTags、TraitCollection、EquipmentSlots、Container、GridPosition
- 有阵营（Faction：Player / Enemy）
- 有行动状态（acted 标志，控制每回合只能行动一次）

---

## 单位模板（UnitTemplate）

RON 文件中的不可变单位定义，包含种族、职业、初始属性、技能列表、特质列表和初始装备。

不是运行时实例。不是角色实例。不是 Entity。

关键属性：
- 定义态为 UnitTemplateDef（RON 反序列化用），运行态为 UnitTemplate
- 通过 UnitTemplateRegistry 按 ID 查询
- 包含：id、name、faction、race、background、class、base_attributes（8 维核心属性）、base_attack_range、skill_ids、trait_ids、ai_behavior、initial_equipment
- 从 assets/units/*.ron 加载（Rule/Content 分离）

---

## 种族（Race）

🟩 7.0.2 单位的种族分类，由 Trait 集合 + Modifier 集合构成，决定基础属性和种族特质。

不是职业。不是标签。不是 Class。

关键属性：
- 存储为 UnitRace 组件（String）
- 由 UnitTemplate 的 race 字段决定
- 种族影响初始特质（如"亡灵"种族可能提供亡灵特质）

---

## 职业（Class）

🟩 7.0.2 单位的职业分类，由成长率表 + 技能池 + Trait 集合构成，决定成长率和职业技能。

不是种族。不是技能。不是 Race。

关键属性：
- 存储为 UnitClass 组件（String）
- 由 UnitTemplate 的 class 字段决定
- 职业影响技能列表和特质组合

---

## 等级（Level）

单位的经验等级，影响属性成长。

不是经验值。不是能力。不是 Experience。

关键属性：
- 等级提升触发属性重新计算（通过属性系统的修饰器管线）
- 等级提升时 TraitCollection 需要重建
- 等级影响可学习的技能槽位上限

---

## 经验值（Experience）

积累到阈值后升级的数值资源。

不是等级。不是属性。不是 Level。

关键属性：
- 经验值达到阈值后触发等级提升
- 经验值是运行时累积值，不影响基础属性

---

## 特质（Trait）

参见 `attribute_modifier_rules.md#特质`。

🟩 7.0.1 角色内在的能力修改器，修改属性和提供被动效果。Trait 用于行为扩展，Modifier 用于数值扩展。

不是 Buff（特质是内在的）。不是效果。不是修饰器本身。

关键属性：
- 由种族、职业、天赋提供
- 变化时重建 TraitCollection
- 触发时机：Passive / OnAttack / OnHit / OnKill / OnTurnStart / OnTurnEnd
- 通过 TraitEffect → Effect Pipeline 执行

---

## 特质集合（TraitCollection）

单位上所有特质的集合组件，支持按来源精确增减。

不是 Vec<Trait>。不是单层数据。不是运行时临时计算结果。

关键属性：
- 是 Bevy Component，存储在 Unit Entity 上
- 包含 TraitEntry 列表（trait_id + TraitSource）
- TraitSource 区分来源：Intrinsic（种族/职业/天赋）或 Equipment（装备槽位）
- 支持 has()、add_entry()、remove_by_source()、trait_ids() 操作
- 装备变化或 Buff 变化时需要重建

---

## 必需组件集（Required Components）

Unit Entity 必须挂载的组件集合，通过 Bevy #[require] 宏自动挂载。

不是可选组件。不是运行时添加的。不是可省略的。

关键属性：
- Attributes — 属性组件（核心属性 + 生命资源 + 修饰器栈）
- SkillSlots — 技能槽位
- SkillCooldowns — 技能冷却追踪
- ActiveBuffs — 激活的 Buff 列表
- GameplayTags — 标签位掩码
- PersistentTags — 持久化标签（from_traits + from_equipment）
- TraitCollection — 特质集合
- EquipmentSlots — 装备槽位
- Container — 背包容器
- GridPosition — 格子坐标

---

## 阵营（Faction）

单位所属的战斗阵营，决定敌我关系。

不是种族。不是职业。不是标签。

关键属性：
- Player — 玩家方
- Enemy — 敌方
- 存储在 Unit.faction 字段
- 影响回合行动顺序编排和 AI 行为

---

## 死亡标记（Dead）

🟩 2.1.3 HP 降为 0 时添加的组件标记（Tag Component），触发 Hook 自动清理和 Observer 广播。

不是立即 despawn Entity。不是移除组件。不是 Despawn。

关键属性：
- 添加 Dead 后触发 on_add_dead Hook：标记 acted = true、移除 Selected
- 触发 Dead Observer：发送 CharacterDied Message
- Entity 保留但被排除在行动队列之外
- 死亡后由回合系统在回合结束时清理

---

## 持久化标签（PersistentTags）

不被 rebuild 丢失的标签存储，支持 Trait + Equipment 两层。

不是 GameplayTags（临时标签）。不是运行时计算结果。

关键属性：
- from_traits：Trait 授予的标签（种族/职业/天赋，最持久）
- from_equipment：装备授予的标签（穿脱变化）
- rebuild 时从两层合并到 GameplayTags

---

## 单位业务 ID（UnitId）

🟩 1.2.1 单位的逻辑标识符，用于业务逻辑查找。所有业务实体必须定义专属强类型标识。

不是 Name（Inspector 调试用）。不是 UnitName（UI 显示用）。不是 Entity。

关键属性：
- 存储为 UnitId 组件（String）
- 全局唯一，如 "player_warrior"、"enemy_goblin"
- 用于 UnitTemplateRegistry 查询和战斗日志记录

---

# 领域边界

## 本领域负责

- 单位创建流程（从 UnitTemplate 生成 Entity 并挂载组件）
- 单位模板定义和注册表管理（UnitTemplateRegistry）
- 种族/职业/特质的数据模型和组合规则
- 单位生命周期标记（Dead Hook、Selected、MovingUnit）
- 特质集合管理（TraitCollection：增删查改）
- 持久化标签管理（PersistentTags：Trait + Equipment 两层）
- 单位 Registry（运行时单位查询）

## 本领域不负责

- 属性计算和修饰器栈（由 Attribute Modifier 管线领域负责）
- 效果管线执行（由 Battle/Effect Pipeline 领域负责）
- 技能槽位管理和冷却（由 Skill 领域负责）
- 回合状态机和行动顺序（由 Turn Battle 领域负责）
- Buff 生命周期管理（由 Buff 领域负责）
- 装备穿脱逻辑（由 Equipment 领域负责）
- 用户输入处理（由 Input 领域负责）
- UI 展示与交互（由 UI 架构领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| CharacterDied | Message | Battle/Turn/UI 领域 |
| 单位属性查询 | 函数调用 | 所有领域 |
| 特质效果触发 | 函数调用 | Attribute Modifier 管线领域 |
| Dead 标记添加 | Hook + Observer | Turn（移除队列）、UI（日志） |
| 装备变化重建标签 | 函数调用 | Equipment 领域 |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Alive（默认） | 单位存活，可被选中行动 | Dead |
| Dead | 单位死亡标记，不可行动 | （终态，不转换） |

## 状态转换图

```
Alive → Dead
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Alive | Dead | HP 降为 0，插入 Dead 组件 |
| Dead | （无） | Dead 是终态，Entity 保留但被排除行动 |

---

# 不变量

## 不变量1：单位创建必须通过 UnitTemplate 🟥 1.1.3

任意时刻：

🟩 1.1.3 所有战斗单位必须通过 UnitTemplate 生成。禁止代码中硬编码单位属性值（如直接设置 base_attributes）。

违反表现：

spawn 代码中出现 `attributes.set_base(AttributeKind::Might, 5.0)` 而不来自模板。

---

## 不变量2：必需组件集完整

任意时刻：

Unit Entity 上必须挂载所有 Required Components（Attributes、SkillSlots、SkillCooldowns、ActiveBuffs、GameplayTags、PersistentTags、TraitCollection、EquipmentSlots、Container、GridPosition）。禁止运行时动态添加缺失的必需组件。

违反表现：

Unit Entity 上缺少 SkillSlots 或 Attributes 组件，导致系统查询失败。

---

## 不变量3：Dead 标记后不可行动

回合阶段流转：

Unit 添加 Dead 组件后，on_add_dead Hook 必须将 Unit.acted 设为 true，且移除 Selected 标记。

违反表现：

死亡单位仍被选中、死亡单位在行动队列中可被选中行动。

---

## 不变量4：Definition/Instance 分离 🟥 1.1.2

任意时刻：

🟩 1.1.2 UnitTemplate（定义态）不可变，不可被运行时修改。所有运行时变化通过修饰器管线作用于 Unit 实例的 Attributes。

违反表现：

战斗中修改 UnitTemplate 的 base_attributes 值，导致后续生成的单位也受影响。

---

## 不变量5：TraitCollection 与 GameplayTags 同步

任意时刻：

TraitCollection 变化（添加/移除特质）后，PersistentTags.from_traits 必须同步更新，GameplayTags 必须从 PersistentTags 两层合并重建。

违反表现：

TraitCollection 已变化但 GameplayTags 未更新，导致标签匹配错误。

---

## 不变量6：单位业务 ID 全局唯一

任意时刻：

所有运行时 Unit Entity 的 UnitId 值在单场战斗中必须唯一。

违反表现：

两个 Unit Entity 拥有相同的 UnitId，导致查找歧义。

---

# 业务规则

## 规则1：单位创建流程 🟩 1.1.3

禁止：
- 硬编码单位属性值（必须从 UnitTemplate 读取）
- 跳过 Required Components 自动挂载
- 生成 Unit 后不填充生命资源（fill_vital_resources）

必须：
- 从 UnitTemplate 读取 base_attributes 并调用 set_base 设置
- 调用 fill_vital_resources() 将当前 HP/MP/Stamina 设为最大值
- 应用 trait 被动效果（apply_passive_traits）授予标签和修饰器
- 穿戴初始装备并应用装备效果

允许：
- 模板不存在时跳过该单位并输出 error 日志
- 初始装备定义不存在时跳过该装备并输出 warn 日志

---

## 规则2：死亡处理流程

禁止：
- HP ≤ 0 时直接 despawn Entity
- 手动添加 Dead 组件而不触发 Hook 和 Observer
- 死亡后不广播 CharacterDied Message

必须：
- HP 降为 0 时只插入 Dead 组件（通过 Hook 自动清理）
- Dead Observer 统一发送 CharacterDied Message
- 死亡单位在回合结束时由 TurnOrder 重建排除

允许：
- Dead 标记后 Entity 保留用于 UI 展示（飘字、死亡动画）

---

## 规则3：Trait 系统管理

禁止：
- 绕过 TraitCollection 直接修改单位标签
- 硬编码特质效果（必须从 TraitRegistry 查询 TraitData）
- 跳过 TraitEffectHandlerRegistry 分发

必须：
- 特质效果通过 TraitEffectHandlerRegistry 查找处理器分发
- Passive 触发时机的特质在单位生成时应用（apply_passive_traits）
- TraitCollection 变化后重建 PersistentTags 和 GameplayTags

允许：
- 通过 TraitSource 区分来源进行精确移除

---

## 规则4：Definition/Instance 分离 🟥 1.1.2

禁止：
- 运行时修改 UnitTemplate 的任何字段
- 从 UnitTemplate 引用可变引用进行修改
- 将 UnitTemplate 存储在 Unit Entity 上作为 Component

必须：
- UnitTemplate 只读，通过 UnitTemplateRegistry.get() 获取不可变引用
- 运行时变化通过修饰器管线（add_modifier / remove_modifiers_from）作用于 Unit

允许：
- UnitTemplateRegistry 在游戏启动时一次性加载和注册

---

# 流程管线

## 单位创建管线

```
UnitTemplate RON → 反序列化 → 解析属性 → 创建 Entity → 挂载组件 → 应用特质 → 穿戴装备 → 填充生命资源
```

### Step1：加载 UnitTemplate

输入：assets/units/*.ron 文件路径
处理：通过 RegistryLoader 机制反序列化为 UnitTemplateDef，转换为 UnitTemplate
输出：UnitTemplate 实例
禁止：修改 UnitTemplateDef 的原始数据

### Step2：解析属性

输入：UnitTemplate
处理：从 base_attributes 构建 Attributes 组件，设置核心属性基础值和基础攻击范围
输出：Attributes 组件
禁止：跳过 set_base 直接赋值

### Step3：创建 Entity 并挂载组件

输入：模板数据 + 解析后的组件
处理：spawn Unit Entity，通过 #[require] 自动挂载必需组件
输出：Unit Entity
禁止：遗漏任何 Required Component

### Step4：应用特质效果

输入：TraitCollection + TraitRegistry + TraitEffectHandlerRegistry
处理：apply_passive_traits 授予标签和属性修饰器
输出：GameplayTags + AttributeModifierInstance 列表
禁止：跳过 Passive 触发时机检查

### Step5：穿戴初始装备

输入：initial_equipment 列表 + EquipmentRegistry
处理：逐件装备到 EquipmentSlots，应用装备效果（修饰器 + 标签）
输出：更新后的 Attributes + PersistentTags + TraitCollection
禁止：装备不存在时静默跳过（必须输出日志）

### Step6：填充生命资源

输入：Attributes 组件
处理：fill_vital_resources() 将当前 HP/MP/Stamina 设为最大值
输出：完整的生命资源值
禁止：跳过此步骤（单位将以 0 HP 生成）

---

## 等级提升管线

```
经验累积 → 触发升级 → 属性成长 → TraitCollection 重建 → ViewModel 更新
```

### Step1：经验累积

输入：战斗经验获取事件
处理：累加单位经验值
输出：更新后的经验值
禁止：经验值超过阈值时不触发升级

### Step2：触发升级

输入：经验值 + 升级阈值
处理：判定是否达到升级条件，递增等级
输出：新的等级值
禁止：跳过等级递增

### Step3：属性成长

输入：新的等级值 + 职业成长率
处理：通过属性系统的修饰器管线重新计算属性
输出：更新后的 Attributes
禁止：直接修改 base 值（必须通过修饰器管线）

### Step4：TraitCollection 重建

输入：新等级解锁的特质
处理：重建 TraitCollection，重新应用被动效果
输出：更新后的 TraitCollection + GameplayTags
禁止：跳过标签重建

---

# 数据结构

## UnitTemplate（单位模板-运行时）

职责：存储从 RON 加载的不可变单位定义

结构：
- id：String — 业务标识符（如 "player_warrior"）
- name：String — 显示名称（如 "战士"）
- faction：Faction — 阵营
- race：String — 种族名称
- background：String — 背景描述
- class：String — 职业名称
- base_attributes：HashMap — 8 维核心属性基础值
- base_attack_range：u32 — 基础攻击范围
- skill_ids：Vec — 初始技能 ID 列表
- trait_ids：Vec — 初始特质 ID 列表
- ai_behavior：String — AI 行为模板 ID
- initial_equipment：Vec — 初始装备（槽位 → 装备定义 ID）

要求：
- 定义态不可变，禁止运行时修改
- 通过 UnitTemplateRegistry.get() 获取不可变引用
- 从 assets/units/*.ron 加载

---

## UnitTemplateDef（单位模板-反序列化用）

职责：RON 反序列化中间态，转换为 UnitTemplate

结构：
- version：u32 — 配置版本号（默认 0）
- 其余字段同 UnitTemplate

要求：
- 实现 From<UnitTemplateDef> for UnitTemplate
- version 缺失时默认为 0（兼容旧配置）

---

## TraitCollection（特质集合组件）

职责：管理单位的所有特质条目，支持按来源精确增减

结构：
- entries：Vec — TraitEntry 列表（trait_id + TraitSource）

要求：
- 是 Bevy Component
- new(trait_ids) 将所有条目标记为 Intrinsic 来源
- add_entry(trait_id, source) 添加条目
- remove_by_source(source) 按来源精确移除
- has(trait_id) 查询是否存在
- trait_ids() 返回去重的 trait_id 列表

---

## PersistentTags（持久化标签组件）

职责：存储 Trait 和 Equipment 授予的标签，rebuild 时不丢失

结构：
- from_traits：GameplayTags — Trait 授予的标签
- from_equipment：GameplayTags — 装备授予的标签

要求：
- 两层标签合并后赋值给 GameplayTags 组件
- Trait 变化时更新 from_traits
- 装备变化时更新 from_equipment

---

## Dead（死亡标记组件）

职责：标记单位死亡，触发 Hook 和 Observer

结构：
- 无字段（Tag 组件）

要求：
- on_add_dead Hook：标记 acted = true，移除 Selected
- Dead Observer：发送 CharacterDied Message
- 不立即 despawn Entity

---

## UnitId（单位业务 ID 组件）

职责：单位的逻辑标识符，用于业务查找和日志

结构：
- 0：String — 业务 ID（如 "player_warrior"）

要求：
- 全局唯一（单场战斗内）
- 实现 Hash + Eq，支持 HashSet/HashMap 使用
- 用于 UnitTemplateRegistry 查询和战斗日志

---

# 禁止事项

禁止：代码中硬编码单位属性值

原因：违反数据驱动原则，新增/修改单位需要修改 Rust 代码而非 RON 配置

违反后果：内容扩展需要修改代码、热重载失效、多场战斗数据不一致

---

禁止：立即 despawn 死亡单位 Entity

原因：死亡单位需要保留用于 UI 展示（飘字、死亡动画）、回合结算、战斗日志

违反后果：死亡单位瞬间消失、战斗日志缺少死亡记录、回合结算时查询失败

---

禁止：运行时修改 UnitTemplate 定义

原因：Definition/Instance 分离是架构核心原则，修改定义会污染全局配置

违反后果：后续生成的单位使用被污染的配置、热重载失效

---

禁止：绕过 TraitCollection 直接修改 GameplayTags

原因：标签由 Trait 和 Equipment 两层授予，直接修改会破坏一致性

违反后果：标签与特质不同步、ModifierRule 匹配错误、战斗日志不准确

---

禁止：Unit Entity 缺少必需组件

原因：Required Components 是系统查询的基础，缺失会导致系统 panic

违反后果：ECS 查询失败、运行时 panic、系统无法执行

---

禁止：Trait 效果绕过 TraitEffectHandlerRegistry 分发

原因：Handler 分发是特质系统的扩展点，硬编码破坏扩展性

违反后果：新增特质效果类型需要修改分发代码，违反开闭原则

---

禁止：在 Character 领域内执行效果管线

原因：效果管线执行属于 Battle/Effect Pipeline 领域职责

违反后果：角色系统与战斗系统耦合、无法独立测试

---

禁止：UnitTemplate 不设置 version 字段

原因：version 字段用于配置版本管理和兼容性检查

违反后果：旧配置无法识别、版本迁移失败

---

# AI 修改规则

## 宪法合规检查清单

修改本领域代码前，必须逐项确认：
- 🟩 1.1.2 Definition/Instance 分离：UnitTemplate 不可变，运行时变化通过修饰器管线
- 🟩 1.1.3 Rule/Content 分离：单位定义通过 RON 配置，不硬编码
- 🟩 1.1.6 组合优先于继承：角色差异化通过 Component + Trait + Modifier 组合实现
- 🟩 1.2.1 使用强类型 ID（UnitId），不裸传 Entity
- 🟩 7.0.1 分层扩展：Trait 用于行为扩展，Modifier 用于数值扩展
- 🟩 7.0.2 核心定义：种族=Trait+Modifier，职业=成长率+技能池+Trait

## 领域事件清单

本领域产生的领域事件（🟩 2.2.6 领域事件是唯一业务事实源）：
- `UnitSpawned` — 单位生成，携带 unit_id、template_id、faction
- `UnitDied` — 单位死亡，携带 unit_id、killer_id
- `TraitApplied` — 特质效果应用，携带 unit_id、trait_id、effect
- `LevelUp` — 等级提升，携带 unit_id、old_level、new_level

> 🟩 2.2.7 新增领域事件必须先更新白名单文档
> 🟩 13.10.1 所有核心业务事实通过领域事件表达，日志、回放、UI 均监听同一事件源

## 如果新增单位模板

允许：
- 在 assets/units/*.ron 中添加新的 RON 配置文件
- 确保 id、name、faction、base_attributes、skill_ids、trait_ids 完整

禁止：
- 在 spawn 代码中硬编码新单位属性
- 修改 UnitTemplate 结构添加新字段（除非所有模板同步更新）

优先检查：
- UnitTemplateRegistry.register_defaults() 是否包含新模板的默认值
- base_attributes 是否包含所有 8 维核心属性
- trait_ids 和 skill_ids 中的 ID 是否在对应 Registry 中注册

---

## 如果修改特质系统

允许：
- 在 assets/traits/*.ron 中添加新的 TraitDefinition
- 新增 TraitEffect 变体（需同步修改 TraitEffectHandlerRegistry）

禁止：
- 修改 apply_passive_traits 的签名（影响所有调用方）
- 绕过 TraitEffectHandlerRegistry 直接处理特质效果

优先检查：
- TraitEffectHandlerRegistry 中是否注册了新效果类型的处理器
- Passive 触发时机是否正确（非 Passive 特质不应在生成时应用）
- TraitCollection 变化后 GameplayTags 是否同步重建

---

## 如果修改单位创建流程

允许：
- 在 spawn_unit_from_template 中添加新的组件挂载
- 修改 fill_vital_resources 的调用位置

禁止：
- 跳过任何 Required Component 的挂载
- 跳过 fill_vital_resources 调用
- 在创建流程中执行游戏逻辑（只做数据准备）

优先检查：
- 所有 Required Components 是否通过 #[require] 自动挂载
- 特质被动效果是否在装备穿戴前应用（避免重复计算）
- 初始装备效果是否正确应用到 PersistentTags 和 TraitCollection

---

## 如果修改死亡处理

允许：
- 修改 on_add_dead Hook 中的清理逻辑
- 调整 Dead Observer 的广播内容

禁止：
- 直接 despawn 死亡单位 Entity
- 移除 Dead Observer（死亡通知必须广播）
- 修改 Dead 标记的语义（不是立即清除）

优先检查：
- Dead Hook 是否正确标记 acted = true
- Dead Observer 是否发送 CharacterDied Message
- TurnOrder 重建时是否排除 Dead 单位

---

## 如果测试失败

排查顺序：
1. 检查 Unit Template 中的 ID 是否在 Registry 中注册
2. 检查 Required Components 是否完整挂载
3. 检查 fill_vital_resources 是否被调用
4. 检查 TraitCollection 变化后 GameplayTags 是否同步
5. 检查 Dead 标记后 Hook 和 Observer 是否触发
