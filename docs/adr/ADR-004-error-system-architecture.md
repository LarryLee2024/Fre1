# ADR-004: 分领域错误体系架构

## 状态

Accepted

## 背景

当前代码中存在错误处理不规范的情况：
- 部分模块使用全局统一的 AppError 大枚举（违反 §14.9.1）
- 部分业务代码使用 unwrap()、expect()（违反 §14.9.4）
- 规则失败与程序错误混用（违反 §14.9.2）

需要建立分领域错误体系，实现：
1. 每个领域定义独立错误枚举
2. 严格区分四类失败场景
3. 核心业务层禁止 Panic

## 引用的领域规则

- `docs/AI开发宪法.md` §14.9.1 — 分领域错误原则
- `docs/AI开发宪法.md` §14.9.2 — 失败分类学（强制区分）
- `docs/AI开发宪法.md` §14.9.3 — 错误强制要求
- `docs/AI开发宪法.md` §14.9.4 — 业务层 Panic 禁令

## 决策

采用「分领域错误枚举」架构：

### 错误分类（四类失败）

| 类型 | 说明 | 示例 | 实现方式 |
|------|------|------|----------|
| **RuleFailure** | 业务规则正常不满足 | 法力不足、目标超出范围 | 结果枚举，禁止用 Result::Err |
| **DomainError** | 领域内预期内异常 | 技能配置不存在、BuffID 无效 | 领域错误枚举，Result::Err |
| **InfrastructureError** | 底层通用能力异常 | 资源加载失败、存档 IO 错误 | 基础设施错误，Result::Err |
| **Bug** | 非法状态、逻辑断言失败 | 状态机非法跳转、数据一致性破坏 | panic!() 或 unreachable!() |

### 错误枚举设计

```rust
// 战斗领域错误
#[derive(Debug, thiserror::Error)]
pub enum BattleError {
    #[error("B001: 技能配置不存在: {skill_id}")]
    SkillNotFound { skill_id: SkillId },
    
    #[error("B002: 目标实体不存在: {target}")]
    TargetNotFound { target: Entity },
    
    #[error("B003: 伤害计算溢出: {damage}")]
    DamageOverflow { damage: f32 },
}

// 技能领域错误
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("S001: 法力不足: 需要 {required}, 当前 {current}")]
    InsufficientMp { required: f32, current: f32 },
    
    #[error("S002: 冷却未结束: 剩余 {turns} 回合")]
    CooldownNotReady { turns: u32 },
    
    #[error("S003: 目标超出范围: 距离 {distance}, 范围 {range}")]
    TargetOutOfRange { distance: u32, range: u32 },
}

// Buff 领域错误
#[derive(Debug, thiserror::Error)]
pub enum BuffError {
    #[error("BF001: Buff 配置不存在: {buff_id}")]
    BuffNotFound { buff_id: BuffId },
    
    #[error("BF002: Buff 实例不存在: {instance_id}")]
    InstanceNotFound { instance_id: u64 },
}
```

核心原则：
1. **分领域定义**：每个业务领域独立错误枚举
2. **错误码格式**：领域前缀+序号（如 B001、S001）
3. **完整上下文**：所有错误携带相关实体 ID、状态与参数
4. **Panic 禁令**：核心业务层禁止 unwrap/expect/panic

## Module Design

```
src/
└── core/
    └── error/
        ├── mod.rs              # 模块入口，导出通用错误类型
        ├── battle_error.rs     # 战斗领域错误
        ├── skill_error.rs      # 技能领域错误
        ├── buff_error.rs       # Buff 领域错误
        ├── inventory_error.rs  # 背包领域错误
        └── game_result.rs      # GameResult<T> 类型别名
```

### 职责划分

| 文件 | 职责 |
|------|------|
| `mod.rs` | 统一导出所有错误类型 |
| `battle_error.rs` | 战斗领域错误定义 |
| `skill_error.rs` | 技能领域错误定义 |
| `buff_error.rs` | Buff 领域错误定义 |
| `inventory_error.rs` | 背包领域错误定义 |
| `game_result.rs` | GameResult<T> = Result<T, AppError> |

## Communication Design

### Message（跨 Feature 广播）
不涉及。错误处理是模块内部逻辑。

### Observer（局部响应）
不涉及。

### Hook（组件固有行为）
不涉及。

### 函数调用
模块内部函数返回 Result<T, DomainError>。

## 边界定义

### 允许
- 每个领域定义独立错误枚举
- 使用 thiserror 派生错误
- 错误携带完整上下文信息
- 基础设施层定义 GameResult<T> 类型别名

### 禁止
- 🟥 禁止使用全局统一的 AppError 大枚举
- 🟥 禁止使用 anyhow::Error、Box<dyn Error> 作为业务层返回类型
- 🟥 禁止在核心业务层使用 unwrap()、expect()、panic!()
- 🟥 禁止仅返回无上下文的错误变体

## Forbidden（禁止事项）

- 🟥 禁止：使用全局统一的 AppError 大枚举 — 理由：错误自带领域上下文，AI 可天然识别边界（§14.9.1）
- 🟥 禁止：使用 anyhow::Error、Box<dyn Error> 作为业务层返回类型 — 理由：避免单一错误枚举无限膨胀（§14.9.1）
- 🟥 禁止：在核心业务层使用 unwrap()、expect()、panic!() — 理由：保证核心战斗逻辑的稳定性（§14.9.4）
- 🟥 禁止：仅返回无上下文的错误变体 — 理由：所有错误必须携带完整上下文信息（§14.9.3）
- 🟥 禁止：将正常业务规则不满足作为 Err 返回 — 理由：规则失败不属于程序错误（§14.9.2）

## Definition / Instance Design

### Definition（不可变配置）
不涉及。错误类型是纯数据定义。

### Instance（运行时状态）
不涉及。

## 后果

### 正面
1. **类型安全**：编译期区分不同领域错误
2. **可读性**：错误码+上下文信息直接可读
3. **可维护性**：每个领域独立演进，不会无限膨胀
4. **稳定性**：核心业务层禁止 Panic，保证运行稳定

### 负面
1. **迁移成本**：现有全局 AppError 需要逐步迁移
2. **代码量增加**：每个领域需要定义错误枚举

## 替代方案

### 方案1：使用全局 AppError 大枚举
优点：简单统一
缺点：单一枚举无限膨胀，难以维护
**结论：否决** — 违反 §14.9.1

### 方案2：使用 anyhow::Error
优点：灵活方便
缺点：失去类型信息，无法编译期检查
**结论：否决** — 违反 §14.9.1

### 方案3：使用 Box<dyn Error>
优点：通用性强
缺点：失去具体错误类型，难以处理
**结论：否决** — 违反 §14.9.1

## 架构合规性检查

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为）
- [x] 符合 Feature First 原则（core/error/ 作为独立模块）
- [x] 符合分领域错误原则（每个领域独立枚举）
- [x] 符合失败分类学（四类失败严格区分）
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs）
- [x] 所有禁止事项已明确列出
- [x] 已检查 docs/AI开发宪法.md §14.9
