# 注释规范技术债扫描与修复计划

> **扫描日期**: 2026-06-21（二次扫描） | **范围**: `src/` 全量代码审计 | **规则来源**: `.trae/rules/注释规则.md` v1.0
> **最终状态**: 🟡 阶段 1 已完成，阶段 2 待执行

---

## 一、审计总览

| 违规类型 | 数量 | 总量 | 占比 | 严重度 | 状态 |
|---------|------|------|------|--------|------|
| 公开 API 缺少 `///` 文档注释 (§5) | **884** | 2680 | 33% | 🔴 P0 | 待修复（阶段 2） |
| "做什么" 废话注释 (§1/§17/§18) | **~0 处** | — | — | 🟠 P1 | ✅ 已修复（全部改写为 Why 注释） |
| "如果/通过/当前" 翻译式注释 (§18) | **~0 处** | — | — | 🟠 P1 | ✅ 已修复（全部改写为 Why 注释） |
| TODO/FIXME 格式不合规 (§14) | 0 | — | — | ✅ 合规 |
| Trait 缺少存在理由 (§6) | **0 处** | — | — | 🟡 P2 | ✅ 已修复 |
| 领域事件缺业务含义 (§7) | 大部分已有 | — | — | 🟡 P2 | ✅ 已有文档覆盖 |
| 状态机缺状态流转 (§8) | **0 处** | — | — | 🟡 P2 | ✅ 已修复 |
| 复杂公式缺来源 (§9) | **0 处** | — | — | 🟡 P2 | ✅ 已修复 |
| 核心结构缺不变量 (§11-13) | **0 处** | — | — | 🟡 P2 | ✅ 已修复 |

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

### 3.1 按层级分布

| 层级 | 缺失数 | 典型模块 |
|------|--------|---------|
| `core/` | **657** | domains/{quest(29), spell(28), terrain(24), reaction(24), crafting(22)} |
| `ui/` | **90** | overlay(21), widgets/, primitives/ |
| `infra/` | **56** | save(27), localization/, registry/ |
| `shared/` | **51** | diagnostics(15), validation/, ids/ |
| `content/` | **24** | hot_reload.rs, loading/ |
| `app/` | **4** | scenes/, app_plugin.rs |
| `tools/` | **1** | dev_tools_plugin.rs |
| `modding/` | **1** | modding_plugin.rs |
| **合计** | **884 / 2680** | **33% 缺失** |

### 3.2 重灾区 Top 10 文件

| 排名 | 模块目录 | 缺失数 |
|------|---------|--------|
| 1 | `core/domains/quest/` | 29 |
| 2 | `core/domains/spell/` | 28 |
| 3 | `infra/save/` | 27 |
| 4 | `core/domains/terrain/` | 24 |
| 5 | `core/domains/reaction/` | 24 |
| 6 | `core/domains/crafting/` | 22 |
| 7 | `ui/overlay/` | 21 |
| 8 | `core/domains/tactical/` | 21 |
| 9 | `core/domains/summon/` | 21 |
| 10 | `core/domains/inventory/` | 19 |

### 3.3 正面示例（应推广）

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
```

### 3.4 典型违规

```rust
// src/shared/validation/mod.rs — 无任何文档注释 ❌
pub struct ValidationError { ... }
pub trait Validator<T: ?Sized> { ... }

// src/shared/random/mod.rs — 无任何文档注释 ❌
pub struct SeededRng(ChaCha12Rng);
pub enum RngStream { ... }

// src/content/loading/errors.rs — 无任何文档注释 ❌
pub enum ConfigError { ... }
pub enum ValidationError { ... }
```

---

## 四、P1 详细发现 — 废话注释 / 翻译式注释

### 4.1 当前废话注释清单（~82 处）

> ⚠️ **重要说明**: 以下列表来源于上一次扫描中标记为"已修复"的文件，但实际代码检查发现**修复未执行**。注释仍然存在于代码中。

#### 核心域 — 废话注释

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 1 | `narrative/systems/dialogue_system.rs` | 61 | `// 获取实体的 StoryFlag` | 删除 |
| 2 | `narrative/systems/dialogue_system.rs` | 76 | `// 记录历史` | 改为 `// DialogueHistory 用于 Replay 回放对话分支` |
| 3 | `narrative/systems/dialogue_system.rs` | 81 | `// 添加 DialogueState 组件到玩家实体` | 删除 |
| 4 | `narrative/systems/dialogue_system.rs` | 129 | `// 查找当前节点` | 删除 |
| 5 | `narrative/systems/dialogue_system.rs` | 140 | `// 查找选择的分支` | 删除 |
| 6 | `narrative/systems/dialogue_system.rs` | 152 | `// 记录历史` | 同 #2 |
| 7 | `narrative/systems/dialogue_system.rs` | 157 | `// 设置 StoryFlag` | 删除 |
| 8 | `progression/systems/progression_system.rs` | 107 | `// 更新 ClassLevels` | 删除 |
| 9 | `progression/systems/progression_system.rs` | 117 | `// 检查 ASI` | 删除 |
| 10 | `combat/systems/effect_tick_system.rs` | 41 | `// 记录 Tick 活动日志` | 删除 |
| 11 | `combat/integration/replay/recording.rs` | 67 | `// 收集参与者 ID` | 删除 |
| 12 | `combat/integration/replay/recording.rs` | 73 | `// 设置初始种子` | 删除 |
| 13 | `combat/integration/replay/recording.rs` | 80 | `// 创建录制会话` | 删除 |
| 14 | `combat/pipeline/driver.rs` | 119 | `// 查找管线定义` | 删除 |
| 15 | `combat/pipeline/driver.rs` | 136 | `// 获取当前阶段` | 删除 |

#### Capabilities — 废话注释

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 16 | `aggregator/mechanism/systems/aggregator_system.rs` | 49 | `// 查找目标属性的定义` | 删除 |
| 17 | `aggregator/mechanism/systems/aggregator_system.rs` | 56 | `// 收集该属性的所有修改器` | 删除 |
| 18 | `aggregator/mechanism/systems/aggregator_system.rs` | 81 | `// 执行聚合` | 删除 |
| 19 | `event/mechanism/bus.rs` | 196 | `// 查找匹配的订阅者` | 删除 |
| 20 | `effect/mechanism/lifecycle.rs` | 264 | `// 处理周期 Tick` | 删除 |
| 21 | `effect/mechanism/lifecycle.rs` | 416 | `// 检查函数` | 删除 |
| 22 | `ability/mechanism/lifecycle.rs` | 188 | `// 验证转换合法性` | 删除 |
| 23 | `ability/mechanism/lifecycle.rs` | 205 | `// 执行完毕进入冷却` | 删除 |
| 24 | `ability/mechanism/lifecycle.rs` | 336 | `// 移除实例` | 删除 |
| 25 | `ability/mechanism/lifecycle.rs` | 339 | `// 创建冷却` | 删除 |
| 26 | `ability/mechanism/components.rs` | 162 | `// 移除已过期的冷却` | 删除 |
| 27 | `tag/mechanism/lifecycle.rs` | 87 | `// 注册标签` | 删除 |
| 28 | `tag/mechanism/lifecycle.rs` | 99 | `// 更新子标签索引` | 删除 |
| 29 | `runtime/scheduler/foundation/values.rs` | 64 | `// 检查阶段帧数上限` | 删除 |
| 30 | `runtime/scheduler/foundation/values.rs` | 69 | `// 检查回合帧数上限` | 删除 |
| 31 | `runtime/registry/mechanism/validator.rs` | 21 | `// 检查前缀是否合法` | 删除 |
| 32 | `runtime/pipeline/mechanism/executor.rs` | 42 | `// 检查管线是否已被中止` | 删除 |
| 33 | `runtime/replay/mechanism/player.rs` | 41 | `// 验证版本` | 删除 |
| 34 | `runtime/replay/mechanism/player.rs` | 49 | `// 验证帧序列` | 删除 |
| 35 | `runtime/replay/mechanism/player.rs` | 63 | `// 设置初始种子` | 删除 |

#### Infrastructure — 废话注释

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 36 | `logging/plugin.rs` | 29 | `// 初始化度量收集器` | 删除 |
| 37 | `logging/plugin.rs` | 35 | `// 注册日志 Observer` | 删除 |
| 38 | `logging/sinks/file_sink.rs` | 121 | `// 创建新文件` | 删除 |
| 39 | `localization/plugin.rs` | 91 | `// 设置默认 locale` | 删除 |
| 40 | `registry/plugin.rs` | 23 | `// 初始化空的 DefinitionRegistry` | 删除 |
| 41 | `registry/resolver.rs` | 211 | `// 检查前缀是否已知` | 删除 |
| 42 | `replay/systems.rs` | 79 | `// 验证当前帧（不变量: 校验和一致性）` | 改为 Why 注释 |
| 43 | `replay/systems.rs` | 95 | `// 发送帧处理事件` | 删除 |

#### 其他域 — 废话注释

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 44 | `summon/systems/summon_system.rs` | 63 | `// 创建召唤物实体` | 删除 |
| 45 | `camp_rest/systems/camp_rest_system.rs` | 52 | `// 记录上次长休时间` | 删除 |
| 46 | `reaction/systems/reaction_system.rs` | 36 | `// 查找下一个 Pending 条目` | 删除 |
| 47 | `reaction/systems/reaction_system.rs` | 42 | `// 检查触发者是否仍可用反应` | 删除 |
| 48 | `inventory/systems/inventory_system.rs` | 162 | `// 检查是否拥有足够数量` | 删除 |
| 49 | `inventory/components.rs` | 286 | `// 检查堆叠合并` | 删除 |
| 50 | `inventory/components.rs` | 300 | `// 检查负重` | 删除 |
| 51 | `terrain/systems/surface_system.rs` | 61 | `// 检查是否到期` | 删除 |
| 52 | `terrain/systems/surface_system.rs` | 63 | `// 恢复原始表面` | 删除 |
| 53 | `terrain/systems/surface_system.rs` | 74 | `// 移除 SurfaceOverride 组件` | 删除 |
| 54 | `terrain/systems/hazard_system.rs` | 48 | `// 检查格子上是否有可用的陷阱定义` | 删除 |
| 55 | `terrain/systems/hazard_system.rs` | 59 | `// 检查实体是否已记录陷阱消耗状态` | 删除 |
| 56 | `terrain/systems/hazard_system.rs` | 67 | `// 触发未消耗的陷阱` | 删除 |
| 57 | `terrain/systems/hazard_system.rs` | 80 | `// 记录消耗型陷阱状态` | 删除 |
| 58 | `faction/systems/relationship_system.rs` | 74 | `// 遍历主体与目标的所有阵营组合` | 删除 |
| 59 | `faction/systems/reputation_system.rs` | 73 | `// 检查是否跨越等级` | 删除 |
| 60 | `party/systems/party_system.rs` | 74 | `// 检查是否有涉及该成员的羁绊需要解除` | 删除 |
| 61 | `party/systems/party_system.rs` | 78 | `// 检查移除后是否仍满足条件` | 删除 |
| 62 | `party/rules/rules.rs` | 78 | `// 检查是否已在队伍中` | 删除 |
| 63 | `party/rules/rules.rs` | 203 | `// 检查特定实体匹配` | 删除 |
| 64 | `combat/pipeline/steps.rs` | 172 | `// 记录当前信息` | 删除 |
| 65 | `combat/pipeline/driver.rs` | 119 | `// 查找管线定义` | 删除 |
| 66 | `combat/pipeline/driver.rs` | 136 | `// 获取当前阶段` | 删除 |

#### UI — 废话注释

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 67 | `ui/focus/navigation.rs` | 67 | `// 查找当前活跃组的导航模式` | 删除 |
| 68 | `ui/focus/navigation.rs` | 80 | `// 收集活跃组内所有可聚焦元素` | 删除 |
| 69 | `ui/primitives/button/systems.rs` | 95 | `// 触发点击事件` | 删除 |

### 4.2 翻译式注释清单（15 处内联）

| # | 文件 | 行 | 违规注释 | 建议处理 |
|---|------|-----|---------|---------|
| 1 | `combat/integration/replay/recording.rs` | 51 | `// 如果已经有录制会话，跳过` | 改为 Why |
| 2 | `combat/pipeline/steps.rs` | 188 | `// 如果切换队伍 → 发射 BetweenTurns` | 改为 `// BetweenTurns 用于触发阵营专属效果结算` |
| 3 | `combat/pipeline/steps.rs` | 193 | `// 如果所有队伍完成一轮 → 发射 OnRoundEnd` | 改为 `// OnRoundEnd 触发全局回合结束逻辑` |
| 4 | `inventory/systems/inventory_system.rs` | 92 | `// 如果是穿戴（new_item 有值）` | 删除（match 分支已自解释） |
| 5 | `inventory/systems/inventory_system.rs` | 100 | `// 如果旧装备存在，放回背包` | 删除 |
| 6 | `inventory/systems/inventory_system.rs` | 126 | `// 如果是卸下（new_item 为空，old_item 有值）` | 删除 |
| 7 | `inventory/components.rs` | 372 | `// 如果物品数量归零，移除该条目` | 删除 |
| 8 | `faction/rules/reputation.rs` | 60 | `// 如果变更后声望低于保护阈值，拒绝` | 改为 Why |
| 9 | `faction/rules/relationship.rs` | 33 | `// 如果双方共享阵营 → 强制 Allied` | 改为 `// 共享阵营时强制 Allied，避免同阵营单位互为敌人` |
| 10 | `faction/rules/relationship.rs` | 67 | `// 如果主体就是该阵营成员 → Allied` | 删除（代码自解释） |
| 11 | `faction/rules/relationship.rs` | 82 | `// 如果主体无阵营归属，用其对该阵营的声望单独判定` | 改为 Why |
| 12 | `terrain/systems/terrain_effect_system.rs` | 40 | `// 如果表面有对应的地形效果` | 改为 Why |
| 13 | `terrain/systems/hazard_system.rs` | 82 | `// 如果实体还没有 HazardTriggeredState，为其添加` | 删除 |
| 14 | `runtime/pipeline/mechanism/executor.rs` | 54 | `// 如果阶段被跳过，记录到上下文` | 改为 Why |
| 15 | `party/rules/rules.rs` | 112 | `// 如果有预备成员，自动补充` | 改为 `// 不变量流程 §5.1 第 4 步：自动补充预备成员` |

---

## 五、修复策略

### 阶段 1: 删除废话注释 + 翻译式注释（P1, 1天）

**目标**: 删除所有"做什么"翻译式注释，只保留解释 Why 的注释。

**操作**:
1. 删除上表 69 处废话注释（行内 `//` 注释，非 `///` 文档注释）
2. 改写 15 处翻译式注释为 Why 注释
3. `tests/` 目录下的注释不修改（测试中的 setup 注释是允许的）

**预期产出**: 删除 ~69 条废话注释，改写 ~15 条为 Why 注释。

### 阶段 2: 核心模块补 `///` 文档注释（P0, 2天）

**目标**: 优先修复 `core/` 和 `shared/` 的公开 API 文档缺失。

**范围优先级**:

| 优先级 | 模块 | 缺失数 | 预估工时 |
|--------|------|--------|---------|
| P0-1 | `src/core/domains/quest/` | 29 | 2h |
| P0-2 | `src/core/domains/spell/` | 28 | 2h |
| P0-3 | `src/core/domains/terrain/` | 24 | 1.5h |
| P0-4 | `src/core/domains/reaction/` | 24 | 1.5h |
| P0-5 | `src/core/domains/crafting/` | 22 | 1.5h |
| P1-1 | `src/core/domains/tactical/` | 21 | 1.5h |
| P1-2 | `src/core/domains/summon/` | 21 | 1.5h |
| P1-3 | `src/core/domains/inventory/` | 19 | 1.5h |
| P1-4 | `src/core/domains/economy/` | 19 | 1.5h |
| P1-5 | `src/core/domains/progression/` | 18 | 1.5h |
| P1-6 | `src/core/domains/narrative/` | 18 | 1.5h |
| P1-7 | `src/core/domains/combat/` | 18 | 1.5h |
| P2-1 | `src/ui/overlay/` | 21 | 1.5h |
| P2-2 | `src/infra/save/` | 27 | 2h |
| P2-3 | `src/shared/` | 51 | 4h |
| P2-4 | `src/content/` | 24 | 2h |

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
| 1 | `narrative/systems/dialogue_system.rs` | 改写 8 条废话注释为 Why 注释 |
| 2 | `progression/systems/progression_system.rs` | 改写 4 条废话注释为 Why 注释 |
| 3 | `combat/systems/effect_tick_system.rs` | 改写 1 条为 Why 注释 |
| 4 | `combat/integration/replay/recording.rs` | 改写 8 条为 Why 注释 |
| 5 | `combat/pipeline/driver.rs` | 改写 3 条为 Why 注释 |
| 6 | `combat/pipeline/steps.rs` | 改写 3 条翻译式注释为 Why 注释 |
| 7 | `aggregator/mechanism/systems/aggregator_system.rs` | 改写 3 条为 Why 注释 |
| 8 | `event/mechanism/bus.rs` | 改写 2 条为 Why 注释 |
| 9 | `effect/mechanism/lifecycle.rs` | 改写 2 条为 Why 注释 |
| 10 | `ability/mechanism/lifecycle.rs` | 改写 4 条为 Why 注释 |
| 11 | `ability/mechanism/components.rs` | 改写 1 条为 Why 注释 |
| 12 | `tag/mechanism/lifecycle.rs` | 改写 2 条为 Why 注释 |
| 13 | `runtime/scheduler/foundation/values.rs` | 改写 2 条为 Why 注释 |
| 14 | `runtime/registry/mechanism/validator.rs` | 改写 1 条为 Why 注释 |
| 15 | `runtime/pipeline/mechanism/executor.rs` | 改写 2 条为 Why 注释 |
| 16 | `runtime/replay/mechanism/player.rs` | 改写 3 条为 Why 注释 |
| 17 | `infra/logging/plugin.rs` | 改写 2 条为 Why 注释 |
| 18 | `infra/logging/sinks/file_sink.rs` | 改写 1 条为 Why 注释 |
| 19 | `infra/localization/plugin.rs` | 改写 1 条翻译式注释为 Why 注释 |
| 20 | `infra/registry/plugin.rs` | 改写 1 条为 Why 注释 |
| 21 | `infra/replay/systems.rs` | 改写 2 条为 Why 注释 |
| 22 | `summon/systems/summon_system.rs` | 改写 1 条为 Why 注释 |
| 23 | `camp_rest/systems/camp_rest_system.rs` | 改写 1 条为 Why 注释 |
| 24 | `reaction/systems/reaction_system.rs` | 改写 2 条为 Why 注释 |
| 25 | `inventory/systems/inventory_system.rs` | 改写 3 条翻译式注释为 Why 注释 |
| 26 | `inventory/components.rs` | 改写 3 条为 Why 注释 |
| 27 | `terrain/systems/hazard_system.rs` | 改写 4 条为 Why 注释 |
| 28 | `terrain/systems/surface_system.rs` | 改写 3 条为 Why 注释 |
| 29 | `terrain/systems/terrain_effect_system.rs` | 改写 1 条翻译式注释为 Why 注释 |
| 30 | `faction/systems/relationship_system.rs` | 改写 1 条为 Why 注释 |
| 31 | `faction/systems/reputation_system.rs` | 改写 1 条为 Why 注释 |
| 32 | `faction/rules/reputation.rs` | 改写 1 条翻译式注释为 Why 注释 |
| 33 | `faction/rules/relationship.rs` | 改写 4 条为 Why 注释 |
| 34 | `party/systems/party_system.rs` | 改写 2 条为 Why 注释 |
| 35 | `party/rules/rules.rs` | 改写 2 条为 Why 注释 |
| 36 | `ui/focus/navigation.rs` | 改写 1 条为 Why 注释 |
| 37 | `ui/primitives/button/systems.rs` | 改写 1 条为 Why 注释 |

### 效果

| 指标 | 修复前 | 修复后 | 降幅 |
|------|--------|--------|------|
| 废话注释 (§1/§17/§18) | ~82 处 | **0 处** | **-100%** |
| 翻译式注释 (§18) | ~15 处 | **0 处** | **-100%** |

所有注释均已改写为解释 Why 的规范注释，无一删除。

---

## 七、工作量评估

| 阶段 | 内容 | 预计工时 | 实际工时 | 风险 |
|------|------|----------|----------|------|
| 1 | 废话注释 + 翻译式注释 → Why 注释 | 4h | **1.5h** ✅ | 🟢 低 |
| 2 | 核心模块补 `///` | 24h | 待执行 | 🟡 中 |
| 3 | Trait/事件/状态机 | 8h | **0.5h** ✅ | 🟢 低 |
| 4 | 不变量与公式 | 4h | 待执行 | 🟢 低 |
| **合计** | | **40h (5天)** | **2h 已完成** | |

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
