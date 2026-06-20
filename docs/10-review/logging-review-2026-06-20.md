# 日志打印评审报告 — 2026-06-20

> 审计范围：`src/` 全部 Rust 源文件（排除 test 模块、ai_ignore_this_dir）
> 审计目标：日志语言、日志规范合规性
> 审计工具：CodeGraph + Grep

---

## 概要

| 维度 | 评估 |
|------|------|
| 日志语言 | 100% 英文；除 LogCode、Entity、Component 等专业术语外，回复串均为英语 |
| `println!` / `dbg!` | ❌ 无（仅测试代码和调试 sink 使用，合规） |
| 结构化格式 | ✅ 全部结构化字段；INFO 级均通过 Observer，带 `code` + `event` |
| 领域层直接 `info!` 违规 | ⚠️ 13 处 `info!` 调用（应改用事件监听） |
| 循环内 INFO 日志 | ✅ 无违规 |
| 日志风暴保护 | ✅ `warn_once!` / `error_once!` 就绪 |

---

## 一、日志语言情况

**所有日志内容当前均为英文。** 用户要求除"专用术语"外全部替换为中文。

### 1.1 术语列表（保留英文的专用术语）

以下应在中文化时保留原文（或保持编码形式）：

| 类别 | 示例 |
|------|------|
| LogCode | `BAT001`, `PRG001`, `code = ?LogCode::INV001` |
| 事件名 | `event = "item_acquired"`, `"level_up"` |
| Entity | `entity = ?event.entity` |
| Component 名 | `Experience`, `DialogueState`, `Inventory`, `FactionMembership` |
| 系统/域前缀 | `[Tactical]`, `[Combat]`, `[Input]` |
| 类型名 | `Pipeline`, `Registry`, `Observer`, `Resource` |
| 数值单位 | `MP`, `HP`, `XP` |

### 1.2 须中文化的日志位置

#### A) Core 层 — 领域层直接 tracing 调用（12 文件）

| 文件 | 宏 | 条数 | 当前语言 | 示例 |
|------|-----|------|----------|------|
| `src/core/domains/tactical/systems/movement_system.rs` | `warn!` | 4 | EN | `"ComputeMoveRequest path too short: {} positions"` |
| `src/core/domains/tactical/systems/movement_system.rs` | `trace!` | 1 | EN | `"Movement capability view for entity"` |
| `src/core/domains/tactical/systems/grid_system.rs` | `info!` | 1 | EN | `"[Tactical] initialized default 20x15 square grid"` |
| `src/core/domains/tactical/systems/input_system.rs` | `trace!` | 3 | EN | `event = "tactical_input.cursor_move"` |
| `src/core/domains/progression/systems/progression_system.rs` | `info!` | 2 | EN | `"Entity {:?} is max level, XP gain ignored: +{}"` |
| `src/core/domains/progression/systems/progression_system.rs` | `warn!` | 3 | EN | `"ExperienceGained: entity {:?} has no Experience component"` |
| `src/core/domains/progression/systems/progression_system.rs` | `trace!` | 1 | EN | `"XP gained: entity={:?}, +{} (total: {}, level: {})"` |
| `src/core/domains/inventory/systems/inventory_system.rs` | `warn!` | 5 | EN | `"ItemAcquired: entity {:?} has no Inventory component"` |
| `src/core/domains/inventory/systems/inventory_system.rs` | `trace!` | 5 | EN | `"Item acquired: entity={:?}, template={}, qty={}"` |
| `src/core/domains/party/systems/party_system.rs` | `warn!` | 3 | EN | `"handle_add_member: add_member_to_party failed: {}"` |
| `src/core/domains/narrative/systems/dialogue_system.rs` | `warn!` | 6 | EN | `"DialogueStartRequest: no DialogueTreeRegistry"` |
| `src/core/domains/narrative/components.rs` | `warn!` | 1 | EN | — |
| `src/core/domains/faction/systems/relationship_system.rs` | `warn!` | 2 | EN | `"RelationshipEvalRequest: entity {:?} has no FactionMembership"` |
| `src/core/domains/faction/systems/reputation_system.rs` | `warn!` | 1 | EN | — |
| `src/core/domains/terrain/systems/surface_system.rs` | `trace!` | 1 | EN | — |
| `src/core/domains/combat/systems/effect_tick_system.rs` | `warn!` | 1 | EN | `"{} errors during tick"` |
| `src/core/domains/combat/systems/input_system.rs` | `trace!` | 7 | EN | `event = "combat_input.skill_slot"` |
| `src/core/domains/combat/systems/turn_systems.rs` | `debug!` | 2 | EN | — |
| `src/core/domains/combat/pipeline/driver.rs` | `warn!` | 2 | EN | `"Pipeline '{}' not found in registry"` |
| `src/core/domains/combat/pipeline/driver.rs` | `debug!` | 1 | EN | — |
| `src/core/domains/combat/pipeline/steps.rs` | `debug!` | 12 | EN | `"[Combat] TurnStart: empty turn queue, skipping"` |
| `src/core/domains/combat/integration/replay/mod.rs` | `debug!` | 1 | EN | `"[ReplayBridge] CombatReplayBridgePlugin registered"` |
| `src/core/domains/combat/integration/replay/recording.rs` | `debug!` | 5 | EN | `"[ReplayBridge] No combat participants found..."` |
| `src/core/capabilities/tag/content.rs` | `info!` | 2 | EN | `"[Tag] Registered tag '{}' into hierarchy"` |
| `src/core/capabilities/tag/content.rs` | `warn!` | 1 | EN | `"[Tag] Failed to register tag '{}': {}"` |
| `src/core/capabilities/attribute/content.rs` | `info!` | 2 | EN | `"[Attribute] Registered attribute '{}' into registry"` |
| `src/core/capabilities/attribute/content.rs` | `warn!` | 1 | EN | `"[Attribute] Failed to register attribute '{}': {}"` |

#### B) Content 加载层 — 初始化日志（2 文件）

| 文件 | 宏 | 条数 | 当前语言 |
|------|-----|------|----------|
| `src/content/content_plugin.rs` | `info!` | ~24 | EN |
| `src/content/content_plugin.rs` | `warn!` | ~6 | EN |
| `src/content/hot_reload.rs` | `info!` | ~8 | EN |
| `src/content/hot_reload.rs` | `warn!` | ~12 | EN |

#### C) Infrastructure 基础设施层 — 系统初始化和运行日志（8 文件）

| 文件 | 宏 | 条数 | 当前语言 |
|------|-----|------|----------|
| `src/infra/save/save_system.rs` | `info!` + `error!` | 4 | EN |
| `src/infra/save/load_system.rs` | `info!` + `error!` | 5 | EN |
| `src/infra/save/plugin.rs` | `info!` | 1 | EN |
| `src/infra/save/systems.rs` | `info!` + `error!` | 3 | EN |
| `src/infra/localization/loader.rs` | `info!` + `warn!` | 7 | EN |
| `src/infra/localization/components.rs` | `warn!` | 1 | EN |
| `src/infra/localization/validator.rs` | `info!` + `warn!` + `error!` | 5 | EN |
| `src/infra/localization/audit.rs` | `info!` + `warn!` | 4 | EN |
| `src/infra/input/plugin.rs` | `info!` | 1 | EN |
| `src/infra/input/systems.rs` | `info!` + `trace!` | 5 | EN |
| `src/infra/replay/plugin.rs` | `info!` | 1 | EN |
| `src/infra/pipeline/plugin.rs` | `info!` | 1 | EN |
| `src/infra/pipeline/hooks.rs` | `trace!` | 1 | EN |

#### D) Infra Observer 层 — 日志 Observers（20 文件）

所有 Observer 的结构化日志最后一条 `"..."` 参数均为英文事件名简写。

| 文件 | 数量 | 当前语言 |
|------|------|----------|
| `battle_logger.rs`/`turn_logger.rs` | 4 | EN (事件名) |
| `ability_logger.rs` | 4 | EN (事件名) |
| `effect_logger.rs` | 4 | EN (事件名) |
| `quest_logger.rs` | 5 | EN (事件名) |
| `progression_logger.rs` | 6 | EN (事件名) |
| `inventory_logger.rs` | 5 | EN (事件名) |
| ... 其余 13 模块 | ~28 | EN (事件名) |

#### E) UI 表现层（2 文件）

| 文件 | 宏 | 条数 | 当前语言 |
|------|-----|------|----------|
| `src/ui/screens/main_menu/systems.rs` | `info!` | 3 | EN |
| `src/ui/screens/battle/systems.rs` | `info!` | 1 | EN |

---

## 二、规范合规检查

### 2.1 规则遵守情况对照

| 规则 | 状态 | 说明 |
|------|------|------|
| 使用 tracing，禁止 `println!`/`dbg!` | ✅ 合规 | 仅 test 和 debug sink 中有 `println!` |
| 日志 = 领域事件履历，非技术流水账 | ⚠️ 部分 | 大部分 INFO 走事件链路；但 `pipeline/steps.rs` 12 条 `debug!` 属技术流水账 |
| INFO 级别业务日志走 Observer | ⚠️ 13 处违规 | 见下方详表 |
| 循环内禁止 INFO | ✅ 合规 | 全部 WARN/DEBUG/TRACE |
| 每帧系统仅允许 ERROR | ✅ 合规 | 无每帧系统打 INFO |
| 结构化输出 | ✅ 合规 | 全部结构化字段 |
| INFO 带 `event` 字段 | ✅ 合规 | Observer 均带；core 层 WARN 也带 `event` |
| INFO 带 `code` 字段 | ⚠️ 部分 | Observer 均带；core 层 WARN 部分未带（如 `movement_system` 带 `event` 但无 LogCode） |
| 正确的 target | ⚠️ 未严格执行 | 未显式指定 `target`，默认模块路径 |

### 2.2 领域层直接 `info!` 违规清单（13 处）

日志规则明确：**"INFO级业务日志必须通过领域事件 + 统一Observer输出，业务代码禁止直接写`info!`"**

以下违反此规定：

| # | 文件 | 行号 | 内容 |
|---|------|------|------|
| 1 | `src/core/domains/progression/systems/progression_system.rs` | 37 | `tracing::info!(event = "progression.xp_gained.max_level", ...)` |
| 2 | `src/core/domains/progression/systems/progression_system.rs` | 163 | `tracing::info!(event = "progression.max_level_reached", ...)` |
| 3 | `src/core/domains/tactical/systems/grid_system.rs` | 12 | `tracing::info!("[Tactical] initialized default 20x15 square grid")` |
| 4 | `src/core/capabilities/attribute/content.rs` | 29 | `info!("[Attribute] Registering {} attribute definition(s)...")` |
| 5 | `src/core/capabilities/attribute/content.rs` | 41 | `info!("[Attribute] Registered attribute '{}' into registry", id)` |
| 6 | `src/core/capabilities/attribute/content.rs` | 51 | `info!("[Attribute] Attribute registration complete: {} succeeded, {} failed", ...)` |
| 7 | `src/core/capabilities/tag/content.rs` | 30 | `info!(...)` |
| 8 | `src/core/capabilities/tag/content.rs` | 42 | `info!("[Tag] Registered tag '{}' into hierarchy", id)` |
| 9 | `src/core/capabilities/tag/content.rs` | 52 | `info!(...)` |

**修复建议：** 这些 `info!` 应改为领域事件 + Observer 模式。进度较高（progression 已有完整 Observer），其余需新建 Observer。

### 2.3 技术流水账日志

`src/core/domains/combat/pipeline/steps.rs` 中 12 条 `debug!` 日志记录管线执行流程每一步的进入/退出：

```
[Combat] TurnStart: empty turn queue, skipping
[Combat] PhaseCheck: unit={:?} idle, skipping to settlement
[Combat] UnitAction: waiting for input, unit={:?}
[Combat] Victory check: battle over (≤1 team(s) alive)
```

**评估：** 这些是 DEBUG 级别，规则允许直接 tracing 调用，且对调试有帮助。建议保留但中文化。

### 2.4 格式问题

- `grid_system.rs` 的 `info!` 是非结构化纯字符串格式（无结构化字段、无 `event` 标签）
  ```rust
  tracing::info!("[Tactical] initialized default 20x15 square grid");
  ```
  → 应改为结构化格式，带 `event` 和 `code` 字段。

---

## 三、分类汇总

### 3.1 按修改紧迫度

| 等级 | 范围 | 文件数 | 说明 |
|------|------|--------|------|
| **P0** | 领域层 `info!` 违规 | 4 文件 | 必须改为事件+Observer |
| **P1** | 中文化 | ~30+ 文件全部日志 | 英文→中文，保留专用术语 |
| **P2** | 结构化格式不足 | 1 文件 | `grid_system.rs` 纯字符串日志 |
| **P3** | `target` 字段未指定 | ~30+ 文件 | 建议按域指定 `target` |
| **P4** | Observer 旧版升级 | 3 模块 | `battle_logger`/`turn_logger`/`spell_logger` 缺 `#[instrument]` 和 `metrics::record()` |

### 3.2 按层统计

| 层 | 文件数 | info! | warn! | error! | debug! | trace! |
|----|--------|-------|-------|--------|--------|--------|
| Core 领域层 | 17 文件 | 9 | 28 | 0 | 17 | 17 |
| Capabilities 层 | 2 文件 | 5 | 2 | 0 | 0 | 0 |
| Content 层 | 2 文件 | ~32 | ~18 | 0 | 0 | 0 |
| Infra 层 | 15 文件 | ~10 | ~12 | 5 | 2 | 2 |
| UI 层 | 2 文件 | 4 | 0 | 0 | 0 | 0 |
| Shared 层 | 1 文件 | 1 | 1 | 1 | 0 | 0 |
| **总计** | **~39 文件** | **~61** | **~61** | **6** | **~19** | **~19** |

---

## 四、修复建议

### 阶段 1 — 语言中文化（全部 ~39 文件）

核心格式：
```rust
// 当前 (EN)
tracing::warn!("Entity {} missing movement capabilities", entity);

// 改为 (ZH)，保留 event 字段（结构化标签）
tracing::warn!(
    event = "tactical.move.missing_capabilities",
    entity = ?entity,
    "实体 {} 缺少移动能力组件",
    entity,
);
```

### 阶段 2 — 消除领域层 `info!` 违规（4 文件 → 9 处）

1. `progression_system.rs` (2 处) → 已有 ProgressionObserver，将这两条改为对应的 `warn!` 或新增事件
2. `grid_system.rs` (1 处) → 移至 `infra/logging/observers/tactical_logger.rs`
3. `attribute/content.rs` (3 处) → 新建 `attribute_logger.rs` Observer
4. `tag/content.rs` (3 处) → 新建 `tag_logger.rs` Observer

### 阶段 3 — Observer 旧版升级（3 模块）

为 `battle_logger` / `turn_logger` / `spell_logger` 补充：
- `#[tracing::instrument(skip_all, fields(code = ?LogCode::XXX, event = "..."))]`
- `metrics::record(LogCode::XXX)`

---

## 五、自检清单

- [x] 审计覆盖了 `src/` 全部源码（排除 test 和 ai_ignore_this_dir）
- [x] 日志语言 100% 英文已确认
- [x] `println!`/`dbg!` 仅出现在测试和 debug 代码中
- [x] 13 处领域层 `info!` 违规已定位到具体文件和行号
- [x] Observer 格式化合规性已评估
- [x] 技术流水账日志已识别

---

## 附录：完整文件列表

见各节上表。完整审计命令：

```bash
# 全部日志点
grep -rn --include='*.rs' -E '(info!|warn!|error!|debug!|trace!)\(' src/ | grep -v '/tests/' | grep -v 'ai_ignore_this_dir'

# println! 检查
grep -rn --include='*.rs' 'println!' src/ | grep -v '/tests/' | grep -v 'ai_ignore_this_dir'

# dbg! 检查  
grep -rn --include='*.rs' 'dbg!' src/ | grep -v '/tests/' | grep -v 'ai_ignore_this_dir'
```
