# UI 架构领域

Version: 1.1
Status: Proposed

UI 架构领域管理游戏逻辑（Core）与视觉表现（UI）之间的分离规则、ViewModel 模式和单向数据流。

核心原则：
- 逻辑/表现强制分离：Core 不知道 UI 的存在，UI 不直接访问 Core 的 ECS 组件
- ViewModel 是 UI 渲染的唯一真相源：Core 通过系统更新 ViewModel，UI 只读 ViewModel
- UI 意图通过 UiCommand 单向传递：UI 不直接修改任何游戏状态

## 宪法合规矩子

| 条款 | 级别 | 落地规则 |
|------|------|----------|
| 1.1.4 逻辑/表现强制分离 | 🟥 | Core 禁止引用 UI 类型，UI 禁止直接操作 Core ECS |
| 1.3.2 依赖方向铁则 | 🟥 | 表现层→应用层→领域层，Core 不依赖 bevy_ui crate |
| 10.0.1 bevy_ui + bevy_egui | 🟩 | 产品 UI 用 bevy_ui，调试工具用 bevy_egui |
| 10.0.2 状态单向流动 | 🟥 | UI 只展示状态，禁止保存业务真相；业务逻辑禁止操作 UI 组件 |
| 10.0.3 UI 独立 Feature | 🟩 | UI 作为独立 Feature 模块存在 |
| 10.0.7 临时状态隔离 | 🟥 | 选中单位/悬停格子等 UI 临时状态不混入业务事实，不参与存档/回放 |
| 11.7 CQRS Lite | 🟩 | 读路径无副作用，写路径收口到 command_handler |
| 22.8 分层架构 | 🟥 | Core→UI 依赖方向，禁止反向依赖 |

---

# 术语定义

## ViewModel（视图模型）

Core 系统维护的 Bevy Resource，包含 UI 渲染所需的全部显示数据。Core 通过系统从 ECS 组件提取数据并填充 ViewModel，UI 只读 ViewModel 进行渲染。

不是 Component。不是 Entity。不是 Core 的模型数据本身。

关键属性：
- 是 Bevy Resource，挂载在全局，不是 Entity 上的 Component
- 由 Core 层的系统（如 `update_selected_unit_view`）写入
- 由 UI 层的系统只读消费
- 只包含渲染需要的扁平化数据，不包含游戏逻辑数据
- 一个 ViewModel 对应一种 UI 展示功能（如 `SelectedUnitView` 对应单位信息面板）

---

## UiCommand（UI 命令）

UI 层向 Core 层发送的 Message，表达用户操作意图。UiCommand 是 Bevy Message 类型，通过 `commands.write_message()` 发送，由 `command_handler` 消费执行。

不是直接函数调用。不是 ECS 组件修改。不是 ViewModel 更新。

关键属性：
- 是 Bevy Message（`#[derive(Message)]`）
- 由 UI 交互系统发送（点击、按键等）
- 由 `command_handler.rs` 中的 `handle_ui_commands` 系统统一消费
- 只携带意图数据（Entity ID、坐标、技能 ID 等），不携带游戏状态
- 一次只处理一个用户意图，禁止在 UI 系统中直接处理多个命令

---

## 逻辑/表现分离（Logic/Presentation Separation）

Core 系统只处理游戏逻辑（伤害计算、Buff 结算、回合管理等），UI 系统只处理表现逻辑（面板渲染、飘字、音效等），两者通过 ViewModel 和 Message 单向通信，禁止互相直接调用。

不是"UI 直接读 ECS 组件获取数据"。不是"Core 推送显示数据给 UI 控件"。

关键属性：
- Core 系统中禁止引用任何 UI 模块的类型
- UI 系统中禁止直接查询 Core 层的 ECS 组件（必须通过 ViewModel）
- Core 事件通过 Message 广播，UI 的 Observer 或系统响应这些 Message
- 表现层只负责将数据展示为视觉元素

---

## UI 层

表现层的一部分，负责游戏界面的渲染、用户交互和视觉反馈。位于 `src/ui/` 目录，是跨层模块，依赖 ViewModel only。

不是 Core 层。不是 Infrastructure 层。不是 App 层。

关键属性：
- 目录路径：`src/ui/`
- 只依赖 ViewModel（Bevy Resource）和 UiCommand（Bevy Message）
- 不直接 use Core、Infrastructure 等层的模块类型
- 包含面板、组件、主题、VFX、屏幕等表现子模块

---

## View Query（视图查询）

UI 系统从 ViewModel Resource 中读取显示数据的模式。通过 `Res<SelectedUnitView>` 等 Resource 查询获取数据，不通过 `Query<(&Attributes, &Unit, ...)>` 直接访问 ECS 组件。

不是直接 ECS Query。不是 World 访问。

关键属性：
- UI 系统参数中使用 `Res<XxxView>` 读取 ViewModel
- 禁止使用 `Query<(&Attributes, ...)>` 等直接查询 Core 组件
- ViewModel 是扁平化的字符串和数值，不包含 Entity 引用或游戏对象
- View Query 是只读操作

---

## View Binding（视图绑定）

ViewModel 字段到 UI 控件的连接方式。UI 系统从 ViewModel 读取字段值，创建或更新 UI 节点。是单向绑定：ViewModel 变化时 UI 重新渲染，UI 操作不反向修改 ViewModel。

不是直接组件绑定（Entity ↔ Widget）。不是双向数据绑定。

关键属性：
- 单向绑定：ViewModel → UI 控件
- 通过 `ResChanged<SelectedUnitView>` 检测变化触发重绘
- UI 控件只展示数据，不存储游戏状态
- 一个 ViewModel 字段可以对应多个 UI 控件的显示

---

## UI State（UI 状态）

只存在于 UI 层的本地状态，如选中的单位 Entity、鼠标悬停的格子、面板的可见性。这些状态不影响游戏逻辑，只影响 UI 表现。

🟥 **10.0.7 临时状态隔离**：UI 交互状态（选中单位、悬停格子、技能预览）属于表现层临时状态，绝对禁止混入业务事实状态，不参与存档、不进入回放。

不是游戏状态。不是 Core 层管理的状态。

关键属性：
- 🟥 存储在 UI 层的 Resource 或 Component 中（如 `HoveredEntity`、`UiFocusState`、`BlocksGameInput`）
- 🟥 只被 UI 系统读写
- 🟥 Core 层禁止读写 UI State
- 🟥 重置或丢失不会影响游戏逻辑正确性
- 🟥 不参与 Battle Replay（回放不依赖 UI 状态）

---

## Notification（通知）

Core 层产生的游戏事件，通过 Bevy Message 广播给 UI 层，UI 层响应并显示反馈。如 `DamageApplied`（伤害飘字）、`HealApplied`（治疗日志）、`CharacterDied`（击败提示）。

不是请求。不是命令。不是双向通信。

关键属性：
- 由 Core 系统通过 `commands.write_message()` 发送
- 由 UI 层的系统通过 `MessageReader<T>` 读取
- 是 Core → UI 的单向通信
- UI 只负责展示通知内容，不修改游戏状态
- Notification 被消费后即消失，不持久化

---

## CQRS（命令查询职责分离） [宪法 11.7 🟩]

🟥 命令查询职责分离是本架构的灵魂。在 Bevy ECS 中，读写分离通过 Resource 和 Message 天然实现。

不是直接函数调用。不是 ECS 组件修改。不是 ViewModel 更新。

关键属性：
- 🟥 读路径（Query）：UI 系统只从 ViewModel Resource 读取，绝不直接查询 ECS 组件
- 🟥 写路径（Message）：UI 系统只发送 UiCommand Message，绝不直接修改 ECS
- 🟥 Core 处理（唯一入口）：CommandHandler 是所有用户意图的唯一执行点
- 🟩 CommandHandler 唯一入口使游戏逻辑触发点完全可追踪（方便日志和回放）
- 🟩 为未来网络同步（UiCommand 直接发给服务器）铺平道路

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

## 表现层隔离（Presentation Layer Isolation） [宪法 10.0.2 🟥]

🟥 UI 层必须 NOT 直接修改领域组件（Core ECS 组件），只能通过 UiCommand Message 传递意图。业务逻辑禁止直接操作 UI 组件。

不是"UI 直接读 ECS 组件获取数据"。不是"Core 推送显示数据给 UI 控件"。

关键属性：
- 🟥 UI 层禁止 `commands.entity(e).insert(...)` 创建游戏逻辑 Entity
- 🟥 UI 层禁止 `commands.entity(e).remove::<...>()` 直接移除组件
- 🟥 UI 层禁止 `next_state.set(...)` 直接设置状态
- 🟥 UI 层禁止直接调用 Core 系统函数
- 🟥 Core 系统禁止直接操作 UI 组件（如 `spawn_vfx()`、`play_sfx()`）
- 🟩 所有用户操作必须经过 command_handler 处理

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

# 领域边界

## 本领域负责

- ViewModel 的定义和维护规则
- UiCommand 的定义和分发规则
- UI 层的文件组织和模块边界
- View Query 的访问模式
- View Binding 的单向绑定规则
- UI State 的本地管理规则
- 模态面板的输入焦点管理（`BlocksGameInput`）
- UI 焦点状态（`UiFocusState`）的维护
- 主题系统（`UiTheme`）的统一样式管理
- UI Notification 的消费和展示规则

## 本领域不负责

- Core 层的游戏逻辑（由 Battle、Skill、Buff 等领域负责）
- ViewModel 数据的来源逻辑（由各 Core 模块通过系统填充）
- Message 的定义和广播规则（由 ECS Communication 领域负责）
- Layer 依赖方向的判定（由 Layer Architecture 领域负责）
- 具体面板的布局设计（由各 UI 模块内部决定）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 用户操作意图 | UiCommand（Message） | Core（通过 command_handler） |
| 游戏事件反馈 | Notification（Message） | UI（展示飘字/日志） |
| ViewModel 数据更新 | Resource 变更 | UI（只读消费） |
| UI 焦点状态变更 | Resource 变更 | Input（读取 `UiFocusState`） |
| 模态面板打开/关闭 | `BlocksGameInput` 标记 | Input（读取焦点状态） |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Idle | UI 等待用户输入 | ProcessingCommand |
| ProcessingCommand | command_handler 正在处理 UiCommand | Idle, DisplayingFeedback |
| DisplayingFeedback | 展示 Core 事件的视觉反馈 | Idle |

## 状态转换图

```
Idle → ProcessingCommand → Idle
                ↓
         DisplayingFeedback → Idle
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Idle | ProcessingCommand | 收到 UiCommand Message |
| ProcessingCommand | Idle | 命令处理完成，游戏状态已更新 |
| ProcessingCommand | DisplayingFeedback | 命令触发了 Core 事件（如 DamageApplied） |
| DisplayingFeedback | Idle | 反馈动画/日志写入完成 |

---

# 不变量

## 不变量1：UI 不直接查询 Core ECS 组件 [宪法 10.0.2 🟥]

任意时刻：

🟥 UI 层的所有系统参数中，禁止出现 `Query<(&Attributes, &Unit, ...)>` 等直接查询 Core 组件的形式。UI 只能通过 `Res<SelectedUnitView>` 等 ViewModel Resource 获取显示数据。

违反表现：

UI 系统函数签名中出现 `Query<(&Attributes, ...)>` 或 `Query<(&ActiveBuffs, ...)>`。

---

## 不变量2：Core 不引用 UI 类型 [宪法 1.3.2 🟥]

任意时刻：

🟥 Core 层的 `use` 语句中不出现 `crate::ui::` 路径。Core 系统不知道 UI 模块的存在。

违反表现：

`core/` 目录下的文件中出现 `use crate::ui::UiTheme`、`use crate::ui::SelectedUnitView` 等语句。

---

## 不变量3：ViewModel 是 UI 渲染的唯一真相源 [宪法 10.0.2 🟥]

任意时刻：

🟥 UI 面板和组件显示的所有游戏数据必须来自 ViewModel Resource。不存在 UI 直接从 ECS 组件读取数据的路径。

违反表现：

UI 系统绕过 ViewModel 直接使用 `Res<Attributes>` 或 `Query<&SkillSlots>` 获取数据。

---

## 不变量4：UI State 存活于 UI 层 [宪法 10.0.7 🟥]

任意时刻：

🟥 UI 本地状态（悬停实体、焦点状态、面板可见性）只存储在 UI 层的 Resource 或 Component 中。Core 层禁止读写 UI State。UI 临时状态不参与存档、不进入回放。

违反表现：

Core 系统读取 `Res<HoveredEntity>` 或 `Res<UiFocusState>`；UI State 被写入存档文件。

---

## 不变量5：所有用户操作通过 UiCommand 传递

任意时刻：

用户的所有游戏操作（选中单位、移动、攻击、使用技能、结束回合）必须通过发送 UiCommand Message 实现。禁止在 UI 系统中直接修改 ECS 组件。

违反表现：

UI 系统中出现 `commands.entity(e).insert(Selected)` 而不通过 UiCommand（注意：`command_handler` 中的 `insert(Selected)` 是 Core 行为，由 command_handler 在处理命令时执行，不是 UI 系统直接执行）。

---

## 不变量6：Core 模块不依赖 bevy_ui crate

任意时刻：

Core 模块的 Cargo.toml 不依赖 bevy_ui crate。这是编译期可验证的物理隔离。Core 层成为纯粹的规则引擎，可零成本剥离 UI 运行 Dedicated Server 或进行 Headless 自动化测试。

违反表现：

Core 模块的 Cargo.toml 中出现 bevy_ui 依赖；或 Core 模块的 use 语句中出现 crate::ui:: 路径。

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

## 不变量7：ViewModel 禁止包含 Entity 引用

任意时刻：

ViewModel 字段类型仅限 String、f32、u32、i32、bool、enum 等基础类型。禁止使用 Entity、Vec<Entity>、HashMap<Entity, ...> 等 Bevy ECS 类型。ECS 中 Entity ID 会被回收复用（Generation 机制），缓存 Entity 会导致脏数据。

违反表现：

ViewModel 中出现 `entity: Entity` 字段；或 `buffs: Vec<ActiveBuff>` 包含 ECS 组件引用。

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

## 不变量8：ViewModel 安全铁律

任意时刻：

ViewModel 必须遵守三条铁律：（1）禁止包含 Entity 引用；（2）只允许基础类型，无 Bevy 依赖；（3）只表达"展示什么"，不表达"为什么这样"。ViewModel 是纯展示快照，不含任何业务语义。

违反表现：

ViewModel 中出现 `can_attack: bool`（业务判断结果）或 `reason_cannot_attack: String`（Core 的解释）。

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

# 规则

## 规则1：UI 只读 ViewModel

允许：
- UI 系统通过 `Res<SelectedUnitView>` 等 Resource 读取显示数据
- UI 系统通过 `ResChanged<T>` 检测 ViewModel 变化触发重绘
- UI 系统使用 ViewModel 中的字符串和数值创建 UI 控件

禁止：
- UI 系统通过 `Query<(&Attributes, ...)>` 直接查询 Core 组件
- UI 系统通过 `Res<Attributes>` 等直接读取 Core Resource
- UI 系统在 `World` 中直接访问游戏实体

必须：
- UI 系统的 `update_*_view` 函数在 ViewModel 的 `Changed` 检测失败时提前返回
- UI 只消费 ViewModel，不产生或修改 ViewModel

---

## 规则2：UI 只输出 UiCommand

允许：
- UI 交互系统（点击、按键）发送 UiCommand Message
- command_handler 在 InGame 状态且玩家回合时消费 UiCommand
- UiCommand 携带 Entity ID、坐标、技能 ID 等意图数据

禁止：
- UI 系统直接修改 ECS 组件（如 `commands.entity(e).insert(CombatIntent{...})`）
- UI 系统直接调用 Core 系统函数
- UI 系统绕过 UiCommand 直接设置 `NextState<TurnPhase>`

必须：
- UiCommand 的所有变体必须在 `ui/events.rs` 中定义
- 所有 UiCommand 消费逻辑集中在 `command_handler.rs`
- command_handler 只在 `player_turn` 条件下运行

---

## 规则3：Core 通过系统更新 ViewModel

允许：
- Core 层的 `update_*_view` 系统在 `Update` 阶段运行
- 系统从 ECS 组件读取数据，填充到 ViewModel Resource
- 系统使用 `ResChanged<T>` 或 `is_changed()` 优化跳过无变化的更新

禁止：
- Core 系统直接操作 UI 控件
- Core 系统引用任何 UI 模块的类型
- ViewModel 更新系统包含游戏逻辑（只做数据提取和格式化）

必须：
- 每种 ViewModel 有对应的 `update_*_view` 系统
- ViewModel 更新在 `Update` 阶段执行（不是 `FixedUpdate`）
- ViewModel 是 Bevy Resource（`#[derive(Resource)]`），不是 Component

---

## 规则4：Notification 单向消费

允许：
- Core 事件（DamageApplied、HealApplied 等）通过 Message 广播
- UI 系统通过 `MessageReader<T>` 消费 Notification
- UI 系统将 Notification 转化为视觉反馈（飘字、日志条目）

禁止：
- UI 系统在消费 Notification 时修改游戏状态
- Core 系统在发送 Notification 时包含 UI 特定数据（如颜色、字号）
- Notification 被多个 UI 系统重复消费后产生副作用

必须：
- Notification 消费系统只执行表现逻辑（写日志、生成飘字 Entity）
- Notification 消费系统在 `Update` 阶段运行
- Notification 的数据由 Core 负责填充，UI 负责展示

---

## 规则5：UI State 本地管理

允许：
- UI 层维护 `HoveredEntity`、`UiFocusState` 等本地状态 Resource
- UI 层使用 `BlocksGameInput` 标记组件控制模态面板的输入阻止
- `update_ui_focus_state` 系统自动同步焦点状态

禁止：
- Core 系统读取 UI State Resource
- UI State 丢失影响游戏逻辑
- 在 Core 层定义 UI State 类型

必须：
- UI State 类型定义在 `src/ui/` 目录
- UI State 默认值对游戏无副作用
- 模态面板必须标记 `BlocksGameInput`

---

## 规则6：主题统一样式管理

允许：
- 所有 UI 样式常量集中在 `UiTheme` Resource 中
- 颜色、字号、间距等通过 `UiTheme` 访问
- 换肤只需修改 `UiTheme` 默认值

禁止：
- 在 UI 面板代码中硬编码颜色、字号、间距值
- 不同面板使用不同的样式常量

必须：
- UI 控件的颜色通过 `Res<UiTheme>` 获取
- `faction_color` 等辅助函数统一提供阵营颜色

---

## 规则7：服务器模式兼容

允许：
- Core + Infrastructure 在无 UI 环境下独立运行（服务器模式）
- 服务器模式使用相同的 Core 逻辑
- 通过 headless 测试验证 Core 逻辑正确性

禁止：
- 服务器模式注册 UiPlugin
- Core 逻辑依赖 UI 存在才能运行

必须：
- Core 层不引用任何 UI 类型（`use crate::ui::`）
- 服务器模式只加载 Core + Infrastructure Plugin
- Core 逻辑在无 UI 环境下行为一致

---

## 规则8：架构审查检查表

每个触及 UI 边界的 PR 必须通过以下 6 项检查：

1. **UI use 语句检查**：UI 系统的 `use` 语句是否只引用 UI 内部类型和 ViewModel？
2. **UI 写入检查**：UI 系统是否通过 UiCommand 传递用户操作？禁止直接修改 ECS
3. **Core use 语句检查**：Core 系统的 `use` 语句是否不包含 `crate::ui::`？
4. **ViewModel 数据检查**：ViewModel 是否只包含扁平化渲染数据？是否不包含 Entity 引用？
5. **模态面板检查**：模态面板是否标记了 `BlocksGameInput`？输入焦点状态是否正确维护？
6. **Notification 检查**：Notification 中是否不包含 UI 特定数据（颜色、字号、动画参数）？

---

## 规则9：ViewModel Push 模式

允许：
- 利用 Bevy Component Hooks（on_insert / on_mut）在 Core 组件变化时主动 Push 数据到 ViewModel
- 高频变更组件（Health、AP）使用 Push 模式，避免每帧遍历
- 低频变更组件（Name、Class）使用 Pull 模式（Changed<T>），简单可靠

禁止：
- 所有 ViewModel 更新都使用 Pull 模式（性能浪费）
- Push 模式中访问不存在的 Entity
- Push 模式中包含游戏逻辑计算

必须：
- Push 模式仅在变化的是当前选中单位时才更新 ViewModel
- Push 模式与 Pull 模式可混合使用（按组件变更频率选择）

| 场景 | 推荐模式 | 理由 |
|------|---------|------|
| 高频变更组件（Health、AP） | Push (Hook) | 避免每帧遍历 |
| 低频变更组件（Name、Class） | Pull (Changed<T>) | 简单可靠 |
| 跨单位聚合数据（回合队列） | Pull + 脏标记 | 需要遍历所有单位 |
| 实时战斗面板 | Push (Observer) | 需要即时响应 |

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

## 规则10：UI 创建 Entity 限制

允许：
- UI 层创建 UI 节点树 Entity（NodeBundle、TextBundle 等）
- UI 层的 Entity 仅限 UI 节点树，且必须由 UI 系统管理生命周期

禁止：
- UI 层创建游戏逻辑 Entity（如 units、items 等）
- UI 层持有游戏 Entity 引用跨帧
- UI 层对游戏 Entity 的引用超过当前帧

必须：
- UI 层对游戏 Entity 的引用最长生命周期 = 当前帧
- 帧结束后必须丢弃引用，下一帧从 ViewModel 重新获取

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

# 管线

## UI 读管线

```
Core 变更 ECS 状态 → update_*_view 系统提取数据 → ViewModel Resource 更新 → UI 系统读取 ViewModel → UI 渲染
```

### Step1：Core 变更 ECS 状态

输入：游戏逻辑执行结果（伤害、治疗、Buff 等）
处理：Core 系统修改 ECS 组件
输出：ECS 组件状态变更
禁止：Core 系统在此步骤中直接操作 UI

### Step2：update_*_view 系统提取数据

输入：变更后的 ECS 组件
处理：`update_selected_unit_view` 等系统从 ECS 读取数据，填充 ViewModel
输出：ViewModel Resource 数据更新
禁止：提取过程中包含游戏逻辑计算

### Step3：UI 系统读取 ViewModel

输入：更新后的 ViewModel Resource
处理：UI 系统通过 `Res<XxxView>` 读取数据
输出：UI 控件的显示数据
禁止：UI 系统绕过 ViewModel 直接查询 ECS

### Step4：UI 渲染

输入：UI 控件显示数据
处理：创建或更新 Bevy UI 节点
输出：屏幕像素
禁止：渲染过程中修改任何游戏状态

---

## UI 写管线

```
用户点击 UI → UI 系统发送 UiCommand → command_handler 消费 → Core 修改 ECS 状态 → update_*_view 更新 ViewModel → UI 重绘
```

### Step1：用户点击 UI

输入：鼠标/键盘事件
处理：UI 交互系统检测用户操作
输出：操作类型（点击坐标、按键等）
禁止：在此步骤中直接修改游戏状态

### Step2：UI 系统发送 UiCommand

输入：操作类型
处理：构造 `UiCommand::SelectUnit` 等 Message
输出：UiCommand Message
禁止：构造不合法的 UiCommand（如缺少必要字段）

### Step3：command_handler 消费

输入：UiCommand Message
处理：`handle_ui_commands` 系统执行命令对应的逻辑
输出：游戏状态变更（CombatIntent、NextState 等）
禁止：command_handler 中执行 UI 渲染逻辑

### Step4：Core 修改 ECS 状态

输入：command_handler 的执行结果
处理：ECS 系统执行游戏逻辑
输出：ECS 组件状态更新
禁止：Core 系统在此步骤中引用 UI 类型

### Step5：update_*_view 更新 ViewModel

输入：变更后的 ECS 状态
处理：ViewModel 更新系统提取数据
输出：ViewModel Resource 更新
禁止：跳过此步骤直接让 UI 渲染旧数据

---

## UI 通知管线

```
Core 事件发生 → 系统发送 Notification Message → UI 系统消费 → UI 显示反馈 → Notification 被消费
```

### Step1：Core 事件发生

输入：游戏逻辑执行结果（伤害、死亡、Buff 等）
处理：Core 系统通过 `commands.write_message()` 发送 Notification
输出：Notification Message（DamageApplied、HealApplied 等）
禁止：在 Notification 中包含 UI 特定数据

### Step2：UI 系统消费

输入：Notification Message
处理：`MessageReader<T>` 读取消息内容
输出：格式化后的显示数据（日志文本、飘字内容）
禁止：在消费过程中修改游戏状态

### Step3：UI 显示反馈

输入：格式化后的显示数据
处理：生成飘字 Entity 或写入 CombatLog
输出：视觉反馈
禁止：反馈内容影响游戏逻辑（如飘字不扣血）

### Step4：Notification 被消费

输入：已处理的 Notification
处理：Bevy Message 系统自动清理已读消息
输出：消息队列清空
禁止：手动持久化 Notification 数据

---

## ViewModel Push 管线

利用 Bevy Component Hooks 在 Core 组件变化时主动 Push 数据到 ViewModel，替代每帧 Pull。

```
Core 组件变化 → on_mut Hook 触发 → 检查是否为当前选中单位 → 更新 ViewModel Resource
```

### Hook 注册阶段

输入：Core 组件类型（如 Health）
处理：register_component_hooks::<Health>().on_mut(...)
 输出：Hook 注册完成
禁止：在 Hook 中包含游戏逻辑计算

### 变化检测阶段

输入：Core 组件变化事件
处理：检查变化的 Entity 是否为当前选中单位
输出：是否需要更新 ViewModel
禁止：对所有 Entity 的变化都更新 ViewModel（只更新选中单位）

### ViewModel 更新阶段

输入：变化后的 Core 组件数据
处理：更新 ViewModel Resource 的对应字段
输出：ViewModel Resource 更新
禁止：在 Push 模式中访问不存在的 Entity

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

# 数据结构

## SelectedUnitView（选中单位视图模型）

职责：存储当前悬停/选中单位的全部显示数据，供 UI 面板读取

结构：
- name：String — 单位名称
- race：String — 种族名称
- class：String — 职业名称
- grid_coord：IVec2 — 格子坐标
- hp / max_hp：i32 — 生命值
- mp / max_mp：i32 — 魔法值
- stamina / max_stamina：i32 — 体力值
- core_attrs：Vec<CoreAttrEntry> — 核心属性（8 维）
- combat_attrs：Vec<DerivedAttrEntry> — 战斗衍生属性
- support_attrs：Vec<DerivedAttrEntry> — 辅助衍生属性
- skills：Vec<SkillEntry> — 技能列表
- traits：Vec<TraitEntry> — 特质列表
- buffs：Vec<BuffEntry> — Buff 列表
- equipment：Vec<EquipmentSlotEntry> — 装备槽列表
- inventory：Vec<InventoryEntry> — 背包条目
- is_selected：bool — 是否被选中

要求：
- 是 Bevy Resource（`#[derive(Resource)]`），不是 Component
- 默认值所有字段为空/零/false
- 由 `update_selected_unit_view` 系统在 HoveredEntity 变化时填充
- 只包含渲染数据，不包含 Entity 引用或游戏对象

---

## CombatPreviewView（战斗预览视图模型）

职责：存储战斗预览的显示数据（伤害估算、命中率等）

结构：
- is_visible：bool — 是否显示预览
- estimated_damage：i32 — 预估伤害
- hit_rate：i32 — 命中率百分比
- crit_rate：i32 — 暴击率百分比
- is_lethal：bool — 是否致死

要求：
- 是 Bevy Resource
- 只在 SelectTarget 阶段可见
- 由 `update_combat_preview_view` 系统维护

---

## TurnInfoView（回合信息视图模型）

职责：存储当前回合的全局信息（回合数、行动顺序等）

结构：
- turn_number：u32 — 当前回合数
- is_player_turn：bool — 是否玩家回合
- turn_order：Vec<(String, bool)> — 行动顺序（名称, 是否玩家方）
- current_index：usize — 当前行动索引

要求：
- 是 Bevy Resource
- 由 `update_turn_info_view` 系统在 TurnState 变化时填充
- 不包含 Entity 引用

---

## UiCommand（UI 命令枚举）

职责：封装用户的所有操作意图，作为 Bevy Message 传递

结构：
- SelectUnit：携带 Entity — 选中玩家单位
- MoveUnit：携带 IVec2 坐标 — 移动到目标格子
- Attack：无参数 — 选择基础攻击
- Skill：携带 skill_id String — 选择技能
- SelectTarget：携带 IVec2 坐标 — 选择攻击目标
- Wait：无参数 — 待机
- Cancel：无参数 — 取消当前操作
- EndTurn：无参数 — 结束回合
- 菜单命令：StartGame、ContinueGame、SelectStage、ConfirmStage 等

要求：
- 派生 `Message` trait（`#[derive(Message)]`）
- 变体携带最小必要数据
- 禁止携带游戏状态数据

---

## UiFocusState（UI 焦点状态）

职责：追踪是否有模态面板正在阻止游戏输入

结构：
- blocks_input：bool — 是否有面板阻止输入

要求：
- 是 Bevy Resource
- 只被 UI 层的输入系统写入
- 变化时触发 `update_selected_unit_view` 更新
- Core 层不读取此 Resource

---

## UiLocalState（UI 本地状态）

职责：存储 UI 层的本地状态，如面板开关、选中项、悬停实体等

结构：
- selected_panel：可选值 — 当前选中的面板类型
- is_unit_panel_open：布尔 — 单位面板是否打开
- hovered_grid：可选值 — 鼠标悬停的格子坐标

要求：
- 是 Bevy Resource（`#[derive(Resource, Default)]`）
- 只被 UI 层系统读写
- 不被 Core 层系统读写
- 丢失不影响游戏逻辑
- 默认值对游戏无副作用

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

## HoveredEntity（悬停实体）

职责：记录当前鼠标悬停/最后点击的单位实体，驱动 ViewModel 更新

结构：
- entity：Option<Entity> — 悬停的实体，None 表示无悬停

要求：
- 是 Bevy Resource
- 只被 UI 层的输入系统写入
- 变化时触发 `update_selected_unit_view` 更新
- Core 层不读取此 Resource

---

# 禁止事项

禁止：UI 系统通过 `Query<(&Attributes, ...)>` 直接查询 Core ECS 组件

原因：UI 只能通过 ViewModel 获取显示数据，直接查询 ECS 会破坏 Logic/Presentation 分离，导致 UI 与游戏逻辑耦合

违反后果：UI 无法独立于 Core 测试，替换 UI 实现时需要修改 Core 代码

---

禁止：Core 系统 `use crate::ui::` 任何模块

原因：Core 是纯游戏规则层，引用 UI 类型会破坏可移植性（无法在无 UI 的服务器模式下运行）

违反后果：Core 依赖 UI 实现，无法独立测试 Core 逻辑

---

禁止：UI 系统直接修改 ECS 组件（如 `commands.entity(e).insert(Selected)`）

原因：用户操作必须通过 UiCommand Message 传递给 command_handler，由 Core 层决定如何修改状态

违反后果：游戏状态被 UI 绕过逻辑层修改，状态变更不可追踪、不可回滚

---

禁止：command_handler 中执行 UI 渲染逻辑（如生成飘字、更新面板）

原因：command_handler 是 Core 行为，负责处理命令和修改游戏状态。表现逻辑属于 UI 层

违反后果：Core 逻辑与表现耦合，无法在无渲染环境下执行命令

---

禁止：ViewModel 包含 Entity 引用或游戏对象指针

原因：ViewModel 是只读快照，包含 Entity 会让 UI 有路径绕过 ViewModel 直接操作游戏状态

违反后果：UI 可能通过 Entity 引用修改 Core 组件，破坏单向数据流

---

禁止：UI State 丢失影响游戏逻辑

原因：UI State 是纯表现层状态（如悬停高亮、面板展开），游戏逻辑不依赖 UI State

违反后果：UI 崩溃或重置导致游戏逻辑异常

---

禁止：在 Notification 中包含 UI 特定数据（颜色、字号、动画参数）

原因：Notification 是 Core → UI 的单向消息，Core 不应知道 UI 的展示细节

违反后果：Core 与 UI 实现耦合，修改 UI 样式需要修改 Core 代码

---

禁止：UI 面板代码中硬编码颜色、字号、间距

原因：样式必须通过 `UiTheme` 统一管理，硬编码会导致样式不一致且难以维护

违反后果：换肤时需要逐文件修改，不同面板样式不统一

---

禁止：`handle_ui_commands` 在非玩家回合执行

原因：只有玩家回合才应响应 UI 操作，AI 回合由 AI 系统驱动

违反后果：玩家在 AI 回合中操作导致状态不一致

---

禁止：Core 代码中出现 `cfg(feature = "ui")` 作为业务逻辑分支

原因：Core 层不知道 UI 的存在，使用条件编译区分 UI 有无违反 Logic/Presentation 分离。Core 逻辑必须在有无 UI 环境下行为一致。

违反后果：Core 依赖 UI feature，无法在服务器模式下独立运行。业务逻辑与 UI 实现耦合。

---

禁止：UI 创建游戏逻辑 Entity

原因：UI 层禁止 `commands.spawn(...)` 创建游戏逻辑 Entity。UI 的 Entity 仅限 UI 节点树（NodeBundle、TextBundle 等），且必须由 UI 系统管理生命周期。

违反后果：UI 创建的 Entity 不受 Core 层管理，导致游戏逻辑混乱。

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

禁止：UI 持有 Entity 引用跨帧

原因：UI 层对游戏 Entity 的引用最长生命周期 = 当前帧。帧结束后必须丢弃，下一帧从 ViewModel 重新获取。跨帧持有引用会导致 Entity 被回收复用后读到脏数据。

违反后果：Entity 被回收复用后 UI 读到错误数据，导致显示异常或 Panic。

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

# AI 修改规则

## 如果新增 ViewModel

允许：
- 在 `src/ui/view_models.rs` 中定义新的 Resource 类型
- 在 `UiPlugin::build` 中 `init_resource::<NewView>()`
- 编写对应的 `update_new_view` 系统

禁止：
- 在 ViewModel 中包含 Entity 引用
- 在 ViewModel 中包含游戏逻辑计算
- 在 Core 模块中定义 ViewModel 类型

优先检查：
- ViewModel 是否是 Resource（不是 Component）
- update 系统是否只在 `Changed` 时更新
- ViewModel 是否只包含扁平化的渲染数据

---

## 如果新增 UiCommand 变体

允许：
- 在 `src/ui/events.rs` 的 `UiCommand` 枚举中追加新变体
- 在 `command_handler.rs` 的 `handle_ui_commands` 中添加处理分支
- 新变体携带最小必要数据

禁止：
- 在 UI 系统中直接处理新命令（必须通过 command_handler）
- 新变体携带游戏状态数据
- 在 Core 模块中直接构造 UiCommand

优先检查：
- 新变体是否在 `events.rs` 中定义
- command_handler 中是否正确处理
- 是否在 `player_turn` 条件下运行

---

## 如果新增 UI 面板

允许：
- 在 `src/ui/panels/` 中创建新面板模块
- 面板只通过 `Res<XxxView>` 读取数据
- 面板模块注册为子 Plugin

禁止：
- 面板直接 `Query<(&Attributes, ...)>` 查询 Core 组件
- 面板硬编码颜色值（使用 `Res<UiTheme>`）
- 面板在 `Update` 阶段修改游戏状态

优先检查：
- 面板的 `use` 语句是否只引用 UI 内部类型和 ViewModel
- 面板是否通过 UiCommand 传递用户操作
- 面板是否标记了 `BlocksGameInput`（如果是模态面板）

---

## 如果修改 ViewModel 结构

允许：
- 在 ViewModel 中添加新的渲染数据字段
- 调整 ViewModel 字段的类型（仅限基础类型）

禁止：
- 在 ViewModel 中包含 Entity 引用
- 在 ViewModel 中包含游戏逻辑计算
- 在 ViewModel 中使用 Bevy ECS 类型（Entity、Vec<Entity> 等）
- ViewModel 包含业务语义（如 can_attack、reason_cannot_attack）

优先检查：
- 新字段是否只包含扁平化渲染数据
- 新字段是否不包含 Entity 引用
- update_*_view 系统是否正确填充新字段
- Push 模式（如适用）是否正确注册了 Hook

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md

---

## 如果修改 Notification 消费

允许：
- 在 `src/ui/` 中新增或修改 `MessageReader<T>` 消费系统
- 将 Notification 转化为飘字、日志条目等视觉反馈

禁止：
- 在消费系统中修改游戏状态
- 在消费系统中引用 Core 内部类型
- 消费系统产生影响游戏逻辑的副作用

优先检查：
- 消费系统是否只执行表现逻辑
- Notification 的数据格式是否与 Core 侧一致
- 消费系统是否在正确的 Schedule 中运行

---

## 如果测试失败

排查顺序：
1. 检查 UI 系统是否直接查询了 Core ECS 组件（违反不变量 1）
2. 检查 command_handler 是否在正确的状态下运行（`player_turn` 条件）
3. 检查 ViewModel 的 `update_*_view` 系统是否在 `Changed` 时正确触发
4. 检查 UiCommand 是否正确发送和消费
5. 检查 Notification 是否被正确广播和消费
6. 检查 Core 模块是否不依赖 bevy_ui（违反不变量 6）
7. 检查 ViewModel 是否不包含 Entity 引用（违反不变量 7）
8. 检查 ViewModel 是否只包含基础类型（违反不变量 8）

> **优化来源**: docs/architecture/ui_domain_boundary_rules.md
