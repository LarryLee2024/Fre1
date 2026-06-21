---
id: 06-ui.README
title: UI/Presentation Architecture — Projection、ViewModel、Screen、Widget
status: in_progress
owner: presentation-architect
created: 2026-06-20
updated: 2026-06-20
tags:
  - ui
  - presentation
  - projection
  - viewmodel
  - widget
  - navigation
  - architecture
  - theme
  - localization
  - bevy-0.19
  - bsn
---

# UI/Presentation Architecture — 表现层架构总纲

> **职责**: @presentation-architect | **上游输入**: `docs/02-domain/`（领域规则）
> **核心约束**: 逻辑与表现分离，核心逻辑不依赖 UI
> **Bevy 0.19 BSN 策略**（宪法第九编）：`src/app/scenes/` ✅ / `src/ui/screens/` 🟥 / `src/ui/widgets/` 🟥，详见 `architecture.md §2.5`

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
| UI 架构设计 | Projection/ViewModel/Screen/Widget 分层方案 | `docs/06-ui/01-architecture/` |
| 设计系统 | 原子组件、复合组件、StyleToken、本地化 | `docs/06-ui/02-design-system/` |
| Screen/Overlay 设计 | 页面导航、浮层、Overlay 生命周期 | `docs/06-ui/03-screens/` |
| 数据流设计 | ViewModel/Projection 转换 | `docs/06-ui/04-data-flow/` |
| 测试策略 | Widget/Screen/Projection 测试方案 | `docs/06-ui/05-testing/` |

---

## 2. 目录结构

```
06-ui/
├── README.md                           ← 本文件（索引 + 总纲）
├── 01-architecture/
│   ├── architecture.md                 ── L3 UI 架构总纲（定位、数据流、通信、宪法规则）
│   ├── application-layer.md            ── UI 应用层（UiIntent/UiCommand/UiEvent + 完整映射链）
│   └── implementation-patterns.md      ── 实现模式（Widget/Screen/ViewModel/Overlay 的 Bevy ECS 骨架）
├── 02-design-system/
│   ├── widget-atoms.md                 ── Primitives 原语层组件契约（Button/ProgressBar/Text/Panel/List/Modal）
│   ├── widget-composites.md            ── 复合组件详细设计（Molecule/Organism，组合原语）
│   ├── theme-localization.md           ── 主题与本地化（StyleToken、Theme、UiTextKey）
│   └── focus-binding.md                ── 焦点导航与数据绑定（Focusable/FocusGroup/Dirty<T>/UiBinding）
├── 03-screens/
│   ├── screen-lifecycle.md             ── Screen 与 Widget 生命周期（生命周期状态机、Contract、组合规则，源自 screen-lifecycle.md）
│   ├── screens.md                      ── Screen 详细设计（Battle/MainMenu/Inventory/Shop/Settings/SaveLoad）
│   ├── navigation-overlay.md           ── 导航与浮层（ScreenStack、Overlay 分层、Focus 系统）
│   └── overlays.md                     ── Overlay 详细设计（Tooltip/DamageText/Notification/Loading/Debug）
├── 04-data-flow/
│   └── projection-viewmodel.md         ── Projection 与 ViewModel（防火墙、Dirty<T>、UiStore）
└── 05-testing/
    └── testing.md                      ── UI 测试策略（Widget 单元/Screen 集成/快照/Mock）
```

---

## 3. 与上游文档的关系

本目录的文档基于三个上游设计文档提取、结构化后产出。以下是文档溯源映射表：

| 本文档 | 上游 ADR-055 | 上游 Domain Rules | 上游 Data Schema | 覆盖主题 |
|--------|-------------|-------------------|-----------------|---------|
| `01-architecture/architecture.md` | §1-§3 (UI 定位), §6 (Root 分层), §10 (Plugin), §13 (Content 数据流), §14 (铁律) | §INV-UI-009 (铁律) | §1, §3, §16-§17 | L3 定位、数据流、通信、宪法规则 |
| `01-architecture/application-layer.md` | §3 (数据流), §Communication Design (通信表) | §1 (UiIntent/UiCommand/UiAction), §5.3 (用户输入流程) | §10 (UiCommand), §11 (UiAction) | UiIntent/UiCommand/UiEvent + Intent→Command→GameCommand链 |
| `02-design-system/widget-atoms.md` | §5.4 (Contract 模式), §8 (Widget 分类) | §8 (Contract 清单) | §4, §25 (WidgetFactory) | 每个 Atom Widget 的 Props/Events/State/样式/变体 |
| `02-design-system/widget-composites.md` | §5.4 (Contract 模式), §8 (Widget 分类) | §1 (Molecule/Organism 分层) | — | 复合组件（Molecule/Organism）的组成/Props/Layout/Events/State |
| `02-design-system/theme-localization.md` | §4.3-§4.4 (FontSize/FontSource), §7 (状态分级) | §1 (StyleToken/Theme), §INV-UI-007, §INV-UI-008 | §5 (StyleToken), §6 (UiSettings), §13 (ID) | StyleToken、Theme、UiTextKey、LocalizedText |
| `02-design-system/focus-binding.md` | §4.2 (Dirty<T>), §11 (UiBinding) | §1 (Focusable/FocusGroup/Dirty), §5.3 (焦点流程), §PROHIBIT-UI-011 | §8 (Focus), §9 (Dirty), §23 (UiBinding) | Focusable/FocusGroup 导航规则、Dirty<T> 消费、UiBinding |
| `03-screens/screen-lifecycle.md` | §5.4-§5.5 (Contract/Screen), §8 (Persistent), §9 (映射), §12 (WidgetFactory) | §1-§2 (术语、状态机), §5.8 (Schema), §8 (Contract 清单) | §7 (Navigation), §24 (Schema) | Screen/Widget 生命周期、Contract、组合规则 |
| `03-screens/screens.md` | §5.5 (Screen 组合), §9 (GameState 映射) | §2.1 (Screen 状态机) | §7 (Navigation) | Battle/MainMenu/Inventory/Shop/Settings/SaveLoad 详细设计 |
| `03-screens/navigation-overlay.md` | §6 (UI Root 分层), §7 (状态分级) | §1 (Overlay/Focus/ScreenStack 定义), §2.1 (Screen 状态机), §5.3-§5.7 (流程) | §7 (Navigation), §8 (Focus), §17 (Save) | ScreenStack、Overlay 分层、Focus 系统 |
| `03-screens/overlays.md` | §6 (UI Root 分层) | §5.5-§5.7 (Notification/Modal/Tooltip 流程) | §4.8-§4.9 (NotificationVm, ModalVm) | Tooltip/DamageText/Notification/Loading/Debug 详细设计 |
| `04-data-flow/projection-viewmodel.md` | §3 (数据流), §5.1 (防火墙), §11 (UiBinding), §12 (WidgetFactory), §13 (Content 数据流) | §1 (Projection 定义), §5.1 (更新流程), §6 (事件映射), §7 (ViewModel) | §2, §4, §9 (Dirty), §11 (UiAction), §23 (UiBinding), §25-§26 | Projection 纯函数、ViewModel 规范、Dirty<T>、UiStore |
| `05-testing/testing.md` | §14 (测试) | §5.9 (三层测试流程) | §15 (验证规则) | Widget 单元/Screen 集成/快照/Mock Projection/Test Fixtures |

### 3.1 引用约定

文档中使用以下方式引用上游内容：

- `ADR-055 §3` — 引用 ADR 的第 3 节
- `domain rules §5.1` — 引用领域规则文档的第 5.1 节
- `schema §4.1` — 引用数据架构文档的第 4.1 节
- `§INV-UI-001` — 引用领域规则中的不变量

### 3.2 设计完整性

三个上游文档中所有设计概念均已覆盖，未引入任何上游不存在的新设计概念。

| 上游概念 | 覆盖文档 | 状态 |
|---------|---------|------|
| L3 UI 层定位 | architecture.md §2 | 已覆盖 |
| 四层数据流 | architecture.md §4 | 已覆盖 |
| 通信机制 | architecture.md §5 | 已覆盖 |
| 宪法级规则 | architecture.md §6 | 已覆盖 |
| Plugin 注册 | architecture.md §8 | 已覆盖 |
| Screen 生命周期 | screen-lifecycle.md §2 | 已覆盖 |
| Widget 生命周期 | screen-lifecycle.md §3 | 已覆盖 |
| Widget Contract | screen-lifecycle.md §3.4 | 已覆盖 |
| GameState 映射 | screen-lifecycle.md §4 | 已覆盖 |
| Projection 防火墙 | projection-viewmodel.md §2 | 已覆盖 |
| ViewModel 规范 | projection-viewmodel.md §3 | 已覆盖 |
| Dirty<T> 机制 | projection-viewmodel.md §4 | 已覆盖 |
| UiStore 容器 | projection-viewmodel.md §5 | 已覆盖 |
| Domain Event 映射 | projection-viewmodel.md §6 | 已覆盖 |
| Content 数据流 | projection-viewmodel.md §7 | 已覆盖 |
| UiBinding 反 Marker | projection-viewmodel.md §8 | 已覆盖 |
| ScreenStack 导航 | navigation-overlay.md §3 | 已覆盖 |
| Overlay 分层 | navigation-overlay.md §4 | 已覆盖 |
| Focus 系统 | navigation-overlay.md §5 | 已覆盖 |
| Save/Persistence | navigation-overlay.md §6 | 已覆盖 |
| StyleToken 体系 | theme-localization.md §2 | 已覆盖 |
| Theme 系统 | theme-localization.md §3 | 已覆盖 |
| Localization 规范 | theme-localization.md §4 | 已覆盖 |
| UiSettings 持久化 | theme-localization.md §5 | 已覆盖 |
| UI 状态分级 | architecture.md §7 | 已覆盖 |
| Screen 状态机 (6 状态) | screen-lifecycle.md §2.2-§2.3 | 已覆盖 |
| Invariants (9 条) | 分布在各文档对应章节 | 已覆盖 |
| Forbidden (11 条) | 分布在各文档对应章节 | 已覆盖 |
| 流程定义 (11 个) | 分布在各文档对应章节 | 已覆盖 |
| UiTextKey 命名规范 | theme-localization.md §4.2 | 已覆盖 |
| UiSettings 验证规则 | theme-localization.md §5.3 | 已覆盖 |
| Reflect 注册要求 | projection-viewmodel.md §4.3 | 已覆盖 |
| Schema 治理 | screen-lifecycle.md §3.4.3 | 已覆盖 |
| WidgetFactory | architecture.md §9 | 已覆盖 |
| Widget Contract 详细设计（每个 Widget 的 Props/Events/State） | widget-atoms.md | 已覆盖 |
| Screen 详细设计（Layout/ViewModel/Command 完整清单） | screens.md | 已覆盖 |
| Overlay 详细设计（层级/数据源/生命周期/交互） | overlays.md | 已覆盖 |
| UiIntent → UiCommand → GameCommand 映射链 | application-layer.md | 已覆盖 |
| UiAction 与 UiEvent 定义 | application-layer.md | 已覆盖 |
| Focusable/FocusGroup 导航规则 | focus-binding.md §2 | 已覆盖 |
| Dirty<T> 消费流程与 UiBinding 反 Marker | focus-binding.md §3-§4 | 已覆盖 |
| Widget 单元测试/Screen 集成/快照/Mock | testing.md | 已覆盖 |

## 4. 架构层次概述

（各层次完整设计参见对应文档）

| 层次 | 职责 | 对应文档 |
|------|------|---------|
| **Projection** | Domain → ViewModel 的纯函数转换，防火墙层 | `04-data-flow/projection-viewmodel.md §2` |
| **ViewModel** | UI 状态的投影，Widget 的唯一数据源 | `04-data-flow/projection-viewmodel.md §3` |
| **Screen** | 页面级容器，组合 Widget/Organism，与 GameState 对应 | `03-screens/screen-lifecycle.md §2`, `03-screens/screens.md` |
| **Primitives** | 可复用的最小 UI 元素，独立 Plugin，有明确 Contract。**唯一允许直接操作 Bevy UI 底层实现的层** | `03-screens/screen-lifecycle.md §3`, `02-design-system/widget-atoms.md` |
| **Widget Composite** | Molecule（3-5 原语）/ Organism（多个 Molecule + 原语）组合，关联 ViewModel | `02-design-system/widget-composites.md` |
| **Application** | UiIntent/UiCommand/UiEvent 输入意图与命令通道 | `01-architecture/application-layer.md` |
| **Navigation** | ScreenStack 管理 push/pop/replace | `03-screens/navigation-overlay.md §3` |
| **Overlay** | Tooltip/Notification/Modal 等独立浮层 | `03-screens/navigation-overlay.md §4`, `03-screens/overlays.md` |
| **Focus** | 键盘/手柄焦点导航 | `03-screens/navigation-overlay.md §5`, `02-design-system/focus-binding.md §2` |
| **Binding** | ViewModel → Widget 数据绑定（Dirty<T>/UiBinding） | `02-design-system/focus-binding.md §3-§4` |
| **StyleToken** | 语义化颜色/间距/字体引用 | `02-design-system/theme-localization.md §2` |
| **Theme** | 主题定义，聚合所有 StyleToken | `02-design-system/theme-localization.md §3` |
| **Localization** | UiTextKey + LocalizedText 国际化 | `02-design-system/theme-localization.md §4` |
| **Testing** | Widget 单元/Screen 集成/快照/Mock Projection | `05-testing/testing.md` |

## 5. 通信机制

### 5.1 UI → Domain

```
User Input → UiIntent → UiAction → UiCommand → GameCommand → Domain
```

详见 `architecture.md §5`。

### 5.2 Domain → UI

```
Domain Event → Observer → Projection → ViewModel → Widget 刷新
```

详见 `architecture.md §5`。

### 5.3 禁止事项

- 🟥 禁止 UI Widget 直接修改 Component
- 🟥 禁止 Domain System 直接操作 UI 节点
- 🟥 禁止 Event/Message 传递 UI 专用数据

## 6. 文件状态

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `README.md` | 🟡 in_progress | presentation-architect | 2026-06-21 |
| `01-architecture/architecture.md` | 🟡 需更新目录结构 | presentation-architect | 2026-06-21 |
| `01-architecture/application-layer.md` | 🟡 需代码对齐 | presentation-architect | 2026-06-21 |
| `01-architecture/implementation-patterns.md` | ✅ 完整 | presentation-architect | 2026-06-21 |
| `02-design-system/widget-atoms.md` | ✅ 完整 | presentation-architect | 2026-06-20 |
| `02-design-system/widget-composites.md` | 🟡 需补全复合组件 | presentation-architect | 2026-06-20 |
| `02-design-system/theme-localization.md` | ✅ 完整 | presentation-architect | 2026-06-20 |
| `02-design-system/focus-binding.md` | 🟡 需补全 FocusNavigation | presentation-architect | 2026-06-20 |
| `03-screens/screen-lifecycle.md` | 🟡 需更新 Screen 列表 | presentation-architect | 2026-06-20 |
| `03-screens/screens.md` | 🟡 需补全 Inventory 等 | presentation-architect | 2026-06-20 |
| `03-screens/navigation-overlay.md` | 🟡 需补全 ScreenStack 细节 | presentation-architect | 2026-06-20 |
| `03-screens/overlays.md` | 🟡 需补全 Overlay 规格 | presentation-architect | 2026-06-20 |
| `04-data-flow/projection-viewmodel.md` | 🟡 需补全 ViewModel 字段 | presentation-architect | 2026-06-20 |
| `05-testing/testing.md` | 🟡 需补全测试规格 | presentation-architect | 2026-06-20 |
