# Battle 模块测试评审报告

Version: 1.0
Date: 2026-06-11
Reviewer: Test Guardian
Scope: `src/battle/` 全部代码文件 + `tests/` 中相关外部测试
Standard: `docs/test_spec.md` (Bevy SRPG Testing Constitution v3.1)
Domain Reference: `docs/domain/battle_rules_v1.md` + `docs/domain/effect_pipeline_v1.md`

---

# 1. 评审范围

## 1.1 源代码文件

| 文件 | 行数 | 内联测试数 | 测试覆盖状态 |
|------|------|-----------|-------------|
| `mod.rs` | 19 | 0 | N/A（模块声明） |
| `combat.rs` | 35 | 4 | 良好 |
| `events.rs` | 263 | 9 | 良好 |
| `log.rs` | 312 | 5 | 部分覆盖 |
| `record.rs` | 684 | 11 | 良好 |
| `pipeline/mod.rs` | 48 | 0 | N/A（管线编排） |
| `pipeline/intent.rs` | 251 | 0 | **未覆盖** |
| `pipeline/generate.rs` | 199 | 0 | **未覆盖**（外部测试部分覆盖） |
| `pipeline/modify.rs` | 71 | 0 | **未覆盖** |
| `pipeline/execute.rs` | 420 | 6 | 良好 |
| `pipeline/trait_trigger.rs` | 299 | 4 | 良好 |
| `plugin.rs` | 55 | 0 | N/A（插件注册） |

**内联测试总计：39 个**

## 1.2 外部测试文件（与 battle 相关）

| 文件 | 测试数 | 覆盖范围 |
|------|--------|----------|
| `tests/legacy/combat_pipeline.rs` | 22 | 伤害计算、Handler generate、EffectQueue、预览一致性 |
| `tests/legacy/edge_cases.rs` | 12 | 边界条件、修饰符叠加、空操作、类型不匹配 |
| `tests/golden/golden_battle.rs` | 3 | BattleRecord 快照测试 |
| `tests/feature/death.rs` | 4 | 死亡处理完整流程 |
| `tests/feature/buff.rs` | ~4+ | Buff 生命周期（含 Effect Pipeline） |

**外部测试总计：~45 个**

**全部测试总计：~84 个**

---

# 2. 评审标准

依据 `test_spec.md` 以下条款逐项评审：

| 条款 | 内容 | 评审重点 |
|------|------|----------|
| §3 Testing Philosophy | 测试验证 Behavior，不验证 Implementation | 断言是否关注 What 而非 How |
| §4 Test Pyramid | 70% Unit / 20% Integration / 8% Replay / 2% E2E | 各层级比例是否合理 |
| §5 Test Categories | Unit/Integration/Replay/Regression/E2E 定义 | 是否有缺失类别 |
| §6 Determinism Rules | 禁止随机、固定 Seed | 测试是否确定性 |
| §7 Test Case Schema | Test ID / Title / Given / When / Then / Assertions | 测试结构是否规范 |
| §7.1 Standard Test Data | Unit_001 / Unit_002 / Unit_003 | 是否使用标准数据 |
| §9 Coverage Strategy | 100% 核心领域规则覆盖 | 领域不变量是否全部测试 |
| §10 Error Testing | Invalid Input / Boundary Values | 边界和错误场景是否覆盖 |
| §13 AI Constraints | 禁止测试私有实现 | 是否越界测试内部细节 |
| §13.1 AI Self-Check | 6 项自检标注 | 是否有自检标注 |

---

# 3. 领域不变量覆盖评审

依据 `battle_rules_v1.md` 中定义的 7 个不变量 + `effect_pipeline_v1.md` 中定义的 6 个不变量：

## 3.1 Battle 领域不变量

| 不变量 | 描述 | 测试覆盖 | 评审结论 |
|--------|------|----------|----------|
| INV-BTL-1 | EffectQueue 执行后清空 | **部分覆盖** | `execute_effects` 测试隐式验证（drain 消费），但无显式断言 queue 为空 |
| INV-BTL-2 | 伤害下限 ≥ 1 | **覆盖** | `combat_pipeline.rs` 的 `伤害下限为1` + `modify.rs` 中 `max(1)` |
| INV-BTL-3 | 治疗上限 ≤ MaxHp | **覆盖** | `execute.rs` 的 `apply_heal_effect_不超过maxhp` |
| INV-BTL-4 | 死亡判定一致性（HP≤0 → Dead Tag） | **覆盖** | `death.rs` 的 `致命伤害触发死亡` + `execute.rs` 的 `致死添加dead标记` |
| INV-BTL-5 | 管线严格顺序 G→M→E | **部分覆盖** | 系统编排通过 `.chain()` + `.after()` 保证，但无测试验证跳步不会发生 |
| INV-BTL-6 | CombatIntent 消费后清除 | **无覆盖** | `clear_combat_intent()` 在 `intent.rs` 中调用，但无测试验证执行后字段为 None |
| INV-BTL-7 | 属性修改必须通过 Modifier | **部分覆盖** | Modify 阶段通过 ModifierRuleRegistry 修饰，但无测试验证 Generate 阶段不修改 HP |

## 3.2 Effect Pipeline 领域不变量

| 不变量 | 描述 | 测试覆盖 | 评审结论 |
|--------|------|----------|----------|
| INV-EFF-1 | 管线严格顺序 | 同 INV-BTL-5 | 系统编排保证，无显式测试 |
| INV-EFF-2 | EffectQueue 执行后清空 | 同 INV-BTL-1 | 隐式覆盖 |
| INV-EFF-3 | 伤害下限 ≥ 1 | 同 INV-BTL-2 | 覆盖 |
| INV-EFF-4 | base_amount 首次记录不覆盖 | **无覆盖** | 无测试验证 Modify 阶段 `if base_amount.is_none()` 逻辑 |
| INV-EFF-5 | Generate 不修改 ECS | **无覆盖** | 无测试验证 Generate 阶段除 EffectQueue 外无 ECS 副作用 |
| INV-EFF-6 | 属性修改必须通过 Modifier | 同 INV-BTL-7 | 部分覆盖 |

**覆盖率：4/13 完全覆盖，4/13 部分覆盖，5/13 无覆盖**

---

# 4. 业务规则覆盖评审

| 规则 | 描述 | 测试覆盖 | 评审结论 |
|------|------|----------|----------|
| BR-BTL-1 | 效果管线（G→M→E 三步） | **覆盖** | `combat_pipeline.rs` + `execute.rs` 覆盖各阶段 |
| BR-BTL-2 | CombatIntent 是唯一攻击通道 | **部分覆盖** | AI 和玩家共用 Effect Pipeline 在 `generate.rs` 中实现，但无集成测试验证 |
| BR-BTL-3 | 死亡处理（Dead Tag + Hook + Message） | **覆盖** | `death.rs` 完整覆盖 |
| BR-BTL-4 | Trait 触发（OnAttack/OnHit/OnKill） | **覆盖** | `trait_trigger.rs` 内联测试 + `execute.rs` 隐式覆盖 |
| BR-BTL-5 | 行动路由（route_after_action） | **无覆盖** | `intent.rs` 中的 `route_after_action` 零测试 |
| BR-BTL-6 | 战斗记录 | **覆盖** | `record.rs` 内联测试 + `golden_battle.rs` 快照测试 |
| BR-BTL-7 | EffectHandler trait 分发 | **覆盖** | `combat_pipeline.rs` + `edge_cases.rs` |

**覆盖率：4/7 完全覆盖，1/7 部分覆盖，2/7 无覆盖**

---

# 5. 现有测试质量评审

## 5.1 内联测试逐文件评审

### combat.rs（4 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `曼哈顿距离_相邻格子` | 距离=1 | Behavior | 良好 |
| `曼哈顿距离_对角线` | 距离=7 | Behavior | 良好 |
| `曼哈顿距离_同一位置` | 距离=0 | Behavior — 边界测试 | 良好 |
| `曼哈顿距离_负坐标` | 负数坐标 | Behavior — 边界测试 | 良好 |

**问题：** 无。纯函数测试，行为验证，边界覆盖充分。

### events.rs（9 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `character_died_消息字段` | 字段正确 | **Implementation** — 检查结构体字段 | 轻微违规 |
| `damage_applied_消息字段` | 字段正确 | **Implementation** — 检查结构体字段 | 轻微违规 |
| `heal_applied_消息字段` | 字段正确 | **Implementation** | 轻微违规 |
| `dot_applied_消息字段` | 字段正确 | **Implementation** | 轻微违规 |
| `hot_applied_消息字段` | 字段正确 | **Implementation** | 轻微违规 |
| `stun_applied_消息字段` | 字段正确 | **Implementation** | 轻微违规 |
| `on_character_died_从行动队列移除` | 队列移除 | Behavior | 良好 |
| `on_character_died_队列中无死亡单位则不变` | 无变化 | Behavior — 边界测试 | 良好 |
| `on_character_died_多次死亡消息按序移除` | 多次移除 | Behavior | 良好 |

**问题：**
- 前 6 个测试仅验证 Message 结构体字段赋值，属于实现细节测试。Message 的字段结构由编译器保证，不需要测试。应改为验证 Message 的行为效果（如 `on_character_died` 测试那样）。
- 缺少 `current_index` 调整的边界测试（死亡单位在 current_index 之前/之后时索引修正）

### log.rs（5 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `战斗日志_添加一条` | 添加功能 | **Implementation** — 检查 entries.len() | 轻微违规 |
| `战斗日志_多条日志` | 多条添加 | **Implementation** — 检查 entries.len() | 轻微违规 |
| `战斗日志_超过最大条数截断最旧` | 截断行为 | Behavior | 良好 |
| `战斗日志_刚好等于最大条数不截断` | 边界 | Behavior — 边界测试 | 良好 |
| `战斗日志_多片段拼接` | 片段结构 | **Implementation** — 检查 entries[0].len() | 轻微违规 |

**问题：**
- `log.rs` 是 UI 展示模块，按 test_spec.md §1.1 Non-goals，UI 视觉测试不在范围内。但 `CombatLog` 的数据行为（截断、添加）属于逻辑层，可以测试。
- `添加一条` 和 `多条日志` 测试价值低，仅验证 `push` 方法有效，可删除。
- 缺少 `log_turn_change` 系统的测试（回合切换日志生成）
- 缺少 `update_combat_log` 系统的测试（但这是纯 UI 系统，按 Non-goals 可豁免）

### record.rs（11 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `战斗记录_记录伤害` | 记录功能 | **Implementation** — 检查 entries.len() | 轻微违规 |
| `战斗记录_按实体查询` | 查询功能 | Behavior | 良好 |
| `战斗记录_最近N条` | recent 功能 | Behavior | 良好 |
| `战斗记录_清空` | clear 功能 | Behavior | 良好 |
| `战斗记录_回合开始更新回合号` | 回合号更新 | Behavior | 良好 |
| `战斗记录_按回合查询` | entries_for_turn | Behavior | 良好 |
| `战斗记录_实体统计` | stats_for | Behavior | 良好 |
| `战斗记录_序列化反序列化` | RON 序列化 | Behavior — 数据契约 | 良好 |
| `伤害分解_基础构建` | DamageBreakdown 构建 | **Implementation** — 检查字段 | 轻微违规 |
| `伤害分解_无修饰符` | 无修饰符 | Behavior | 良好 |
| `伤害分解_多修饰符叠加` | 多修饰符 | Behavior | 良好 |

**问题：**
- `战斗记录_记录伤害` 仅验证 `entries.len() == 1`，应验证记录内容正确性
- `伤害分解_基础构建` 仅验证字段赋值，属于实现细节
- 缺少 `stats_for` 的击杀数计算边界测试（无致死伤害时 kills=0）
- 缺少 `entries_for_turn` 的边界测试（查询不存在的回合）

### pipeline/execute.rs（6 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `apply_damage_effect_扣血` | 伤害扣血 | Behavior | 良好 |
| `apply_damage_effect_致死添加dead标记` | 死亡判定 | Behavior | 良好 |
| `apply_heal_effect_回血` | 治疗回血 | Behavior | 良好 |
| `apply_heal_effect_不超过maxhp` | 治疗上限 | Behavior | 良好 |
| `apply_buff_effect_正常施加` | Buff 施加 | **Implementation** — 检查 buffs 内容 | 轻微违规 |
| `apply_buff_effect_未知buff静默跳过` | 错误处理 | Behavior — 错误测试 | 良好 |

**问题：**
- `apply_buff_effect_正常施加` 检查 `buffs.iter().any(|b| b.name == "攻+5")`，验证了 Buff 名称而非行为效果（如属性变化），属于实现细节测试
- 缺少 EffectQueue 执行后为空的显式断言（INV-BTL-1）
- 缺少 DamageApplied Message 内容验证
- 缺少 OnHit/OnKill Trait 触发的集成测试

### pipeline/trait_trigger.rs（4 个测试）

| 测试 | 验证目标 | Behavior/Implementation | 合规性 |
|------|----------|------------------------|--------|
| `on_attack_触发apply_buff` | OnAttack 触发 | Behavior | 良好 |
| `on_hit_触发apply_buff` | OnHit 触发 | Behavior | 良好 |
| `passive_trait_不触发` | Passive 不触发 | Behavior — 边界测试 | 良好 |
| `多个on_attack_trait_全部触发` | 多 Trait 触发 | Behavior | 良好 |

**问题：** 无。测试质量高，行为验证，边界覆盖。

### pipeline/intent.rs（0 个测试）

**严重缺失** — 包含以下关键逻辑无测试：

- `execute_action_on_enter`：玩家/AI 行动执行（设置冷却、acted 标记、清除 CombatIntent）
- `wait_action_on_enter`：待机处理
- `route_after_action`：行动后路由（跳过死亡单位、队列耗尽进 TurnEnd）
- `clear_combat_intent`：CombatIntent 清除
- 晕眩跳过行动逻辑

### pipeline/generate.rs（0 个测试）

**严重缺失** — 虽然外部测试 `combat_pipeline.rs` 覆盖了 Handler generate，但 `generate_combat_effects` 系统本身无测试：

- 晕眩检查跳过攻击
- 冷却检查跳过攻击
- AI 通过 CombatIntent.source_entity 查找攻击者
- OnAttack Trait 触发

### pipeline/modify.rs（0 个测试）

**严重缺失** — Modify 阶段无任何测试：

- Damage 修饰 + base_amount 首次记录
- Heal 修饰
- ApplyBuff/Cleanse 不修饰
- 伤害下限保护 `max(1)`

---

# 6. test_spec.md 合规性评审

## 6.1 §3 Testing Philosophy — 测试行为而非实现

| 评级 | 说明 |
|------|------|
| **基本合规** | 大部分测试验证行为，但 events.rs 和 log.rs 中有约 8 个测试仅验证结构体字段赋值 |

具体违规：
- `events.rs` 的 6 个消息字段测试：`assert_eq!(msg.amount, 15)` — 验证结构体字段赋值，不是行为
- `log.rs` 的 `战斗日志_添加一条`：`assert_eq!(log.entries.len(), 1)` — 验证 push 方法有效
- `record.rs` 的 `战斗记录_记录伤害`：`assert_eq!(record.entries.len(), 1)` — 同上
- `execute.rs` 的 `apply_buff_effect_正常施加`：检查 Buff 名称而非属性变化

## 6.2 §4 Test Pyramid — 测试金字塔

| 层级 | 要求 | 实际 | 差距 |
|------|------|------|------|
| Unit Test | 70% | ~61 个（73%） | 比例略高 |
| Integration Test | 20% | ~19 个（23%） | 基本达标 |
| Replay Test | 8% | 3 个（4%） | **不足** |
| E2E Test | 2% | 0 个（0%） | 可接受 |

**结论：测试金字塔基本合理，但 Replay 测试不足。Golden Battle 测试属于 Replay 类别，但仅 3 个场景。**

## 6.3 §6 Determinism Rules — 确定性

| 评级 | 说明 |
|------|------|
| **合规** | 所有测试均为确定性，无随机性。`execute_effects` 使用 `&mut World` 模式，测试中手动构建 App 和 Entity |

## 6.4 §7 Test Case Schema — 测试用例结构

| 评级 | 说明 |
|------|------|
| **不合规** | 所有测试缺少 Test ID、Given/When/Then 结构化注释 |

## 6.5 §7.1 Standard Test Data — 标准测试数据

| 评级 | 说明 |
|------|------|
| **部分合规** | 外部测试使用 `UnitBuilder::warrior()` / `UnitBuilder::goblin()` 等辅助函数，但与标准测试数据 Unit_001/002/003 的属性值不一致。内联测试使用 `make_test_attrs()` 自定义数据 |

## 6.6 §9 Coverage Strategy — 领域规则覆盖

| 评级 | 说明 |
|------|------|
| **不合规** | 13 个不变量中仅 4 个完全覆盖，7 个业务规则中 2 个无覆盖 |

## 6.7 §10 Error Testing — 错误测试

| 评级 | 说明 |
|------|------|
| **部分合规** | 有部分错误测试（未知 Buff 跳过、处理器不存在、类型不匹配），但缺少以下场景 |

缺失的错误测试：
- CombatIntent 为空时 generate/execute 的行为
- 技能 ID 不存在时 generate 的行为
- 目标坐标无单位时 generate 的行为
- EffectQueue 为空时 execute 的行为
- 死亡单位被攻击时的行为
- 同阵营攻击被过滤

## 6.8 §13 AI Constraints — 禁止测试私有实现

| 评级 | 说明 |
|------|------|
| **轻微违规** | 部分测试验证结构体字段赋值（events.rs），而非行为效果 |

## 6.9 §13.1 AI Self-Check — 自检标注

| 评级 | 说明 |
|------|------|
| **不合规** | 无任何测试文件包含 AI Self-Check 标注 |

---

# 7. 问题分类统计

## 7.1 按严重程度

| 严重程度 | 数量 | 描述 |
|----------|------|------|
| **P0 Critical** | 3 | intent.rs 零测试、generate.rs 系统无测试、INV-BTL-6（CombatIntent 清除）无覆盖 |
| **P1 High** | 4 | modify.rs 零测试、route_after_action 无测试、INV-EFF-4（base_amount 首次记录）无覆盖、INV-EFF-5（Generate 不修改 ECS）无覆盖 |
| **P2 Medium** | 6 | Test Case Schema 不合规、Standard Test Data 不一致、AI Self-Check 缺失、EffectQueue 清空无显式断言、错误测试不足、Replay 测试不足 |
| **P3 Low** | 4 | 消息字段测试为实现细节、日志测试价值低、Buff 测试检查名称而非行为、record 测试验证 len 而非内容 |

## 7.2 按问题类型

| 类型 | 数量 | 具体问题 |
|------|------|----------|
| 覆盖缺失 | 7 | intent.rs、generate.rs 系统、modify.rs、route_after_action、INV-BTL-6/7、INV-EFF-4/5 |
| 规范不合规 | 3 | Test Case Schema、Standard Test Data、AI Self-Check |
| 测试质量 | 5 | 消息字段测试、日志 len 测试、Buff 名称测试、record len 测试、EffectQueue 无显式断言 |
| 错误测试不足 | 4 | 空 CombatIntent、无效技能 ID、无目标坐标、死亡单位被攻击 |

---

# 8. 优先级建议

## P0 — 必须立即补充（阻塞合并）

### 8.1 集成测试：CombatIntent 消费后清除（INV-BTL-6）

- **文件**：`tests/feature/combat_intent.rs`（新建）
- **目标**：验证 Execute 和 WaitAction 后 CombatIntent 的 source_entity / target_coord / skill_id 均为 None
- **方法**：构建含玩家/AI 单位的 App，推进到 ExecuteAction/WaitAction，检查 CombatIntent 字段

### 8.2 集成测试：行动路由 route_after_action

- **文件**：`tests/feature/combat_intent.rs`
- **目标**：验证以下场景
  - 队列中下一个单位存活 → SelectUnit
  - 下一个单位已死亡（HP≤0）→ 跳过，继续前进
  - 队列耗尽 → TurnEnd
  - 下一个是 AI → 重置 AiTimer

### 8.3 集成测试：generate_combat_effects 系统

- **文件**：`tests/feature/combat_pipeline.rs`（新建）
- **目标**：验证系统级行为
  - 晕眩单位跳过攻击
  - 冷却中的技能跳过攻击
  - AI 通过 CombatIntent.source_entity 正确查找攻击者
  - OnAttack Trait 在 Generate 末尾触发

## P1 — 尽快补充（1 周内）

### 8.4 单元测试：modify_effects 阶段

- **文件**：`src/battle/pipeline/modify.rs` 内联测试
- **目标**：
  - Damage 修饰 + base_amount 首次记录
  - Heal 修饰
  - ApplyBuff/Cleanse 不修饰
  - 伤害下限保护 `max(1)`

### 8.5 单元测试：base_amount 首次记录不覆盖（INV-EFF-4）

- **文件**：`src/battle/pipeline/modify.rs` 内联测试
- **目标**：验证 `if base_amount.is_none()` 逻辑，首次 Modify 记录 base_amount，后续不覆盖

### 8.6 集成测试：EffectQueue 执行后清空（INV-BTL-1）

- **文件**：`tests/feature/combat_pipeline.rs`
- **目标**：显式断言 `app.world().resource::<EffectQueue>().pending.is_empty()`

### 8.7 错误测试补充

- 空 CombatIntent 时 generate/execute 的行为
- 无效技能 ID 时 generate 的行为
- 目标坐标无单位时 generate 的行为
- 死亡单位被攻击时的行为

## P2 — 计划补充（2 周内）

### 8.8 Test Case Schema 合规

为所有测试添加 Test ID 和 Given/When/Then 注释。Test ID 命名规则：

| 前缀 | 范围 |
|------|------|
| BTL-CBT-xxx | combat.rs 测试 |
| BTL-EVT-xxx | events.rs 测试 |
| BTL-LOG-xxx | log.rs 测试 |
| BTL-REC-xxx | record.rs 测试 |
| BTL-EXE-xxx | execute.rs 测试 |
| BTL-TRT-xxx | trait_trigger.rs 测试 |
| BTL-INT-xxx | intent.rs 测试（新增） |
| BTL-GEN-xxx | generate.rs 测试（新增） |
| BTL-MOD-xxx | modify.rs 测试（新增） |

### 8.9 Standard Test Data 适配

统一 `UnitBuilder` 与标准测试数据的属性值映射：

| UnitBuilder | 标准数据 | 属性 |
|-------------|----------|------|
| `warrior()` | Unit_001 | HP=100, ATK=30, DEF=10, SPD=10 |
| `mage()` | Unit_002 | HP=80, ATK=40, DEF=5, SPD=12 |
| `tank()` | Unit_003 | HP=150, ATK=20, DEF=20, SPD=5 |

当前 `warrior()` 的属性值（Might=5 → ATK=10, Vitality=5 → MaxHP=30）与标准数据不一致，需要调整或创建映射层。

### 8.10 AI Self-Check 标注

为所有测试文件添加 §13.1 要求的自检标注。

### 8.11 Replay 测试补充

扩展 `golden_battle.rs`，增加以下场景：
- 技能攻击（含修饰符）
- 多回合战斗流程
- AI 行动完整流程
- Trait 触发（OnAttack/OnHit/OnKill）

## P3 — 低优先级

### 8.12 清理实现细节测试

- 删除 `events.rs` 中 6 个仅验证结构体字段的消息测试（或改为验证行为效果）
- 删除 `log.rs` 中 `战斗日志_添加一条` 和 `多条日志` 低价值测试
- 修改 `execute.rs` 的 `apply_buff_effect_正常施加`：验证属性变化而非 Buff 名称
- 修改 `record.rs` 的 `战斗记录_记录伤害`：验证记录内容而非 entries.len()

### 8.13 补充 events.rs 边界测试

- `on_character_died` 中 `current_index` 调整逻辑的边界测试
- 死亡单位在 current_index 之前/之后时的索引修正

---

# 9. 测试矩阵

## 9.1 领域不变量 × 测试覆盖

| 不变量 | Unit Test | Integration Test | Replay Test | 状态 |
|--------|:---------:|:----------------:|:-----------:|------|
| INV-BTL-1 EffectQueue 清空 | 隐式 | **需补充显式** | — | 不足 |
| INV-BTL-2 伤害下限 | 覆盖 | 覆盖 | — | 达标 |
| INV-BTL-3 治疗上限 | 覆盖 | — | — | 达标 |
| INV-BTL-4 死亡判定 | 覆盖 | 覆盖 | 覆盖 | 达标 |
| INV-BTL-5 管线顺序 | — | 系统编排保证 | — | 不足 |
| INV-BTL-6 CombatIntent 清除 | — | **需补充** | — | 缺失 |
| INV-BTL-7 属性修改通过 Modifier | 部分覆盖 | — | — | 不足 |
| INV-EFF-4 base_amount 首次记录 | **需补充** | — | — | 缺失 |
| INV-EFF-5 Generate 不修改 ECS | **需补充** | — | — | 缺失 |

## 9.2 业务规则 × 测试覆盖

| 规则 | Unit Test | Integration Test | Replay Test | 状态 |
|------|:---------:|:----------------:|:-----------:|------|
| BR-BTL-1 效果管线 | 覆盖 | 覆盖 | 覆盖 | 达标 |
| BR-BTL-2 CombatIntent 唯一通道 | 部分覆盖 | **需补充** | — | 不足 |
| BR-BTL-3 死亡处理 | 覆盖 | 覆盖 | 覆盖 | 达标 |
| BR-BTL-4 Trait 触发 | 覆盖 | 部分覆盖 | — | 良好 |
| BR-BTL-5 行动路由 | — | **需补充** | — | 缺失 |
| BR-BTL-6 战斗记录 | 覆盖 | 覆盖 | 覆盖 | 达标 |
| BR-BTL-7 Handler 分发 | 覆盖 | 覆盖 | — | 达标 |

## 9.3 管线阶段 × 测试覆盖

| 阶段 | 内联测试 | 外部测试 | 错误测试 | 状态 |
|------|:--------:|:--------:|:--------:|------|
| Generate | 0 | 22（combat_pipeline） | 2（edge_cases） | 外部覆盖，内联缺失 |
| Modify | 0 | 0 | 0 | **缺失** |
| Execute | 6 | 4（death）+ 3（golden） | 1 | 良好 |
| Intent/Route | 0 | 0 | 0 | **缺失** |
| Trait Trigger | 4 | 0 | 0 | 良好 |
| Record | 11 | 3（golden） | 0 | 良好 |
| Events | 9 | 0 | 0 | 良好 |
| Combat | 4 | 0 | 0 | 良好 |
| Log | 5 | 0 | 0 | 部分（UI 为主） |

---

# 10. 总体评估

| 维度 | 评级 | 说明 |
|------|------|------|
| 领域规则覆盖 | **C** | 13 个不变量中 4 个完全覆盖，7 个业务规则中 4 个完全覆盖。核心管线阶段有外部测试保护 |
| 测试金字塔 | **B** | Unit/Integration 比例基本合理，Replay 测试偏少 |
| 测试质量 | **B** | 大部分测试验证行为，约 8 个测试为实现细节（消息字段、len 检查） |
| 规范合规 | **D** | Test Case Schema、Standard Test Data、AI Self-Check 均不合规 |
| 错误/边界覆盖 | **C** | 有部分错误测试，但缺少关键错误场景 |
| 确定性 | **A** | 所有测试均为确定性 |
| 外部测试覆盖 | **B** | combat_pipeline.rs 和 death.rs 提供了良好的集成测试覆盖 |

**综合评级：C+**

核心问题：
1. **intent.rs 和 modify.rs 零内联测试** — 两个关键管线阶段完全无内联测试保护
2. **INV-BTL-6（CombatIntent 清除）无覆盖** — 这是防止"下一次行动误读上一次意图"的关键不变量
3. **route_after_action 无测试** — 行动路由是回合推进的核心逻辑

但相比 AI 模块，Battle 模块的外部测试覆盖较好，`combat_pipeline.rs` 和 `death.rs` 提供了重要的集成测试保护，`golden_battle.rs` 提供了 Replay 测试基础。

---

# 11. 行动计划摘要

| 优先级 | 行动项 | 预计新增测试数 |
|--------|--------|---------------|
| P0 | 集成测试：CombatIntent 清除验证 | 2 |
| P0 | 集成测试：route_after_action 路由 | 4 |
| P0 | 集成测试：generate_combat_effects 系统行为 | 4 |
| P1 | 内联测试：modify_effects 阶段 | 5 |
| P1 | 内联测试：base_amount 首次记录 | 2 |
| P1 | 集成测试：EffectQueue 清空显式断言 | 1 |
| P1 | 错误测试补充 | 4 |
| P2 | Test Case Schema 合规 | 0（重构现有） |
| P2 | Standard Test Data 适配 | 0（重构现有） |
| P2 | AI Self-Check 标注 | 0（添加注释） |
| P2 | Replay 测试补充 | 4 |
| P3 | 清理实现细节测试 | 0（修改/删除现有） |
| P3 | events.rs 边界测试 | 2 |

**预计新增测试：28 个，重构现有测试：~8 个**
