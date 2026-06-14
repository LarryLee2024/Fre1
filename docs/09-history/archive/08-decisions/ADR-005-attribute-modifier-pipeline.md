---
id: history.archive.adr.ADR-005-attribute-modifier-pipeline
title: ADR-005-attribute-modifier-pipeline
status: archived
owner: architect
created: 2026-06-14
updated: 2026-06-14
---

# ADR-005: 属性与 Modifier 管线架构

## 状态

Accepted

## 背景

当前代码中属性系统存在不规范的情况：
- 部分代码直接修改最终属性值（违反 §8.0.3）
- Modifier 来源不统一，难以追踪
- 派生属性计算公式分散在多个模块

需要建立统一的属性与 Modifier 管线，实现：
1. 所有属性修改必须通过 Modifier 管线
2. Modifier 来源可追踪
3. 派生属性公式集中管理

## 引用的领域规则

- `docs/AI开发宪法.md` §8.0.1 — 属性分类强制分离
- `docs/AI开发宪法.md` §8.0.2 — 派生属性计算
- `docs/AI开发宪法.md` §8.0.3 — 修改规范（禁止直接修改最终属性值）
- `docs/AI开发宪法.md` §8.0.4 — 来源统一
- `docs/AI开发宪法.md` §8.0.5 — 公式集中管理
- `docs/AI开发宪法.md` §8.0.6 — 配置不可变

## 决策

采用「Modifier 管线」架构：

### 属性分类

| 类型 | 说明 | 示例 | 修改方式 |
|------|------|------|----------|
| **Primary Stat** | 基础属性 | Might, Vitality, Agility | Base Value |
| **Derived Stat** | 派生属性 | Attack, Defense, Speed | 公式计算 |
| **Vital Resource** | 生命资源 | HP, MP, Stamina | 直接设置 |

### Modifier 管线

```
Base Value + Σ(Modifier) = Final Value
```

Modifier 来源优先级：
1. **Base Value**：角色基础值（Definition 配置）
2. **Trait Modifier**：特质加成（最持久）
3. **Equipment Modifier**：装备加成（穿脱变化）
4. **Buff Modifier**：Buff 加成（临时）
5. **Temporary Modifier**：临时修改（一次性效果）

### Modifier 数据结构

```rust
#[derive(Debug, Clone, Copy)]
pub struct AttributeModifier {
    pub kind: AttributeKind,
    pub op: ModifierOp,
    pub value: f32,
    pub source: ModifierSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierOp {
    Add,
    Multiply,
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierSource {
    Base,
    Trait { trait_id: String },
    Equipment { instance_id: u64 },
    Buff { instance_id: u64 },
    Consumable { entity: Entity },
}
```

核心原则：
1. **禁止直接修改**：所有属性修改必须通过 Modifier 管线
2. **来源可追踪**：每个 Modifier 记录来源
3. **公式集中管理**：派生属性计算公式统一在 core/attribute 模块

## Module Design

```
src/
└── core/
    └── attribute/
        ├── mod.rs              # 模块入口，导出公共接口
        ├── attribute_kind.rs   # 属性类型枚举
        ├── attributes.rs       # 属性容器，管理 Modifier
        ├── modifier.rs         # Modifier 数据结构
        ├── derived.rs          # 派生属性计算公式
        └── formula.rs          # 属性计算公式集中管理
```

### 职责划分

| 文件 | 职责 |
|------|------|
| `mod.rs` | 统一导出所有属性类型 |
| `attribute_kind.rs` | 属性类型枚举定义 |
| `attributes.rs` | 属性容器，管理 Modifier 添加/移除/计算 |
| `modifier.rs` | Modifier 数据结构定义 |
| `derived.rs` | 派生属性计算公式 |
| `formula.rs` | 所有属性计算公式集中管理 |

## Communication Design

### Message（跨 Feature 广播）
不涉及。属性计算是模块内部逻辑。

### Observer（局部响应）
不涉及。

### Hook（组件固有行为）
不涉及。

### 函数调用
模块内部函数直接调用 Modifier 管线。

## 边界定义

### 允许
- 属性容器管理 Modifier 添加/移除
- 派生属性通过公式实时计算
- 每个 Modifier 记录来源

### 禁止
- 🟥 禁止直接修改最终属性值
- 🟥 禁止绕过 Modifier 管线修改属性
- 🟥 禁止在 UI 层修改业务属性
- 🟥 禁止分散派生属性计算公式

## Forbidden（禁止事项）

- 🟥 禁止：直接修改最终属性值 — 理由：所有属性修改必须通过 Modifier 管线（§8.0.3）
- 🟥 禁止：绕过 Modifier 管线修改属性 — 理由：Modifier 来源必须可追踪（§8.0.4）
- 🟥 禁止：在 UI 层修改业务属性 — 理由：逻辑与表现强制分离（§1.1.4）
- 🟥 禁止：分散派生属性计算公式 — 理由：所有公式必须集中管理（§8.0.5）
- 🟥 禁止：使用继承实现属性差异化 — 理由：组合绝对优先于继承（§1.1.6）

## Definition / Instance Design

### Definition（不可变配置）
- `AttributeKind`：属性类型枚举
- `ModifierOp`：修饰符操作类型
- `ModifierSource`：修饰符来源类型

### Instance（运行时状态）
- `Attributes`：属性容器，管理 Modifier 列表
- `AttributeModifier`：单个 Modifier 实例

## 后果

### 正面
1. **可追踪**：每个属性修改都有来源记录
2. **可测试**：Modifier 管线可独立测试
3. **可扩展**：新增 Modifier 来源只需添加 ModifierSource 变体
4. **可调试**：属性计算过程透明

### 负面
1. **迁移成本**：现有直接修改属性的代码需要迁移
2. **代码量增加**：需要定义 Modifier 数据结构和管线

## 替代方案

### 方案1：直接修改属性值
优点：简单直接
缺点：无法追踪来源，难以调试
**结论：否决** — 违反 §8.0.3

### 方案2：使用 Observer 监听属性变化
优点：利用 Bevy 原生能力
缺点：Observer 用于局部响应，不适合属性计算
**结论：否决** — 不适合 Modifier 管线场景

### 方案3：使用事件系统通知属性变化
优点：松耦合
缺点：事件膨胀，性能开销
**结论：否决** — 属性计算是模块内部逻辑

## 架构合规性检查

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为）
- [x] 符合 Feature First 原则（core/attribute/ 作为独立模块）
- [x] 符合 Definition/Instance 分离（AttributeKind 是 Definition，Attributes 是 Instance）
- [x] 符合 Modifier 管线统一原则
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs）
- [x] 所有禁止事项已明确列出
- [x] 已检查 docs/AI开发宪法.md §8
