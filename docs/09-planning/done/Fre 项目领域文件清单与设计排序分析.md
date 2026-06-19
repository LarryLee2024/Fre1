现在我已完整阅读了架构文档。以下是我的全面分析。

---

## Fre 项目领域文件清单与设计排序分析

### 分析方法论

基于架构文档的七层结构和 DDD 领域驱动设计原则，我按以下维度对项目进行全面领域分析：

1. **架构截面**：Capabilities（15 机制域）+ Domains（15 业务域），共计 30 个领域
2. **依赖关系**：按 Capabilities 内部依赖图（基石→聚合→逻辑骨架→行为表现→运行时）逐层展开
3. **交付节奏**：对齐架构文档第八十节"阶段规划"（P0-P7）
4. **内聚原则**：遵循"一个领域只负责一件事"，避免领域膨胀

### 一、领域总览：30 个领域文件清单

项目需要 30 份领域规则文档，分为两大领域族：

---

### 领域族 A：Capabilities（15 个机制领域）

> 玩法无关的通用能力机制，对标 UE GAS。所有业务复杂度向 Domain 沉淀。

| # | 领域文件 | 分组 | 核心职责 |
|---|---------|------|---------|
| A01 | `tag_domain.md` | 核心基石 | 标签定义、位掩码层级、标签查询与同步 |
| A02 | `attribute_domain.md` | 核心基石 | 属性定义、基础值/当前值分离、属性分类 |
| A03 | `modifier_domain.md` | 核心基石 | 修改器运算（Add/Mul/Override）、优先级排序 |
| A04 | `aggregator_domain.md` | 核心基石 | 属性聚合管线（Base→Add→Mul→Override→Clamp→Final） |
| A05 | `gameplay_context_domain.md` | 聚合层 | 统一数据载体、溯源链、上下文构建 |
| A06 | `spec_domain.md` | 逻辑骨架 | 三层分离（Def→Spec→Instance）、Spec 生命周期 |
| A07 | `condition_domain.md` | 逻辑骨架 | 统一条件检查（Tag 需求/属性阈值/资源充足） |
| A08 | `trigger_domain.md` | 逻辑骨架 | 技能激活条件、触发类型、触发条件评估 |
| A09 | `event_domain.md` | 逻辑骨架 | 系统间结构化通信、事件总线、订阅管理 |
| A10 | `ability_domain.md` | 行为表现 | 技能状态机、消耗/冷却规则、激活生命周期 |
| A11 | `targeting_domain.md` | 行为表现 | 目标类型、范围筛选、网格目标选择 |
| A12 | `execution_domain.md` | 行为表现 | 执行计算分类、自定义执行、伤害/治疗计算 |
| A13 | `effect_domain.md` | 行为表现 | 效果生命周期（施加→持续→到期→移除） |
| A14 | `stacking_domain.md` | 行为表现 | 堆叠类型、同源/异源堆叠、溢出处理 |
| A15 | `cue_domain.md` | 行为表现 | 表现层信号（VFX/SFX/动画）、触发时机 |

---

### 领域族 B：Domains（15 个业务领域）

> 承载全部玩法复杂度，玩法迭代的唯一载体。

| # | 领域文件 | 分组 | 核心职责 |
|---|---------|------|---------|
| B01 | `combat_domain.md` | P2 战斗 | 战斗编排、回合规则、胜负判定、伤害结算 |
| B02 | `spell_domain.md` | P2 战斗 | 法术释放、法术位、专注、豁免、升环 |
| B03 | `reaction_domain.md` | P2 战斗 | 机会攻击、援护格挡、法术反制、反应队列 |
| B04 | `tactical_domain.md` | P2 战术 | 网格移动、夹击/背刺/高地、掩体 |
| B05 | `terrain_domain.md` | P2 地形 | 地形效果、陷阱、表面交互、困难地形 |
| B06 | `progression_domain.md` | P3 养成 | 经验等级、职业多职、天赋、子职、ASI |
| B07 | `inventory_domain.md` | P3 养成 | 背包、装备槽位、物品使用、掉落规则 |
| B08 | `party_domain.md` | P3 养成 | 编队、换人、羁绊、队伍全局 buff |
| B09 | `camp_rest_domain.md` | P3 养成 | 长休/短休、资源恢复、营地事件 |
| B10 | `narrative_domain.md` | P4 叙事 | 对话流程、分支选项、故事标记、演出控制 |
| B11 | `quest_domain.md` | P4 叙事 | 任务生命周期、目标追踪、奖励发放 |
| B12 | `faction_domain.md` | P4 叙事 | 阵营声望、关系演变、声望奖励 |
| B13 | `economy_domain.md` | P5 经济 | 货币体系、商店买卖、物价波动、赃物交易 |
| B14 | `crafting_domain.md` | P5 经济 | 装备锻造、附魔词条、升级、分解 |
| B15 | `summon_domain.md` | P5 经济 | 召唤物生命周期、专注维持、AI 控制 |

---

### 二、领域设计排序

排序原则按 **5 个维度综合评定**：

1. **依赖度**（权重 30%）：被依赖越多的领域越优先 — 如 Tag/Attribute 被所有下游依赖
2. **基础性**（权重 25%）：是否构成其他领域的前提 — 如 Condition 是所有技能/装备的前置检查
3. **数据流方向**（权重 20%）：数据从产生到消费的流向 — 如 Modifier→Aggregator→Attribute
4. **模块耦合度**（权重 15%）：高内聚低耦合领域可并行设计，高耦合领域需串行
5. **架构阶段**（权重 10%）：对齐 P0-P7 阶段规划，确保可落地

#### 排序结果

```
第1梯队 ─ 核心基石（P0/无依赖）
  A01 Tag ─── 标签系统（最底层，所有领域依赖标签标识）
  A02 Attribute ─── 属性系统（最底层，所有数值系统依赖属性定义）
  A03 Modifier ─── 修改器（依赖 Attribute，被 Aggregator/Effect 依赖）
  A04 Aggregator ─── 聚合器（依赖 Modifier+Attribute，能力系统核心）

  排序依据：Tag 和 Attribute 是唯二零依赖的基础领域，必须最先定义。
  Modifier 引用 AttributeId（依赖 Attribute），Aggregator 编排 Modifier 执行顺序
  （依赖 Modifier+Attribute）。这4个领域构成能力系统的"数学公理"，上层所有机制
  都是它们的组合。

第2梯队 ─ 聚合与骨架（P1/依赖基石）
  A05 GameplayContext ─── 上下文（被所有下游领域依赖为数据载体）
  A06 Spec ─── 规格（依赖 Tag，Def→Spec→Instance 的核心桥梁）
  A07 Condition ─── 条件（依赖 Tag+Attribute，被 Ability/Effect/Inventory 依赖）
  A08 Trigger ─── 触发器（依赖 Tag+Condition，被 Ability 系统消费）
  A09 Event ─── 事件总线（零依赖，但被全系统消费）

  排序依据：GameplayContext 是跨全系统的统一数据载体，所有 Effect/Execution 都
  需要它，必须优先于行为表现层。Condition 在下游被大量引用（技能激活条件、装备
  条件、对话条件），必须在行为层之前稳定下来。Trigger 是 Ability 的前置条件，
  Event 是 Domain 间通信的唯二通道（另一个是 Trigger），属于基础通信设施。

第3梯队 ─ 行为表现（P1/依赖骨架）
  A10 Ability ─── 技能（依赖 Condition+Trigger+Spec+GameplayContext）
  A11 Targeting ─── 目标选择（依赖 GameplayContext）
  A12 Execution ─── 执行计算（依赖 GameplayContext）
  A13 Effect ─── 效果（依赖 Modifier+Tag+Condition+GameplayContext）
  A14 Stacking ─── 堆叠（依赖 Effect）
  A15 Cue ─── 表现信号（依赖 Event，被 Infra 层消费）

  排序依据：这是 Capabilities 最复杂的层次，Ability 依赖前面几乎所有领域，
  必须在前两层稳定后才能设计。Effect 是 Skill→Buff→Damage 链路的中间环节，
  被 Stacking 依赖。Cue 处于逻辑层与表现层的边界，依赖 Event 设施。

第4梯队 ─ 战斗核心（P2/依赖 Capabilities 全15领域）
  B04 Tactical ─── 战术（依赖 Targeting+Condition，网格是战斗空间基础）
  B05 Terrain ─── 地形（依赖 Effect+Event，附着在网格上）
  B01 Combat ─── 战斗编排（依赖 Tactical+Terrain+Ability+Effect+
                  Execution+Aggregator+Condition+Event）
  B02 Spell ─── 法术（依赖 Combat+Ability+Effect+Condition+Event）
  B03 Reaction ─── 反应（依赖 Combat+Ability+Event+Trigger）

  排序依据：Tactical 是 SRPG 战斗的空间基础（网格/移动/站位），必须先于 Combat
  定义。Terrain 附着于网格，是 Tactical 的自然扩展。Combat 是战斗全流程编排者，
  依赖前面所有领域，是整个项目复杂度最高的领域，必须在 Capabilities 和空间/地形
  领域稳定后设计。Spell 和 Reaction 是 Combat 的子流程，必须在 Combat 框架确定
  后才能精确定义。

第5梯队 ─ 成长养成（P3/依赖 Capabilities + 战斗域）
  B06 Progression ─── 成长（依赖 Attribute+Condition+Event）
  B07 Inventory ─── 背包（依赖 Condition+Effect+Modifier+Event）
  B08 Party ─── 队伍（依赖 Event+Modifier）
  B09 CampRest ─── 营地（依赖 Event+Effect+Combat）

  排序依据：Progression（经验/等级/天赋）定义了角色的成长曲线，是养成系统的
  基础。Inventory 管理物品的获取/使用/装备，依赖 Progression 的等级条件
  （装备等级需求）。Party 管理队伍组成，依赖 CampRest 的营地换人机制。
  CampRest 涉及战斗间恢复，依赖 Combat 的战斗结束事件。

第6梯队 ─ 叙事内容（P4/依赖养成域）
  B10 Narrative ─── 叙事（依赖 Event+Condition+Faction）
  B11 Quest ─── 任务（依赖 Event+Condition+Inventory+Progression）
  B12 Faction ─── 阵营（依赖 Event）

  排序依据：Faction 是叙事系统的基础设施（对话分支/任务奖励受声望影响），
  但本身依赖最轻，可独立设计。Narrative 依赖 Faction 的声望数据（不同声望
  触发不同对话）。Quest 是叙事的结构化载体，依赖 Inventory（物品奖励）和
  Progression（经验奖励），必须在两者稳定后设计。

第7梯队 ─ 经济系统（P5/依赖养成+叙事域）
  B13 Economy ─── 经济（依赖 Event+Faction）
  B14 Crafting ─── 制作（依赖 Effect+Modifier+Inventory）
  B15 Summon ─── 召唤（依赖 Effect+Condition+Event+Combat）

  排序依据：Economy 是货币/物价/商店系统，依赖 Faction 的声望折扣。
  Crafting 是 Inventory 的扩展（产出物品进入背包），依赖装备和效果系统。
  Summon 涉及战斗中的额外单位，依赖 Combat 的战斗规则和 Effect 的持续时间
  系统，是战斗域的扩展能力。
```

#### 整体依赖拓扑图

```
                    Capabilities 领域
                    ┌──────────────────────────────────┐
                    │  Tag  Attribute  Modifier         │ ← 第1梯队
                    │              ↓                    │
                    │         Aggregator                │
                    └──────────────────────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │  GameplayContext  Spec            │
                    │  Condition  Trigger  Event         │ ← 第2梯队
                    └──────────────────────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │  Ability  Targeting  Execution    │
                    │  Effect  Stacking  Cue           │ ← 第3梯队
                    └──────────────────────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │         业务 Domain                │
                    │                                   │
                    │  Tactical → Terrain               │ ← 第4梯队（P2）
                    │      ↓                            │
                    │    Combat → Spell → Reaction      │
                    │                                   │
                    │  Progression → Inventory          │ ← 第5梯队（P3）
                    │      ↓                            │
                    │  Party ←→ CampRest                │
                    │                                   │
                    │  Faction → Narrative → Quest      │ ← 第6梯队（P4）
                    │                                   │
                    │  Economy → Crafting → Summon      │ ← 第7梯队（P5）
                    └──────────────────────────────────┘
```

---

### 三、排序依据总体原则

| 原则 | 说明 | 应用示例 |
|------|------|---------|
| **依赖驱动** | 被依赖多的领域先设计 | Tag 被 20+ 领域引用，排第1 |
| **数据流向** | 数据产生者先于消费者 | Modifier 先于 Aggregator（产生修改数据→消费聚合） |
| **语义完整性** | 实体定义先于行为规则 | Attribute 定义属性类型先于 Progression 的经验消耗 |
| **阶段对齐** | 优先当前阶段需要的领域 | Combat 虽依赖众多但为 P2 核心，在其依赖稳定后立即介入 |
| **并行潜力** | 同一梯队内部可并行设计 | 第1梯队 Tag/Attribute 无相互依赖，可 2-4 人并行 |
| **风险递减** | 高风险/高不确定性领域后置 | Summon（涉及 AI+定位+生命周期组合）排最后 |

### 四、设计优先级建议

**最高优先级 — 立即开始（P0-P1）**：A01~A15（全部 Capabilities 领域）
- 这是所有 Domains 的基础，没有稳定的 Capabilities 层，业务域无法设计
- A01~A04 可并行（零依赖彼此）
- A05~A09 可在 A01~A04 完成后立即启动
- A10~A15 需前两个子梯队完成后开始

**高优先级 — 紧随其后（P2）**：B01~B05（战斗核心域）
- 这是项目能否"玩起来"的关键
- B04 Tactical 优先于 B01 Combat（空间是 SRPG 的基础）
- B02 Spell 和 B03 Reaction 可作为 B01 的子流程并行设计

**中优先级（P3）**：B06~B09（成长养成域）
- 战斗能跑通后立即接入养成系统
- B06 Progression 优先（定义角色成长曲线）

**低优先级（P4-P5）**：B10~B15（叙事/经济域）
- 叙事和经济是"锦上添花"层，需要核心玩法稳定后再设计
- B12 Faction 可提前（依赖轻，但被 B10/B11/B13 依赖）

### 五、阶段执行计划

| 批次 | 领域文件 | 建议工期 | 并行可能性 |
|------|---------|---------|-----------|
| **Batch 0** | A01 Tag, A02 Attribute | 2个领域并行 | 完全并行（零依赖） |
| **Batch 1** | A03 Modifier, A04 Aggregator, A09 Event | 3个领域并行 | A03依赖A02，A04依赖A03+A02，但可同批次顺序设计 |
| **Batch 2** | A05 GameplayContext, A06 Spec, A07 Condition, A08 Trigger | 4个领域并行 | A05/A09可独立，A06/A07/A08依赖前序但可同一批次 |
| **Batch 3** | A10 Ability, A11 Targeting, A12 Execution, A13 Effect, A14 Stacking, A15 Cue | 6个领域顺序设计 | A14→A13→A10串行链，其余可并行 |
| **Batch 4** | B04 Tactical, B05 Terrain, B12 Faction | 3个领域并行 | Faction依赖轻可提前，Terrain与Tactical耦合小 |
| **Batch 5** | B01 Combat, B02 Spell, B03 Reaction | 3个领域 | B01设计框架后B02/B03并行精化 |
| **Batch 6** | B06 Progression, B07 Inventory | 2个领域 | 可并行（共享 Condition 但业务独立） |
| **Batch 7** | B08 Party, B09 CampRest | 2个领域 | 可并行（耦合度低） |
| **Batch 8** | B10 Narrative, B11 Quest | 2个领域 | B12完成后可并行 |
| **Batch 9** | B13 Economy, B14 Crafting, B15 Summon | 3个领域 | B13/B14可并行，B15需等待Combat稳定 |

**总计**：30 个领域文件，建议分 10 个批次（每个批次可 1-2 人负责），预计核心设计周期约 4-6 周（不含评审迭代）。

### 六、下一个动作建议

这份清单和排序已为接下来的领域建模做好准备。**建议从 Batch 0 的 A01 Tag 和 A02 Attribute 开始**——这两个领域零依赖彼此、无外部依赖，是项目的"公理层"，可以立即启动各自领域规则文档的详细撰写。是否需要我接下来对 Batch 0 的两个领域输出完整的领域模型文档？

从 Batch 0 的 A01 Tag 和 A02 Attribute 开始——这两个领域零依赖彼此、无外部依赖，是项目的"公理层"

进入 Batch 1（Modifier + Aggregator + Event）：
```
Modifier（A03）—— 依赖 Attribute（已就绪）
    ↓
Aggregator（A04）—— 依赖 Modifier + Attribute（两者已就绪）
    │
Event（A09）—— 零依赖，可并行设计
```

Batch 3 是 Capabilities 的"行为表现层"——这 6 个领域构成了技能系统的完整执行链路：
```
Spec → Trigger → Condition
                    │
                    ▼
Ability → Targeting → Execution → Effect → Stacking → Cue
  ├─ 技能激活        │         │        │         │       │
  ├─ 消耗/冷却       │         │        │         │       │
  └─ 状态管理     目标选择   伤害计算  生命周期   堆叠    表现信号
```