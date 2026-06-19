---
id: 06-ui.README
title: UI/Presentation Architecture — Projection、ViewModel、Screen、Widget
status: draft
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - presentation
  - projection
  - viewmodel
  - widget
  - navigation
---

# UI/Presentation Architecture — 表现层架构总纲

> **职责**: @presentation-architect | **上游输入**: `docs/02-domain/`（领域规则）
> **核心约束**: 逻辑与表现分离，核心逻辑不依赖 UI

本文档是 Fre 项目 UI/表现层架构的索引和规范。Presentation 层是 DDD 三层中的最高层（L3），负责将领域状态投影为 UI 可消费的 ViewModel，并管理屏幕、组件、导航。

---

## 1. 架构定位

```
Domain Layer (领域层)             ── 纯业务逻辑，零 UI 依赖
      │
      ▼
Projection (投影层)               ── Domain State → ViewModel 的转换
      │
      ▼
ViewModel (视图模型层)             ── UI 状态持有，Screen + Widget 消费
      │
      ▼
Screen / Widget (表现层)          ── Bevy UI 渲染
```

### 1.1 核心原则

- **逻辑与表现分离** — 核心逻辑不依赖 UI，Domain 不知道 Screen/Widget 的存在
- **单向数据流** — Domain → Projection → ViewModel → Widget（禁止反向）
- **Projection 防火墙** — Widget 只能消费 ViewModel，不能直接访问 Domain 数据
- **Widget Contract** — 每个 Widget 定义明确的输入（Props）、输出（Events）、状态

### 1.2 Presentation Architect 职责

| 职责 | 输出 | 对应目录 |
|------|------|---------|
| UI 架构设计 | Projection/ViewModel/Screen/Widget 分层方案 | `docs/06-ui/` |
| Navigation 设计 | 屏幕导航、Scene 转换 | `docs/06-ui/navigation.md` |
| ViewModel 设计 | 视图模型结构 | `docs/06-ui/viewmodel.md` |
| Widget 设计 | 可复用组件契约 | `docs/06-ui/widgets/` |
| Projection 设计 | Domain → ViewModel 转换模式 | `docs/06-ui/projection.md` |
| UI State 管理 | UI 本地状态的管理模式 | `docs/06-ui/ui-state.md` |

---

## 2. 目录结构

```
06-ui/
├── README.md                     ← 本文件（索引）
├── architecture.md               ── UI 整体架构（Projection/ViewModel/Screen/Widget）
├── navigation.md                 ── 导航与 Screen 生命周期
├── projection.md                 ── Domain → ViewModel 投影规则
├── viewmodel.md                  ── ViewModel 结构规范
├── ui-state.md                   ── UI 本地状态管理
├── widgets/                      ── Widget 组件定义
│   ├── widget-contract.md        ── Widget 契约标准
│   ├── hp-bar.md                 ── HP 条组件
│   ├── action-bar.md             ── 行动栏组件
│   ├── inventory-panel.md        ── 背包面板组件
│   ├── dialogue-box.md           ── 对话组件
│   └── party-panel.md            ── 队伍面板组件
└── screens/                      ── Screen 定义
    ├── battle-screen.md          ── 战斗界面
    ├── map-screen.md             ── 地图界面
    ├── camp-screen.md            ── 营地界面
    ├── menu-screen.md            ── 菜单界面
    └── dialogue-screen.md        ── 对话界面
```

---

## 3. 架构层次

### 3.1 Projection（投影层）

领域状态 → ViewModel 的纯函数转换：

```rust
pub fn project_unit_status(unit: &Unit) -> UnitStatusVM {
    UnitStatusVM {
        name_key: unit.name_key.clone(),
        hp_percent: unit.hp as f32 / unit.max_hp as f32,
        has_action: unit.action_points > 0,
    }
}
```

- 🟩 必须是纯函数（无副作用、无 System 依赖）
- 🟩 每个 Domain 应有对应的 Projection 模块
- 🟥 禁止在 Projection 中修改 Domain 状态

### 3.2 ViewModel（视图模型）

Widget 消费的数据结构：

```rust
#[derive(Clone)]
pub struct UnitStatusVM {
    pub name_key: LocalizationKey,
    pub hp_percent: f32,
    pub has_action: bool,
}
```

- 🟩 ViewModel 只包含 Widget 需要的数据
- 🟩 ViewModel 不含业务方法
- 🟥 禁止 ViewModel 直接引用 Domain 类型

### 3.3 Screen（屏幕）

顶级 UI 容器，管理一组 Widget：

- **生命周期**: OnEnter → Setup → Widgets ← Input → OnExit
- **导航**: Screen Stack（支持 Push/Pop）
- **状态**: 每个 Screen 持有自己的 UI 状态

### 3.4 Widget（组件）

可复用 UI 组件的契约标准：

```rust
pub struct HpBarProps {
    pub current: u32,
    pub max: u32,
}

pub enum HpBarEvent {
    Hovered,
    Clicked,
}
```

- 🟩 Widget = Props（输入）+ Events（输出）+ State（本地）
- 🟥 禁止 Widget 直接查询 ECS World

---

## 4. 通信机制

### 4.1 UI → Domain

用户操作通过 Input System → Commands → Domain：

```
User Input → InputCollector → Command → Domain System
```

### 4.2 Domain → UI

领域状态通过 Observer → Projection → ViewModel：

```
Domain Change → Observer → Projection → ViewModel → Widget更新
```

### 4.3 禁止事项

- 🟥 禁止 UI Widget 直接修改 Component
- 🟥 禁止 Domain System 直接操作 UI 节点
- 🟥 禁止 Event/Message 传递 UI 专用数据

---

## 5. 文件状态

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `README.md` | 🟡 draft | presentation-architect | 2026-06-20 |

---

*本文档由 @presentation-architect 维护。所有 UI 架构变更需经过 Presentation Architect 审查。*
