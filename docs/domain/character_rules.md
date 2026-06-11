# Character 领域

Version: 1.0

Character 领域管理战场单位的定义、生成、身份标识、阵营划分、位置管理和 Trait 扩展。

核心原则：
- Definition / Instance 分离
- 组合优于继承（Trait + Modifier 组合能力）
- Entity 只是 ID

---

# 术语定义

## Unit

战场上的战斗单位实例，拥有运行时状态。

不是 UnitTemplate。UnitTemplate 是配置，Unit 是运行时。

关键属性：
- faction：阵营归属（Player / Enemy）
- acted：本回合是否已行动

---

## UnitTemplate

单位的配置定义，描述一个单位"是什么"。

不是 Unit。UnitTemplate 不可变，Unit 可变。

关键属性：
- id：唯一标识
- base_attributes：仅 8 维核心属性
- skill_ids / trait_ids：引用 ID 列表
- initial_equipment：初始装备映射

---

## UnitTemplateRegistry

所有 UnitTemplate 的注册表，启动时从 RON 加载。

不是运行时状态容器。仅提供查找，不存储实例。

关键属性：
- templates：id → UnitTemplate 映射

---

## TraitCollection

单位拥有的 Trait 条目集合，按来源分组管理。

不是 TraitRegistry。TraitCollection 是实例，TraitRegistry 是定义。

关键属性：
- entries：TraitEntry 列表，每条记录 trait_id + source

---

## TraitSource

Trait 的来源标记，区分内在来源和装备来源。

不是 TraitTrigger。TraitSource 标记"从哪来"，TraitTrigger 标记"何时触发"。

关键属性：
- Intrinsic：种族/职业/天赋
- Equipment { slot }：装备，记录具体槽位

---

## GridPosition

单位在地图上的格子坐标。

不是 Transform。GridPosition 是逻辑坐标，Transform 是渲染位置。

关键属性：
- coord：IVec2 格子坐标

---

## Faction

阵营枚举，决定敌我关系。

不是 Team。本项目只有 Player / Enemy 两个阵营。

关键属性：
- Player：玩家方
- Enemy：敌方

---

## Dead

死亡标记 Tag Component，表示单位已死亡。

不是 is_dead: bool。Tag Component 优于 bool 字段。

关键属性：
- on_add Hook：自动标记 acted = true，移除 Selected

---

## MovingUnit

移动动画组件，挂在正在移动的单位上。

不是 GridPosition 变化。MovingUnit 控制动画过程，GridPosition 在动画完成后更新。

关键属性：
- path：路径坐标序列
- next_phase：移动完成后的回调阶段

---

## PersistentTags

持久化标签集合，分两层追踪来源。

不是 GameplayTags。PersistentTags 是容器，GameplayTags 是最终合并结果。

关键属性：
- from_traits：Trait 授予的标签
- from_equipment：装备授予的标签

---

# 领域边界

## 本领域负责

- 单位定义（UnitTemplate）和注册表（UnitTemplateRegistry）
- 单位实例（Unit）的生成和身份组件
- 阵营（Faction）和格子坐标（GridPosition）
- 死亡标记（Dead）的 Hook 行为
- TraitCollection 的增删管理
- 移动动画（MovingUnit）
- PersistentTags 的来源分层

## 本领域不负责

- 属性计算（由 stat_system 领域负责）
- Trait 定义和效果处理（由 trait_rules 领域负责）
- 装备穿脱逻辑（由 equipment_rules 领域负责）
- Buff 管理（由 buff_rules 领域负责）
- 伤害和死亡判定（由 battle_rules 领域负责）
- 移动范围计算和寻路（由 map_rules 领域负责）
- UI 展示（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 单位生成完成 | 直接函数调用 | stat / trait / equipment |
| 单位死亡 | Dead Tag Hook | battle / ui |
| 移动完成 | TurnPhase 切换 | turn |
| Trait 变化 | rebuild_trait_effects | trait |

---

# 生命周期

## 单位生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Template | 配置定义，不可变 | — |
| Spawned | 已生成，在战场上 | Acting, Dead |
| Acting | 正在行动（移动/攻击） | Spawned, Dead |
| Dead | 已死亡 | — |

## 状态转换图

Template → Spawned → Acting → Spawned
                  ↘ Dead

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Template | Spawned | OnEnter(InGame) 触发 spawn_units |
| Spawned | Acting | 玩家/AI 选择该单位行动 |
| Acting | Spawned | 行动完成（Wait / 攻击结束） |
| Acting | Dead | HP ≤ 0 且 Dead Tag 被添加 |
| Spawned | Dead | HP ≤ 0 且 Dead Tag 被添加 |

## 移动动画生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Idle | 无移动 | Moving |
| Moving | 正在播放移动动画 | Idle |

| 从 | 到 | 条件 |
|----|-----|------|
| Idle | Moving | 插入 MovingUnit 组件 |
| Moving | Idle | is_finished() = true，移除 MovingUnit |

---

# 不变量

## 不变量1：Unit 完整性

任意时刻：

每个拥有 Unit 组件的 Entity 必须同时拥有 Attributes, SkillSlots, SkillCooldowns, ActiveBuffs, GameplayTags, PersistentTags, TraitCollection, EquipmentSlots, Container, GridPosition。

违反表现：

单位缺少必要组件，系统访问时 panic。

---

## 不变量2：UnitTemplate 不可变

任意时刻：

UnitTemplate 的内容在加载后不可被修改。

违反表现：

多个单位共享同一模板时，修改模板影响所有实例。

---

## 不变量3：Dead 标记一致性

任意时刻：

如果单位 HP ≤ 0，则必须拥有 Dead 组件。

违反表现：

死亡单位仍可行动、被选中、被攻击。

---

## 不变量4：TraitCollection 来源一致

TraitCollection 操作完成后：

每个 TraitEntry 的 source 必须与实际来源一致。

违反表现：

装备穿脱后残留旧 Trait，或内在 Trait 被误删。

---

## 不变量5：GridPosition 与 OccupancyGrid 同步

移动动画完成后：

GridPosition 必须与 OccupancyGrid 中的占用记录一致。

违反表现：

单位逻辑坐标与地图占用不一致，寻路和攻击范围计算错误。

---

# 业务规则

## 规则1：单位生成

禁止：
- 跳过 Required Components
- 运行时创建新的 UnitTemplate
- 直接修改 UnitTemplate

必须：
- 通过 UnitTemplate 生成运行时实例
- 生成时调用 fill_vital_resources() 推导 Vital Resources
- 生成时调用 apply_passive_traits() 应用被动 Trait
- 生成时穿戴 initial_equipment（跳过需求检查）
- 穿戴装备后调用 rebuild_trait_effects()

允许：
- 无配置文件时使用内置默认模板

---

## 规则2：阵营归属

禁止：
- 在运行时改变单位阵营

必须：
- faction 决定敌我关系
- 阵营颜色由 faction_color() 统一映射

允许：
- AI 行为根据阵营选择策略

---

## 规则3：死亡处理

禁止：
- 直接删除 Entity
- 跳过 Dead Tag 直接处理死亡逻辑
- 在 HP 变化时内联死亡处理

必须：
- HP ≤ 0 时添加 Dead Tag Component
- Dead Hook 自动标记 acted = true
- Dead Hook 自动移除 Selected

允许：
- Observer 响应死亡（播放动画、刷新 UI）
- Message 广播死亡（CharacterDied）

---

## 规则4：TraitCollection 管理

禁止：
- 为每种能力来源写独立逻辑
- 硬编码 Trait 效果
- 装备穿脱时不更新 TraitCollection

必须：
- Trait 变化时重建 TraitCollection
- 装备穿脱时调用 remove_by_source(Equipment { slot })
- 内在 Trait 标记为 Intrinsic，不被装备操作误删

允许：
- 通过 TraitEffectHandler 分发效果

---

## 规则5：PersistentTags 分层

禁止：
- 不区分来源直接修改 GameplayTags

必须：
- from_traits 和 from_equipment 分层追踪
- 装备穿脱只修改 from_equipment
- 最终 GameplayTags = from_traits | from_equipment

---

## 规则6：移动动画

禁止：
- 在动画未完成时更新 GridPosition
- 在动画未完成时切换 TurnPhase

必须：
- 逐格线性插值
- is_finished() 后才更新 GridPosition
- 动画完成后移除 MovingUnit 和 PathArrow

允许：
- speed 控制每格耗时

---

# 流程管线

## 单位生成管线

查找模板 → 构建属性 → 构建 Trait → 应用被动 Trait → 穿戴初始装备 → 重建 Trait 效果 → 重建 GameplayTags → Spawn Entity

### Step1：查找模板

输入：LevelConfig 中的 deploy.template
处理：从 UnitTemplateRegistry 查找
输出：UnitTemplate
禁止：模板不存在时静默跳过

### Step2：构建属性

输入：UnitTemplate.base_attributes
处理：设置 8 维核心属性基础值，调用 fill_vital_resources()
输出：Attributes 组件
禁止：直接设置 Vital Resources 基础值

### Step3：构建 Trait

输入：UnitTemplate.trait_ids
处理：创建 TraitCollection，全部标记为 Intrinsic
输出：TraitCollection 组件
禁止：跳过 Intrinsic 标记

### Step4：应用被动 Trait

输入：TraitCollection + TraitRegistry + HandlerRegistry
处理：收集 Passive 触发的标签和属性修饰符
输出：GameplayTags + AttributeModifierInstance 列表
禁止：处理非 Passive 触发类型

### Step5：穿戴初始装备

输入：UnitTemplate.initial_equipment
处理：跳过需求检查直接装备
输出：EquipmentSlots 更新
禁止：对初始装备执行需求检查

### Step6：重建 Trait 效果

输入：装备后的 TraitCollection
处理：rebuild_trait_effects()
输出：更新后的 GameplayTags 和 Attributes
禁止：跳过此步骤

### Step7：Spawn Entity

输入：所有组件
处理：插入 Unit + Required Components + 子实体
输出：Unit Entity
禁止：遗漏任何 Required Component

---

# 数据结构

## Unit（Instance）

职责：战斗单位的核心身份组件

结构：
- faction：阵营枚举 — Player / Enemy
- acted：布尔 — 本回合是否已行动

要求：
- 生成时自动插入 9 个 Required Components
- acted 在回合开始时重置为 false

---

## UnitTemplate（Definition）

职责：单位的配置定义

结构：
- id：字符串 — 唯一标识
- name：字符串 — UI 显示名称
- faction：阵营 — Player / Enemy
- race：字符串 — 种族标识
- class：字符串 — 职业标识
- base_attributes：映射 — 仅 8 维核心属性
- base_attack_range：整数 — 基础攻击范围
- skill_ids：字符串列表 — 技能引用
- trait_ids：字符串列表 — Trait 引用
- ai_behavior：字符串 — AI 行为配置 ID
- initial_equipment：槽位→装备ID 列表 — 初始装备

要求：
- 不可变，加载后不可修改
- version 字段使用 serde(default)，旧配置兼容

---

## TraitCollection（Instance）

职责：单位拥有的 Trait 条目集合

结构：
- entries：TraitEntry 列表 — 每条记录 trait_id + source

要求：
- add_entry 时记录来源
- remove_by_source 时精确清理
- trait_ids() 返回去重列表

---

## TraitSource（值对象）

职责：标记 Trait 的来源

结构：
- Intrinsic：内在来源（种族/职业/天赋）
- Equipment { slot }：装备来源（记录槽位）

要求：
- 装备穿脱时使用 Equipment 变体
- 内在 Trait 使用 Intrinsic 变体

---

## GridPosition（Instance）

职责：单位在地图上的逻辑坐标

结构：
- coord：IVec2 — 格子坐标

要求：
- 默认值为 ZERO
- 移动动画完成后更新

---

## MovingUnit（临时 Instance）

职责：移动动画控制

结构：
- path：IVec2 列表 — 路径坐标序列
- current_index：整数 — 当前目标路径索引
- speed：浮点 — 每格耗时（秒）
- elapsed：浮点 — 当前格已用时间
- next_phase：TurnPhase — 移动完成后切换的阶段

要求：
- is_finished() = current_index >= path.len()
- 动画完成后必须移除

---

## PersistentTags（Instance）

职责：持久化标签容器，分来源追踪

结构：
- from_traits：GameplayTags — Trait 授予
- from_equipment：GameplayTags — 装备授予

要求：
- 最终 GameplayTags = from_traits | from_equipment
- 装备穿脱只修改 from_equipment

---

# 禁止事项

禁止：把 Entity 当对象使用

原因：Entity 只是 ID，不承载行为

违反后果：逻辑与数据耦合，无法用 ECS 系统处理

---

禁止：运行时修改 UnitTemplate

原因：UnitTemplate 是共享配置，修改影响所有实例

违反后果：多个单位共享模板时产生意外副作用

---

禁止：跳过 Dead Tag 直接处理死亡

原因：死亡是状态标记，不是即时删除

违反后果：死亡单位仍可行动、被选中、被攻击

---

禁止：在 HP 变化时内联死亡处理

原因：死亡处理由 Dead Hook + Observer + Message 分层处理

违反后果：死亡逻辑散落各处，难以维护和扩展

---

禁止：装备穿脱时不更新 TraitCollection

原因：装备提供 Trait，穿脱必须同步

违反后果：单位拥有已卸下装备的 Trait，属性计算错误

---

禁止：子实体拦截鼠标事件

原因：子实体（HP条、标签）是视觉元素，不应拦截输入

违反后果：点击 HP 条无法选中单位

---

# AI 修改规则

## 如果新增单位类型

允许：
- 新增 UnitTemplate RON 配置
- 新增 Trait 定义 RON 配置
- 新增初始装备组合

禁止：
- 修改 Unit 组件结构
- 修改 spawn_unit_from_template 流程
- 硬编码新单位逻辑

优先检查：
- UnitTemplate 字段是否覆盖新单位需求
- Trait 效果是否需要新 Handler

---

## 如果新增 Trait 效果类型

允许：
- 新增 TraitEffectHandler 实现并注册

禁止：
- 修改 TraitData 的方法
- 修改 apply_passive_traits 流程

优先检查：
- TraitEffectHandlerRegistry 注册
- TraitEffect 枚举是否需要扩展

---

## 如果修改死亡处理

允许：
- 新增 Observer 响应死亡
- 新增 Message 广播死亡事件

禁止：
- 修改 Dead Hook 的固有行为
- 在 HP 变化时内联死亡逻辑

优先检查：
- Dead::on_add_dead Hook
- CharacterDied Message 的消费者

---

## 如果测试失败

排查顺序：
1. 检查 Unit 的 Required Components 是否完整
2. 检查 TraitCollection 来源标记是否正确
3. 检查 UnitTemplate RON 配置是否合法
4. 检查 Dead Hook 是否正确触发
5. 检查实现代码是否绕过规则
