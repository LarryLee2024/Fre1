# 日志规范合规性评审分析报告

## 一、当前状态总结

### 已有基础设施
| 组件 | 状态 | 位置 |
|------|------|------|
| `CombatLog` (UI日志) | ✅ 已实现 | `src/battle/log.rs` |
| `BattleRecord` (回放日志) | ✅ 已实现 | `src/battle/record.rs` |
| `CombatLogPlugin` | ✅ 已注册 | `src/battle/mod.rs` |
| `combat_log_handler.rs` | ✅ 已实现 | `src/ui/combat_log_handler.rs` |
| `RegistryLoader` (配置加载) | ✅ 已实现 | `src/core/registry_loader.rs` |
| `CampaignLoader` (战役加载) | ✅ 已实现 | `src/campaign/loader.rs` |
| tracing 日志框架 | ⚠️ 未显式初始化 | 依赖 Bevy 默认 |

### 日志统计
- **总计**: ~100 条日志语句，分布在 34 个文件
- **info!**: 少量（主要在 `battle/events.rs`, `turn/order.rs`）
- **debug!**: 中等（主要在 `ui/combat_log_handler.rs`）
- **warn!**: 少量（主要在 `core/registry_loader.rs`）
- **error!**: 少量（主要在 `core/registry_loader.rs`, `campaign/loader.rs`）
- **trace!**: 仅 `battle/record.rs` 的 BattleEntry 记录

### 已有日志覆盖的模块
| 模块 | 日志覆盖 | 评价 |
|------|----------|------|
| `core/registry_loader.rs` | ✅ 完整 | 目录不存在、解析失败、空目录均有日志 |
| `campaign/loader.rs` | ✅ 完整 | 关卡ID验证、解析失败、读取失败均有日志 |
| `battle/events.rs` | ✅ 有 | CharacterDied 有 info! |
| `ui/combat_log_handler.rs` | ✅ 有 | 各事件有 debug! |
| `battle/record.rs` | ✅ 有 | 各 BattleEntry 有 trace! |

---

## 二、关键差距分析

### 🔴 Critical — 违反宪法条款

| # | 差距 | 违反条款 | 影响 |
|---|------|----------|------|
| C1 | 缺少 `docs/conventions/logging.md` | §14 + 26日志.md | AI 无法遵守日志白名单 |
| C2 | 日志缺少 `event` 结构化字段 | §14.3.1 | 无法按事件名检索 |
| C3 | 日志 target 不统一 | §14.3.2 | 无法按领域过滤 |
| C4 | 无 `BattleLogEvent` 统一事件体系 | §14.5 | 战斗日志与运行日志混用 |

### 🟡 High — 重要遗漏（业务事件）

| # | 差距 | 影响 |
|---|------|------|
| H1 | `BattleStarted/BattleEnded` 无日志 | 无法追踪战斗生命周期 |
| H2 | `UnitMoved` 无日志 | 无法追踪单位移动 |
| H3 | `BuffApplied/BuffRemoved/BuffExpired` 无日志 | 无法追踪 Buff 状态 |
| H4 | `SkillActivated` 无日志 | 无法追踪技能使用 |
| H5 | `EquipmentEquipped/Unequipped` 无日志 | 无法追踪装备变更 |
| H6 | `LevelCompleted` 无日志 | 无法追踪关卡完成 |

### 🟡 High — 重要遗漏（基础设施/调试）

| # | 差距 | 影响 |
|---|------|------|
| H7 | 字体加载失败无日志 | UI 显示异常时无法排查 |
| H8 | `AssetServer` 加载失败无日志 | 资源缺失时无法定位 |
| H9 | 注册表加载统计无日志 | 不知道实际加载了多少配置 |
| H10 | 关卡配置加载统计无日志 | 不知道实际加载了多少关卡 |
| H11 | 状态转换无日志 | 游戏卡住时无法排查 |
| H12 | `GameOverState` 变化无日志 | 胜负判定无法追踪 |

### 🟢 Medium — 需改进

| # | 差距 | 影响 |
|---|------|------|
| M1 | 部分日志在循环体内 | 性能风险 |
| M2 | 无 `RUST_LOG` 分级过滤示例 | 开发体验差 |
| M3 | `BattleEntry` 不是 Bevy Event/Message | 无法使用 Observer 模式 |
| M4 | `unwrap()` 在非测试代码中使用 | 运行时 panic 无日志 |
| M5 | `expect()` 在非测试代码中使用 | 运行时 panic 无日志 |

---

## 三、领域事件日志缺口矩阵

### 3.1 业务事件（按模块分类）

| 领域事件 | 当前日志状态 | 建议日志级别 | 建议 target | 优先级 |
|----------|-------------|-------------|-------------|--------|
| `BattleStarted` | ❌ 无 | INFO | battle | P0 |
| `BattleEnded` | ❌ 无 | INFO | battle | P0 |
| `TurnStarted` | ✅ 有 (trace) | INFO | turn | P1 |
| `TurnEnded` | ✅ 有 (trace) | INFO | turn | P1 |
| `UnitMoved` | ❌ 无 | INFO | character | P0 |
| `UnitAttacked` | ✅ 有 (debug) | INFO | battle | P1 |
| `UnitDamaged` | ✅ 有 (trace) | INFO | battle | P1 |
| `UnitDied` | ✅ 有 (info) | INFO | battle | P1 |
| `BuffApplied` | ❌ 无 | INFO | buff | P0 |
| `BuffRemoved` | ❌ 无 | INFO | buff | P0 |
| `BuffExpired` | ❌ 无 | INFO | buff | P0 |
| `SkillActivated` | ❌ 无 | INFO | skill | P0 |
| `EquipmentEquipped` | ❌ 无 | INFO | equipment | P1 |
| `EquipmentUnequipped` | ❌ 无 | INFO | equipment | P1 |
| `HealApplied` | ✅ 有 (debug) | INFO | battle | P1 |
| `StunApplied` | ✅ 有 (debug) | INFO | battle | P1 |
| `DotApplied` | ✅ 有 (debug) | INFO | buff | P1 |
| `HotApplied` | ✅ 有 (debug) | INFO | buff | P1 |
| `LevelCompleted` | ❌ 无 | INFO | campaign | P0 |
| `QuestCompleted` | ❌ 无 | INFO | campaign | P1 |
| `TurnOrderChanged` | ❌ 无 | INFO | turn | P1 |

### 3.2 基础设施事件（调试/排查用）

| 事件 | 当前日志状态 | 建议日志级别 | 建议 target | 说明 |
|------|-------------|-------------|-------------|------|
| `AssetLoadFailed` | ❌ 无 | ERROR | assets | 字体/纹理加载失败 |
| `RegistryLoaded` | ⚠️ 部分有 | INFO | core | 注册表加载完成统计 |
| `RegistryEmpty` | ⚠️ 有 (warn) | WARN | core | 注册表为空回退默认 |
| `RegistryParseError` | ⚠️ 有 (error) | ERROR | core | RON 解析失败 |
| `ConfigValidationFailed` | ⚠️ 部分有 | ERROR | core | 配置校验失败 |
| `StateChanged` | ❌ 无 | DEBUG | turn | 状态转换记录 |
| `GameOverTriggered` | ❌ 无 | INFO | turn | 胜负判定触发 |
| `SaveFailed` | ❌ 无 | ERROR | campaign | 存档失败 |
| `LoadFailed` | ❌ 无 | ERROR | campaign | 读档失败 |
| `InvariantViolation` | ❌ 无 | ERROR | battle | 运行时不变量违反 |

---

## 四、各模块详细缺口分析

### 4.1 资产加载模块 (`src/assets.rs`)

**当前状态**: 无任何日志

**缺失场景**:
| 场景 | 严重程度 | 建议日志 |
|------|----------|----------|
| 字体文件不存在 | HIGH | `error!(target: "assets", path = %CN_FONT, "字体文件加载失败")` |
| 字体文件损坏 | HIGH | `error!(target: "assets", path = %CN_FONT, error = %e, "字体文件加载失败")` |
| AssetServer 未就绪 | HIGH | `error!(target: "assets", "AssetServer 未初始化")` |

### 4.2 注册表加载模块 (`src/core/registry_loader.rs`)

**当前状态**: ✅ 日志较完整

**已有日志**:
- 目录不存在 → warn
- 文件读取失败 → error
- 文件解析失败 → error
- 目录为空 → warn
- 单文件加载完成 → info

**缺失场景**:
| 场景 | 严重程度 | 建议日志 |
|------|----------|----------|
| 加载完成统计 | LOW | `info!(target: "core", registry = %name, count = total, "注册表加载完成")` |
| 默认数据回退 | INFO | `info!(target: "core", registry = %name, "使用默认数据")` |

### 4.3 战役加载模块 (`src/campaign/loader.rs`)

**当前状态**: ✅ 日志较完整

**已有日志**:
- 关卡ID验证失败 → error
- 解析失败 → error
- 读取失败 → error
- 目录为空 → warn
- 战役加载完成 → info

**缺失场景**:
| 场景 | 严重程度 | 建议日志 |
|------|----------|----------|
| 加载完成统计 | LOW | `info!(target: "campaign", count = campaigns.len(), "战役加载完成")` |

### 4.4 回合系统 (`src/turn/`)

**当前状态**: ⚠️ 部分日志

**已有日志**:
- TurnStarted/TurnEnded → trace (via BattleRecord)

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| 回合开始 | INFO | `info!(target: "turn", event = "TurnStarted", turn = turn_number)` | `turn/order.rs` |
| 回合结束 | INFO | `info!(target: "turn", event = "TurnEnded", turn = turn_number)` | `turn/order.rs` |
| 状态转换 | DEBUG | `debug!(target: "turn", from = ?old, to = ?new, "状态转换")` | `turn/state.rs` |
| 胜负判定 | INFO | `info!(target: "turn", event = "GameOverTriggered", result = ?state)` | `turn/victory_check.rs` |
| 行动队列重建 | DEBUG | `debug!(target: "turn", queue_len = queue.len(), "行动队列重建")` | `turn/order.rs` |

### 4.5 战斗系统 (`src/battle/`)

**当前状态**: ⚠️ 部分日志

**已有日志**:
- CharacterDied → info
- DamageApplied/HealApplied/DotApplied/HotApplied/StunApplied → trace (via BattleRecord)
- CombatLogHandler → debug

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| 战斗开始 | INFO | `info!(target: "battle", event = "BattleStarted", map = %map_id)` | `battle/mod.rs` |
| 战斗结束 | INFO | `info!(target: "battle", event = "BattleEnded", result = ?result, turns = turns)` | `battle/mod.rs` |
| 伤害计算 | DEBUG | `debug!(target: "battle", attacker = %name, base = base, modified = modified)` | `battle/pipeline/modify.rs` |
| 技能触发 | INFO | `info!(target: "battle", event = "SkillActivated", unit = %name, skill = %skill_id)` | `battle/pipeline/` |

### 4.6 Buff 系统 (`src/buff/`)

**当前状态**: ❌ 几乎无日志

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| Buff 施加 | INFO | `info!(target: "buff", event = "BuffApplied", unit = %name, buff = %buff_id, source = %source)` | `buff/apply.rs` |
| Buff 移除 | INFO | `info!(target: "buff", event = "BuffRemoved", unit = %name, buff = %buff_id, reason = %reason)` | `buff/resolve.rs` |
| Buff 过期 | INFO | `info!(target: "buff", event = "BuffExpired", unit = %name, buff = %buff_id)` | `buff/resolve.rs` |
| Buff 叠加 | DEBUG | `debug!(target: "buff", unit = %name, buff = %buff_id, stacks = stacks)` | `buff/apply.rs` |

### 4.7 技能系统 (`src/skill/`)

**当前状态**: ❌ 几乎无日志

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| 技能激活 | INFO | `info!(target: "skill", event = "SkillActivated", unit = %name, skill = %skill_id)` | `skill/` |
| 技能使用失败 | WARN | `warn!(target: "skill", unit = %name, skill = %skill_id, error = ?err)` | `skill/domain/` |
| 技能冷却 | DEBUG | `debug!(target: "skill", unit = %name, skill = %skill_id, remaining = cd)` | `skill/` |

### 4.8 装备系统 (`src/equipment/`)

**当前状态**: ❌ 几乎无日志

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| 装备穿戴 | INFO | `info!(target: "equipment", event = "EquipmentEquipped", unit = %name, item = %item_id, slot = %slot)` | `equipment/equip.rs` |
| 装备卸下 | INFO | `info!(target: "equipment", event = "EquipmentUnequipped", unit = %name, item = %item_id)` | `equipment/equip.rs` |
| 装备效果应用 | DEBUG | `debug!(target: "equipment", item = %item_id, effect = ?effect)` | `equipment/equip.rs` |

### 4.9 角色系统 (`src/character/`)

**当前状态**: ⚠️ 部分日志

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| 单位移动 | INFO | `info!(target: "character", event = "UnitMoved", unit = %name, from = ?from, to = ?to)` | `character/movement_execution.rs` |
| 单位生成 | INFO | `info!(target: "character", event = "UnitSpawned", unit = %name, faction = ?faction)` | `character/spawn.rs` |
| 单位死亡 | INFO | `info!(target: "character", event = "UnitDied", unit = %name)` | `character/` |
| 属性变更 | DEBUG | `debug!(target: "character", unit = %name, attr = %attr, old = old, new = new)` | `character/` |

### 4.10 UI 系统 (`src/ui/`)

**当前状态**: ⚠️ 部分日志

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| 设置加载失败 | WARN | `warn!(target: "ui", error = %e, "设置加载失败，使用默认值")` | `ui/settings.rs` |
| 设置保存失败 | ERROR | `error!(target: "ui", error = %e, "设置保存失败")` | `ui/settings.rs` |
| UI 命令执行 | DEBUG | `debug!(target: "ui", command = ?cmd, "UI命令执行")` | `ui/command_handler.rs` |

### 4.11 存档系统 (`src/core/snapshot.rs`)

**当前状态**: 未确认

**缺失场景**:
| 场景 | 严重程度 | 建议日志 | 位置 |
|------|----------|----------|------|
| 存档创建 | INFO | `info!(target: "campaign", event = "SaveCreated", path = %path)` | `core/snapshot.rs` |
| 存档加载 | INFO | `info!(target: "campaign", event = "SaveLoaded", path = %path)` | `core/snapshot.rs` |
| 存档失败 | ERROR | `error!(target: "campaign", error = %e, "存档操作失败")` | `core/snapshot.rs` |
| 存档损坏 | ERROR | `error!(target: "campaign", path = %path, "存档文件损坏")` | `core/snapshot.rs` |

---

## 五、运行时异常场景日志

### 5.1 Entity 生命周期异常

| 场景 | 严重程度 | 建议日志 |
|------|----------|----------|
| 访问已删除 Entity | ERROR | `error!(target: "battle", entity = ?e, "访问已删除实体")` |
| Entity 缺失必要组件 | ERROR | `error!(target: "battle", entity = ?e, component = %name, "实体缺少必要组件")` |
| Dead 单位参与战斗 | ERROR | `error!(target: "battle", entity = ?e, "死亡单位参与战斗")` |

### 5.2 资源初始化异常

| 场景 | 严重程度 | 建议日志 |
|------|----------|----------|
| Resource 未初始化 | ERROR | `error!(target: "core", resource = %name, "资源未初始化")` |
| Registry 为空 | WARN | `warn!(target: "core", registry = %name, "注册表为空，使用默认数据")` |
| 配置引用无效 | ERROR | `error!(target: "core", ref = %ref, "配置引用无效")` |

### 5.3 状态机异常

| 场景 | 严重程度 | 建议日志 |
|------|----------|----------|
| 非法状态转换 | ERROR | `error!(target: "turn", from = ?old, to = ?new, "非法状态转换")` |
| 状态卡住 | WARN | `warn!(target: "turn", state = ?current, frames = count, "状态持续过久")` |
| 重复进入状态 | DEBUG | `debug!(target: "turn", state = ?current, "重复进入状态")` |

### 5.4 战斗不变量违反

| 场景 | 严重程度 | 建议日志 |
|------|----------|----------|
| 伤害为负 | ERROR | `error!(target: "battle", damage = dmg, "伤害为负")` |
| HP 超出范围 | ERROR | `error!(target: "battle", hp = hp, max = max, "HP超出范围")` |
| 行动队列越界 | ERROR | `error!(target: "turn", index = idx, len = len, "行动队列越界")` |
| 回合数异常 | ERROR | `error!(target: "turn", turn = num, "回合数异常")` |

---

## 六、unwrap()/expect() 风险点

以下非测试代码使用了 `unwrap()` 或 `expect()`，可能导致运行时 panic 但无日志：

| 文件 | 行号 | 代码 | 风险 |
|------|------|------|------|
| `src/ai/behavior.rs:80` | 80 | `.expect("至少需要一个 AI 行为定义")` | AI 配置缺失时 panic |
| `src/ai/behavior.rs:185` | 185 | `.unwrap()` | RON 解析失败时 panic |
| `src/equipment/definition.rs:293` | 293 | `.unwrap()` | 测试代码（可接受） |
| `src/inventory/transfer.rs:70` | 70 | `.unwrap()` | 运行时可能 panic |
| `src/inventory/transfer.rs:158` | 158 | `.unwrap()` | 运行时可能 panic |

**建议**: 将非测试代码的 `unwrap()`/`expect()` 替换为 `Result` + `error!` 日志

---

## 七、七个 Agent 任务说明书

---

### 📋 Agent 1: @refactor-guardian — 技术债扫描

#### 任务目标
扫描现有代码中的日志违规和风险点，输出技术债清单

#### 任务范围
- 扫描所有 `src/**/*.rs` 文件中的日志调用和 `unwrap()`/`expect()` 风险点
- 检查每个日志调用是否符合宪法 §14 要求

#### 检查清单
1. **§14.1.1 禁令**: 是否存在 `println!` 或 `dbg!`
2. **§14.3.1 结构化**: INFO 级别日志是否携带 `event` 字段
3. **§14.3.2 Target**: 日志 `target` 是否与所属 Feature 目录名一致
4. **§14.4 禁令**: 是否在每帧系统中输出 INFO/DEBUG 日志
5. **§14.4 禁令**: 是否在循环/迭代器内部输出 INFO 日志
6. **§14.6 上下文**: ERROR 级别日志是否包含完整上下文
7. **unwrap 风险**: 非测试代码中的 `unwrap()`/`expect()` 是否需要替换

#### 输出格式
```markdown
# 技术债清单 — 日志合规性扫描

## Debt-LOG-001: [问题类型]
- **位置**: src/path/to/file.rs:line
- **严重程度**: Critical / High / Medium / Low
- **违反条款**: 宪法 §14.X.X
- **问题描述**: 具体问题
- **建议修复**: 具体方案
```

#### 预期成果
- `docs/refactor/debt_log_compliance.md` — 完整技术债清单

---

### 📋 Agent 2: @architect — 架构设计

#### 任务目标
设计日志架构 ADR，定义日志系统边界和通信模式

#### 任务范围
- 设计 `docs/adr/ADR-XXX-logging-architecture.md`
- 定义日志系统与现有模块的通信边界

#### ADR 必须包含
1. **Module Design**: 日志相关模块的职责划分
2. **Communication Design**:
   - Message: 跨 Feature 的日志事件（BattleLogEvent）
   - Observer: 同 Feature 内的状态变化响应
   - Hook: 组件添加/移除时的日志
   - 函数调用: 模块内日志
3. **边界定义**:
   - 允许：哪些模块可以产生日志
   - 禁止：哪些访问路径被禁止
4. **Forbidden**:
   - 🟥 禁止：直接使用 `println!`/`dbg!`
   - 🟥 禁止：在循环体内输出 INFO 日志
   - 🟥 禁止：日志包含业务逻辑
5. **Definition/Instance Design**:
   - Definition: 日志配置（级别、target）
   - Instance: 运行时日志记录

#### 预期成果
- `docs/adr/ADR-XXX-logging-architecture.md`

---

### 📋 Agent 3: @domain-designer — 领域建模

#### 任务目标
定义日志领域规则，建立日志白名单

#### 任务范围
- 输出 `docs/domain/logging_rules_v1.md`
- 定义允许记录的领域事件清单

#### 领域模型必须包含

##### 1. 统一术语
| 术语 | 定义 | 职责边界 |
|------|------|----------|
| 运行日志 | tracing 框架输出的程序可观测性日志 | 负责：程序调试、运维监控；不负责：玩家可见 |
| 战斗日志 | BattleLogEvent 事件体系，用于 UI 战斗记录 | 负责：战斗履历、回放、录像；不负责：程序调试 |
| 领域事件 | 业务动作边界产生的结构化日志 | 负责：业务流程可追溯；不负责：技术流水账 |

##### 2. 不变量
- **日志只记录业务事实**: 禁止记录系统运行、函数进入退出
- **INFO 级别最多一条**: 一个完整业务动作最多输出一条 INFO 日志
- **ERROR 必须有上下文**: 所有 ERROR 级别日志必须包含完整复现上下文

##### 3. 禁止事项
- 🟥 禁止：记录函数进入/退出
- 🟥 禁止：记录系统执行开始/结束
- 🟥 禁止：在每帧系统中输出 INFO/DEBUG
- 🟥 禁止：在循环内部输出 INFO 日志

##### 4. 日志白名单（允许记录的事件）
```
battle: BattleStarted, BattleEnded, UnitAttacked, UnitDamaged, UnitDied, UnitHealed, SkillActivated
turn: TurnStarted, TurnEnded, TurnOrderChanged, GameOverTriggered
character: UnitMoved, UnitSpawned, UnitDespawned
buff: BuffApplied, BuffRemoved, BuffExpired, BuffStackChanged
skill: SkillActivated, SkillEffectApplied, SkillFailed
equipment: EquipmentEquipped, EquipmentUnequipped
campaign: LevelCompleted, QuestAccepted, QuestCompleted, SaveCreated, SaveLoaded
core: RegistryLoaded, ConfigValidationFailed
assets: AssetLoadFailed
```

#### 预期成果
- `docs/domain/logging_rules_v1.md`

---

### 📋 Agent 4: @feature-developer — 功能实现

#### 任务目标
实现日志基础设施和补全遗漏的日志

#### 任务范围
分为四个阶段执行

##### Phase 1: 基础设施（必须先做）
1. **创建 `docs/conventions/logging.md`** — 日志规范文档
2. **验证 tracing 初始化** — 确保 `LogPlugin` 或自定义 tracing subscriber 正确配置
3. **统一现有日志格式** — 为所有 INFO 级别日志添加 `event` 字段

##### Phase 2: 补全业务日志（基于缺口矩阵）
按优先级补全以下日志：

| 优先级 | 事件 | 位置 | 日志内容 |
|--------|------|------|----------|
| P0 | BattleStarted | `src/battle/mod.rs` | event, map_id, factions |
| P0 | BattleEnded | `src/battle/mod.rs` | event, result, turns |
| P0 | UnitMoved | `src/character/movement_execution.rs` | event, unit, from, to |
| P0 | BuffApplied | `src/buff/apply.rs` | event, unit, buff_id, source |
| P0 | BuffRemoved | `src/buff/resolve.rs` | event, unit, buff_id, reason |
| P0 | BuffExpired | `src/buff/resolve.rs` | event, unit, buff_id |
| P0 | SkillActivated | `src/skill/` | event, unit, skill_id, target |
| P0 | LevelCompleted | `src/turn/victory_check.rs` | event, level_id, result, turns |
| P1 | EquipmentEquipped | `src/equipment/equip.rs` | event, unit, item, slot |
| P1 | EquipmentUnequipped | `src/equipment/equip.rs` | event, unit, item |
| P1 | TurnStarted | `src/turn/order.rs` | event, turn_number |
| P1 | TurnEnded | `src/turn/order.rs` | event, turn_number |

##### Phase 3: 补全基础设施日志
| 优先级 | 场景 | 位置 | 日志内容 |
|--------|------|------|----------|
| P0 | 字体加载失败 | `src/assets.rs` | error, path, error |
| P1 | 注册表加载统计 | `src/core/registry_loader.rs` | info, registry, count |
| P1 | 状态转换 | `src/turn/state.rs` | debug, from, to |
| P1 | GameOverState 变化 | `src/turn/victory_check.rs` | info, result |
| P2 | 设置加载失败 | `src/ui/settings.rs` | warn, error |
| P2 | 存档操作 | `src/core/snapshot.rs` | info/error, path |

##### Phase 4: 替换 unwrap() 为 Result + 日志
| 文件 | 替换方案 |
|------|----------|
| `src/ai/behavior.rs:80` | `.expect()` → `Result` + `error!` |
| `src/ai/behavior.rs:185` | `.unwrap()` → `Result` + `error!` |
| `src/inventory/transfer.rs:70` | `.unwrap()` → `Result` + `warn!` |
| `src/inventory/transfer.rs:158` | `.unwrap()` → `Result` + `warn!` |

#### 必须遵守
- **§14.3.1**: 所有 INFO 日志必须携带 `event` 字段
- **§14.3.2**: 日志 `target` 必须与 Feature 目录名一致
- **§14.4**: 禁止在循环体内输出 INFO 日志
- **§5.0.1**: 单一职责 — 日志只记录，不执行业务逻辑

#### 自检清单
```
Feature First: PASS/FAIL
Definition/Instance: PASS/FAIL
Rule/Content: PASS/FAIL
Effect Pipeline: PASS/FAIL
Modifier Pipeline: PASS/FAIL
Architecture Violation: NONE/XXX
日志格式合规: PASS/FAIL
日志target合规: PASS/FAIL
循环内无INFO日志: PASS/FAIL
unwrap()已替换: PASS/FAIL
```

#### 预期成果
- `docs/conventions/logging.md`
- 修改 `src/**/*.rs` 中的日志调用
- `cargo build` + `cargo test` 通过

---

### 📋 Agent 5: @test-guardian — 测试守护

#### 任务目标
验证日志系统测试覆盖，确保日志行为符合领域规则

#### 任务范围
1. **审查现有测试**: 检查是否有测试覆盖日志行为
2. **新增日志测试**: 为关键日志行为添加测试

#### 测试重点
| 测试类型 | 测试内容 | 验证目标 |
|----------|----------|----------|
| Unit | `BattleRecord::record()` 写入正确 | 日志记录行为 |
| Unit | `CombatLog::push()` 不超过 MAX_LOG_LINES | 日志截断行为 |
| Unit | `RegistryLoader::load_from_dir()` 返回正确数据 | 配置加载行为 |
| Unit | `CampaignLoader::load_campaigns()` 验证 level_id | 配置校验行为 |
| Integration | 战斗事件触发正确的日志输出 | 事件→日志联动 |
| Integration | 日志 target 正确设置 | 按领域过滤能力 |
| Integration | `unwrap()` 替换后的错误处理 | 错误恢复能力 |

#### 禁止事项
- 🟥 禁止：测试日志的具体文本内容（实现细节）
- 🟥 禁止：测试日志调用次数（实现细节）
- 🟥 禁止：删除测试来消除失败

#### 输出格式
```
## Test Guardian Report — 日志合规性

### Test Plan
[列出需要测试的业务规则]

### Test Matrix
| 规则 | 测试类型 | 断言目标 | 状态 |

### Coverage Report
PASS / FAIL
```

#### 预期成果
- `docs/testing/logging_test_plan.md`
- `src/**/tests/` 中的日志相关测试

---

### 📋 Agent 6: @code-reviewer — 代码审查

#### 任务目标
审查所有日志相关代码，确保合规

#### 任务范围
审查所有修改过日志的文件

#### 审查清单（日志专项）
1. **§14.1.1 框架**: 是否使用 `tracing`（`bevy::log`）
2. **§14.1.2 定位**: 是否记录业务事件事实
3. **§14.3.1 结构化**: INFO 日志是否携带 `event` 字段
4. **§14.3.2 Target**: 日志 target 是否与 Feature 目录名一致
5. **§14.4 禁令**:
   - 是否在每帧系统中输出 INFO/DEBUG
   - 是否在循环内部输出 INFO
   - 是否输出技术流水账
6. **§14.5 分离**: 运行日志与战斗日志是否分离
7. **§14.6 上下文**: ERROR 是否有完整上下文
8. **unwrap 风险**: 非测试代码是否还有 `unwrap()`/`expect()`

#### 输出格式
```
## Code Review Report — 日志合规性

### ✅ 通过的检查
- [列出通过的项]

### ❌ 发现的问题

#### [严重程度] 问题标题
- **位置**: file.rs:line
- **规则**: 宪法 §14.X.X
- **说明**: 为什么这是问题
- **建议**: 如何修复

### 📋 总结
- Critical: X 个
- High: Y 个
- Medium: Z 个
- Low: W 个

### 🎯 结论
PASS / FAIL
```

#### 预期成果
- `docs/reviews/code_review_logging.md`

---

## 八、执行顺序建议

```
┌─────────────────────────────────────────────────────────────┐
│  Phase 0: 分析与设计（并行）                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ @refactor-   │  │ @architect   │  │ @domain-     │       │
│  │ guardian     │  │              │  │ designer     │       │
│  │ 技术债扫描   │  │ 架构设计ADR  │  │ 领域规则     │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│         ↓                  ↓                  ↓              │
├─────────────────────────────────────────────────────────────┤
│  Phase 1: 实现（依赖 Phase 0 输出）                          │
│  ┌──────────────────────────────────────────────────┐       │
│  │ @feature-developer                                │       │
│  │ 1. 创建 docs/conventions/logging.md              │       │
│  │ 2. 统一现有日志格式                               │       │
│  │ 3. 补全遗漏的日志                                 │       │
│  │ 4. 替换 unwrap() 为 Result + 日志                 │       │
│  └──────────────────────────────────────────────────┘       │
│                           ↓                                  │
├─────────────────────────────────────────────────────────────┤
│  Phase 2: 验证（并行）                                       │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │ @test-       │  │ @code-       │                        │
│  │ guardian     │  │ reviewer     │                        │
│  │ 测试验证     │  │ 代码审查     │                        │
│  └──────────────┘  └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 九、关键参考文档

| 文档 | 路径 | 用途 |
|------|------|------|
| AI开发宪法 | `docs/AI开发宪法.md` | §14 日志条款（最高优先级） |
| 日志规范建议 | `docs/其他/26日志.md` | 最佳实践参考 |
| 架构规范 | `docs/architecture.md` | 模块边界约束 |
| 测试规范 | `docs/testing_spec.md` | 测试分层策略 |
| 现有领域规则 | `docs/domain/done/` | 领域术语对齐 |

---

## 十、总结

本评审识别出：

| 类型 | 数量 | 说明 |
|------|------|------|
| Critical | 4 | 违反宪法条款，必须立即修复 |
| High (业务) | 6 | 重要业务事件无日志 |
| High (基础设施) | 6 | 调试/排查所需日志缺失 |
| Medium | 5 | 需要改进的问题 |
| unwrap 风险 | 5 | 非测试代码中的 panic 风险 |

建议按 Phase 0 → Phase 1 → Phase 2 的顺序执行，预计可将日志合规性从当前的 ~40% 提升至 95%+。
