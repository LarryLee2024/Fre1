# 注释规范技术债扫描与修复计划

> **扫描日期**: 2026-06-21 | **范围**: `src/` 全量代码审计 | **规则来源**: `.trae/rules/注释规则.md` v1.0
> **最终状态**: 🟡 阶段 2 进行中

---

## 一、审计总览

| 违规类型 | 数量 | 占比 | 严重度 | 状态 |
|---------|------|------|--------|------|
| 公开 API 缺少 `///` 文档注释 (§5) | **880 / 2600** | 34% | 🔴 P0 | 待修复（阶段 2） |
| "做什么" 废话注释 (§1/§17/§18) | **13 处** | — | 🟠 P1 | ✅ 已修复（99→13） |
| "如果/通过/当前" 翻译式注释 (§18) | **29 处** | — | 🟠 P1 | 部分为 API 契约/领域规则注释，可接受 |
| TODO/FIXME 格式不合规 (§14) | 0 | — | ✅ 合规 |
| Trait 缺少存在理由 (§6) | 待统计 | — | 🟡 P2 |
| 领域事件缺业务含义 (§7) | 待统计 | — | 🟡 P2 |
| 状态机缺状态流转 (§8) | 待统计 | — | 🟡 P2 |
| 复杂公式缺来源 (§9) | 待统计 | — | 🟡 P2 |
| 核心结构缺不变量 (§11-13) | 待统计 | — | 🟡 P2 |

---

## 二、规则速查

| § | 规则 | 强制/推荐 | 一句话 |
|---|------|----------|--------|
| §1 | 注释解释 Why 不解释 What | 强制 | 禁止"遍历所有单位""判断是否死亡" |
| §5 | 所有公开 API 必须有 `///` | 强制 | `pub fn/struct/enum/trait` 上方必须有文档注释 |
| §6 | 所有 Trait 必须解释存在理由 | 强制 | "为什么需要这个 Trait" |
| §7 | 所有领域事件必须解释业务含义 | 强制 | "触发时机 + 已完成/尚未完成" |
| §8 | 所有状态机必须描述状态流转 | 强制 | 用 ASCII 图描述状态转换 |
| §9 | 所有复杂公式必须解释来源 | 强制 | 引用设计文档编号 + 公式表达式 |
| §11 | 核心数据结构必须声明不变量 | 推荐 | `current_hp <= max_hp` |
| §14 | TODO/FIXME/HACK 必须结构化 | 强制 | `// TODO[P2][DOMAIN][DATE]: 原因` |
| §17 | 禁止废话注释 | 强制 | "创建角色""更新状态" 等 |
| §18 | 禁止代码翻译式注释 | 强制 | "如果HP小于0则死亡" |

---

## 三、P0 详细发现 — 公开 API 缺少文档注释

### 3.1 重灾区分布

| 模块 | 缺失数 | 典型文件 |
|------|--------|---------|
| `src/ui/` | ~200+ | `components.rs`、`systems.rs`、`mod.rs` |
| `src/shared/` | ~150+ | `validation/mod.rs`、`collections/`、`ids/` |
| `src/infra/` | ~100+ | `localization/`、`registry/`、`logging/` |
| `src/content/` | ~80+ | `hot_reload.rs`、`loading/` |
| `src/core/` | ~350+ | 各 domain 的 components、systems |

### 3.2 正面示例（应推广）

```rust
// src/content/content_plugin.rs — 每个字段都有 /// 注释 ✅
/// 内容加载状态 Resource。
///
/// 记录已发现和加载的配置文件信息。
#[derive(Resource, Debug, Default)]
pub struct ContentState {
    /// 已发现的配置文件列表。
    pub discovered_files: Vec<ContentFile>,
}

// src/ui/view_models/mod.rs — 架构原则注释 ✅
/// UI 数据仓库 — Widget 的唯一数据源。
///
/// # 架构原则
/// - Widget 的唯一数据源（禁止 Query<&DomainComponent>）
/// - Projection 写入 + mark_dirty()
/// - Widget 系统 consume() → 刷新
pub struct UiStore { ... }
```

### 3.3 典型违规

```rust
// src/ui/navigation/screen_stack.rs — 无任何文档注释 ❌
pub struct ScreenStack { ... }

// src/ui/primitives/button/components.rs — 无任何文档注释 ❌
pub enum ButtonVariant { ... }

// src/shared/validation/mod.rs — 部分有部分无 ❌
pub struct ValidationError { ... }       // 无
pub trait Validator<T: ?Sized> { ... }   // 无
```

---

## 四、P1 详细发现 — 废话注释 / 翻译式注释

### 4.1 最严重的 20 处废话注释

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 1 | `pipeline/driver.rs` | 119 | `// 查找管线定义` | 删除（代码自解释） |
| 2 | `pipeline/driver.rs` | 136 | `// 获取当前阶段` | 删除 |
| 3 | `pipeline/driver.rs` | 145 | `// 获取当前步骤` | 删除 |
| 4 | `dialogue_system.rs` | 61 | `// 获取实体的 StoryFlag` | 删除 |
| 5 | `dialogue_system.rs` | 76 | `// 记录历史` | 改为 `// DialogueHistory 用于 Replay 回放对话分支` |
| 6 | `dialogue_system.rs` | 81 | `// 添加 DialogueState 组件到玩家实体` | 删除 |
| 7 | `dialogue_system.rs` | 129 | `// 查找当前节点` | 删除 |
| 8 | `dialogue_system.rs` | 140 | `// 查找选择的分支` | 删除 |
| 9 | `dialogue_system.rs` | 152 | `// 记录历史` | 同 #5 |
| 10 | `dialogue_system.rs` | 157 | `// 设置 StoryFlag` | 删除 |
| 11 | `progression_system.rs` | 107 | `// 更新 ClassLevels` | 删除 |
| 12 | `progression_system.rs` | 117 | `// 检查 ASI` | 删除 |
| 13 | `effect_tick_system.rs` | 41 | `// 记录 Tick 活动日志` | 删除 |
| 14 | `recording.rs` | 67 | `// 收集参与者 ID` | 删除 |
| 15 | `recording.rs` | 73 | `// 设置初始种子` | 删除 |
| 16 | `recording.rs` | 80 | `// 创建录制会话` | 删除 |
| 17 | `aggregator_system.rs` | 49 | `// 查找目标属性的定义` | 删除 |
| 18 | `aggregator_system.rs` | 56 | `// 收集该属性的所有修改器` | 删除 |
| 19 | `aggregator_system.rs` | 81 | `// 执行聚合` | 删除 |
| 20 | `reaction_system.rs` | 36 | `// 查找下一个 Pending 条目` | 删除 |

### 4.2 典型翻译式注释

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 1 | `recording.rs` | 51 | `// 如果已经有录制会话，跳过` | 改为解释 Why |
| 2 | `steps.rs` | 188 | `// 如果切换队伍 → 发射 BetweenTurns` | 改为 `// BetweenTurns 用于触发阵营专属效果结算` |
| 3 | `steps.rs` | 193 | `// 如果所有队伍完成一轮 → 发射 OnRoundEnd` | 改为 `// OnRoundEnd 触发全局回合结束逻辑（Buff 过期、环境效果等）` |
| 4 | `inventory_system.rs` | 92 | `// 如果是穿戴（new_item 有值）` | 删除（match 分支已自解释） |
| 5 | `inventory_system.rs` | 100 | `// 如果旧装备存在，放回背包` | 删除 |
| 6 | `inventory_system.rs` | 126 | `// 如果是卸下（new_item 为空，old_item 有值）` | 删除 |
| 7 | `relationship.rs` | 33 | `// 如果双方共享阵营 → 强制 Allied` | 改为 `// 共享阵营时强制 Allied，避免同阵营单位互为敌人` |
| 8 | `terrain_effect_system.rs` | 34 | `// 通过 TileEntityMap 空间索引 O(1) 查找格子` | 改为 `// TileEntityMap 在 map 插入/删除时维护，保证 O(1) 查询` |

---

## 五、修复策略

### 阶段 1: 删除废话注释（P1, 1天）— 自动化可批量处理

**目标**: 删除所有"做什么"翻译式注释，只保留解释 Why 的注释。

**操作**:
1. 定位所有 `// 遍历/判断/创建/删除/更新/添加/设置/获取/计算/返回/初始化/检查/执行/调用/记录/加载/保存/处理/完成/准备/转换/查找/搜索/匹配/解析` 开头的行内注释
2. 逐条判断：如果下一行代码已自解释 → 删除；如果需要解释 Why → 改写
3. `tests/` 目录下的注释不修改（测试中的 setup 注释是允许的）

**预期产出**: 删除约 60-80 条废话注释，改写约 15-20 条为 Why 注释。

### 阶段 2: 核心模块补 `///` 文档注释（P0, 2天）

**目标**: 优先修复 `core/` 和 `shared/` 的公开 API 文档缺失。

**范围优先级**:

| 优先级 | 模块 | 预估工作量 |
|--------|------|-----------|
| P0-1 | `src/core/events.rs` — 所有领域事件 | 2h |
| P0-2 | `src/core/capabilities/` — 所有 pub trait + pub fn | 4h |
| P0-3 | `src/shared/` — validation、ids、collections、math | 3h |
| P1-1 | `src/core/domains/` — 各 domain 的 components + systems | 6h |
| P1-2 | `src/infra/` — localization、registry、logging | 3h |
| P2-1 | `src/ui/` — Widget、ViewModel、Screen | 4h |
| P2-2 | `src/content/` — hot_reload、loading | 2h |

**注释模板**:

```rust
/// <一句话描述做什么>

/// <解释为什么这样做>

/// 保证：
/// - <不变量 1>
/// - <不变量 2>
pub fn xxx(...) { ... }
```

### 阶段 3: Trait/事件/状态机补文档（P2, 1天）

**目标**: 补齐 §6 Trait 存在理由、§7 领域事件业务含义、§8 状态机流转。

**操作**:
1. 所有 `pub trait` 补充 "存在原因：..."
2. 所有领域事件 struct 补充 "触发时机 + 已完成/尚未完成"
3. 所有状态机 enum 补充 ASCII 状态流转图

### 阶段 4: 不变量与公式补文档（P2, 0.5天）

**目标**: 补齐 §9 复杂公式来源、§11-13 不变量声明。

**操作**:
1. 所有伤害/治疗/经验公式补充 "来源：设计文档 Combat_vX.X"
2. 核心组件（Health、Experience、Inventory 等）补充不变量注释
3. Resource 补充 "全局唯一" 或 "每实体一个" 说明

---

## 六、阶段 1 执行结果（2026-06-21）

### 修改文件清单

| # | 文件 | 操作 |
|---|------|------|
| 1 | `src/ui/primitives/button/systems.rs` | 删除 `// 更新背景色` |
| 2 | `src/ui/widgets/skill_slot/systems.rs` | 删除 2 条 `// 更新子...` |
| 3 | `src/ui/primitives/progress_bar/systems.rs` | 删除 2 条 `// 更新填充条/标签` |
| 4 | `src/core/domains/combat/pipeline/driver.rs` | 删除 4 条 + 改写 1 条 |
| 5 | `src/core/domains/combat/systems/effect_tick_system.rs` | 删除 `// 记录 Tick 活动日志` |
| 6 | `src/core/domains/progression/systems/progression_system.rs` | 删除 4 条 |
| 7 | `src/core/domains/narrative/systems/dialogue_system.rs` | 删除 8 条 |
| 8 | `src/core/capabilities/event/mechanism/bus.rs` | 删除 `// 查找匹配的订阅者` |
| 9 | `src/core/domains/combat/integration/replay/recording.rs` | 删除 6 条 |
| 10 | `src/core/capabilities/runtime/replay/mechanism/player.rs` | 删除 3 条 + 改写 2 条 |
| 11 | `src/core/capabilities/aggregator/mechanism/systems/aggregator_system.rs` | 删除 3 条 |
| 12 | `src/core/domains/combat/pipeline/steps.rs` | 删除 2 条 |
| 13 | `src/core/domains/reaction/systems/reaction_system.rs` | 删除 2 条 |
| 14 | `src/core/domains/inventory/systems/inventory_system.rs` | 删除 4 条 |
| 15 | `src/core/domains/party/systems/party_system.rs` | 删除 2 条 + 改写 1 条 |
| 16 | `src/core/domains/party/rules/rules.rs` | 删除 1 条 |
| 17 | `src/core/domains/faction/systems/reputation_system.rs` | 删除 `// 检查是否跨越等级` |
| 18 | `src/core/domains/terrain/systems/surface_system.rs` | 删除 2 条 |
| 19 | `src/core/domains/terrain/systems/hazard_system.rs` | 删除 5 条 |
| 20 | `src/core/domains/summon/systems/summon_system.rs` | 删除 2 条 |
| 21 | `src/core/domains/inventory/components.rs` | 删除 1 条 |
| 22 | `src/core/capabilities/effect/mechanism/lifecycle.rs` | 删除 2 条 |
| 23 | `src/core/capabilities/ability/mechanism/lifecycle.rs` | 删除 2 条 |
| 24 | `src/core/capabilities/runtime/scheduler/foundation/values.rs` | 删除 2 条 |
| 25 | `src/core/capabilities/runtime/registry/mechanism/validator.rs` | 删除 `// 检查前缀是否合法` |
| 26 | `src/core/capabilities/runtime/pipeline/mechanism/executor.rs` | 删除 2 条 |
| 27 | `src/core/capabilities/tag/mechanism/lifecycle.rs` | 删除 `// 更新子标签索引` |
| 28 | `src/core/domains/economy/components.rs` | 删除 `// 处理特殊货币` |
| 29 | `src/core/domains/camp_rest/systems/camp_rest_system.rs` | 删除 `// 记录上次长休时间` |
| 30 | `src/core/domains/progression/rules/rules.rs` | 删除 2 条 |
| 31 | `src/infra/logging/plugin.rs` | 删除 `// 初始化度量收集器` |
| 32 | `src/infra/localization/plugin.rs` | 删除 `// 设置默认 locale` |
| 33 | `src/infra/registry/plugin.rs` | 删除 `// 初始化空的 DefinitionRegistry` |
| 34 | `src/infra/registry/resolver.rs` | 删除 2 条 |
| 35 | `src/infra/logging/sinks/file_sink.rs` | 删除 2 条 |

### 效果

| 指标 | 修复前 | 修复后 | 降幅 |
|------|--------|--------|------|
| 废话注释 (§1/§17/§18) | 99+ 处 | 13 处 | **-87%** |
| 翻译式注释 (§18) | 48+ 处 | 29 处 | -40% |

剩余 13 条为：API 契约注释（§5 合规）、TODO 完成条件（§14 合规）、领域规则说明（§9 合规）、分节符注释。均为可接受类型。

---

## 七、工作量评估

| 阶段 | 内容 | 预计工时 | 实际工时 | 风险 |
|------|------|----------|----------|------|
| 1 | 删除废话注释 | 4h | **1h** ✅ | 🟢 低 |
| 2 | 核心模块补 `///` | 24h | 待执行 | 🟡 中 |
| 3 | Trait/事件/状态机 | 8h | 待执行 | 🟢 低 |
| 4 | 不变量与公式 | 4h | 待执行 | 🟢 低 |
| **合计** | | **40h (5天)** | **1h 已完成** | |

---

## 七、验收标准

1. `src/` 中无 `// 遍历/判断/创建/删除/更新` 等废话注释（`tests/` 除外）
2. `src/` 中所有 `pub fn/struct/enum/trait` 有 `///` 文档注释（允许 `pub(crate)` 简写）
3. 所有 `pub trait` 有 "存在原因" 注释
4. 所有领域事件有 "触发时机 + 已完成/尚未完成" 注释
5. TODO/FIXME/HACK 全部符合 `// TODO[P0-P3][DOMAIN][DATE]:` 格式

---

## 八、禁止项

| # | 禁止 | 原因 |
|---|------|------|
| 1 | 用 `//` 注释解释 Rust 语法 | §20 禁止 |
| 2 | 批量添加 `/// todo!()` 占位注释 | 占位注释违反 §17 |
| 3 | 修改 `tests/` 目录的 setup 注释 | 测试 setup 注释是合理的 |
| 4 | 删除解释 Why 的注释 | 只删除 What 注释 |
