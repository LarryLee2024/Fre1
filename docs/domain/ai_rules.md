# AI 行为系统领域

Version: 1.0
Status: Proposed

AI 行为系统领域管理敌方和中立单位在战术战斗中的自动决策与行动执行。

核心原则（对标宪法条款）：
- 🟩 AI 通过 Intent + Effect Pipeline 表达和执行行动，与玩家共用同一通道（宪法 7.0.1 分层扩展体系 + 11.5 命令层）
- 🟩 行为配置数据驱动（RON 文件），策略逻辑通过 Trait 注册表分发（宪法 1.1.3 规则与内容强制分离）
- 🟥 AI 决策不访问玩家不可见信息，不直接修改 ECS 组件（宪法 7.0.1：所有属性增减必须进入统一 Modifier 管线）
- 🟩 AI 是独立 Feature 模块（宪法 1.1.1 Feature First + 11.5 所有操作入口为命令）
- 🟥 AI 必须不绕过 Effect/Modifier 管线（宪法 7.0.1：角色数值能力统一通过 Modifier 管线实现）

---

# 术语定义

## AI 行为（AI Behavior）

AI 决策和行动执行的整体机制。从战场扫描到意图生成的完整决策流程，最终产出 CombatIntent 或 MovementIntent。

不是游戏规则。不是玩家输入。

关键属性：
- AI 行为仅在 `AppState::InGame` 且当前单位为 `Faction::Enemy` 时激活
- AI 行为由 `enemy_ai_system` 系统驱动
- AI 行为通过 AiTimer 延迟执行，避免瞬间决策导致不自然感知
- AI 行为的输出与玩家操作共用同一 Effect Pipeline

---

## 行为模板（Behavior Template）

数据驱动的 AI 行为配置，存储为 RON 文件（`assets/ai/*.ron`）。定义单位使用的目标策略、移动策略、技能策略和技能优先级。

不是硬编码逻辑。不是行为树实现。

关键属性：
- 由 `AiBehaviorDef` 定义（Definition，不可变），运行时转换为 `AiBehavior`（Instance，可变）
- 存储在 `AiBehaviorRegistry` 资源中，通过 `id` 字段索引
- 策略字段为字符串名称，运行时通过 `AiStrategyRegistry` 查找 Trait 对象分发
- 未找到指定行为模板时回退到 `default_behavior()`
- 内置四个默认模板：default（默认）、aggressive（激进）、cautious（谨慎）、support（辅助）

---

## 行为树（Behavior Tree）

AI 决策的树形结构，用于表示复杂的条件-动作决策逻辑。

不是脚本。不是状态机。

关键属性：
- 行为树的节点分为选择节点、序列节点和动作节点
- 当前系统未采用行为树，而是使用策略模式（Trait + 注册表分发）
- 行为树是一种可选的扩展方案，但不在当前实现范围内

---

## AI 决策上下文（DecisionContext）

AI 决策时可用的信息集合，包括当前单位快照、所有单位状态、地图可达区域和技能数据。

不是 ECS World。不是全局信息。

关键属性：
- 通过 `UnitSnapshot` 收集所有单位的状态快照（位置、HP、攻击、技能、冷却等）
- 决策上下文仅包含当前可见信息，不访问战争迷雾或手牌信息
- 决策上下文在每次决策时重新构建，不缓存

---

## AI 意图（AiIntent）

AI 决策产生的行动意图，最终表现为 CombatIntent（攻击意图）或 MovementIntent（移动意图）。

不是玩家 UiCommand。不是直接 ECS 变更。

关键属性：
- AI 意图通过 CombatIntent 资源传递攻击意图
- AI 意图通过 MovementIntent 消息传递移动意图
- AI 意图与玩家意图共用同一执行管线（Effect Pipeline）
- AI 意图不直接修改任何 ECS 组件，只设置 Intent

---

## AI 计时器（AiTimer）

AI 行动的延迟机制，在 AI 单位轮到行动时提供短暂延迟，让玩家能看见 AI 的"思考"过程。

不是游戏倒计时。不是单位冷却。

关键属性：
- 存储在 `AiTimer` 资源中，包含一个 `Timer`
- 默认延迟 0.4 秒（`Timer::from_seconds(0.4, TimerMode::Once)`）
- 计时器在路由到下一个 AI 单位时重置
- 仅在 `Faction::Enemy` 单位行动时生效

---

## 策略模板（Strategy Template）

预定义的 AI 战术模式，通过 Trait 实现并在 `AiStrategyRegistry` 中注册。包括目标选择策略、移动策略和技能选择策略。

不是行为模板。不是单位配置。

关键属性：
- 策略模板是 Rust Trait 实现，通过 `strategy_name()` 返回标识字符串
- 目标选择策略（TargetSelector）：Nearest、Weakest、MostDangerous、LowestHpPercent
- 移动策略（MoveSelector）：Aggressive、Cautious、Support
- 技能选择策略（SkillSelector）：PreferSpecial、PreferBasic、ByPriority
- 新增策略只需实现对应 Trait 并注册，无需修改已有代码

---

## 难度级别（Difficulty Level）

控制 AI 精细度的参数，影响 AI 的算力分配、视野范围和行为丰富度。

不是数值加成。不是单位属性修改。

关键属性：
- 难度级别通过行为模板的参数差异体现
- 不同难度级别使用不同的行为模板配置
- 难度差异不应在代码中通过 if-else 硬编码
- 难度差异应在 RON 配置文件中通过不同模板实现

---

# 领域边界

## 本领域负责

- AI 行为模板的定义、加载和注册（`AiBehaviorDef` → `AiBehavior`）
- AI 策略的定义、注册和分发（`AiStrategyRegistry`）
- AI 决策系统的执行（`enemy_ai_system`）
- AI 目标选择（`select_target_coord`）
- AI 移动策略选择（`select_move_coord`）
- AI 技能选择（`select_skill`）
- AI 计时器管理（`AiTimer`）
- AI 行为数据的 RON 加载和默认值注册

## 本领域不负责

- CombatIntent 的执行和 Effect Pipeline 的运行（由 Battle 领域负责）
- MovementIntent 的执行和移动逻辑（由 Input/Map 领域负责）
- 回合阶段的流转和行动队列的编排（由 Turn 领域负责）
- 单位属性的计算和修饰（由 Core 属性系统负责）
- 单位快照的数据结构定义（由 Character 领域负责）
- 地图可达区域的计算（由 Map 领域负责）
- 技能数据的加载和冷却计算（由 Skill 领域负责）
- AI 行为模板的 RON 格式定义和校验（由 Content 系统负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 攻击意图 | Resource（CombatIntent） | Battle（Effect Pipeline 执行） |
| 移动意图 | Message（MovementIntent） | Input/Map（移动执行） |
| 行为配置 ID | Component（AiBehaviorId） | Character（单位绑定） |
| AI 行动延迟 | Resource（AiTimer） | Turn（路由重置） |
| 策略名称映射 | Resource（AiBehaviorRegistry） | Character（单位配置） |

---

# 生命周期

## 状态列表

本领域无状态机，为纯函数式计算。

AI 决策系统在 `AppState::InGame` 状态下由 `Update` 驱动，每次决策为独立的函数调用，不维护跨帧状态。

## 决策触发条件

| 条件 | 含义 | 结果 |
|------|------|------|
| TurnPhase::SelectUnit | 当前阶段为选择单位 | AI 系统开始检查 |
| 当前单位为 Faction::Enemy | 行动队列当前单位是敌方 | AI 开始计时 |
| AiTimer 到期 | 延迟 0.4 秒完成 | AI 执行决策 |
| 当前单位为非敌方 | 行动队列当前单位是玩家 | AI 系统跳过 |

## 决策流程

```
检查阶段 → 检查阵营 → 等待计时器 → 收集快照 → 选择目标 → 选择移动 → 选择技能 → 生成 Intent
```

---

# 不变量

## 不变量1：AI 必须通过 Intent 表达意图（🟥 宪法 11.5.1 所有操作入口为命令）

任意时刻：

🟥 AI 产生的任何行动必须通过 CombatIntent 或 MovementIntent 表达。AI 不得直接修改 ECS 组件。所有操作入口必须转换为标准化业务命令。

违反表现：

AI 系统中出现 `unit.hp -= damage` 或直接操作组件的代码。

---

## 不变量2：AI 与玩家共用 Effect Pipeline（🟥 宪法 7.0.1 所有属性增减必须进入统一 Modifier 管线）

任意时刻：

🟥 AI 的攻击执行必须经过 Generate → Modify → Execute 三步管线。AI 和玩家的伤害计算路径必须完全相同。绝对禁止 AI 绕过 Effect/Modifier 管线直接修改战斗数值与属性。

违反表现：

AI 伤害计算跳过 Modifier 修饰、Trait 触发或 BattleRecord 记录。

---

## 不变量3：AI 决策不访问玩家不可见信息（🟥 宪法 11.7.1 读路径无副作用）

AI 决策阶段：

🟥 AI 只能访问 UnitSnapshot 中记录的信息。AI 不得访问战争迷雾、手牌信息或其他玩家不可见的游戏状态。

违反表现：

AI 单位在玩家视野外做出精确定位决策，或根据玩家手牌选择应对策略。

---

## 不变量4：行为模板必须从 RON 加载（🟩 宪法 1.1.3 规则与内容强制分离）

任意时刻：

🟥 AI 行为配置必须从 `assets/ai/*.ron` 文件加载，通过 `AiBehaviorRegistry` 注册。禁止在 Rust 代码中硬编码行为配置。

违反表现：

行为配置直接写在 Rust 代码中（如 `AiBehaviorDef { id: "xxx".into(), ... }` 出现在非注册逻辑中）。

---

## 不变量5：策略未找到时必须回退默认值（🟩 宪法 1.4.2 领域无副作用）

任意时刻：

当行为模板引用的策略名称在 `AiStrategyRegistry` 中未找到时，必须回退到该策略类别的默认策略（TargetSelector → Nearest，MoveSelector → Aggressive，SkillSelector → PreferSpecial）。

违反表现：

AI 系统因未知策略名称而 panic 或崩溃。

---

# 业务规则

## 规则1：AI 意图生成（🟥 宪法 11.5 命令层 + 7.0.1 Effect Pipeline）

禁止：
- 🟥 AI 直接修改 ECS 组件（如直接设置 `unit.acted = true`）（宪法 7.0.1：所有属性增减必须进入统一 Modifier 管线）
- 🟥 AI 绕过 CombatIntent 独立执行攻击（宪法 11.5.1：所有操作入口为命令）
- 🟥 AI 在非 SelectUnit 阶段执行决策

必须：
- AI 通过 CombatIntent 传递攻击意图（source_entity、target_coord、skill_id）
- AI 通过 MovementIntent 消息传递移动意图（entity、target_coord、source）
- AI 决策完成后切换到 ExecuteAction 或 WaitAction 阶段

允许：
- AI 在决策阶段读取所有单位的 UnitSnapshot
- AI 在决策阶段读取地图和技能数据

---

## 规则2：行为模板管理（🟩 宪法 1.1.3 规则与内容强制分离）

禁止：
- 🟥 在 Rust 代码中硬编码 AI 行为配置（宪法 1.1.3：代码只实现通用规则，所有可变内容必须放在配置文件中）
- 🟥 修改已有的 RON 配置文件格式（宪法 12.1.2 配置稳定性）
- 🟥 在运行时修改 BehaviorTemplate 的 Definition 数据（宪法 1.1.2 定义与实例强制分离）

必须：
- 行为模板从 `assets/ai/*.ron` 加载
- 行为模板通过 `AiBehaviorRegistry` 注册
- 未找到指定行为时回退到 `default_behavior()`

允许：
- 新增 RON 文件定义新行为模板
- 通过 `register_defaults()` 注册内置默认行为
- 为不同难度级别创建不同的行为模板文件

---

## 规则3：策略扩展（🟩 宪法 1.1.6 组合绝对优先于继承）

禁止：
- 🟥 修改已有策略的 `strategy_name()` 返回值
- 🟥 在策略实现中直接修改 ECS 组件（宪法 7.0.1：策略是纯计算逻辑，不负责执行）
- 🟥 硬编码策略分发逻辑（必须通过注册表）（宪法 1.1.3 规则与内容强制分离）

必须：
- 新增策略通过实现对应 Trait 并注册到 `AiStrategyRegistry`
- 策略的 `strategy_name()` 返回值与 RON 配置中的字符串对应
- 策略未找到时回退到默认策略

允许：
- 新增 TargetSelector、MoveSelector、SkillSelector 的实现
- 在 `AiStrategyRegistry::register_defaults()` 中注册新策略
- 通过 RON 配置引用新策略名称

---

## 规则4：AI 计时器

禁止：
- 修改 AiTimer 的默认延迟时间（0.4 秒）
- 在 AI 计时器未到期时执行决策
- 跳过 AI 计时器直接执行决策

必须：
- AI 单位行动前等待 AiTimer 到期
- 路由到下一个 AI 单位时重置 AiTimer
- 路由到玩家单位时不重置 AiTimer

允许：
- 在测试中调整 AiTimer 延迟
- 通过难度级别调整 AiTimer 延迟（通过配置，非代码 if-else）

---

## 规则5：难度差分（🟩 宪法 1.1.3 规则与内容强制分离）

禁止：
- 🟥 在代码中通过 if-else 区分难度级别（宪法 1.1.3：代码只实现通用规则，所有可变内容必须放在配置文件中）
- 🟥 在代码中硬编码难度相关的数值差异（宪法 12.4.1 平衡参数全配置化）
- 🟥 修改已有难度级别模板的行为逻辑

必须：
- 不同难度级别通过不同的行为模板（RON 文件）体现
- 难度差异在行为模板的参数中体现（如策略名称、技能优先级）
- 难度差异不修改策略 Trait 的实现逻辑

允许：
- 为新难度级别创建新的 RON 配置文件
- 在行为模板中配置不同的策略组合
- 通过技能优先级列表调整 AI 行为倾向

---

# 流程管线

## AI 决策管线

```
扫描战场 → 评估选项 → 选择最佳 → 生成 Intent
```

### Step1：扫描战场

输入：TurnOrder 当前单位、所有存活单位的 ECS 组件
处理：收集所有单位的 UnitSnapshot，过滤敌方和友方位置
输出：Vec<UnitSnapshot>、玩家位置列表
禁止：访问非公开信息（战争迷雾、手牌等）

### Step2：评估选项

输入：UnitSnapshot 列表、当前单位快照、地图可达区域
处理：根据行为模板选择策略，计算可达格子、目标坐标、可用水技能
输出：目标坐标、可达格子集合、技能候选列表
禁止：修改任何 ECS 组件

### Step3：选择最佳

输入：目标坐标、可达格子、技能候选列表
处理：根据移动策略选择最佳移动位置，根据技能策略选择最佳技能
输出：最终移动坐标、最终技能 ID、攻击目标实体
禁止：跳过策略注册表直接调用策略实现

### Step4：生成 Intent

输入：最终移动坐标、最终技能 ID、攻击目标实体
处理：设置 CombatIntent（攻击意图）或发送 MovementIntent（移动意图）
输出：CombatIntent 资源更新、MovementIntent 消息
禁止：直接修改单位 HP 或其他 ECS 组件

---

## AI 执行管线

```
AiIntent → CombatIntent/MovementIntent → Effect Pipeline → 等待 → 下一单位
```

### Step1：AiIntent 转化

输入：AI 决策管线的输出（CombatIntent 或 MovementIntent）
处理：将 AI 意图转化为标准的 CombatIntent 资源或 MovementIntent 消息
输出：CombatIntent 资源设置、MovementIntent 消息发送
禁止：AI 直接执行攻击逻辑

### Step2：Effect Pipeline 执行

输入：CombatIntent 资源
处理：执行 Generate → Modify → Execute 三步管线
输出：伤害/治疗效果、可能的单位死亡
禁止：跳过 Effect Pipeline 的任何步骤

### Step3：等待执行完成

输入：Effect Pipeline 执行结果
处理：等待动画完成、UI 更新、消息消费
输出：执行完成信号
禁止：在执行完成前开始下一个单位的决策

### Step4：路由到下一单位

输入：TurnOrder 队列、当前单位状态
处理：标记当前单位已行动，前进到下一个存活单位
输出：TurnPhase 状态切换
禁止：跳过死亡单位检测

---

# 数据结构

## AiBehaviorDef（行为模板定义）

职责：定义 AI 行为的 RON 反序列化结构（Definition，不可变）

结构：
- version：u32 — 模板版本号
- id：String — 行为唯一标识（如 "default"、"aggressive"）
- name：String — 行为显示名称（如 "默认"、"激进"）
- target_strategy：String — 目标选择策略名称（对应 TargetSelector 的 strategy_name）
- move_strategy：String — 移动策略名称（对应 MoveSelector 的 strategy_name）
- skill_strategy：String — 技能选择策略名称（对应 SkillSelector 的 strategy_name）
- skill_priority：Vec<String> — 技能使用优先级列表（从高到低）

要求：
- 从 `assets/ai/*.ron` 文件反序列化
- 策略字段的字符串值必须在 AiStrategyRegistry 中注册
- 新增字段必须添加 `#[serde(default)]` 以保持向后兼容

---

## AiBehavior（行为模板运行时）

职责：AI 行为的运行时数据（Instance，可变）

结构：
- id：String — 行为唯一标识
- name：String — 行为显示名称
- target_strategy：String — 目标选择策略名称
- move_strategy：String — 移动策略名称
- skill_strategy：String — 技能选择策略名称
- skill_priority：Vec<String> — 技能使用优先级列表

要求：
- 由 AiBehaviorDef 通过 From 转换生成
- 存储在 AiBehaviorRegistry 中
- 运行时不可修改（由 BehaviorTemplate Definition 约束）

---

## AiBehaviorRegistry（行为注册表）

职责：管理所有 AI 行为模板的注册和查找

结构：
- behaviors：HashMap<String, AiBehavior> — 按 id 索引的行为模板

要求：
- 必须实现 RegistryLoader trait
- 必须提供 `default_behavior()` 方法
- 未知 id 时回退到 default_behavior()
- 注册表为空时 panic（至少需要一个行为定义）

---

## UnitSnapshot（单位快照）

职责：AI 决策时的单位状态快照，避免 ECS 借用冲突

结构：
- entity：Entity — 单位实体
- faction：Faction — 阵营（Player/Enemy）
- coord：IVec2 — 网格坐标
- atk：f32 — 攻击力
- hp：f32 — 当前生命值
- max_hp：f32 — 最大生命值
- mov：u32 — 移动力
- attack_range：u32 — 攻击范围
- acted：bool — 是否已行动
- skill_ids：Vec<String> — 可用技能列表
- cooldowns：SkillCooldowns — 技能冷却状态
- ai_behavior_id：String — AI 行为模板 ID
- tags：GameplayTags — 单位标签

要求：
- 在每次决策时重新构建，不缓存
- 仅包含 AI 决策所需的信息
- 不包含玩家不可见信息

---

## AiStrategyRegistry（策略注册表）

职责：管理所有 AI 策略 Trait 对象的注册和按名称查找

结构：
- target_selectors：HashMap<String, Box<dyn TargetSelector>> — 目标选择策略
- move_selectors：HashMap<String, Box<dyn MoveSelector>> — 移动策略
- skill_selectors：HashMap<String, Box<dyn SkillSelector>> — 技能选择策略

要求：
- 必须在 App 初始化时注册所有默认策略
- 未知策略名称时回退到该类别的默认策略
- TargetSelector 默认回退：Nearest
- MoveSelector 默认回退：Aggressive
- SkillSelector 默认回退：PreferSpecial

---

# 禁止事项

🟥 禁止：AI 直接修改 ECS 组件（如直接设置 unit.acted、unit.hp 等）（宪法 7.0.1：所有属性增减必须进入统一 Modifier 管线）

原因：AI 直接修改 ECS 组件会绕过 Effect Pipeline，跳过 Modifier 修饰、Trait 触发和 BattleRecord 记录，导致 AI 和玩家的战斗计算不一致。

违反后果：AI 伤害计算不走 Modifier 管线、AI 行为与玩家行为不一致、测试无法覆盖 AI 伤害、战斗日志缺失。

---

🟥 禁止：AI 访问玩家不可见信息（战争迷雾、手牌信息等）（宪法 11.7.1 读路径无副作用）

原因：AI 应该和玩家在相同的信息条件下决策，否则玩家会感到不公平。AI 使用作弊信息会破坏游戏体验。

违反后果：玩家感知到 AI 作弊、游戏体验下降、难度设计失效。

---

🟥 禁止：在 Rust 代码中硬编码 AI 行为配置（if-else 难度分支、硬编码策略选择等）（宪法 1.1.3 规则与内容强制分离）

原因：违反 Rule/Content 分离原则。AI 行为的配置应由 RON 文件驱动，代码只包含策略逻辑。

违反后果：新增难度级别需要修改 Rust 代码、行为调整需要重新编译、热重载失效。

---

🟥 禁止：AI 绕过 CombatIntent 独立执行攻击（宪法 11.5.1 所有操作入口为命令 + 7.0.1 Effect Pipeline）

原因：CombatIntent 是攻击意图的唯一通道，AI 和玩家共用同一 Effect Pipeline。AI 独立执行会导致使用不同的伤害计算路径。

违反后果：AI 伤害计算不一致、修饰规则失效、战斗日志缺失。

---

🟥 禁止：修改已有的 RON 配置文件格式而不更新 AiBehaviorDef（宪法 12.1.2 配置稳定性）

原因：RON 文件格式与 AiBehaviorDef 的反序列化结构严格对应。修改格式而不更新定义会导致反序列化失败。

违反后果：AI 行为模板加载失败、游戏启动时 panic。

---

🟥 禁止：在策略实现中直接修改 ECS 组件（宪法 7.0.1：策略是纯计算逻辑，不负责执行）

原因：策略 Trait 是纯计算逻辑，只负责选择最优方案，不负责执行。修改 ECS 组件属于执行层职责。

违反后果：策略层和执行层职责混淆、策略无法独立测试。

---

🟥 禁止：跳过 AiTimer 直接执行 AI 决策（宪法 11.5.2 命令无差别执行）

原因：AiTimer 为玩家提供 AI 思考的视觉反馈。跳过计时器会导致 AI 行动瞬间完成，玩家无法感知 AI 的决策过程。

违反后果：AI 行动无延迟、玩家体验下降、游戏节奏不自然。

---

🟥 禁止：在非 SelectUnit 阶段执行 AI 决策（宪法 11.1.1 阶段划分标准化）

原因：AI 决策系统只在 SelectUnit 阶段有效。在其他阶段执行会导致决策时序混乱和状态不一致。

违反后果：AI 在错误的回合阶段执行决策、回合状态机混乱。

---

# AI 修改规则（🟥 宪法 21.1 AI反模式黑名单 — 生成前必须对照检查）

> AI 最容易违反的 24 条规则中与 AI 领域直接相关的：
> - ❌ #1 把 Entity 当面向对象实例（宪法 2.1.1）
> - ❌ #5 业务逻辑直接操作 UI 组件（宪法 1.1.4）
> - ❌ #8 为未明确的未来需求提前设计复杂架构（宪法 1.5.2）
> - ❌ #19 核心领域逻辑直接依赖 Bevy ECS 类型（宪法 1.4.1）
> - ❌ #20 预览/仿真等读路径带有副作用（宪法 11.7.1）
> - ❌ #23 跨模块直接修改其他 Feature 的内部状态（宪法 3.0.4）

## 如果新增行为模板

允许：
- 在 `assets/ai/` 目录下新增 RON 文件
- 在 `AiBehaviorRegistry::register_defaults()` 中注册内置默认行为
- 在 Character 领域的单位模板中引用新行为 ID

禁止：
- 在 Rust 代码中硬编码新行为的配置值
- 修改已有行为模板的字段格式
- 新增行为模板时不提供默认值

优先检查：
- RON 文件中的策略名称是否在 AiStrategyRegistry 中注册
- 行为 ID 是否与已有 ID 冲突
- 新模板是否通过 `AiBehaviorDef` 反序列化测试

---

## 如果新增策略实现

允许：
- 在 `src/ai/strategy.rs` 中实现新的 TargetSelector / MoveSelector / SkillSelector Trait
- 在 `AiStrategyRegistry::register_defaults()` 中注册新策略
- 在 RON 配置中引用新策略的 `strategy_name()`

禁止：
- 修改已有策略的 `strategy_name()` 返回值
- 在策略实现中直接修改 ECS 组件
- 新增策略后不注册到 AiStrategyRegistry

优先检查：
- `strategy_name()` 返回值是否与 RON 配置中的字符串对应
- 策略实现是否为纯计算逻辑（不修改 ECS 组件）
- 策略是否已注册到 AiStrategyRegistry
- 是否需要同步更新默认回退策略

---

## 如果修改难度差分

允许：
- 为不同难度级别创建不同的 RON 行为模板文件
- 在行为模板中配置不同的策略组合和技能优先级
- 在关卡配置中为不同难度引用不同的行为模板 ID

禁止：
- 在代码中通过 if-else 区分难度级别
- 修改已有策略 Trait 的实现逻辑来适配难度
- 在 Rust 代码中硬编码难度相关数值

优先检查：
- 难度差异是否完全在 RON 配置中体现
- 是否有代码中的 if-else 难度分支需要清理
- 新难度模板的策略名称是否全部在 AiStrategyRegistry 中注册

---

## 如果修改 AI 决策逻辑

允许：
- 修改 enemy_ai_system 中的决策流程顺序
- 新增决策前的条件检查
- 调整 UnitSnapshot 收集的信息字段

禁止：
- 在决策逻辑中直接修改 ECS 组件
- 跳过 CombatIntent 设置直接执行攻击
- 修改 CombatIntent 的字段结构（需在 Battle 领域协调）

优先检查：
- 决策逻辑是否仍通过 CombatIntent/MovementIntent 表达意图
- 决策逻辑是否仍遵循 Scan → Evaluate → Select → Intent 流程
- 是否需要同步更新 UnitSnapshot 的字段定义
- 新逻辑是否在 TurnPhase::SelectUnit 阶段执行

---

## 如果测试失败

排查顺序：
1. 检查 AI 是否通过 CombatIntent/MovementIntent 表达意图（不变量1）
2. 检查策略名称是否在 AiStrategyRegistry 中注册（回退逻辑）
3. 检查行为模板 RON 文件是否可正确反序列化
4. 检查 AiTimer 是否在正确时机重置
5. 检查 TurnPhase 是否为 SelectUnit 时才执行 AI 决策
6. 检查 Effect Pipeline 是否完整执行 Generate → Modify → Execute（不变量2）
