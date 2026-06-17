# Character 领域

Version: 2.0

## Purpose

Character 领域管理战场单位的定义、生成、身份标识、阵营划分、位置管理和 Trait 扩展。遵循 Definition / Instance 分离，Entity 只是 ID，状态标记使用 Tag Component。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| Unit | 战场上的战斗单位实例，拥有运行时状态 | ≠ UnitTemplate：Unit 是运行时，Template 是配置 |
| UnitTemplate | 单位的配置定义，描述一个单位"是什么" | ≠ Unit：Template 不可变，Unit 可变 |
| TraitCollection | 单位拥有的 Trait 条目集合，按来源分组管理 | ≠ TraitRegistry：Collection 是实例，Registry 是定义 |
| TraitSource | Trait 的来源标记 | ≠ TraitTrigger：Source 标记"从哪来"，Trigger 标记"何时触发" |
| GridPosition | 单位在地图上的格子坐标 | ≠ Transform：GridPosition 是逻辑坐标，Transform 是渲染位置 |
| Faction | 阵营枚举，决定敌我关系 | ≠ Team：本项目只有 Player / Enemy |
| Dead | 死亡标记 Tag Component | ≠ is_dead: bool：Tag Component 优于 bool 字段 |
| MovingUnit | 移动动画组件 | ≠ GridPosition 变化：MovingUnit 控制动画过程 |
| PersistentTags | 持久化标签集合，分两层追踪来源 | ≠ GameplayTags：PersistentTags 是容器，GameplayTags 是合并结果 |

---

## Responsibilities

### Owns

- 单位定义（UnitTemplate）和注册表
- 单位实例的生成和身份组件
- 阵营和格子坐标
- 死亡标记（Dead）的 Hook 行为
- TraitCollection 的增删管理
- 移动动画（MovingUnit）
- PersistentTags 的来源分层

### Does Not Own

- 属性计算 → stat_system
- Trait 定义和效果处理 → trait_rules
- 装备穿脱逻辑 → equipment_rules
- Buff 管理 → buff_rules
- 伤害和死亡判定 → battle_rules
- 移动范围计算和寻路 → map_rules
- UI 展示 → ui_rules

---

## Invariants

### INV-CHR-01：Unit 完整性 🟥

每个拥有 Unit 组件的 Entity 必须同时拥有所有 Required Components（Attributes, SkillSlots, SkillCooldowns, ActiveBuffs, GameplayTags, PersistentTags, TraitCollection, EquipmentSlots, Container, GridPosition）。

违反：单位缺少必要组件，系统访问时 panic。

### INV-CHR-02：UnitTemplate 不可变 🟥

宪法：1.1.2

UnitTemplate 的内容在加载后不可被修改。多个实例共享同一模板。

违反：修改模板影响所有实例。

### INV-CHR-03：Dead 标记一致性 🟥

宪法：2.1.4

如果单位 HP ≤ 0，则必须拥有 Dead Tag。禁止直接删除 Entity，禁止用 bool 代替 Tag。

违反：死亡单位仍可行动、被选中、被攻击。

### INV-CHR-04：TraitCollection 来源一致 🟥

每个 TraitEntry 的 source 必须与实际来源一致。

违反：装备穿脱后残留旧 Trait，或内在 Trait 被误删。

### INV-CHR-05：GridPosition 与 OccupancyGrid 同步 🟥

移动动画完成后，GridPosition 必须与 OccupancyGrid 中的占用记录一致。

违反：单位逻辑坐标与地图占用不一致。

### INV-CHR-06：Entity 只是 ID 🟥

宪法：2.1.1

Entity 不承载行为，禁止把 Entity 当对象调用方法。

违反：逻辑与数据耦合，无法用 ECS 系统处理。

### INV-CHR-07：死亡处理分层 🟥

宪法：5.0

死亡处理由 Dead Hook（固有行为）+ Observer（局部响应）+ Message（广播）分层处理，禁止在 HP 变化时内联死亡逻辑。

违反：死亡逻辑散落各处，难以维护和扩展。

---

## State Machine

### 单位状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Template | 配置定义，不可变 | — |
| Spawned | 已生成，在战场上 | Acting, Dead |
| Acting | 正在行动 | Spawned, Dead |
| Dead | 已死亡 | — |

```
Template → Spawned → Acting → Spawned
                    ↘ Dead
```

### 移动动画状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Idle | 无移动 | Moving |
| Moving | 正在播放移动动画 | Idle |

```
Idle → Moving → Idle
```

---

## Business Rules

### BR-CHR-01：单位生成

- 通过 UnitTemplate 生成运行时实例
- 推导 Vital Resources
- 应用被动 Trait
- 穿戴初始装备（跳过需求检查）
- 穿戴装备后重建 Trait 效果

### BR-CHR-02：阵营归属

- faction 决定敌我关系
- 阵营颜色统一映射
- 运行时不可改变阵营

### BR-CHR-03：死亡处理

- HP ≤ 0 时添加 Dead Tag Component
- Dead Hook 自动标记已行动 + 移除选中
- Observer 响应死亡（播放动画、刷新 UI）
- Message 广播死亡（CharacterDied）

### BR-CHR-04：TraitCollection 管理

- Trait 变化时重建 TraitCollection
- 装备穿脱时按来源精确移除
- 内在 Trait 标记为 Intrinsic，不被装备操作误删
- 通过 TraitEffectHandler 分发效果

### BR-CHR-05：PersistentTags 分层

- from_traits 和 from_equipment 分层追踪
- 装备穿脱只修改 from_equipment
- 最终 GameplayTags = from_traits | from_equipment

### BR-CHR-06：移动动画

- 逐格线性插值
- 动画完成后才更新 GridPosition
- 动画完成后移除 MovingUnit

---

## Pipelines

### 单位生成管线

查找模板 → 构建属性 → 构建 Trait → 应用被动 Trait → 穿戴初始装备 → 重建 Trait 效果 → 重建标签 → 生成实体

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 查找模板 | 关卡配置中的模板 ID | UnitTemplate | 模板不存在时静默跳过 |
| 构建属性 | 核心属性基础值 | Attributes 组件 | 禁止直接设置 Vital Resources 基础值 |
| 构建 Trait | Trait ID 列表 | TraitCollection | 全部标记为 Intrinsic |
| 应用被动 Trait | TraitCollection + 注册表 | 标签 + 修饰符 | 禁止处理非 Passive 触发类型 |
| 穿戴初始装备 | 初始装备映射 | EquipmentSlots | 跳过需求检查 |
| 重建 Trait 效果 | 装备后的 TraitCollection | 更新后的标签和属性 | 禁止跳过此步骤 |
| 生成实体 | 所有组件 | Unit Entity | 禁止遗漏任何 Required Component |

---

## Data Model

### Unit（Instance）

战斗单位的核心身份组件。

- 阵营（Player / Enemy）
- 本回合是否已行动

### UnitTemplate（Definition）

单位的配置定义，不可变。

- 标识：id / name
- 身份：faction / race / class
- 核心：8 维基础属性 + 基础攻击范围
- 引用：技能 ID 列表 / Trait ID 列表 / AI 行为 ID
- 初始装备映射
- 配置来源：RON

### TraitCollection（Instance）

单位拥有的 Trait 条目集合。

- TraitEntry 列表（trait_id + source）
- 按来源精确增删

### TraitSource（值对象）

Trait 的来源标记。

- Intrinsic：内在来源（种族/职业/天赋）
- Equipment { slot }：装备来源

### GridPosition（Instance）

单位在地图上的逻辑坐标。

- IVec2 格子坐标
- 移动动画完成后更新

### MovingUnit（临时 Instance）

移动动画控制。

- 路径坐标序列
- 动画速度
- 移动完成后切换的阶段

### PersistentTags（Instance）

持久化标签容器，分来源追踪。

- from_traits：Trait 授予
- from_equipment：装备授予
- 最终 GameplayTags = from_traits | from_equipment

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 单位生成完成 | 直接函数调用 | stat / trait / equipment |
| 单位死亡 | Dead Tag Hook | battle / ui |
| 移动完成 | TurnPhase 切换 | turn |
| Trait 变化 | rebuild_trait_effects | trait |

---

## Change Rules

### 新增单位类型

- 允许：新增 UnitTemplate RON 配置 + 新增 Trait 定义 RON
- 禁止：修改 Unit 组件结构、修改生成流程、硬编码新单位逻辑
- 检查：UnitTemplate 字段是否覆盖需求、Trait 效果是否需要新 Handler

### 新增 Trait 效果类型

- 允许：新增 TraitEffectHandler 实现 + 注册
- 禁止：修改 TraitData 方法、修改 apply_passive_traits 流程
- 检查：注册表注册、TraitEffect 枚举是否需要扩展

### 修改死亡处理

- 允许：新增 Observer 响应死亡 + 新增 Message 广播
- 禁止：修改 Dead Hook 固有行为、在 HP 变化时内联死亡逻辑
- 检查：Dead Hook、CharacterDied Message 的消费者

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
| INV-CHR-02 | 运行时修改 UnitTemplate | Definition/Instance 分离 | 改为修改运行时实例 |
| INV-CHR-03 | 用 bool 代替 Tag Component | 宪法 2.1.4 | 改用 Dead Tag Component |
| INV-CHR-06 | Entity 承载行为 | Entity 只是 ID | 改为 ECS 系统处理 |
| INV-CHR-07 | HP 变化时内联死亡逻辑 | 通信三原则 | 改为 Dead Hook + Observer + Message |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证生成管线各步骤
- 集成测试：验证完整单位生命周期
- Bug 修复必须先编写重现测试

排查顺序：
1. Unit 的 Required Components 是否完整
2. TraitCollection 来源标记是否正确
3. UnitTemplate RON 配置是否合法
4. Dead Hook 是否正确触发
5. 实现代码是否绕过规则
