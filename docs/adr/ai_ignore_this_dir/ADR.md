# SRPG 项目架构文档

## 一、项目概览

基于 Bevy 0.18 的回合制战棋游戏，采用 Feature First 模块化架构，数据驱动设计。项目包含 13 个业务模块 + 1 个调试模块，通过 ECS + Message + Observer + Hook 四层通信机制实现松耦合。

---

## 二、顶层目录结构

```
src/
├── main.rs              # App 入口，插件注册与分层编排
├── lib.rs               # 公共模块导出
├── assets.rs            # 全局资源（字体等）
├── input.rs             # 输入处理（Pointer + cursor_to_coord）
│
├── core/                # 核心基础设施（跨模块共享）
│   ├── attribute/       # 属性系统：8维核心 + 衍生 + 修饰符栈
│   ├── effect/          # 效果管线：EffectDef → PendingEffect → EffectResult
│   ├── tag.rs           # 标签系统：位掩码 GameplayTag
│   ├── tag_def.rs       # 标签定义注册
│   ├── attribute_def.rs # 属性定义注册
│   ├── modifier_rule.rs # 修饰规则：数据驱动的伤害/治疗修饰
│   ├── registry_loader.rs # 通用 RON 注册表加载器
│   └── snapshot.rs      # 场景快照（战斗回放/存档）
│
├── character/           # 角色模块
│   ├── components.rs    # Unit, Faction, GridPosition, Dead, UnitName, UnitId
│   ├── marker.rs        # 标记组件
│   ├── movement.rs      # 移动动画
│   ├── spawn.rs         # 从模板生成单位
│   ├── template.rs      # UnitTemplate 数据驱动定义
│   └── traits/          # Trait 扩展体系
│       ├── types.rs     # TraitTrigger, TraitEffect, TraitDefinition
│       └── handlers.rs  # TraitEffectHandler trait 分发
│
├── battle/              # 战斗模块
│   ├── pipeline/        # Effect Pipeline（generate → modify → execute）
│   │   ├── intent.rs    # CombatIntent 资源
│   │   ├── generate.rs  # 步骤1：生成效果
│   │   ├── modify.rs    # 步骤2：修饰效果
│   │   ├── execute.rs   # 步骤3：执行效果
│   │   └── trait_trigger.rs # Trait 触发器
│   ├── events.rs        # 战斗 Message（CharacterDied, DamageApplied 等）
│   ├── record.rs        # 战斗记录（BattleRecord + DamageBreakdown）
│   ├── log.rs           # 战斗日志
│   └── combat.rs        # 战斗辅助函数
│
├── buff/                # Buff 模块
│   ├── domain.rs        # BuffData/BuffDef 定义 + BuffRegistry
│   ├── instance.rs      # ActiveBuffs 组件 + BuffInstance
│   ├── apply.rs         # 穿戴/移除 Buff（修改 Attributes + Tags）
│   └── resolve.rs       # 持续效果结算（DoT/HoT/晕眩/tick）
│
├── skill/               # 技能模块
│   ├── domain/          # SkillData/SkillDef + SkillRegistry
│   ├── slots.rs         # SkillSlots 组件 + SkillCooldowns
│   └── preview.rs       # 技能效果预览
│
├── equipment/           # 装备模块
│   ├── definition.rs    # EquipmentDef + EquipmentSlot + Rarity + EquipmentRegistry
│   ├── instance.rs      # EquipmentInstance
│   ├── slots.rs         # EquipmentSlots 组件
│   ├── equip.rs         # 穿脱逻辑（EquipItem/UnequipItem Message）
│   └── requirements.rs  # 装备需求检查
│
├── inventory/           # 背包模块
│   ├── definition.rs    # ItemDef + ItemRegistry
│   ├── instance.rs      # ItemInstance + ItemStack + InstanceIdCounter
│   ├── container.rs     # Container 组件
│   ├── battle_bag.rs    # 战斗背包
│   ├── transfer.rs      # 物品转移（TransferItem Message）
│   ├── use_item.rs      # 消耗品使用（UseItem Message）
│   └── resources.rs     # Resources + ResourceStack
│
├── map/                 # 地图模块
│   ├── data.rs          # TerrainDef + LevelConfig + 注册表
│   ├── grid.rs          # GameMap 坐标转换
│   ├── pathfinding/     # BFS 寻路 + 地形消耗
│   └── runtime/         # TerrainGrid + OccupancyGrid
│
├── turn/                # 回合管理模块
│   ├── state.rs         # AppState + TurnPhase + GameSet
│   └── order.rs         # TurnOrder + TurnState + 回合 Message
│
├── ai/                  # AI 模块
│   ├── behavior.rs      # AiBehavior 数据驱动定义 + AiBehaviorRegistry
│   ├── strategy.rs      # TargetSelector/MoveSelector/SkillSelector trait 扩展
│   ├── decision.rs      # 敌方 AI 主系统（队列驱动）
│   ├── targeting.rs     # 目标选择
│   ├── movement.rs      # 移动坐标选择
│   └── skill_select.rs  # 技能选择
│
├── ui/                  # UI 模块
│   ├── events.rs        # UiCommand Message
│   ├── command_handler.rs # UI→Logic 命令处理
│   ├── view_models.rs   # ViewModel 层
│   ├── panels/          # 面板模块
│   ├── widgets/         # 基础 UI 组件
│   ├── theme.rs         # 主题
│   ├── settings.rs      # 设置
│   ├── focus.rs         # 焦点管理
│   ├── highlight.rs     # 高亮标记
│   ├── vfx.rs           # 视觉效果
│   ├── camera.rs        # 摄像机
│   └── action_menu.rs   # 行动菜单
│
└── debug/               # 调试工具模块
    ├── battle_debugger.rs  # F1: 战斗调试器
    ├── buff_viewer.rs      # F2: Buff 查看器
    ├── damage_viewer.rs    # F4: 伤害分解器
    ├── attribute_viewer.rs # F4: 属性查看器
    ├── turn_queue_viewer.rs # F5: 行动队列查看器
    ├── grid_viewer.rs      # 格子可视化
    ├── gizmos_viz.rs       # Gizmos 覆盖层
    ├── equipment_viewer.rs # 装备查看器
    ├── ai_viewer.rs        # AI 查看器
    ├── settings_viewer.rs  # 设置查看器
    ├── stepping_control.rs # 单步执行
    └── overlay.rs          # 调试覆盖层
```

---

## 三、ADR-001 整体架构决策

**Decision:**
采用 Feature First 模块化架构，按业务领域拆分模块（character/battle/buff/skill/equipment/inventory/map/turn/ai/ui），不按技术层拆分。模块内部按职责拆文件，不按代码类型拆文件。

**Consequences:**
- (+) 业务边界清晰，新功能定位快
- (+) 模块可独立开发和测试
- (-) 跨模块通信需要明确的接口设计
- (-) 模块间依赖关系需要严格管理

**Module Design:**

| 层级 | 模块 | 职责 |
|------|------|------|
| 核心层 | `core` | 属性系统、标签系统、效果管线、修饰规则、注册表加载、快照 |
| 数据层 | `skill`, `buff`, `equipment`, `inventory` | 数据驱动定义 + 注册表 + 实例管理 |
| 逻辑层 | `character`, `battle`, `turn`, `ai`, `map` | 游戏逻辑核心 |
| 表现层 | `ui`, `input`, `debug` | 用户交互与可视化 |

**Communication Design:**

| 通信方式 | 适用场景 | 示例 |
|----------|----------|------|
| **Hook** | 组件固有行为 | `Dead::on_add_dead` 标记已行动 |
| **Observer** | 局部响应 | Pointer\<Click\> 触发 UiCommand |
| **Message** | 跨 Feature 广播 | `DamageApplied`, `CharacterDied`, `UiCommand` |
| **函数调用** | 模块内部 | `apply_buff()`, `remove_buff()` |
| **Resource** | 全局只读状态 | `SkillRegistry`, `BuffRegistry`, `TurnOrder` |
| **Component** | 实体级状态 | `Attributes`, `ActiveBuffs`, `EquipmentSlots` |

**Boundary Definition:**
- `core` 不依赖任何业务模块
- 数据层模块（skill/buff/equipment/inventory）不依赖逻辑层
- 逻辑层模块可依赖 core + 数据层
- 表现层只通过 Message/ViewModel 与逻辑层通信

**Rationale:**
Feature First 比技术分层更符合 SRPG 的业务复杂度。战斗、装备、Buff 等领域各自有独立的状态机和规则，按业务拆分使得每个模块的职责边界清晰，避免出现"components.rs 巨文件"和"systems.rs 巨文件"。

---

## 四、ADR-002 插件分层与注册顺序

**Decision:**
插件按四层注册：核心层 → 数据层 → 逻辑层 → 表现层，注册顺序保证依赖先初始化。

**Consequences:**
- (+) 依赖关系显式可见
- (+) 初始化顺序可控
- (-) 新增插件需确认层级归属

**Plugin Registration Order (from main.rs):**

```
1. 核心层：EffectPlugin, ModifierRulePlugin, AttributeDefPlugin, TagDefPlugin
2. 数据层：SkillPlugin, BuffPlugin, AiBehaviorPlugin, EquipmentPlugin, InventoryPlugin
3. 逻辑层：AssetsPlugin, TurnPlugin, MapPlugin, CharacterPlugin, BattlePlugin, AiPlugin
4. 表现层：UiPlugin, InputPlugin, DebugPlugin
```

**Rationale:**
数据层注册表（SkillRegistry, BuffRegistry 等）必须在逻辑层系统运行前就绪。表现层必须最后注册，确保所有 Message 类型已注册。

---

## 五、ADR-003 Effect Pipeline 架构

**Decision:**
战斗效果采用三步管线：`generate → modify → execute`，通过 `OnEnter(TurnPhase::ExecuteAction)` 链式调度。新增效果类型只需实现 `EffectHandler` trait 并注册，无需修改管线代码。

**Consequences:**
- (+) 效果类型可扩展（开闭原则）
- (+) 伤害分解（DamageBreakdown）天然可追踪
- (+) 修饰规则数据驱动，新增规则不改代码
- (-) 管线参数较多（通过内联函数缓解）

**Pipeline Flow:**

```
CombatIntent (Resource)
       ↓
generate_combat_effects
  ├─ EffectHandlerRegistry 分发
  ├─ EffectDef → PendingEffectData
  ├─ trigger_on_attack_traits
  └─ 推入 EffectQueue
       ↓
modify_effects
  ├─ ModifierRuleRegistry 应用规则
  ├─ 记录 base_amount + ModifierEntry[]
  └─ 更新 amount
       ↓
execute_effects
  ├─ 扣血/加Buff/击杀判定
  ├─ 发送 DamageApplied/HealApplied/CharacterDied Message
  └─ 清空 EffectQueue
```

**Extension Points:**
- `EffectHandler` trait：新增效果类型（如 Summon, Dispel）
- `ModifierCalculator` trait：新增修饰规则（如 ElementalResonance）
- `TraitTrigger` 枚举：新增触发时机（如 OnCrit, OnMiss）

**Rationale:**
SRPG 战斗链复杂（装备→护盾→角色→被动技能），管线模式将每步职责隔离，同时保证每步可观测、可回溯。DamageBreakdown 记录 generate→modify→execute 全链路数据，支持战斗回放和调试。

---

## 六、ADR-004 属性系统架构

**Decision:**
属性系统采用三层架构：核心属性（base + modifier stack）→ 衍生属性（实时计算）→ 生命资源（current/max 分离）。所有属性修改统一走 `ModifierSource` 管线。

**Consequences:**
- (+) 属性来源可追踪（Buff/装备/Trait 各有 ModifierSource）
- (+) 衍生属性永远一致（实时计算，无缓存失效问题）
- (+) Buff/装备移除时自动清理修饰符
- (-) 高频查询衍生属性有计算开销（当前可接受）

**Attribute Architecture:**

```
Attributes Component
├── base: HashMap<AttributeKind, f32>     # 8维核心属性基础值
├── current_hp / current_mp / current_stamina  # 生命资源当前值
├── base_attack_range: u32                # 基础攻击范围
└── modifiers: Vec<AttributeModifierInstance>  # 修饰符栈

ModifierSource (枚举)
├── Base                                 # 种族/职业基础值
├── Buff(BuffInstanceId)                 # Buff 来源
├── Equipment(EquipmentSlot)             # 装备来源
├── Trait(String)                        # Trait 来源
└── Temporary(String)                    # 临时修饰

Derived Stat 计算公式（集中管理）：
  MaxHp       = 5 + Vitality * 5
  MaxMp       = Intelligence * 5
  Attack      = Might * 2
  Defense     = Vitality
  MagicAttack = Intelligence * 2
  Initiative  = Agility * 2
  MoveRange   = Agility / 2
  ...
```

**Rationale:**
Primary/Derived 分离保证属性一致性。ModifierSource 枚举使得 Buff/装备移除时能精确清理对应修饰符，避免"到处直接修改最终属性"的问题。

---

## 七、ADR-005 Trait + Modifier 统一扩展体系

**Decision:**
所有能力来源（种族/职业/天赋/装备/Buff）统一通过 Trait + Modifier 体系表达。Trait 定义触发时机和效果，Modifier 定义属性修饰。运行时通过 `TraitCollection` 组件聚合。

**Consequences:**
- (+) 能力来源统一，新增来源不改核心代码
- (+) Trait 触发链可追踪
- (-) Trait 效果分发需要注册表

**Trait Architecture:**

```
TraitDefinition (RON) → TraitData (运行时)
├── trigger: TraitTrigger    # Passive/OnTurnStart/OnAttack/OnHit/OnKill
└── effects: Vec<TraitEffect>
    ├── GrantTag(GameplayTag)
    ├── ModifyAttribute(AttributeModifierDef)
    └── ApplyBuff { buff_id, duration }

TraitCollection Component  # 挂载在 Unit Entity 上
├── entries: Vec<TraitEntry>
│   ├── source: TraitSource  # Race/Class/Equipment/Buff/Talent
│   └── data: TraitData
```

**Trait Source Mapping:**

| 来源 | TraitSource | 示例 |
|------|-------------|------|
| 种族 | Race | 飞行（忽略地形消耗） |
| 职业 | Class | 战士（近战加成） |
| 装备 | Equipment | 火焰武器（OnAttack 施加燃烧） |
| Buff | Buff | 狂暴（OnTurnStart 增加攻击） |
| 天赋 | Talent | 龙裔（OnKill 恢复HP） |

**Rationale:**
组合优于继承。角色由 Trait + Modifier 集合构成，而不是继承树。装备 = Modifier + Trait + Tag + Rule 四层架构，Buff/Debuff = 临时 Trait，所有能力来源统一进入 Modifier 管线。

---

## 八、ADR-006 数据驱动架构

**Decision:**
所有配置内容（单位/技能/装备/Buff/地形/AI行为/关卡）通过 RON 文件外部定义，代码只解释配置。采用 `Definition / Instance` 分离和 `Rule / Content` 分离。

**Consequences:**
- (+) 新增内容不改代码
- (+) 配置可热重载
- (-) 配置结构需保持稳定
- (-) 配置引用关系需校验

**Registry Architecture:**

```
RegistryLoader trait (通用加载器)
├── load_from_dir()        # 每文件单条记录
├── load_from_dir_vec()    # 每文件多条记录
├── register_item()        # 注册一条记录
└── register_defaults()    # 兜底默认值

各注册表：
├── UnitTemplateRegistry   # assets/units/*.ron
├── SkillRegistry          # assets/skills/*.ron
├── BuffRegistry           # assets/buffs/*.ron
├── EquipmentRegistry      # assets/equipment/*.ron
├── ItemRegistry           # assets/items/*.ron
├── TerrainRegistry        # assets/terrains/*.ron
├── LevelRegistry          # assets/maps/*.ron
├── AiBehaviorRegistry     # assets/ai/*.ron
├── ModifierRuleRegistry   # assets/modifier_rules/*.ron
└── TraitRegistry          # assets/traits/*.ron
```

**Definition / Instance 分离:**

| 模块 | Definition（不可变配置） | Instance（运行时状态） |
|------|------------------------|----------------------|
| Buff | BuffData / BuffDef | BuffInstance / ActiveBuffs |
| Skill | SkillData / SkillDef | SkillSlots / SkillCooldowns |
| Equipment | EquipmentDef | EquipmentInstance / EquipmentSlots |
| Item | ItemDef | ItemInstance / ItemStack |
| Unit | UnitTemplate | Unit + Attributes + ActiveBuffs |

**RON 反序列化双类型模式:**
每个领域定义两种类型：`XxxDef`（RON 反序列化用，使用 `TagName` 字符串）和 `XxxData`（运行时，使用 `GameplayTag` 位掩码）。`From<XxxDef> for XxxData` 实现转换。

**Rationale:**
RON 使用字符串（TagName）可读性好，运行时使用位掩码（GameplayTag）查询性能 O(1)。双类型模式兼顾配置可读性和运行时性能。

---

## 九、ADR-007 回合状态机架构

**Decision:**
回合管理采用 `AppState`（主状态）+ `TurnPhase`（子状态）+ `TurnOrder`（行动队列）三层架构。系统通过 `OnEnter(TurnPhase)` 钩子调度，AI 通过队列驱动。

**Consequences:**
- (+) 阶段转换显式可控
- (+) AI 和玩家共享同一套行动流程
- (-) 阶段数量需控制

**State Machine:**

```
AppState
├── MainMenu
├── InGame
│   └── TurnPhase (SubState)
│       ├── SelectUnit      # 选择单位 / AI 自动决策
│       ├── MoveUnit        # 移动阶段
│       ├── ActionMenu      # 行动菜单
│       ├── SelectTarget    # 选择攻击目标
│       ├── ExecuteAction   # 执行攻击（Effect Pipeline）
│       ├── WaitAction      # 待机
│       └── TurnEnd         # 回合结束
└── GameOver
```

**Turn Flow:**

```
SelectUnit
  ├─ [Player] 点击单位 → UiCommand::SelectUnit → MoveUnit
  └─ [Enemy]  AiTimer 到期 → 自动决策 → MoveUnit → ExecuteAction → WaitAction
       ↓
MoveUnit
  ├─ [Player] 点击目标格 → UiCommand::MoveUnit → ActionMenu
  └─ [Enemy]  自动移动 → ExecuteAction
       ↓
ActionMenu → SelectTarget / Wait / Cancel
       ↓
SelectTarget → ExecuteAction
       ↓
ExecuteAction
  ├─ generate → modify → execute (Pipeline)
  ├─ trigger_on_attack_traits / on_hit_traits / on_kill_traits
  └─ → WaitAction
       ↓
WaitAction → 下一个单位 / TurnEnd
       ↓
TurnEnd → resolve_status_effects → SelectUnit (新回合)
```

**Rationale:**
队列驱动模式（TurnOrder）比阵营轮换更灵活，支持敏捷排序、中途死亡移除、AI 插入等场景。SubState 保证 TurnPhase 仅在 InGame 时激活。

---

## 十、ADR-008 地图系统架构

**Decision:**
地图采用 Grid 数据优先，Tile 不作为 Entity。地形数据由 `TerrainGrid` 纯数据存储，单位占位由 `OccupancyGrid` 独立管理，寻路数据运行时生成。

**Consequences:**
- (+) 地图数据与渲染分离
- (+) 寻路不依赖 Entity 查询
- (+) OccupancyGrid 实时更新，O(1) 查询
- (-) 地形修改需手动同步 TerrainGrid

**Map Architecture:**

```
GameMap (Resource)           # 地图尺寸 + 坐标转换
├── width, height, tile_size
├── coord_to_world()
└── world_to_coord()

TerrainGrid (Resource)      # 地形数据唯一真相源
├── cells: HashMap<IVec2, String>  # 坐标→地形ID
└── 从 LevelConfig 构建

OccupancyGrid (Resource)    # 单位占位唯一真相源
├── occupied: HashMap<IVec2, Entity>  # 坐标→占用Entity
├── 每帧从 Unit 位置重建
└── is_occupied_except() 支持自身移动

TerrainRegistry (Resource)  # 地形定义注册表
├── terrains: HashMap<String, TerrainDef>
└── move_cost, defense_bonus, passable, color

寻路：
├── BFS 计算可移动范围
├── TerrainCostCalculator trait（支持标签修正消耗）
└── reconstruct_path() 回溯最短路径
```

**Rationale:**
Tile 不作为 Entity 避免了大量无行为 Entity 的开销。TerrainGrid + OccupancyGrid 分离使得地形数据和占位数据各自独立更新，寻路算法只需读取两个 Resource。

---

## 十一、ADR-009 UI 架构

**Decision:**
UI 采用三层架构：`UiCommand`（意图层）→ `ViewModel`（状态层）→ `Panel/Widget`（展示层）。UI 不保存业务真相，只展示状态。

**Consequences:**
- (+) UI 与逻辑完全解耦
- (+) UI 可独立开发和测试
- (-) ViewModel 需要手动同步

**UI Architecture:**

```
Input Layer (input.rs)
├── Pointer<Click> Observer → UiCommand Message
├── cursor_to_coord → UiCommand Message
└── 右键取消 → UiCommand::Cancel

Command Layer (command_handler.rs)
├── handle_ui_commands: UiCommand → 游戏状态变更
├── 不直接操作 UI 组件
└── 通过 NextState<TurnPhase> / CombatIntent / Commands 驱动逻辑

ViewModel Layer (view_models.rs)
├── SelectedUnitView    # 选中单位信息
├── TurnInfoView        # 回合信息
├── CombatPreviewView   # 战斗预览
├── HoveredEntity       # 悬停实体
├── GameOverState       # 游戏结束状态
└── 各 Entry 类型（CoreAttrEntry, BuffEntry, SkillEntry 等）

Presentation Layer
├── panels/             # 功能面板
│   ├── unit_info.rs    # 单位信息面板
│   ├── turn_indicator.rs # 回合指示器
│   ├── combat_log_panel.rs # 战斗日志
│   ├── inventory_panel.rs  # 背包面板
│   └── action_hint.rs  # 操作提示
├── widgets/            # 基础组件
│   ├── layout.rs       # 布局工具
│   ├── popup.rs        # 弹窗
│   └── resource_bar.rs # 资源条
├── action_menu.rs      # 行动菜单
├── combat_vfx_handler.rs # 战斗飘字
└── combat_log_handler.rs # 日志表现
```

**Message Flow (UI → Logic → UI):**

```
User Click → Pointer<Click> Observer
  → UiCommand::SelectUnit Message
    → handle_ui_commands System
      → commands.entity(e).insert(Selected)
      → show_move_range()
    → update_selected_unit_view System
      → SelectedUnitView Resource 更新
        → UnitInfo Panel 刷新显示
```

**Rationale:**
UI 监听状态变化刷新自己，业务逻辑不直接操作 UI。ViewModel 层隔离了 ECS Query 复杂性，Panel 系统只读 ViewModel Resource。

---

## 十二、ADR-010 AI 架构

**Decision:**
AI 采用数据驱动 + 策略 trait 扩展点架构。行为定义从 RON 加载，运行时通过 `AiStrategyRegistry` trait 对象分发。AI 与玩家共享同一套 Effect Pipeline。

**Consequences:**
- (+) 新增 AI 策略不改核心代码
- (+) 不同单位可有不同行为模式
- (-) trait 对象有轻微运行时开销

**AI Architecture:**

```
AiBehavior (RON → 运行时)
├── target_strategy: String   # "Nearest" / "Weakest" / "MostDangerous"
├── move_strategy: String     # "Aggressive" / "Cautious" / "Support"
├── skill_strategy: String    # "PreferSpecial" / "PreferBasic" / "ByPriority"
└── skill_priority: Vec<String>

AiStrategyRegistry (Resource)
├── target_selectors: HashMap<String, Box<dyn TargetSelector>>
├── move_selectors: HashMap<String, Box<dyn MoveSelector>>
└── skill_selectors: HashMap<String, Box<dyn SkillSelector>>

AI 决策流程：
SelectUnit (Enemy turn)
  → AiTimer tick
  → select_target_coord()  # 目标选择
  → select_move_coord()    # 移动选择
  → select_skill()         # 技能选择
  → 设置 CombatIntent
  → NextState(ExecuteAction)  # 进入统一 Effect Pipeline
```

**Rationale:**
AI 不走独立的攻击逻辑，而是设置 `CombatIntent` 后进入统一的 Effect Pipeline，保证玩家和 AI 的伤害计算、Buff 触发、修饰规则完全一致。

---

## 十三、ADR-011 跨模块通信矩阵

**Decision:**
模块间通信通过 Message 实现松耦合，模块内部优先函数调用。

**Message Registry:**

| Message | 发送方 | 接收方 | 用途 |
|---------|--------|--------|------|
| `UiCommand` | input → command_handler | UI→Logic 意图 |
| `DamageApplied` | battle/execute | ui/combat_vfx, ui/combat_log, battle/record | 伤害通知 |
| `HealApplied` | battle/execute | ui/combat_log, battle/record | 治疗通知 |
| `CharacterDied` | battle/execute | battle/events (移除队列+despawn), ui/combat_log, battle/record | 死亡通知 |
| `StunApplied` | buff/resolve | ui/combat_log, battle/record | 晕眩通知 |
| `DotApplied` | buff/resolve | ui/combat_log, battle/record | DoT 通知 |
| `HotApplied` | buff/resolve | ui/combat_log, battle/record | HoT 通知 |
| `EquipItem` | ui/inventory_panel | equipment/equip | 穿戴装备 |
| `UnequipItem` | ui/inventory_panel | equipment/equip | 脱卸装备 |
| `ItemEquipped` | equipment/equip | ui/combat_log | 装备已穿 |
| `ItemUnequipped` | equipment/equip | ui/combat_log | 装备已脱 |
| `EquipFailed` | equipment/equip | ui (未来) | 穿戴失败 |
| `UseItem` | ui/inventory_panel | inventory/use_item | 使用物品 |
| `ItemUsed` | inventory/use_item | ui (未来) | 使用完成 |
| `TransferItem` | ui | inventory/transfer | 物品转移 |
| `ItemTransferred` | inventory/transfer | ui (未来) | 转移完成 |
| `TurnStarted` | turn | battle/record | 回合开始 |
| `TurnEnded` | turn | battle/record | 回合结束 |
| `ForceEndTurn` | ui/command_handler | turn | 强制结束回合 |

**Rationale:**
Message 实现跨 Feature 广播，发送方不需要知道接收方是谁。模块内部（如 `apply_buff()` / `remove_buff()`）直接函数调用，避免过度事件化。

---

## 十四、ADR-012 调试与可观测性架构

**Decision:**
调试工具作为独立 Feature 模块，使用 bevy_egui 实现运行时面板，通过 `DebugPanelState` Resource 管理显隐。核心系统支持单步执行与状态回溯。

**Consequences:**
- (+) 调试工具不影响生产代码
- (+) 所有面板可独立开关
- (-) egui 面板需要单独维护

**Debug Architecture:**

```
DebugPanelState (Resource)
├── show_battle_debugger: bool    # F1
├── show_buff_viewer: bool        # F2
├── show_damage_attribute: bool   # F4
├── damage_attribute_tab: u32     # F4 Tab 切换
└── show_turn_queue: bool         # F5

调试面板：
├── Battle Debugger (F1)   # 回合状态+当前行动单位+事件统计
├── Buff Viewer (F2)       # Buff 列表+修饰符详情
├── Damage & Attribute (F4) # 伤害分解 / 属性查看器 Tab 切换
├── Turn Queue Viewer (F5) # 行动队列预览
├── Grid Viewer            # 格子可视化
├── Gizmos Overlay (F3)    # 游戏内覆盖层
├── Stepping Control (F6/F7) # 暂停/单步
└── World Inspector (F12)  # bevy-inspector-egui

可观测性：
├── BattleRecord           # 结构化战斗记录（支持回放）
├── DamageBreakdown        # 伤害全链路分解
├── tracing 结构化日志      # 统一日志框架
└── bevy_remote (BRP)      # 远程调试协议
```

**Rationale:**
Inspector、Replay、Debug Panel 优先于日志堆砌。关键系统必须支持单步执行与状态回溯。DamageBreakdown 记录 generate→modify→execute 全链路，是战斗回放的核心数据。

---

## 十五、模块依赖关系图

```
                    ┌─────────┐
                    │  core   │  ← 无外部依赖
                    └────┬────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
    ┌─────┴─────┐  ┌────┴────┐  ┌─────┴─────┐
    │ attribute │  │  effect │  │    tag    │
    │ modifier  │  │ handler │  │ modifier  │
    └─────┬─────┘  └────┬────┘  │   rule   │
          │              │       └─────┬─────┘
          │              │             │
    ┌─────┴──────────────┴─────────────┴─────┐
    │                                         │
    │  skill ← buff ← equipment ← inventory  │  ← 数据层
    │                                         │
    └─────┬──────────────┬──────────────┬─────┘
          │              │              │
    ┌─────┴─────┐  ┌────┴────┐  ┌─────┴─────┐
    │ character │  │  battle │  │    map    │  ← 逻辑层
    └─────┬─────┘  └────┬────┘  └─────┬─────┘
          │              │              │
    ┌─────┴─────┐  ┌────┴────┐         │
    │    ai     │  │  turn   │─────────┘
    └───────────┘  └────┬────┘
                        │
              ┌─────────┼─────────┐
              │         │         │
        ┌─────┴──┐ ┌───┴───┐ ┌──┴───┐
        │   ui   │ │ input │ │debug │  ← 表现层
        └────────┘ └───────┘ └──────┘
```

---

## 十六、关键设计模式总结

| 模式 | 应用位置 | 目的 |
|------|----------|------|
| **Definition / Instance 分离** | Buff, Skill, Equipment, Item, Unit | 配置不可变，运行时可变 |
| **Registry + RON** | 所有数据模块 | 数据驱动，新增内容不改代码 |
| **Effect Pipeline** | battle/pipeline | 战斗效果可追踪、可扩展 |
| **Modifier Stack** | core/attribute | 属性来源统一，移除可清理 |
| **Trait + Modifier 统一扩展** | character/traits | 所有能力来源统一机制 |
| **Command Pattern** | UiCommand | UI 不操作 ECS，只发意图 |
| **ViewModel** | ui/view_models | UI 只读 ViewModel，不直接 Query |
| **Strategy Pattern** | ai/strategy | AI 策略可扩展，数据驱动 |
| **Tag Bitmask** | core/tag | O(1) 标签查询 |
| **Hook + Observer + Message** | 全局 | 固有行为/局部响应/跨模块广播 |
| **SubState** | turn/state | 回合阶段仅在 InGame 激活 |
| **Required Components** | character/components | Unit 生成时自动插入依赖组件 |

---

**文档版本**: v1.0  
**生成日期**: 2026-06-11  
**项目**: Bevy SRPG
