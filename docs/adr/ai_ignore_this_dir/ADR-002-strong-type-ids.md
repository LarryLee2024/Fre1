# ADR-002: 强类型标识约定

## 状态

Accepted

## 背景

当前代码中部分模块直接使用裸 `Entity` 作为业务标识跨模块传递（如 battle 模块传递 attacker/target Entity），违反宪法 §1.2.1「核心领域必须使用强类型ID」。

直接使用裸 Entity 导致：
- 编译期无法区分不同业务实体（UnitId vs SkillId）
- 日志、错误信息中输出 Entity(42u64) 无法直接定位业务对象
- 跨模块传参容易混淆（把 SkillId 传成 UnitId 编译器不报错）

## 引用的领域规则

- `docs/AI开发宪法.md` §1.2.1 — 核心领域必须使用强类型ID
- `docs/AI开发宪法.md` §1.2.2 — 对外输出必须使用可读标识

## 决策

采用「强类型 ID 包装器」模式：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuffId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(pub String);
```

核心原则：
1. **编译期类型安全**：不同业务实体使用不同类型，传参错误编译器报错
2. **可读性优先**：日志、错误信息输出字符串 ID 而非 Entity 编号
3. **String 内部存储**：内部是 String，使用时通过 .0 访问

## Module Design

```
src/
└── core/
    └── id/
        ├── mod.rs          # 模块入口，导出所有 ID 类型
        ├── unit_id.rs      # UnitId 定义
        ├── skill_id.rs     # SkillId 定义
        ├── buff_id.rs      # BuffId 定义
        └── item_id.rs      # ItemId 定义
```

### 职责划分

| 文件 | 职责 |
|------|------|
| `mod.rs` | 统一导出所有 ID 类型 |
| `unit_id.rs` | 单位标识定义与工厂方法 |
| `skill_id.rs` | 技能标识定义与工厂方法 |
| `buff_id.rs` | Buff 标识定义与工厂方法 |
| `item_id.rs` | 物品标识定义与工厂方法 |

## Communication Design

### Message（跨 Feature 广播）
不涉及。ID 类型是纯数据定义，不涉及通信。

### Observer（局部响应）
不涉及。

### Hook（组件固有行为）
不涉及。

### 函数调用
所有模块内部使用强类型 ID 进行函数调用。

## 边界定义

### 允许
- 每个业务领域定义自己的强类型 ID
- ID 内部使用 String 存储可读标识
- 使用 .0 访问内部字符串

### 禁止
- 🟥 禁止直接使用裸 Entity 作为业务标识跨模块传递
- 🟥 禁止在日志、错误信息中打印 Entity 编号
- 🟥 禁止不同业务实体共用同一个 ID 类型

## Forbidden（禁止事项）

- 🟥 禁止：直接使用裸 Entity 作为业务标识跨模块传递 — 理由：编译期无法区分不同业务实体（§1.2.1）
- 🟥 禁止：日志、错误信息中打印 Entity(xxx) 原生格式 — 理由：必须使用可读业务 ID（§1.2.2）
- 🟥 禁止：不同业务实体共用同一个 ID 类型 — 理由：类型安全是核心价值

## Definition / Instance Design

### Definition（不可变配置）
不涉及。ID 类型是纯数据定义。

### Instance（运行时状态）
- `UnitId`：单位运行时标识
- `SkillId`：技能运行时标识
- `BuffId`：Buff 运行时标识
- `ItemId`：物品运行时标识

## 后果

### 正面
1. **类型安全**：编译期防止传参错误
2. **可读性**：日志、错误信息直接可读
3. **可维护性**：半年后日志依然可定位业务对象

### 负面
1. **迁移成本**：现有裸 Entity 传递需要逐步迁移
2. **代码量增加**：每个 ID 类型需要定义结构体

## 替代方案

### 方案1：使用 newtype 模式（无 Deref）
优点：更强的类型隔离
缺点：使用时需要频繁解包，代码冗余
**结论：否决** — 过于繁琐，影响开发效率

### 方案2：使用字符串常量标识
优点：简单直接
缺点：无编译期类型检查
**结论：否决** — 失去类型安全的核心价值

## 架构合规性检查

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为）
- [x] 符合 Feature First 原则（core/id/ 作为独立模块）
- [x] 符合 Definition/Instance 分离（ID 类型是运行时数据）
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs）
- [x] 所有禁止事项已明确列出
- [x] 已检查 docs/AI开发宪法.md §1.2
