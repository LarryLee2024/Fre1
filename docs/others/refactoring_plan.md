# 系统性重构执行计划

> 依据：`docs/12.md`（代码评审优先级）、`docs/13.md`（DDD + ECS 融合）、`docs/14.md`（死代码清理）
> 原则：架构正确性 → 可维护性 → 扩展性 → 调试性 → 测试性 → 性能 → 风格
> 目标：消除架构债务，统一领域模型，补齐关键测试，为 3-5 年业务发展奠基

---

## 第零部分：指导原则（源自 12.md + 13.md + 14.md）

以下原则从三篇文档中提炼，贯穿整个重构过程，每个阶段的决策都应回溯到此：

### 0.1 DDD 战略原则（源自 13.md）

| # | 原则 | 含义 | 对应铁律 |
|---|------|------|---------|
| D1 | **领域优先于技术** | 模块围绕 Battle、Character、Inventory、Quest 组织 | Feature First |
| D2 | **统一领域语言** | 一个概念只有一个名字（Ubiquitous Language） | — |
| D3 | **限界上下文隔离** | Battle 不直接操作 Inventory（Bounded Context） | 模块边界 |
| D4 | **领域事件驱动协作** | Context 间通过 Message/Observer 协作，不直接调用 | Hook/Observer/Message |
| D5 | **规则集中管理** | 一个规则只有一个真相来源（Single Source of Truth） | 统一 Modifier 管线 |
| D6 | **ECS 负责执行，领域负责建模** | 不把企业 OOP 搬进 Bevy | Component 存状态，System 存行为 |

### 0.2 评审优先级（源自 12.md）

```
架构边界 → ECS 设计 → SRPG 规则统一性 → 扩展性 → 测试体系 → 调试能力 → 性能 → 命名与格式
★★★★★     ★★★★★     ★★★★★           ★★★★☆   ★★★★☆     ★★★★☆     ★★★☆☆   ★★☆☆☆
```

### 0.3 死代码清理十层法（源自 14.md）

每次重构完成后，必须按此清单审查：

| 层级 | 检查项 | 当前状态 |
|------|--------|---------|
| L1 | 死代码（Clippy dead_code/unused） | 仅 1 处 `#[allow(dead_code)]` |
| L2 | 幽灵 Feature（未注册 Plugin） | ✅ 无幽灵 Plugin |
| L3 | 永远不触发的 System（无 MessageWriter） | ⚠️ 4 个断路消息 |
| L4 | 重复规则系统 | ✅ 无重复规则 |
| L5 | 失效抽象（Trait 只有一个实现） | ✅ 所有 Trait 有 3-7 个实现 |
| L6 | 僵尸配置字段 | ⚠️ 4 个僵尸字段 |
| L7 | 历史兼容层（Old/New 并存） | ✅ 无历史兼容层 |
| L8 | 测试覆盖反向找（零测试 = 可能无人用） | ⚠️ ai/decision 零测试 |
| L9 | 战斗链路审计（谁产生谁消费） | ⚠️ 7 条消息链路不完整 |
| L10 | CODE GRAVEYARD 审查表 | ❌ 尚未建立 |

### 0.4 12.md 专属评审项

| 评审项 | 问题 | 当前状态 |
|--------|------|---------|
| Trait 体系 | 是否已统一职业/装备/Buff/天赋 | ✅ 统一 |
| Modifier 体系 | 是否所有属性修改都走同一管线 | ✅ 统一 |
| BattleRecord | 是否已成为战斗真相来源（SSOT） | ⚠️ 需验证 — CombatLog 与 BattleRecord 是否存在真相分裂 |
| Definition/Instance | 是否彻底隔离 | ✅ 核心模块已分离 |
| Message/Observer | 是否形成清晰的战斗链路 | ⚠️ 4 条断路 + 3 条无消费 |
| 数据驱动 | 新增职业/技能是否无需改核心代码 | ⚠️ 需验证 — 是否存在 match JobType 硬编码 |

---

## 第一部分：现状诊断

### 1.1 做得好的（不需要重构）

| 维度 | 评估 | 证据 |
|------|------|------|
| Feature First | ★★★★★ | 13 个顶级模块全部按业务划分，无技术拆分目录 |
| Definition / Instance 分离 | ★★★★☆ | buff/equipment/inventory 严格分离，character/skill 用 Component 替代 Instance（合理） |
| 数据驱动 | ★★★★☆ | 14 个 Def 类型 + 18 个 Registry，RON 配置体系成熟 |
| 统一 Modifier 管线 | ★★★★★ | 伤害/属性/状态效果各一套管线，无重复规则 |
| Trait 体系 | ★★★★★ | 8 个 Trait 全部有 3-7 个实现，都是真正的扩展点 |
| 测试体系 | ★★★★☆ | 621 个测试，5 层金字塔，核心规则覆盖优秀 |
| Plugin 注册 | ★★★★★ | 35 个 Plugin 全部注册，无幽灵 Plugin |

### 1.2 需要重构的（按严重程度排序）

| # | 问题 | 严重度 | 来源 | 影响 |
|---|------|--------|------|------|
| P1 | core 层反向依赖 buff 层 | ★★★★★ | 12.md/架构边界 | 层级倒置，循环依赖风险 |
| P2 | 4 条消息链路断路 | ★★★★★ | 14.md/幽灵Feature | EquipItem/UnequipItem/UseItem/TransferItem 永远不触发 |
| P3 | 跨 Feature 直接操作内部组件 | ★★★★☆ | 12.md/模块边界 | battle/buff 直接修改 character 组件 |
| P4 | pub use xxx::* 全量导出 | ★★★★☆ | 12.md/模块边界 | 33 处全量 re-export，公共接口模糊 |
| P5 | BuffData.is_stun 僵尸字段 | ★★★☆☆ | 14.md/僵尸配置 | 与 STUN 标签重复，违反 Tag 优于 bool |
| P6 | SkillData.priority 僵尸字段 | ★★☆☆☆ | 14.md/僵尸配置 | 配置赋值但运行时从未读取 |
| P7 | UnitTemplate.background 僵尸字段 | ★★☆☆☆ | 14.md/僵尸配置 | 配置赋值但运行时从未使用 |
| P8 | 6 个文件超过 500 行 | ★★★☆☆ | 12.md/文件失控 | record.rs 684行, equip.rs 653行 |
| P9 | AI decision 零测试 | ★★★☆☆ | 12.md/测试覆盖 | 核心决策逻辑无测试 |
| P10 | UI 层测试覆盖不足 | ★★☆☆☆ | 12.md/测试覆盖 | command_handler 317行零测试 |
| P11 | Component 行为泄漏（待审计） | ★★★☆☆ | 12.md/#4 | 需检查 Component impl 中是否混入业务逻辑 |
| P12 | Resource 滥用风险（待审计） | ★★★☆☆ | 12.md/#5 | 47 个 Resource 中是否有 Manager 类退化 |
| P13 | Observer 风暴风险（待审计） | ★★☆☆☆ | 12.md/#6,#22 | 需检查是否存在高频触发的 Observer |
| P14 | God Query（待审计） | ★★☆☆☆ | 12.md/#7 | 需检查 Query 元组是否超过合理数量 |
| P15 | BattleRecord vs CombatLog 真相分裂 | ★★★★☆ | 12.md/专属评审 | 两个系统记录同一事件，是否存在不一致 |
| P16 | 数据驱动硬编码（待审计） | ★★★☆☆ | 12.md/#13 | 是否存在 match JobType / if skill_id == "xxx" 硬编码 |
| P17 | pub 可见性过宽（待审计） | ★★☆☆☆ | 12.md/#25 | pub/pub(crate)/pub(super) 是否合理 |
| P18 | CODE GRAVEYARD 审查表缺失 | ★★★☆☆ | 14.md/#10 | 无持续性的死代码检查机制 |
| P19 | God Object / Manager 模式（待审计） | ★★★☆☆ | 12.md/#3 | 是否存在 XxxManager/XxxController 命名的巨型结构 |
| P20 | 扩展性验证缺失 | ★★★★☆ | 12.md/#11,#12 | 未验证"新增职业/技能只需改配置"的扩展性承诺 |
| P21 | Bug 回归测试流程缺失 | ★★★☆☆ | 12.md/#16 | 未建立"每修 Bug 必写回归测试"的流程规则 |
| P22 | 战斗回放能力未验证 | ★★★☆☆ | 12.md/#18 | BattleRecord 存在但重放/定位Bug/录像能力未验证 |
| P23 | 命名不表达业务（待审计） | ★★☆☆☆ | 12.md/#23 | 是否存在 process_data() 类泛化命名 |
| P24 | 14.md 十大高危区域未交叉验证 | ★★★☆☆ | 14.md/底部 | 旧Damage/旧Buff/旧Modifier/Message迁移遗留等未逐项排查 |

---

## 第二部分：重构阶段规划

### 总原则

1. **每个阶段独立可交付**：完成一个阶段后项目必须编译通过、测试全绿
2. **先修架构债务，再修代码质量**：按 12.md 优先级排序
3. **不引入新抽象**：只消除问题，不创造新概念
4. **测试先行**：重构前补测试，重构后验证测试
5. **DDD 负责建模，ECS 负责执行**（D6）：不把企业 OOP 搬进 Bevy
6. **限界上下文隔离**（D3）：跨 Feature 通过 Message/Observer 通信，不直接操作内部
7. **规则集中管理**（D5）：一个规则只有一个真相来源

---

### 阶段 0：安全网（前置条件）

**目标**：确保重构前有足够的回归保护

**工作项**：

| # | 任务 | 验收标准 |
|---|------|---------|
| 0.1 | 为断路消息补 Writer 调用点标记 | 每个断路消息添加 `// TODO: Wire up MessageWriter` 注释 |
| 0.2 | 为 core→buff 层级倒置添加架构测试 | 在 tests/ 中添加 `arch_check.rs`，验证 core 模块不依赖业务模块 |
| 0.3 | 为 AI decision 补基础测试 | 至少 3 个决策路径测试 |
| 0.4 | **ECS 审计：Component 行为泄漏**（P11） | 扫描所有 `impl ComponentType` 块，标记含业务逻辑的方法 |
| 0.5 | **ECS 审计：Resource 滥用**（P12） | 扫描 47 个 Resource，标记含 Manager/Controller 语义的 |
| 0.6 | **ECS 审计：Observer 风暴**（P13） | 扫描所有 Observer/trigger 调用点，标记高频触发路径 |
| 0.7 | **ECS 审计：God Query**（P14） | 扫描所有 Query 元组，标记超过 8 个参数的 |
| 0.8 | **真相源审计：BattleRecord vs CombatLog**（P15） | 确认两者是否记录同一事件，是否存在不一致 |
| 0.9 | **数据驱动审计：硬编码**（P16） | 搜索 `match JobType`、`if skill_id ==`、`if class ==` 等硬编码 |
| 0.10 | **可见性审计：pub 过宽**（P17） | 统计 pub vs pub(crate) vs pub(super) 比例 |
| 0.11 | **God Object 审计**（P19） | 搜索 Manager/Controller/Handler 命名模式，标记超过 500 行的 |
| 0.12 | **扩展性验证**（P20） | 模拟新增一个职业和技能，记录需要修改的文件数量 |
| 0.13 | **14.md 十大高危区域交叉验证**（P24） | 逐项排查：旧Damage/旧Buff/旧Modifier/Message迁移遗留/Inventory重构遗留/Trait抽象层/Config字段/调试工具/测试辅助代码/Plugin注册链 |

**产出**：安全网就绪，可以开始重构

---

### 阶段 1：消除层级倒置（P1 - 最高优先级）

**目标**：core 层不再依赖任何业务层模块

**问题分析**：

`core/effect/handler.rs` 中的 `BuffHandler::generate()` 需要查询 `BuffRegistry` 来生成 Buff 效果。这导致基础设施层反向依赖业务层。

**方案**：将 `BuffHandler` 从 core 移出，放入 buff 模块

```
重构前：
  core/effect/handler.rs → 依赖 buff::BuffRegistry（层级倒置）

重构后：
  core/effect/handler.rs → 只保留 DamageHandler, HealHandler, CleanseHandler
  buff/handler.rs → BuffHandler 移入 buff 模块，通过 Registry 注册回 EffectHandlerRegistry
```

**具体步骤**：

| # | 任务 | 文件 |
|---|------|------|
| 1.1 | 将 `BuffHandler` 从 `core/effect/handler.rs` 移至 `buff/handler.rs` | buff/handler.rs（新建）, core/effect/handler.rs |
| 1.2 | `BuffPlugin` 中注册 `BuffHandler` 到 `EffectHandlerRegistry` | buff/plugin.rs |
| 1.3 | 移除 `core/effect/handler.rs` 中对 `buff::BuffRegistry` 的依赖 | core/effect/handler.rs |
| 1.4 | 更新 `EffectHandler` trait 的注册机制，支持跨模块注册 | core/effect/mod.rs |
| 1.5 | 添加架构守卫测试 | tests/arch/boundary.rs |

**验收**：`core/` 目录下无任何 `use crate::buff` 语句

---

### 阶段 2：修复断路消息（P2 - 最高优先级）

**目标**：所有注册的 Message 都有完整的 Writer→Reader 链路

**问题分析**：

4 个消息有 Reader 无 Writer，意味着装备穿脱和物品使用的消息驱动流程完全断路。当前装备穿脱可能通过直接函数调用实现，而非消息驱动。

**方案**：为断路消息接入 Writer，或删除无用的消息+Reader 系统

| 消息 | 现状 | 方案 |
|------|------|------|
| `EquipItem` | 有 Reader（equip.rs），无 Writer | 在 `ui/command_handler.rs` 中接入 MessageWriter |
| `UnequipItem` | 有 Reader（equip.rs），无 Writer | 在 `ui/command_handler.rs` 中接入 MessageWriter |
| `UseItem` | 有 Reader（use_item.rs），无 Writer | 在 `ui/command_handler.rs` 中接入 MessageWriter |
| `TransferItem` | 有 Reader（transfer.rs），无 Writer | 在 `ui/command_handler.rs` 中接入 MessageWriter |
| `EquipFailed` | 有 Writer，无 Reader | 在 UI 中添加装备失败提示消费 |
| `ItemUsed` | 有 Writer，无 Reader | 在 combat_log_handler 中添加消费 |
| `ItemTransferred` | 有 Writer，无 Reader | 在 combat_log_handler 中添加消费 |

**具体步骤**：

| # | 任务 | 文件 |
|---|------|------|
| 2.1 | 确认当前装备穿脱的调用路径（直接调用 vs 消息） | 调研 |
| 2.2 | 在 command_handler 中接入 EquipItem/UnequipItem Writer | ui/command_handler.rs |
| 2.3 | 在 command_handler 中接入 UseItem/TransferItem Writer | ui/command_handler.rs |
| 2.4 | 为 EquipFailed 添加 UI 反馈消费 | ui/panels/ 或 ui/combat_log_handler.rs |
| 2.5 | 为 ItemUsed/ItemTransferred 添加消费 | ui/combat_log_handler.rs |
| 2.6 | 添加消息链路完整性测试 | tests/arch/message_chain.rs |

**验收**：所有注册的 Message 都有至少一个 Writer 和至少一个 Reader

---

### 阶段 3：收紧模块公共接口（P4 - 高优先级）

**目标**：消除 `pub use xxx::*` 全量导出，每个模块只暴露必要的公共类型

**问题分析**：

33 处 `pub use submodule::*` 导致模块的公共接口等于内部所有类型的并集，违反"模块只暴露公共接口"铁律。外部代码可以访问模块内部实现细节。

**方案**：逐模块替换 `pub use xxx::*` 为显式导出列表

**具体步骤**：

| # | 任务 | 文件 |
|---|------|------|
| 3.1 | 为每个模块的 `mod.rs` 替换 `pub use xxx::*` 为显式 `pub use xxx::{A, B, C}` | 所有模块 mod.rs |
| 3.2 | 修复因显式导出导致的编译错误 | 跨模块引用处 |
| 3.3 | 确认所有跨模块 `use crate::xxx::` 引用的类型都在公共接口中 | 全项目 |

**优先级排序**（按模块被依赖程度）：

1. `core/` — 被所有模块依赖，最关键
2. `character/` — 被最多模块依赖
3. `buff/` / `equipment/` / `inventory/` — 业务核心
4. `battle/` / `skill/` / `turn/` / `map/`
5. `ai/` / `ui/` / `debug/`

**验收**：项目中无 `pub use xxx::*` 语句（除了 `prelude` 模式）

---

### 阶段 4：跨 Feature 通信规范化（P3 - 高优先级）

**目标**：battle/buff 不再直接操作 character 的内部组件，通过 Hook/Observer/Message 通信

**问题分析**：

- `battle/pipeline/execute.rs` 直接修改 `Dead`、`GridPosition` 等 character 组件
- `buff/resolve.rs` 直接修改 `Dead`、`PersistentTags` 等 character 组件
- `ui/command_handler.rs` 直接调用 `battle::manhattan_distance` 内部函数

**方案**：

对于直接操作组件的情况，区分两类：

**A 类：ECS 组件操作（可接受）**
- battle 读取 `Unit`、`Faction`、`Attributes` → **合理**，Query 读取公共组件是 ECS 正常模式
- battle 写入 `Dead` tag → **需评估**，添加 Dead 应通过 Hook 触发副作用

**B 类：跨模块函数调用（需修复）**
- `ui/command_handler.rs` 调用 `battle::manhattan_distance` → 移到 `core/` 或 `map/`
- `core/effect/handler.rs` 调用 `buff::apply_buff` → 阶段 1 已修复

**具体步骤**：

| # | 任务 | 文件 |
|---|------|------|
| 4.1 | 将 `manhattan_distance` 移至 `map/` 或 `core/` | battle/combat.rs → map/ |
| 4.2 | 审查 battle→character 写入操作，确认哪些需要改用 Observer | battle/pipeline/execute.rs |
| 4.3 | 审查 buff→character 写入操作，确认哪些需要改用 Observer | buff/resolve.rs |
| 4.4 | 为 Dead tag 的添加确认 Hook 覆盖完整 | character/components.rs |
| 4.5 | **战斗链路审计**（14.md #9）：绘制完整战斗链路图 | 全项目 |
| 4.6 | **BattleRecord SSOT 验证**（12.md 专属评审）：确认 CombatLog 不独立记录真相 | battle/record.rs, battle/log.rs |
| 4.7 | **数据驱动硬编码修复**（P16）：消除 match JobType 等硬编码 | 审计结果指向的文件 |
| 4.8 | **战斗回放能力验证**（P22）：确认 BattleRecord 支持重放/定位Bug/同步验证 | battle/record.rs |

**验收**：
- ui 模块不直接调用 battle 的内部函数
- core 模块不依赖任何业务模块
- 跨 Feature 写入操作通过 Hook/Observer/Message 通信

---

### 阶段 5：清理僵尸代码和配置（P5-P7 - 中优先级）

**目标**：消除所有僵尸字段和冗余标记

**具体步骤**：

| # | 任务 | 文件 |
|---|------|------|
| 5.1 | 删除 `BuffData.is_stun` 字段，晕眩统一走 `GameplayTag::STUN` | buff/domain.rs, RON 文件 |
| 5.2 | 删除 `SkillData.priority` 字段 | skill/domain/types.rs, RON 文件 |
| 5.3 | 删除 `UnitTemplate.background` 字段 | character/template.rs, RON 文件 |
| 5.4 | 为 `version` 字段添加 `#[serde(default)]` + 注释说明兼容性意图 | 所有 Def 类型 |
| 5.5 | 删除 `battle/log.rs` 中的 `#[allow(dead_code)]` | battle/log.rs |
| 5.6 | 运行 `cargo clippy --all-targets` 修复所有警告 | 全项目 |

**验收**：`cargo clippy` 零警告，无僵尸字段

---

### 阶段 6：巨文件拆分（P8 - 中优先级）

**目标**：所有 .rs 文件控制在 500 行以内

**拆分计划**：

| 文件 | 当前行数 | 拆分方案 |
|------|---------|---------|
| `battle/record.rs` | 684 | 拆为 `record/types.rs`（结构体定义）+ `record/functions.rs`（记录函数）+ `record/tests.rs`（测试） |
| `equipment/equip.rs` | 653 | 拆为 `equip/logic.rs`（穿脱逻辑）+ `equip/handlers.rs`（消息处理）+ `equip/tests.rs`（测试） |
| `battle/pipeline/execute.rs` | 615 | 拆为 `execute/damage.rs` + `execute/heal.rs` + `execute/buff.rs` + `execute/mod.rs` |
| `core/modifier_rule.rs` | 611 | 拆为 `modifier_rule/registry.rs` + `modifier_rule/calculators.rs` + `modifier_rule/tests.rs` |
| `core/attribute/mod.rs` | 584 | 拆为 `attribute/ops.rs`（操作方法）+ `attribute/query.rs`（查询方法） |
| `inventory/container.rs` | 515 | 拆为 `container/ops.rs`（增删改查）+ `container/query.rs`（查询方法） |

**验收**：无 .rs 文件超过 500 行

---

### 阶段 7：补齐关键测试（P9-P10 - 中优先级）

**目标**：所有核心业务模块有测试覆盖

**具体步骤**：

| # | 任务 | 目标测试数 |
|---|------|-----------|
| 7.1 | AI decision 补测试 | ≥5 个决策路径测试 |
| 7.2 | 消息链路完整性测试 | 每个注册消息至少 1 个端到端测试 |
| 7.3 | 装备穿脱消息驱动流程测试 | ≥3 个场景测试 |
| 7.4 | 物品使用消息驱动流程测试 | ≥2 个场景测试 |
| 7.5 | UI command_handler 关键路径测试 | ≥3 个命令处理测试 |

**验收**：测试总数 ≥ 650，无模块零测试

---

### 阶段 8：领域语言统一化（DDD 战略层 - 低优先级但高价值）

**目标**：建立 Ubiquitous Language，消除同义异名

**问题分析**（13.md 核心思想）：

当前项目中存在概念混淆：
- "Buff" 和 "StatusEffect" 混用（模块名 buff，但代码注释中有时用 StatusEffect）
- "Trait" 在 Rust 语言层面和游戏领域层面含义不同
- "Def" / "Definition" / "Data" 后缀不统一

**具体步骤**：

| # | 任务 | 说明 |
|---|------|------|
| 8.1 | 建立领域术语表 | 明确 Unit、Trait、Modifier、Buff、Equipment、Inventory 等核心概念的精确定义 |
| 8.2 | 统一 Def 后缀 | `SkillDef` → `SkillDefinition`，`BuffDef` → `BuffDefinition`，或统一为 `XxxDef` |
| 8.3 | 统一 Registry 后缀 | 确认所有注册表命名一致 |
| 8.4 | 在代码注释中统一使用领域术语 | 消除同义异名 |

**验收**：领域术语表文档完成，代码中无同义异名

---

### 阶段 9：ECS 审计修复（P11-P14, P17 - 中优先级）

**目标**：修复阶段 0 审计发现的问题

**依赖**：阶段 0 的审计结果

**具体步骤**：

| # | 任务 | 来源 | 说明 |
|---|------|------|------|
| 9.1 | 修复 Component 行为泄漏 | P11 | 将 Component impl 中的业务逻辑移入 System |
| 9.2 | 修复 Resource 滥用 | P12 | 将 Manager 类 Resource 拆分为 Component + System |
| 9.3 | 修复 Observer 风暴 | P13 | 高频 Observer 改为直接 System 处理 |
| 9.4 | 拆分 God Query | P14 | 超过 8 参数的 Query 拆分为多个职责单一的 System |
| 9.5 | 收紧 pub 可见性 | P17 | 将不必要的 `pub` 降级为 `pub(crate)` 或 `pub(super)` |

**验收**：
- Component impl 中无业务逻辑方法
- 无 Manager/Controller 语义的 Resource
- 无每帧触发的 Observer
- 无超过 8 参数的 Query

---

### 阶段 10：建立 CODE GRAVEYARD 持续审查机制（P18 - 中优先级）

**目标**：建立 14.md 推荐的持续性死代码检查机制，防止幽灵代码再次积累

**背景**（14.md 核心思想）：

> 很多项目最后 30%~50% 的代码其实都是历史遗留。代码还在、测试还绿、编译还过，实际上已经没人用了。这种"幽灵代码"才是长期项目复杂度爆炸的真正来源。

**具体步骤**：

| # | 任务 | 说明 |
|---|------|------|
| 10.1 | 创建 `docs/code_graveyard.md` 审查表 | 包含 14.md #10 的 10 项检查清单 |
| 10.2 | 在 CI 中添加 `cargo clippy --all-targets` 检查 | 零警告才能合并 |
| 10.3 | 在 CI 中添加架构守卫测试 | 验证 core 不依赖业务层、消息链路完整性 |
| 10.4 | 建立每次重构后的审查流程 | 按 0.3 节十层法逐层检查 |

**CODE GRAVEYARD 审查表**（源自 14.md #10）：

| 检查项 | 问题 | 检查方法 |
|--------|------|---------|
| Plugin | 是否还被注册 | 搜索 `pub struct XxxPlugin`，确认在 `add_plugins` 中引用 |
| Message | 是否还有发送方 | 搜索 `MessageWriter<T>`，确认存在 |
| Observer | 是否还有触发源 | 搜索 `trigger()` / `observe()`，确认存在 |
| Component | 是否还有 Query 引用 | 搜索 `Query<&T>` / `Query<&mut T>`，确认存在 |
| Resource | 是否还有读写 | 搜索 `Res<T>` / `ResMut<T>`，确认存在 |
| Trait | 是否还有多个实现 | 搜索 `impl TraitName for`，确认 ≥2 个 |
| Config 字段 | 是否还有读取方 | Rust Analyzer Find References |
| 测试 | 是否覆盖真实业务 | 检查测试是否测试业务规则而非仅构造/析构 |
| Feature 目录 | 是否仍有业务价值 | 确认模块在游戏中被使用 |
| Debug 工具 | 是否仍被使用 | 确认快捷键/面板仍可触发 |

**验收**：审查表文档完成，CI 守卫就绪

---

### 阶段 11：扩展性验证 + 流程建设（P19-P23 - 低优先级但高长期价值）

**目标**：验证数据驱动架构的扩展性承诺，建立 Bug 回归测试流程

**背景**（12.md 核心思想）：

> 新增职业/技能只需改配置即可运行。如果新增一个职业需要改 SkillSystem、DamageSystem、AISystem、UISystem，说明架构有问题。

**具体步骤**：

| # | 任务 | 来源 | 说明 |
|---|------|------|------|
| 11.1 | **扩展性验证：新增职业**（P20） | 12.md/#11 | 模拟新增一个职业，记录需要修改的文件数量。目标：仅修改 RON 配置 |
| 11.2 | **扩展性验证：新增技能**（P20） | 12.md/#12 | 模拟新增一个技能，记录需要修改的文件数量。目标：仅新增 SkillDefinition RON |
| 11.3 | **God Object 修复**（P19） | 12.md/#3 | 根据阶段 0 审计结果，拆分 Manager/Controller 类结构 |
| 11.4 | **建立 Bug 回归测试流程**（P21） | 12.md/#16 | 在项目铁律中增加：每修 Bug 必写回归测试，否则 Bug 会回来 |
| 11.5 | **命名审计**（P23） | 12.md/#23 | 扫描 `process_data()`、`handle_event()` 类泛化命名，改为 `apply_damage()`、`equip_item()` 等业务命名 |
| 11.6 | **战斗回放能力补全**（P22） | 12.md/#18 | 确认 BattleRecord 支持重放、定位 Bug、同步验证；如不支持则补充 |

**验收**：
- 新增职业只需修改 RON 配置（0 行逻辑代码）
- 新增技能只需新增 SkillDefinition RON（0 行逻辑代码）
- Bug 回归测试流程写入项目铁律
- 无 `process_data()` 类泛化命名

---

## 第三部分：执行时间线

```
阶段 0：安全网 + 全面审计  ── 前置条件，必须先完成
  ↓
阶段 1：消除层级倒置      ── P1，架构正确性
  ↓
阶段 2：修复断路消息      ── P2，功能正确性
  ↓
阶段 3：收紧模块接口      ── P4，可维护性
  ↓
阶段 4：跨Feature通信     ── P3，架构边界 + 战斗链路审计 + SSOT 验证 + 回放能力
  ↓
阶段 5：清理僵尸代码      ── P5-P7，代码卫生
  ↓
阶段 6：巨文件拆分        ── P8，代码质量
  ↓
阶段 7：补齐关键测试      ── P9-P10，测试性
  ↓
阶段 8：领域语言统一      ── DDD 战略层，长期价值
  ↓
阶段 9：ECS 审计修复     ── P11-P14,P17，ECS 设计
  ↓
阶段 10：CODE GRAVEYARD  ── P18，持续审查机制
  ↓
阶段 11：扩展性验证+流程  ── P19-P23，扩展性+流程建设
```

**每个阶段的退出条件**：
1. `cargo check` 编译通过
2. `cargo test --lib` 全绿
3. `cargo test` 集成测试全绿
4. 该阶段引入的变更有对应的测试覆盖

---

## 第四部分：不做什么

以下事项**明确排除**在本次重构之外：

| 排除项 | 原因 |
|--------|------|
| 重写战斗管线 | 当前管线架构清晰，无重复规则，不需要重写 |
| 重写 Trait+Modifier 体系 | 已统一，8 个 Trait 都是真正的扩展点 |
| 重写数据驱动体系 | 14 个 Def + 18 个 Registry 运作良好 |
| 引入新的抽象层 | 铁律：架构优先解决当前复杂度 |
| 修改 RON 配置格式 | 铁律：配置兼容性优先于配置优雅 |
| 性能优化 | 铁律：先正确，再优化；先 Profile，再优化 |
| UI 重构 | 表现层重构优先级低于逻辑层 |

---

## 第五部分：风险控制

| 风险 | 缓解措施 |
|------|---------|
| 阶段 1 移动 BuffHandler 破坏注册机制 | 先补测试，再移动；EffectHandlerRegistry 已支持动态注册 |
| 阶段 2 接入 Writer 可能与现有直接调用冲突 | 先确认当前调用路径，再决定是替换还是并存 |
| 阶段 3 显式导出可能导致大量编译错误 | 按模块逐步替换，每步编译验证 |
| 阶段 6 拆分文件可能破坏 `pub use` 链 | 与阶段 3 协同，先收紧接口再拆分 |
| 阶段 4 BattleRecord SSOT 验证可能发现 CombatLog 需要重写 | CombatLog 可能只是 BattleRecord 的视图层，确认后决定是否重构 |
| 阶段 9 ECS 审计可能发现大量问题 | 审计结果决定修复范围，不强制一次性全部修复 |
| 重构过程中引入新 Bug | 每个阶段退出条件：编译通过 + 测试全绿 |
| 14.md 高危区域交叉验证 | 阶段 0 审计时按 14.md 底部 10 大高危区域逐项检查 |

---

## 第六部分：成功指标

| 指标 | 重构前 | 重构后目标 |
|------|--------|-----------|
| core 层依赖业务层 | 1 处（buff） | 0 处 |
| 断路消息 | 4 条 | 0 条 |
| `pub use xxx::*` | 33 处 | 0 处 |
| 僵尸字段 | 4 个 | 0 个 |
| 超 500 行文件 | 6 个 | 0 个 |
| 零测试模块 | 1 个（ai/decision） | 0 个 |
| clippy 警告 | 未知 | 0 个 |
| 测试总数 | 621 | ≥ 650 |
| 跨 Feature 直接写入 | 待审计 | 仅通过 Hook/Observer/Message |
| Component 行为泄漏 | 待审计 | 0 处 |
| Resource 滥用 | 待审计 | 0 处 Manager 类 |
| Observer 风暴 | 待审计 | 0 处高频 Observer |
| God Query（>8参数） | 待审计 | 0 处 |
| BattleRecord SSOT | 待验证 | BattleRecord 为唯一真相源 |
| 数据驱动硬编码 | 待审计 | 0 处 match JobType |
| CODE GRAVEYARD 审查表 | 不存在 | 建立并持续执行 |
| God Object / Manager | 待审计 | 0 处巨型 Manager |
| 扩展性：新增职业 | 未验证 | 仅改 RON 配置（0 行逻辑代码） |
| 扩展性：新增技能 | 未验证 | 仅新增 SkillDefinition RON |
| Bug 回归测试流程 | 不存在 | 写入项目铁律 |
| 战斗回放能力 | 未验证 | 支持重放/定位Bug/同步验证 |
| 命名表达业务 | 待审计 | 0 处 process_data() 类泛化命名 |
| 14.md 十大高危区域 | 未排查 | 逐项验证无幽灵代码 |
