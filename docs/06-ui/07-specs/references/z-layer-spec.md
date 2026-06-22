---
id: 07-specs.z-layer-spec
title: Z-Layer Unified Specification
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - z-layer
  - overlay
  - reference
  - active
---

# Z-Layer Unified Specification

> **职责**: @presentation-architect | **上游**: `06-ui/03-screens/navigation-overlay.md` §4 (Overlay 独立层设计), `06-ui/03-screens/overlays.md` §2 (Overlay 层级), ADR-055 §6 (UI Root 分层)
> **消费端**: 每个 SSPEC 的 §11 Overlay Definition 引用此规范

---

## 1. 设计目的

### 1.1 为什么需要统一的 Z-Layer 规范

Z-Layer 是 UI 元素的垂直堆叠顺序。没有统一规范时，每个 Screen 自行定义 Z 值导致：

- 不同 Screen 的 Overlay 相互覆盖（BattleScreen 的 Tooltip 出现在 MainMenuScreen 的 Modal 上层）
- 同一 Z 值被多个元素使用，渲染顺序不确定
- 新增 Overlay 时不知道分配到哪一层

本文档定义**全局唯一的 Z-Layer 分配表**，所有 Screen、Overlay 的 Z 值从此规范查询。

### 1.2 与现有文档的关系

```
navigation-overlay.md §2.1 (UI Root 分层)
    └── 定义 5 层 UI Root (Screen/Popup/Tooltip/Notification/Debug)
        └── 每层对应一个根 Entity，Z-Layer 规范分配 Z 值
            └── 每个 SSPEC §11 引用此规范

overlays.md §2 (Overlay 层级)
    └── 定义每个 Overlay 位于哪一层
        └── Z-Layer 规范定义每层的精确 Z 值
```

---

## 2. 层级定义

### 2.1 Z 值分配策略

Z 值以 100 为步进，中间留空 99 个整数值作为子层预留：

```
step: 100
gap:  99  (子层扩展范围)
```

**子层预留**: 每个命名层在其 Z 值基础上 +1 ~ +99 可用作同一层内的子排序。例如 `screen_layer` (z=100) 内，Screen 背景 (z=101) < 内容 (z=110) < 交互元素 (z=120)。

### 2.2 七层定义

| # | 层名 | Z 值 | 用途 | 适用场景 | 子层范围 |
|---|------|------|------|---------|---------|
| 1 | `background_layer` | 0 | 页面背景、装饰元素 | Screen 壁纸、地图背景、装饰纹理 | 0-99 |
| 2 | `screen_layer` | 100 | 主 Screen 内容 | Screen 的 root 及所有子 region（Widget 树） | 100-199 |
| 3 | `tooltip_layer` | 200 | 工具提示 | 技能说明 TooltipOverlay, 物品说明 TooltipOverlay | 200-299 |
| 4 | `popup_layer` | 300 | 模态弹窗 | ModalOverlay (确认弹窗), PopupOverlay (商店/设置面板), LoadingOverlay | 300-399 |
| 5 | `notification_layer` | 400 | 通知提示 | NotificationOverlay (获得物品/升级提示/任务更新) | 400-499 |
| 6 | `debug_layer` | 500 | 调试面板 | DebugOverlay (FPS/ViewModel 状态/Entity 计数) | 500-599 |
| 7 | `topmost_layer` | 999 | 紧急覆盖 | 断开连接提示、致命错误遮罩、强制更新提示 | 900-999 |

### 2.3 各层详细说明

#### 2.3.1 background_layer (z=0)

```
用途:   Screen 背景装饰，不包含任何交互元素
z=0:    背景默认值
z=1-99: 子层预留（背景渐变叠加、装饰粒子等）

约束:
  - 不包含任何可交互 Widget
  - 不响应输入事件
  - Screen 切换时与 Screen 一起销毁/创建
```

#### 2.3.2 screen_layer (z=100)

```
用途:   Screen 主内容层，所有 Widget 树节点
z=100:  Screen root 容器
z=101:  Screen 背景层（Panel、Frame 容器）
z=110:  Screen 内容区（文本、图标、列表）
z=120:  Screen 交互元素（按钮、滑块、输入框）
z=121+: 交互子层（下拉菜单展开、内联弹窗）

约束:
  - 每个 Screen 的 Widget 树全部在此层
  - 同一 Screen 的 Widget 间子排序使用 100-199 子层范围
  - 不同 Screen 在同一 Z 层（Screen 切换时互斥，不存在叠加）
```

#### 2.3.3 tooltip_layer (z=200)

```
用途:   工具提示，显示上下文信息
z=200:  TooltipOverlay 默认值
z=201:  子层预留（多级 Tooltip 嵌套、RichTooltip 内的子提示）

约束:
  - 同一时间只显示一个 Tooltip（新 Tooltip 替换旧的）
  - Tooltip 不拦截下层交互（鼠标穿透）
  - 参见 overlays.md §2
```

#### 2.3.4 popup_layer (z=300)

```
用途:   模态弹窗，用户必须响应才能继续交互
z=300:  ModalOverlay 默认值
z=301:  LoadingOverlay（加载进度）
z=302:  ShopScreen Popup（商店界面作为弹出面板）
z=303+: 子层预留（嵌套弹窗、弹窗链）

约束:
  - 弹窗栈深度上限 3（参见 navigation-overlay.md §4.3.2）
  - 弹窗激活时阻止下层 screen_layer 的输入事件
  - 参见 overlays.md §5
```

#### 2.3.5 notification_layer (z=400)

```
用途:   非模态通知，自动消失，不影响用户当前操作
z=400:  NotificationOverlay 默认值
z=401:  BannerNotification（顶部横幅）
z=402:  ToastNotification（右下角 Toast）
z=403+: 子层预留（通知堆叠扩展）

约束:
  - 最多同时显示 3 条通知（参见 overlays.md §4.5）
  - 通知不阻止下层交互
  - 参见 overlays.md §4
```

#### 2.3.6 debug_layer (z=500)

```
用途:   开发调试信息，仅 dev feature 启用
z=500:  DebugOverlay 默认值
z=501+: 子层预留（性能面板、诊断面板、VM 检查器）

约束:
  - 仅在 dev feature 下启用
  - 不阻止下层交互（鼠标穿透）
  - 参见 overlays.md §6
```

#### 2.3.7 topmost_layer (z=999)

```
用途:   紧急覆盖，必须立即引起用户注意
z=999:  紧急覆盖默认值
z=990-998: 子层预留（多优先级紧急覆盖）

约束:
  - 仅在极端条件下使用（网络断开、致命错误、强制更新）
  - 使用前必须通过 @presentation-architect 审查
  - 阻止所有下层输入事件
```

---

## 3. 使用规则

### 3.1 通用规则

```yaml
z_layer_rules:
  R01: "所有 UI 元素在 spawn 时必须分配 Z 值，禁止使用默认 z=0"
  R02: "Z 值从 z-layer-spec.md 查询，禁止在代码中硬编码 Z 常量"
  R03: "同一层内子排序使用子层范围 (z+1 ~ z+99)"
  R04: "跨层覆盖使用命名层 Z 值，不使用中间值"
  R05: "新增层必须通过 @presentation-architect 审查并更新本规范"
  R06: "Screen 内容使用 screen_layer (z=100)，禁止使用其他层"
  R07: "Tooltip 使用 tooltip_layer (z=200)，禁止使用 screen_layer"
  R08: "Modal 使用 popup_layer (z=300)，禁止使用 screen_layer"
```

### 3.2 Overlay 层级分配

Overlay 的层分配由 `overlays.md` 定义，Z 值由本规范分配：

| Overlay | 层名 | Z 值 | 定义位置 |
|---------|------|------|---------|
| TooltipOverlay | tooltip_layer | 200 | overlays.md §2 |
| ModalOverlay | popup_layer | 300 | overlays.md §3 |
| NotificationOverlay | notification_layer | 400 | overlays.md §4 |
| LoadingOverlay | popup_layer | 301 | overlays.md §5 |
| DebugOverlay | debug_layer | 500 | overlays.md §6 |

### 3.3 例外处理

| 例外 | 说明 | 批准 |
|------|------|------|
| DamageTextOverlay | 挂在 BattleScreen 的 screen_layer 下，而非独立 Root 层 | 架构评审批准 (see architecture.md §6.6) |

DamageTextOverlay 是 INV-UI-006 的显式例外。当战斗叠加了多层 UI（如 Tooltip + Modal + DamageText）时，Z 排序为：

```
BattleScreen Widget Tree (z=100-199)
  └── DamageTextOverlay (z=105)     # 在 screen_layer 内，高于内容低于交互
TooltipOverlay (z=200)
ModalOverlay (z=300)
```

---

## 4. Overlay 之间的 Z 排序

### 4.1 多层叠加示例

```
z=999  topmost_layer        [DisconnectOverlay]
z=500  debug_layer          [DebugOverlay]
z=400  notification_layer   [NotificationOverlay]
z=300  popup_layer          [ModalOverlay]
z=301                      [LoadingOverlay]
z=200  tooltip_layer        [TooltipOverlay]
z=100  screen_layer         [BattleScreen Widget Tree]
z=105                      [DamageTextOverlay — 例外]
z=0    background_layer     [Map Background / Decoration]
```

### 4.2 覆盖规则

| 条件 | 行为 |
|------|------|
| Tooltip 显示时 Modal 弹出 | Tooltip 仍可见（Modal 在 z=300，Tooltip 在 z=200） |
| Modal 弹出时 Notification 显示 | Notification 在 Modal 上方（z=400 > z=300） |
| Debug 面板打开时任何操作 | Debug 在所有层之上（z=500 低于紧急覆盖但高于全部内容） |
| 紧急覆盖显示 | 覆盖所有层（z=999），阻止所有输入 |

---

## 5. 各 SSPEC 的引用方式

每个 SSPEC 的 §11 Overlay Definition 必须引用此规范：

```yaml
## §11 Overlay Definition
## Z-Layer 分配遵循 `07-specs/references/z-layer-spec.md`

| Overlay | 用途 | Z-Layer | 类型 |
|---------|------|---------|------|
| TooltipOverlay | 技能说明 | tooltip_layer (200) | Tooltip |
| ModalOverlay | 确认弹窗 | popup_layer (300) | Modal |
```

---

## 6. 维护规则

| 规则 | 说明 |
|------|------|
| 新增层 | 必须通过 @presentation-architect 审查，分配下一个 step=100 的 Z 值 |
| 修改 Z 值 | 不允许修改已分配层的 Z 值（破坏性变更），新层使用下一个 step |
| 废弃层 | 标记为 deprecated，保留 Z 值记录不重新分配 |
| 子层分配 | 在父层 Z 值 +1 ~ +99 范围内取用，新增子层不修改父层 Z 值 |

---

## 附录 A: 与新旧文档的映射

### A.1 与 navigation-overlay.md §2.1 的映射

| navigation-overlay.md 层名 | 本规范层名 | Z 值 |
|---------------------------|-----------|------|
| ScreenLayer | screen_layer | 100 |
| PopupLayer | popup_layer | 300 |
| TooltipLayer | tooltip_layer | 200 |
| NotificationLayer | notification_layer | 400 |
| DebugLayer | debug_layer | 500 |

### A.2 与 SSPEC 模板 §11 Z-Layer 的映射

| SSPEC 模板 §11 层编号 | 本规范层名 | Z 值 |
|----------------------|-----------|------|
| 0 (Screen 主界面层) | screen_layer | 100 |
| 1 (Tooltip 层) | tooltip_layer | 200 |
| 2 (Notification 层) | notification_layer | 400 |
| 3 (Modal 层) | popup_layer | 300 |
| 4 (Popup 层) | popup_layer | 300 |
| 9 (Debug 层) | debug_layer | 500 |

> 注意: SSPEC 模板 §11 的编号（0-9）是层内索引，非 Z 值。Z 值以本规范为准。

---

## 附录 B: 变更记录

| 日期 | 变更 | 原因 |
|------|------|------|
| 2026-06-22 | 初始创建 | 统一 Z-Layer 规范，替代各 SSPEC 自行分配 |

---

*本文档由 @presentation-architect 维护。Z-Layer 变更必须通过 Presentation Architect 审查。*
