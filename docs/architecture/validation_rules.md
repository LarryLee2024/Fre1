# Validation Rules — 全局校验与合法性守卫

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第四节 — 数值校验与合法性守卫

本文档定义游戏状态的全局校验框架、不变量约定和违规处理策略，是防止数据腐败的最后一道防线。

交叉引用：
- `docs/architecture.md` — Effect Pipeline 校验规则、属性系统不变量
- `docs/architecture/layer-contracts.md` — 层间通信校验
- `docs/domain/ui_architecture_rules.md` — UI 数据一致性校验

---

## 1. 校验检查点

### 1.1 必须执行校验的时机

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

### 2.1 属性不变量

| 不变量 | 约束 | 校验方式 | 违规处理 |
|--------|------|---------|---------|
| HP 范围 | `0 ≤ current_hp ≤ max_hp` | 每回合结束 | Clamp |
| MP 范围 | `0 ≤ current_mp ≤ max_mp` | 每回合结束 | Clamp |
| Stamina 范围 | `0 ≤ current_stamina ≤ max_stamina` | 每回合结束 | Clamp |
| MaxHP 正值 | `max_hp > 0` | 关卡加载 | 拒绝 |
| 伤害非负 | `damage ≥ 0` | 每次伤害计算 | Panic |
| 治疗非负 | `heal ≥ 0` | 每次治疗计算 | Panic |

### 2.2 战斗不变量

| 不变量 | 约束 | 校验方式 | 违规处理 |
|--------|------|---------|---------|
| Buff 数量 | `buff_count ≤ MAX_BUFFS_PER_UNIT` | 每次 Buff 施加 | 拒绝 |
| 无孤立 Modifier | 每个 Modifier 必须有 Source | 每回合结束 | Panic |
| 单位位置合法 | `position ∈ map_bounds` | 每次移动后 | Clamp |
| 技能冷却合法 | `cooldown ≤ max_cooldown` | 每回合结束 | Clamp |
| 阵营一致 | 同一单位不能同时属于多个阵营 | 关卡加载 | Panic |

### 2.3 状态不变量

| 不变量 | 约束 | 校验方式 | 违规处理 |
|--------|------|---------|---------|
| 状态机合法 | 转换路径在合法集合内 | 每次状态转换 | Panic |
| 回合队列非空 | `turn_order.len() > 0` 在 InGame 中 | 每回合开始 | Panic |
| 胜负不共存 | 不能同时满足胜利和失败条件 | 每回合结束 | 拒绝失败条件 |

---

## 3. 违规处理策略

### 3.1 三种处理方式

#### Reject（拒绝）

阻止状态变更，记录错误，保持游戏状态不变。

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

自动修正到合法值，记录警告，继续执行。

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

立即终止游戏，生成崩溃报告。

**适用场景**：
- 检测到数据损坏
- 游戏状态不可恢复
- 继续执行会导致安全问题

**示例**：
```rust
fn validate_damage(damage: i32) {
    if damage < 0 {
        panic!(
            damage = damage,
            "Negative damage detected — data corruption, cannot continue"
        );
    }
}
```

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

### 4.1 MOD 内容加载校验

MOD 内容加载时必须执行额外校验：

| 校验项 | 说明 | 处理方式 |
|--------|------|---------|
| ID 唯一性 | MOD 的 ID 不能与已有 ID 冲突 | Reject |
| 无循环引用 | 技能引用的 Buff 不能反向引用技能 | Reject |
| 无权限升级 | MOD 不能访问非 MOD API 的能力 | Reject + 卸载 MOD |
| 数值合法 | 伤害、冷却等数值在合理范围内 | Clamp |
| 格式合规 | RON 文件符合 schema 定义 | Reject |

### 4.2 MOD 校验流程

```
MOD 文件加载
  ↓
Schema 校验（格式合法性）
  ↓
ID 唯一性校验
  ↓
引用完整性校验
  ↓
数值合法性校验
  ↓
权限范围校验
  ↓
├── 全部通过 → 注册到 Registry
└── 任一失败 → 拒绝加载 + 记录 ERROR 日志
```

---

## 5. 校验实现规范

### 5.1 校验函数命名

- 🟩 校验函数以 `validate_` 开头
- 🟩 校验函数只读不写（纯函数）
- 🟩 校验函数返回 `Result<(), ValidationError>`

### 5.2 校验日志

- 🟩 校验失败日志必须包含：违规类型、当前值、期望值、Entity ID（如有）
- 🟩 Clamp 日志使用 WARN 级别
- 🟩 Reject 日志使用 ERROR 级别
- 🟩 Panic 日志使用 error! 宏 + backtrace

### 5.3 校验性能

- 🟩 校验逻辑不应成为性能瓶颈
- 🟩 Release 构建中可选择性关闭非关键校验
- 🟥 关键不变量校验（伤害非负、HP 范围）在任何构建中都必须执行

---

## 6. 校验与调试

### 6.1 Debug 构建增强校验

Debug 构建中启用额外的校验：

- 🟩 每次属性修改后校验不变量
- 🟩 每次状态转换后校验合法性
- 🟩 记录所有 Clamp/Reject 操作

### 6.2 校验失败复现

当检测到校验失败时：

1. 记录失败前的最后 N 个操作（审计日志）
2. 生成状态快照
3. 输出可复现的测试用例（如果可能）

---

## 7. 禁止事项

- 🟥 **Release 构建中跳过所有校验**（关键校验必须保留）
- 🟥 **校验失败时静默忽略**（必须记录日志）
- 🟥 **校验函数修改游戏状态**（校验是只读的）
- 🟥 **校验失败时 crash 但不生成报告**（必须生成崩溃报告）
- 🟥 **MOD 内容跳过校验**（MOD 是最高风险的数据源）
- 🟥 **手动检查代替自动校验**（关键不变量必须自动验证）
- 🟥 **校验逻辑包含业务规则**（校验只检查数值合法性，不执行游戏逻辑）

---

## 8. 实现备注

### 8.1 校验框架

```rust
pub trait Validator {
    fn validate(&self) -> ValidationResult;
}

pub enum ValidationResult {
    Valid,
    Clamp { field: &'static str, from: f32, to: f32 },
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
| `modding-system_rules.md` | MOD 内容校验规则 |
