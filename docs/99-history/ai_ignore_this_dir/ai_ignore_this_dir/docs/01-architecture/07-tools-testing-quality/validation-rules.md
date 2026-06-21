---
id: 01-architecture.validation-rules
title: Validation Rules
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - rules
---

# Validation Rules — 全局校验与合法性守卫

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第四节 — 数值校验与合法性守卫

本文档定义游戏状态的全局校验框架、不变量约定和违规处理策略，是防止数据腐败的最后一道防线。

交叉引用：
- `docs/01-architecture/README.md` — Effect Pipeline 校验规则、属性系统不变量
- `docs/01-architecture/00-overview/layer-contracts.md` — 层间通信校验
- `docs/03-technical/ui-architecture-rules.md` — UI 数据一致性校验

---

## 1. 校验检查点

### 1.1 必须执行校验的时机 🟥

| 检查点 | 触发条件 | 校验范围 |
|--------|---------|---------|
| 回合结束 | `OnExit(TurnPhase::TurnEnd)` | 所有单位状态合法性 |
| 战斗结束 | `OnExit(AppState::InGame)` | 全局不变量 |
| 状态转换 | 任何 `OnExit(AppState::*)` | 状态一致性 |
| 关卡加载后 | `OnEnter(AppState::InGame)` 之后 | 配置数据合法性 |
| 存档加载后 | 存档反序列化完成后 | 数据完整性 |
| MOD 加载后 | MOD 内容注册完成后 | MOD 数据合法性 |

### 1.2 校验执行模式

```
系统执行
  ↓
状态变更
  ↓
到达检查点
  ↓
执行全局校验
  ↓
├── 通过 → 继续
├── 违反（可恢复） → 修正到合法值 + WARN 日志
├── 违反（不可恢复） → 拒绝变更 + ERROR 日志
└── 违反（数据损坏） → PANIC + 崩溃报告
```

---

## 2. 全局不变量

### 2.1 属性不变量 🟥

| 不变量 | 约束 | 校验方式 | 违规处理 |
|--------|------|---------|---------|
| HP 范围 | `0 ≤ current_hp ≤ max_hp` | 每回合结束 | Clamp |
| MP 范围 | `0 ≤ current_mp ≤ max_mp` | 每回合结束 | Clamp |
| Stamina 范围 | `0 ≤ current_stamina ≤ max_stamina` | 每回合结束 | Clamp |
| MaxHP 正值 | `max_hp > 0` | 关卡加载 | 拒绝 |
| 伤害非负 | `damage ≥ 0` | 每次伤害计算 | Panic |
| 治疗非负 | `heal ≥ 0` | 每次治疗计算 | Panic |

### 2.2 战斗不变量 🟥

| 不变量 | 约束 | 校验方式 | 违规处理 |
|--------|------|---------|---------|
| Buff 数量 | `buff_count ≤ MAX_BUFFS_PER_UNIT` | 每次 Buff 施加 | 拒绝 |
| 无孤立 Modifier | 每个 Modifier 必须有 Source | 每回合结束 | Panic |
| 单位位置合法 | `position ∈ map_bounds` | 每次移动后 | Clamp |
| 技能冷却合法 | `cooldown ≤ max_cooldown` | 每回合结束 | Clamp |
| 阵营一致 | 同一单位不能同时属于多个阵营 | 关卡加载 | Panic |

### 2.3 状态不变量 🟥

| 不变量 | 约束 | 校验方式 | 违规处理 |
|--------|------|---------|---------|
| 状态机合法 | 转换路径在合法集合内 | 每次状态转换 | Panic |
| 回合队列非空 | `turn_order.len() > 0` 在 InGame 中 | 每回合开始 | Panic |
| 胜负不共存 | 不能同时满足胜利和失败条件 | 每回合结束 | 拒绝失败条件 |

---

## 3. 违规处理策略

### 3.1 三种处理方式 🟥

#### Reject（拒绝）

阻止状态变更，记录错误，保持游戏状态不变。日志级别：**ERROR**。

**适用场景**：
- 校验失败可能导致数据损坏
- 校验失败是非法操作的结果
- 无法自动修正到合法值

**示例**：
```rust
fn apply_buffValidation(buff_count: usize, max: usize) -> Result<(), BuffError> {
    if buff_count >= max {
        error!(
            current = buff_count,
            max = max,
            "Buff count exceeds maximum, rejecting"
        );
        return Err(BuffError::MaxBuffsExceeded { count: buff_count, max });
    }
    Ok(())
}
```

#### Clamp（修正）

自动修正到合法值，记录警告，继续执行。日志级别：**WARN**。

**适用场景**：
- 可以安全地修正到合法值
- 修正不会破坏游戏逻辑
- 数值偏差在可接受范围内

**示例**：
```rust
fn clamp_hp(hp: i32, max_hp: i32) -> i32 {
    let clamped = hp.clamp(0, max_hp);
    if hp != clamped {
        warn!(
            original = hp,
            clamped = clamped,
            max = max_hp,
            "HP clamped to valid range"
        );
    }
    clamped
}
```

#### Panic（崩溃）

立即终止游戏，生成崩溃报告。日志级别：**error! 宏 + backtrace**。

**适用场景**：
- 检测到数据损坏
- 游戏状态不可恢复
- 继续执行会导致安全问题

**示例**：
```rust
fn validate_damage(damage: i32) {
    if damage < 0 {
        error!(
            damage = damage,
            "Negative damage detected — data corruption, cannot continue"
        );
        // error! 宏会自动捕获堆栈信息，便于定位问题源头
        panic!("Negative damage: data corruption detected");
    }
}
```

### 3.1.1 三级违规处理决策矩阵

| 维度 | Clamp（修正） | Reject（拒绝） | Panic（崩溃） |
|------|-------------|---------------|-------------|
| **日志级别** | `warn!` | `error!` | `error!` + backtrace |
| **游戏状态** | 继续执行 | 状态回滚 | 立即终止 |
| **玩家感知** | 无感知（静默修正） | 操作被拒绝（可提示） | 游戏崩溃 |
| **典型场景** | HP/MP 越界、位置偏移 | Buff 超限、非法操作 | 伤害为负、Modifier 无来源 |
| **数据风险** | 低（可安全降级） | 中（阻止非法变更） | 高（已发生数据损坏） |
| **恢复策略** | 自动修正到合法值 | 保持原状态不变 | 生成崩溃报告 + 遥测上报 |

> **优化来源**: `docs/其他/73.md` — 三级违规处理精准化、校验失败日志分级

### 3.2 处理策略决策表

| 违规类型 | 处理方式 | 理由 |
|---------|---------|------|
| HP 超出范围 | Clamp | 可安全修正到 [0, MaxHP] |
| 伤害为负 | Panic | 数据损坏，不可恢复 |
| Buff 数量超限 | Reject | 拒绝施加新 Buff |
| 单位位置越界 | Clamp | 修正到最近合法位置 |
| 状态机非法转换 | Panic | 游戏逻辑不可恢复 |
| Modifier 无来源 | Panic | 数据损坏 |
| 配置数据非法 | Reject | 使用默认值 |

---

## 4. MOD 校验

### 4.1 MOD 内容加载校验 🟥

🟥 MOD 内容跳过任何一条校验 = 允许恶意 MOD 导致游戏崩溃或内存泄漏（宪法 12.2.2）。

MOD 内容加载时必须执行额外校验：

| 校验项 | 说明 | 处理方式 |
|--------|------|---------|
| ID 唯一性 | MOD 的 ID 不能与已有 ID 冲突 | Reject |
| 无循环引用 | 技能引用的 Buff 不能反向引用技能 | Reject |
| 无权限升级 | MOD 不能访问非 MOD API 的能力 | Reject + 卸载 MOD |
| 数值合法 | 伤害、冷却等数值在合理范围内 | Clamp |
| 格式合规 | RON 文件符合 schema 定义 | Reject |

### 4.2 MOD 校验流程（零信任五步流水线） 🟥

🟥 MOD 是最高风险的数据源，必须视为"零信任"环境。MOD 校验采用严格的五步流水线，任一步骤失败即拒绝加载（宪法 12.2.2）。

```
MOD 文件加载
  ↓
Step 1: Schema 检查（格式合法性）
  └─ RON 文件是否符合预定义 Schema？字段类型是否正确？
  ↓
Step 2: ID 唯一性校验
  └─ MOD 定义的 ID 是否与内置 ID 或其他 MOD 冲突？
  ↓
Step 3: 引用完整性校验
  └─ 技能引用的 Buff 是否存在？Buff 引用的 Modifier 是否存在？无循环引用？
  ↓
Step 4: 权限范围校验
  └─ MOD 是否访问了非 MOD API 的能力？是否越权修改内置数据？
  ↓
Step 5: 数值合法性校验
  └─ 伤害、冷却、 Buff 上限等数值是否在合理范围内？
  ↓
├── 全部通过 → 注册到 Registry
└── 任一失败 → 拒绝加载 + 记录 ERROR 日志 + 卸载 MOD
```

**零信任原则**：MOD 内容跳过任何一条校验 = 允许恶意 MOD 导致游戏崩溃或内存泄漏。五步流水线是防止第三方内容破坏游戏稳定性的绝对防线。

> **优化来源**: `docs/其他/73.md` — MOD 零信任校验流水线（Schema→ID→Ref→Permission→Numeric 五步）

---

## 5. 校验实现规范

### 5.1 校验函数命名 🟩

- 🟩 校验函数以 `validate_` 开头
- 🟩 校验函数只读不写（纯函数）
- 🟩 校验函数返回 `Result<(), ValidationError>`

### 5.1.1 校验函数纯函数铁律 🟥

🟥 **绝对禁止**：校验函数修改游戏状态（宪法 1.1.4 逻辑与表现分离）。校验函数必须是只读纯函数：

- **接收** `&World`（只读引用），绝不接收 `&mut World`
- **返回** `ValidationResult`（`Ok` / `Clamp` / `Reject` / `Panic`），绝不直接修改状态
- **无副作用**：校验函数中不调用 `World::get_mut()`、不发送 Message、不触发事件

```rust
// ✅ 正确：纯函数，只读 World，返回 ValidationResult
pub fn validate_game_state(world: &World) -> Vec<ValidationResult> {
    let mut results = Vec::new();
    for (entity, health) in world.query::<&Health>().iter(world) {
        if health.current < 0 {
            results.push(ValidationResult::Reject {
                field: "health.current",
                reason: "Negative HP",
            });
        }
    }
    results
}

// ❌ 错误：校验函数中修改状态
pub fn validate_and_fix(world: &mut World) {  // ❌ 接收 &mut World
    for (entity, mut health) in world.query::<&mut Health>().iter_mut(world) {
        health.current = health.current.max(0);  // ❌ 直接修改状态
    }
}
```

**核心价值**：将"发现问题"和"解决问题"严格分离。校验函数只负责发现违规，由调用方决定如何处理（Clamp/Reject/Panic），符合单一职责原则。

> **优化来源**: `docs/其他/73.md` — 校验函数纯函数铁律

### 5.2 校验日志 🟥

🟥 校验失败日志必须包含完整的上下文信息（宪法 13.6）。
🟥 Clamp 日志使用 WARN 级别（宪法 13.2）
🟥 Reject 日志使用 ERROR 级别（宪法 13.2）
🟥 Panic 日志使用 error! 宏 + backtrace（宪法 13.2）

### 5.3 校验性能 🟩

- 🟩 校验逻辑不应成为性能瓶颈
- 🟩 Release 构建中可选择性关闭非关键校验
- 🟥 关键不变量校验（伤害非负、HP 范围）在任何构建中都必须执行（宪法 20.1.1 质量预算）

### 5.4 Component Hooks 即时熔断

**当前痛点**：校验多在"回合结束"进行。如果某个 System 在回合中途把 HP 改成了负数，这个"非法状态"可能在回合中途污染其他系统（如 UI 显示负血条、触发错误的死亡判定），直到回合结束才发现。

**解决方案**：利用 Bevy Component Hooks (`on_mut`) 在关键组件被修改的瞬间进行微校验，实现即时熔断：

```rust
// 在 Core 层注册 Hook，Health 被修改瞬间做微校验
app.world_mut()
    .register_component_hooks::<Health>()
    .on_mut(|world, entity, _| {
        let health = world.get::<Health>(entity).unwrap();
        // 即时熔断：current_hp ∈ [0, max_hp]
        if health.current < 0 || health.current > health.max_hp {
            error!(
                entity = ?entity,
                current = health.current,
                max = health.max_hp,
                "Health invariant violated mid-turn — immediate circuit breaker"
            );
            panic!("Health invariant violated on entity {:?}", entity);
        }
    });
```

**熔断策略**：

| 组件 | 微校验规则 | 触发动作 |
|------|-----------|---------|
| `Health` | `current_hp ∈ [0, max_hp]` | Panic（数据损坏） |
| `Attributes` | 所有基础属性 ≥ 0 | Panic（数据损坏） |
| `CombatIntent` | 状态机转换合法 | Panic（逻辑错误） |
| `BuffStack` | `count ≤ MAX_STACKS` | Reject（阻止叠加） |

**关键约束**：Component Hooks 中的校验必须极其轻量（仅做范围检查），避免影响写入性能。复杂校验仍应放在检查点执行。

> **优化来源**: `docs/其他/73.md` — Component Hooks 即时熔断

---

## 6. 校验与调试

### 6.1 Debug vs Release 校验级别

Debug 构建启用全量增强校验，Release 构建保留关键不变量强制校验：

> **优化来源**: `docs/其他/73.md` — Debug vs Release 级别区分

| 校验维度 | Debug 构建 | Release 构建 |
|---------|-----------|-------------|
| 属性范围校验 | ✅ 每次修改后立即校验 | ✅ 关键不变量始终强制 |
| 状态机合法性 | ✅ 每次转换后校验 | ✅ 始终强制 |
| Buff 数量限制 | ✅ 每次施加后校验 | ✅ 始终强制 |
| 伤害非负 | ✅ 每次计算后校验 | ✅ 始终强制（Panic） |
| HP 范围 | ✅ 每次修改后校验 | ✅ 始终强制（Clamp） |
| 位置合法性 | ✅ 每次移动后校验 | ⚠️ 仅回合结束校验 |
| 技能冷却合法性 | ✅ 每次使用后校验 | ⚠️ 仅回合结束校验 |
| Modifier 来源完整性 | ✅ 每次添加后校验 | ✅ 始终强制（Panic） |
| 审计日志记录 | ✅ 所有 Clamp/Reject | ⚠️ 仅 Reject/Panic |

### 6.2 Debug 构建增强校验

Debug 构建中启用额外的校验：

- 🟩 每次属性修改后校验不变量
- 🟩 每次状态转换后校验合法性
- 🟩 记录所有 Clamp/Reject 操作
- 🟩 对比 Core 组件状态与 ViewModel 快照，若不一致则标红警告

### 6.3 校验失败复现

当检测到校验失败时：

1. 记录失败前的最后 N 个操作（审计日志）
2. 生成状态快照
3. 输出可复现的测试用例（如果可能）

---

## 7. 禁止事项 🟥

🟥 **Release 构建中跳过所有校验**（关键校验必须保留，宪法 20.1.1）
🟥 **校验失败时静默忽略**（必须记录日志，宪法 13.6）
🟥 **校验函数修改游戏状态**（校验是只读的，宪法 1.1.4）
🟥 **校验失败时 crash 但不生成报告**（必须生成崩溃报告）
🟥 **MOD 内容跳过校验**（MOD 是最高风险的数据源，宪法 12.2.2）
🟥 **手动检查代替自动校验**（关键不变量必须自动验证）
🟥 **校验逻辑包含业务规则**（校验只检查数值合法性，不执行游戏逻辑）

---

## 8. 实现备注

### 8.1 校验框架

```rust
pub trait Validator {
    fn validate(&self) -> ValidationResult;
}

pub enum ValidationResult {
    Valid,
    Clamp { field: &'static str, from: i32, to: i32 },  // 🟥 使用整数，符合宪法 2.2 数值精度
    Reject { field: &'static str, reason: &'static str },
    Panic { field: &'static str, reason: &'static str },
}

pub fn validate_game_state(world: &World) -> Vec<ValidationResult> {
    let mut results = Vec::new();
    // 遍历所有单位，检查属性不变量
    // 遍历所有 Buff，检查数量限制
    // 检查状态机合法性
    results
}
```

### 8.2 校验注册

```rust
pub struct ValidationPlugin;

impl Plugin for ValidationPlugin {
    fn build(&self, app: &mut App) {
        // 在关键检查点注册校验系统
        app.add_systems(
            OnExit(TurnPhase::TurnEnd),
            validate_turn_end_state
        );
        app.add_systems(
            OnExit(AppState::InGame),
            validate_game_end_state
        );
    }
}
```

---

## 9. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `architecture.md` | 本文档定义校验检查点和违规处理 |
| `performance_budget.md` | 校验逻辑的执行频率影响性能 |
| `infrastructure-design.md` | 审计模块记录校验失败 |
| `ui_architecture_rules.md` | UI 数据一致性校验 |
| `docs/02-domain/modding_system_rules.md` | MOD 内容校验规则 |

---

## 10. 启动时全面校验（Startup Validation） 🟥

🟥 所有配置必须支持 CI 自动校验：引用合法性、数值范围、循环依赖（宪法 12.2.2）。

> **优化来源**：`docs/其他/74借鉴.md` §21 — Blizzard 内部工具：启动时校验所有配置数据

### 10.1 为什么需要启动时校验

很多独立开发者到了后期最痛苦的问题：数据改完后不知道引用是否完整。1000 个技能引用 500 个 Buff，任何一个引用断裂都会导致运行时崩溃。

**暴雪内部工具的最佳实践**：游戏启动时遍历所有 Registry，校验技能/Buff/角色/任务的引用完整性，在问题暴露给玩家之前就捕获。

### 10.2 启动时校验范围

| 校验维度 | 说明 | 失败处理 |
|---------|------|---------|
| **技能引用完整性** | SkillDef 中引用的 EffectDef、BuffDef、RequirementDef 是否存在 | Reject + 阻止该技能注册 |
| **Buff 引用完整性** | BuffDef 中引用的 EffectDef、TriggerDef 是否存在 | Reject + 阻止该 Buff 注册 |
| **角色引用完整性** | CharacterDef 中引用的 SkillId、BuffId、EquipmentId 是否存在 | Reject + 使用默认角色 |
| **任务引用完整性** | QuestDef 中引用的前置/后续 QuestId、奖励 SkillId 是否存在 | Reject + 阻止该任务注册 |
| **Tag 引用完整性** | 所有 TagName 是否在 TagRegistry 中已定义 | Reject + 使用未分类标签 |
| **Formula 引用完整性** | EffectDef 中引用的 FormulaId 是否存在 | Reject + 使用默认公式 |
| **循环依赖检测** | Buff → Skill → Buff 是否存在循环 | Reject + 禁止加载 |

### 10.3 启动时校验流程

```
游戏启动
    ↓
Phase 1: 加载所有 Definition（AssetServer 异步加载）
    ↓
Phase 2: LoadingProgress 屏障等待全部加载完成
    ↓
Phase 3: StartupValidation 系统（一次性执行）
    ├── 遍历 SkillRegistry → 校验每个 SkillDef 的引用
    ├── 遍历 BuffRegistry → 校验每个 BuffDef 的引用
    ├── 遍历 CharacterRegistry → 校验每个 CharacterDef 的引用
    ├── 遍历 QuestRegistry → 校验每个 QuestDef 的引用
    ├── 遍历 FormulaRegistry → 校验每个 FormulaId 的引用
    ├── 交叉引用校验 → 技能引用的 Buff 存在、Buff 引用的 Effect 存在
    └── 循环依赖检测
    ↓
Phase 4: 校验报告
    ├── 全部通过 → AppState::InGame
    ├── 有 Error → 阻止进入游戏 + 输出错误报告
    └── 仅有 Warning → 允许进入 + 输出警告报告
```

### 10.4 Bevy 0.18+ 实现模式

```rust
/// 启动时校验系统 — 在所有 Registry 加载完成后一次性执行
pub fn startup_validation(
    skill_registry: Res<SkillRegistry>,
    buff_registry: Res<BuffRegistry>,
    character_registry: Res<CharacterRegistry>,
    formula_registry: Res<FormulaRegistry>,
) {
    let mut errors = Vec::new();
    
    // 校验所有技能的引用完整性
    for (id, skill_def) in skill_registry.iter() {
        for effect_ref in &skill_def.effect_refs {
            if !formula_registry.contains(effect_ref) {
                errors.push(ValidationError::MissingReference {
                    source: format!("skill:{}", id),
                    target: format!("effect:{}", effect_ref),
                });
            }
        }
        for buff_ref in &skill_def.buff_refs {
            if !buff_registry.contains(buff_ref) {
                errors.push(ValidationError::MissingReference {
                    source: format!("skill:{}", id),
                    target: format!("buff:{}", buff_ref),
                });
            }
        }
    }
    
    // 类似地校验 Buff、Character、Quest...
    
    if !errors.is_empty() {
        error!("Startup validation failed with {} errors", errors.len());
        for err in &errors {
            error!("  {:?}", err);
        }
        panic!("Startup validation failed — fix configuration errors before running");
    }
    
    info!("Startup validation passed ✓");
}
```

### 10.5 与现有校验检查点的关系

| 校验时机 | 本文档章节 | 校验内容 |
|---------|-----------|---------|
| **启动时** | §10（本节） | 配置数据引用完整性、循环依赖 |
| **关卡加载后** | §1.1 | 配置数据合法性 |
| **回合结束** | §1.1 | 运行时状态不变量 |
| **战斗结束** | §1.1 | 全局不变量 |

> 启动时校验 = 内容层面的"编译期检查"，关卡加载后校验 = 运行时数据校验，回合结束校验 = 运行时状态校验。三者互补，不替代。
