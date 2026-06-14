# 技术债清单 — 日志合规扫描

> 扫描日期：2026-06-13
> 扫描范围：`src/` 全量日志点（info!/warn!/error!/debug!/trace!）、unwrap()/expect()、println!/dbg!
> 依据规范：`docs/其他/26日志.md`、`docs/AI开发宪法.md` §14、`.qoder/agents/refactor-guardian.md`

---

## 统计摘要

| 类别 | 发现数 | Critical | High | Medium | Low |
|------|--------|----------|------|--------|-----|
| INFO 缺少 `event` 字段 | 19 | 0 | 19 | 0 | 0 |
| 日志 target 不匹配 | 3 | 0 | 3 | 0 | 0 |
| 日志在循环体内 | 5 | 0 | 2 | 3 | 0 |
| 日志在 Observer/Frame 系统内 | 2 | 0 | 0 | 0 | 2 |
| ERROR 缺少上下文 | 0 | 0 | 0 | 0 | 0 |
| unwrap/expect 非测试代码 | 12 | 2 | 7 | 3 | 0 |
| 业务事件无日志 | 6 | 0 | 6 | 0 | 0 |
| 基础设施无日志 | 3 | 0 | 3 | 0 | 0 |
| **合计** | **50** | **2** | **40** | **6** | **2** |

---

## Debt-001: INFO 缺少 `event` 字段（共 19 处）

**严重程度**: High
**宪法条款**: §14.3.1 — 结构化日志必须包含 `event` 字段

所有 `info!` 调用均缺少 `event` 字段，无法被日志查询系统有效索引。以下逐一列出：

### 战斗模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 1 | `src/battle/events.rs:93` | `info!(unit_id, name, faction, "角色已死亡，从行动队列移除")` | `event` |

### 地图模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 2 | `src/map/data.rs:121` | `info!(map_width, map_height, ...) ` | `event` |
| 3 | `src/map/data.rs:327` | `info!(...) ` | `event` |
| 4 | `src/map/data.rs:362` | `info!(...) ` | `event` |

### 战役模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 5 | `src/campaign/loader.rs:54` | `info!(..., "战役已加载")` | `event` |
| 6 | `src/campaign/progression.rs:22` | `info!(...) ` | `event` |
| 7 | `src/campaign/progression.rs:36` | `info!(...) ` | `event` |

### 装备模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 8 | `src/equipment/definition.rs:247` | `info!(..., "装备已加载")` | `event` |

### 背包模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 9 | `src/inventory/definition.rs:347` | `info!(..., "物品定义已加载")` | `event` |
| 10 | `src/inventory/transfer.rs:119` | `info!(...) ` | `event` |
| 11 | `src/inventory/use_item.rs:106` | `info!(...) ` | `event` |

### 技能模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 12 | `src/skill/domain/mod.rs:44` | `info!(..., "技能已加载")` | `event` |

### AI 模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 13 | `src/ai/behavior.rs:141` | `info!(..., "AI行为已加载")` | `event` |

### Buff 模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 14 | `src/buff/domain.rs:248` | `info!(..., "Buff已加载")` | `event` |

### 角色模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 15 | `src/character/template.rs:252` | `info!(..., "单位模板已加载")` | `event` |
| 16 | `src/character/traits/mod.rs:176` | `info!(..., "Trait已加载")` | `event` |

### Core 模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 17 | `src/core/modifier_rule.rs:385` | `info!(...) ` | `event` |
| 18 | `src/core/registry_loader.rs:126` | `info!(..., count=..., "Registry loaded")` | `event` |

### 回合模块

| # | 位置 | 当前内容 | 缺失字段 |
|---|------|----------|----------|
| 19 | `src/turn/victory_check.rs:246` | `info!(...) ` | `event` |

**影响**: 无法通过日志查询系统按事件类型索引，运维排查时无法快速定位关键事件。

**建议修复**: 统一添加 `event` 字段，格式遵循 `docs/其他/26日志.md` 示例：
```rust
// 修复前
info!(unit_id, name, faction, "角色已死亡，从行动队列移除");

// 修复后
info!(event = "unit_died", unit_id, name, faction, "角色已死亡，从行动队列移除");
```

---

## Debt-002: 日志 target 不匹配（共 3 处）

**严重程度**: High
**宪法条款**: §14.3.2 — 日志 target 必须匹配模块名

| # | 位置 | 当前 target | 应该是 | 模块名 |
|---|------|-------------|--------|--------|
| 1 | `src/battle/record.rs` | `"battle_record"` | `"battle"` | battle |
| 2 | `src/input.rs` | `"input"` | `"input"` | input（可接受） |
| 3 | `src/ui/combat_log_handler.rs` | `"ui"` | `"combat_log"` | combat_log_handler |

**影响**: 日志过滤时无法按模块名精准筛选，运维日志查询不一致。

**建议修复**:
```rust
// battle/record.rs — 移除 target，使用模块名
warn!("BattleRecord: 路径不存在: {}", path.display());

// ui/combat_log_handler.rs — 使用具体子模块名
debug!(target: "combat_log", ...);
```

---

## Debt-003: 日志在循环体内（共 5 处）

**严重程度**: Medium
**宪法条款**: §14.4 — 日志不应在热路径循环中产生过多输出

### 循环内 debug! 级别（高频）

| # | 位置 | 循环上下文 | 级别 |
|---|------|-----------|------|
| 1 | `src/ui/combat_log_handler.rs:27` | `for msg in reader.read()` | debug! |
| 2 | `src/ui/combat_log_handler.rs:79` | `for msg in reader.read()` | debug! |
| 3 | `src/ui/combat_log_handler.rs:105` | `for msg in reader.read()` | debug! |
| 4 | `src/ui/combat_log_handler.rs:131` | `for msg in reader.read()` | debug! |
| 5 | `src/ui/combat_log_handler.rs:151` | `for msg in reader.read()` | debug! |
| 6 | `src/ui/combat_log_handler.rs:177` | `for msg in reader.read()` | debug! |

**影响**: 战斗中每回合可能产生大量 debug 日志，影响性能。

**建议修复**: debug 级别在生产环境通常关闭，风险可控。但建议在循环外添加计数器，仅输出前 N 条：
```rust
let mut count = 0;
for msg in reader.read() {
    if count < 5 {
        debug!(target: "combat_log", ...);
    }
    count += 1;
}
if count > 5 {
    debug!(target: "combat_log", remaining = count - 5, "... more messages");
}
```

---

## Debt-004: 日志在 Observer/Frame 系统内（共 2 处）

**严重程度**: Low
**宪法条款**: §14.4 — 日志不应在高频帧逻辑中产生过多输出

| # | 位置 | 系统类型 | 级别 |
|---|------|----------|------|
| 1 | `src/character/components.rs:102` | `Dead` hook Observer | trace! |
| 2 | `src/character/components.rs:121` | Dead Observer | trace! |

**影响**: trace 级别在生产环境关闭，实际无性能影响。

**建议修复**: 当前行为可接受，无需修改。

---

## Debt-005: unwrap/expect 非测试代码（共 12 处）

**严重程度**: Critical (2处) / High (7处) / Medium (3处)
**宪法条款**: §14.6 — ERROR 日志必须包含上下文；§11 — panic 安全

### Critical — 可能导致游戏崩溃

| # | 位置 | 调用 | 风险说明 |
|---|------|------|----------|
| 1 | `src/ai/behavior.rs:80` | `.expect("至少需要一个 AI 行为定义")` | 如果无 AI 行为定义注册则 panic |
| 2 | `src/ai/strategy.rs:355` | `.expect("...策略必须注册")` | 策略未注册则 panic |

### High — 潜在 panic 风险

| # | 位置 | 调用 | 风险说明 |
|---|------|------|----------|
| 3 | `src/ai/behavior.rs:185` | `.unwrap()` | RON 解析失败则 panic |
| 4 | `src/ai/strategy.rs:367` | `.expect("...策略必须注册")` | 策略未注册则 panic |
| 5 | `src/ai/strategy.rs:379` | `.expect("...策略必须注册")` | 策略未注册则 panic |
| 6 | `src/map/pathfinding/cost.rs:122` | `.expect("GroundCostCalculator 必须存在")` | 默认计算器缺失则 panic |
| 7 | `src/character/movement_execution.rs:174` | `.unwrap()` | A* 寻路计算 |
| 8 | `src/inventory/transfer.rs:70` | `.unwrap()` | def_opt 有 is_some() 保护，但不清晰 |
| 9 | `src/inventory/transfer.rs:158` | `.unwrap()` | def_opt 有 is_some() 保护，但不清晰 |

### Medium — 安全但不够健壮

| # | 位置 | 调用 | 风险说明 |
|---|------|------|----------|
| 10 | `src/buff/resolve.rs` (tests) | `.unwrap()` | 测试代码，可接受 |
| 11 | `src/ui/settings.rs:158,159` | `.expect()` | 测试代码，可接受 |
| 12 | `src/ui/settings.rs:203,204` | `.expect()` | 测试代码，可接受 |

**影响**: Critical 级别在运行时缺少必要资源时直接 panic，导致游戏崩溃而非优雅降级。

**建议修复**:
```rust
// Critical: behavior.rs:80 — 替换为返回 Result
let behavior = registry.get(name)
    .ok_or_else(|| {
        error!(event = "ai_behavior_not_found", name, "AI行为定义未注册");
        GameError::AiBehaviorNotFound(name.to_string())
    })?;

// High: inventory/transfer.rs:70 — 用 if let 替代 unwrap
if let Some(def) = def_opt {
    // use def
} else {
    warn!(event = "item_def_missing", item_id, "物品定义不存在，跳过操作");
    return;
}
```

---

## Debt-006: 业务事件无日志（共 6 类）

**严重程度**: High
**宪法条款**: §14.3.1 — 结构化日志必须记录关键业务事件

| # | 事件类型 | 当前日志 | 影响 |
|---|----------|----------|------|
| 1 | BattleStarted / BattleEnded | 无 | 无法追踪战斗生命周期 |
| 2 | UnitMoved | 无 | 无法追踪单位移动历史 |
| 3 | BuffApplied / BuffRemoved / BuffExpired | 无 | 无法追踪 Buff 状态变化 |
| 4 | SkillActivated | 无 | 无法追踪技能使用情况 |
| 5 | EquipmentEquipped / Unequipped | 无 | 无法追踪装备变更 |
| 6 | LevelCompleted | 无 | 无法追踪关卡进度 |

**影响**: 运维和调试时无法通过日志还原关键业务流程，排查问题效率低下。

**建议修复**: 在每个事件触发点添加结构化日志：
```rust
// 例: battle/events.rs — BattleStarted
info!(event = "battle_started", map_id, turn_count, "战斗开始");

// 例: buff/apply.rs — BuffApplied
info!(event = "buff_applied", buff_id, unit_id, "Buff 已施加");
```

---

## Debt-007: 基础设施无日志（共 3 处）

**严重程度**: High
**宪法条款**: §14.3.1 — 关键基础设施操作必须记录

| # | 位置 | 缺失内容 | 影响 |
|---|------|----------|------|
| 1 | `src/assets.rs` | 完全无日志 | 字体/资源加载失败静默忽略 |
| 2 | `src/turn/state.rs` | 无状态转换日志 | 无法追踪回合状态机流转 |
| 3 | `src/core/snapshot.rs` | 无存档日志 | 无法追踪存档/读档操作 |

**影响**: 资源加载失败、状态机异常、存档损坏等关键问题无法通过日志发现。

**建议修复**:
```rust
// assets.rs — 字体加载
info!(event = "fonts_loaded", count = fonts.len(), "字体资源已加载");

// turn/state.rs — 状态转换
info!(event = "turn_state_transition", from = ?old, to = ?new, "回合状态转换");

// core/snapshot.rs — 存档操作
info!(event = "snapshot_saved", path, "存档已保存");
```

---

## 优先级建议

### 第一批（立即修复）— 2 项 Critical + 40 项 High

1. **Debt-005 Critical** (2 处): 替换 `expect()` 为 `Result` 返回，避免游戏崩溃
2. **Debt-001** (19 处): 所有 `info!` 添加 `event` 字段，统一格式
3. **Debt-006** (6 类): 为关键业务事件添加日志
4. **Debt-007** (3 处): 为基础设施添加日志

### 第二批（本迭代内）— 7 项 High + 5 项 Medium

5. **Debt-005 High** (7 处): 替换 `unwrap()` 为安全访问模式
6. **Debt-002** (3 处): 修正日志 target 为模块名
7. **Debt-003** (5 处): 循环内日志添加计数器限制

### 第三批（可选优化）— 2 项 Low

8. **Debt-004** (2 处): trace 级别在 Observer 内，当前可接受

---

## 交接建议

- **Debt-005 Critical** → 建议调用 **@architect** 评估 panic 安全策略
- **Debt-001 / Debt-006** → 建议调用 **@feature-developer** 执行日志补充
- **Debt-002 / Debt-003** → 建议调用 **@code-reviewer** 复审日志格式一致性
- **Debt-007** → 建议调用 **@feature-developer** 补充基础设施日志

---

## 重构后验证

- [ ] `cargo build 2>&1 | grep -c "warning"` — 编译警告数量不增加
- [ ] 所有 `info!` 调用包含 `event` 字段
- [ ] 无 `println!` / `dbg!` 残留
- [ ] `unwrap()` / `expect()` 在非测试代码中仅存在于已知安全场景
- [ ] `cargo test` 全部通过
