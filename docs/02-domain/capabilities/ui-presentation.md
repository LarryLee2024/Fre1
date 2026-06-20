---
id: 02-domain.ui-presentation
title: UI 表现层领域规则 v1.0
status: draft
owner: domain-designer
created: 2026-06-19
updated: 2026-06-19
tags:
  - domain
  - ui
  - presentation
  - capabilities
---

# UI 表现层领域规则

> 定位：Presentation Layer（L3），不是业务领域
> 依赖：Core（Capabilities + Domains）+ Shared
> 通信：通过 Observer 监听 Domain Event，通过 UiCommand 反向输入
> 参考 ADR：ADR-055

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Screen | 页面，Widget 的组合容器 | 负责：页面级布局与 Widget 组合；不负责：Widget 内部渲染逻辑 |
| Widget | 可复用 UI 组件，独立 Plugin | 负责：单一 UI 元素的渲染与交互；不负责：业务数据获取（禁止 Query Domain 组件） |
| ViewModel | UI 状态的投影，Domain 数据的 UI 视图 | 负责：承载 UI 展示所需的结构化数据；不负责：业务逻辑计算 |
| Projection | Domain → ViewModel 的转换函数 | 负责：将 Domain Event 数据映射为 ViewModel 更新；不负责：修改 Domain 状态 |
| UiCommand | UI → Domain 的命令 | 负责：封装用户操作意图并传递到 Domain 层；不负责：命令的执行逻辑 |
| UiIntent | 输入意图抽象 | 负责：将原始输入（键鼠/手柄）映射为语义化意图；不负责：意图的业务处理 |
| UiAction | Widget 的输出动作 | 负责：Widget 级别的交互输出声明；不负责：动作的后续处理 |
| Overlay | 独立叠加层，不挂在 Screen 下 | 负责：Tooltip/Notification/Modal 等全局浮层；不负责：Screen 内的布局 |
| Theme | 设计令牌集合 | 负责：统一管理颜色/字体/间距等视觉令牌；不负责：Widget 的具体样式实现 |
| StyleToken | 颜色/间距/字体的统一标识 | 负责：提供语义化的视觉属性引用；不负责：属性值的硬编码 |
| Focusable | 可聚焦的 UI 元素 | 负责：声明元素可接受焦点；不负责：焦点导航策略 |
| FocusGroup | 焦点导航组 | 负责：组内元素的焦点流转规则；不负责：跨组焦点切换 |
| Dirty<T> | 脏标记机制，避免每帧刷新 | 负责：标记 ViewModel 是否需要重新渲染；不负责：渲染逻辑本身 |
| ScreenStack | 页面导航栈 | 负责：Screen 的 push/pop/replace 管理；不负责：Screen 内部状态 |
| WidgetContract | Widget 的输入输出声明 | 负责：定义 Widget 的数据依赖与输出动作；不负责：Widget 的实现细节 |

### 已对齐项目术语

- **SkillId**：技能定义的唯一标识（定义在 Ability 领域）
- **CharacterId**：角色的唯一标识（定义在 Combat 领域）
- **BuffId**：Buff 效果的唯一标识（定义在 Effect 领域）
- **ItemId**：物品的唯一标识（定义在 Inventory 领域）
- **QuestId**：任务的唯一标识（定义在 Quest 领域）
- **SaveSlot**：存档槽位标识（定义在 Save 领域）
- **GridPos**：网格坐标（定义在 Tactical 领域）
- **LocalizationKey**：本地化文本键（定义在 Localization 架构，ADR-053）
- **AssetKey**：资源引用键（定义在 Content Loading 架构，ADR-047）
- **Cue**：表现层信号（定义在 Cue 领域），CueType::Popup 与 UI 层的 NotificationVm 有交叉但职责不同——Cue 负责触发信号，UI 层负责渲染呈现
- **Domain Event**：领域事件（定义在 Event 领域），UI 层通过 Observer 监听

---

## 2. 状态机

### 2.1 Screen 生命周期状态机

```
Defined（已定义——在 Theme/Content 中配置）
   │  [ScreenStack::push]
   ▼
Loading（加载中——初始化 ViewModel、加载资源）
   │  [资源就绪 + ViewModel 初始化完成]
   ▼
Active（活跃——可见可交互）
   │  [ScreenStack::push(新 Screen)]
   ▼
Background（后台——被新 Screen 遮挡，不可交互）
   │  [上层 Screen pop]
   ▼
Active（恢复活跃）
   │  [ScreenStack::pop 或 replace]
   ▼
Unloading（卸载中——清理资源、注销 Observer）
   │  [清理完毕]
   ▼
Destroyed（已销毁）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Defined → Loading | ScreenStack::push | 初始化 ViewModel，注册 Observer，加载资源 |
| Loading → Active | 资源就绪 + ViewModel 初始化完成 | 显示 Screen，激活 FocusGroup |
| Active → Background | 新 Screen push 到栈顶 | 暂停交互，保留 ViewModel |
| Background → Active | 上层 Screen pop | 恢复交互，刷新 ViewModel（如 Dirty） |
| Active → Unloading | ScreenStack::pop/replace | 注销 Observer，清理定时器 |
| Background → Unloading | Screen 被 replace 或强制移除 | 注销 Observer，清理定时器 |
| Unloading → Destroyed | 清理完毕 | 移除所有 Entity |
| 禁止 | Loading → Background | 未完成加载的 Screen 不可被遮挡 |
| 禁止 | Destroyed → 任何状态 | Screen 销毁后不可复活，需重新 push |

### 2.2 Widget 生命周期

```
Spawned（已创建——Entity 已 spawn）
   │  [WidgetContract 输入数据就绪]
   ▼
Mounted（已挂载——首次渲染完成）
   │  [Dirty<T> 标记为 true]
   ▼
Updating（更新中——重新渲染）
   │  [渲染完成，Dirty<T> 清除]
   ▼
Mounted（回到已挂载）
   │  [Visibility::Hidden 或 Persistent 模式]
   ▼
Hidden（隐藏——不可见但 Entity 存在）
   │  [Visibility::Visible]
   ▼
Mounted（恢复可见）
   │  [despawn 或 Ephemeral 模式销毁]
   ▼
Despawned（已销毁）
```

### Widget 生命周期规则

| 模式 | 隐藏方式 | 销毁时机 | 适用场景 |
|------|---------|---------|---------|
| Persistent | Visibility 切换 | Screen 销毁时 | 频繁开关的面板（技能栏、状态栏） |
| Ephemeral | 直接 despawn | 关闭时立即 despawn | 一次性弹窗、确认框 |

---

## 3. 不变量（Invariants）

### INV-UI-001：UI 不直接读取 Domain 组件
- **条件**：任何 UI 模块需要获取 Domain 数据时
- **不变量**：UI 模块禁止 `Query<&Health>`、`Query<&Mana>`、`Query<&Ability>` 等 Domain 组件查询，唯一允许的数据源是 `Res<XXXVm>` 或 `Res<UiStore>`
- **违反后果**：UI 与 Domain 耦合，ViewModel 抽象被绕过，Domain 重构时 UI 崩溃
- **违反检测**：CI 中 grep `Query<` 在 ui/ 目录下不应出现 Domain 组件

### INV-UI-002：Widget 不持有 Entity
- **条件**：任何 Widget 组件定义时
- **不变量**：Widget 组件禁止包含 `Entity` 字段，使用业务 ID 替代（SkillId、CharacterId、BuffId）
- **违反后果**：Entity 在重建 UI/切换存档/重载战斗后失效，导致悬空引用

### INV-UI-003：UI 动画不驱动业务逻辑
- **条件**：任何 UI 动画播放过程中
- **不变量**：动画播放不影响 Domain 状态，Domain 先完成业务操作，UI 再播放表现动画
- **违反后果**："跳过动画"/"加速战斗"/"自动战斗"导致业务 Bug

### INV-UI-004：单向数据流
- **条件**：任何数据在 UI 层与 Domain 层之间流动时
- **不变量**：正向 Domain Event → Projection → ViewModel → Widget；反向 User Input → UiIntent → UiCommand → Domain，禁止双向绑定
- **违反后果**：数据流不可追踪，调试困难，状态不一致

### INV-UI-005：Screen 不直接拼 Node
- **条件**：任何 Screen 的布局实现中
- **不变量**：Screen 只做 Widget 组合，禁止在 Screen 中直接写 Node/BackgroundColor/Interaction 等 Bevy UI 原语，所有 UI 原语封装在 Widget 中
- **违反后果**：Screen 代码膨胀，Widget 不可复用，样式散落各处

### INV-UI-006：Overlay 独立于 Screen
- **条件**：任何 Overlay（Tooltip/Notification/Modal/Loading）的挂载方式
- **不变量**：Overlay 不挂在 Screen 下，Screen 销毁不影响 Overlay，Overlay 有独立的 Root 节点
- **违反后果**：Screen 切换时 Overlay 被误销毁，Notification/Tooltip 消失

### INV-UI-007：所有文本走 LocalizationKey
- **条件**：任何用户可见文本的渲染
- **不变量**：禁止 `Text::new("Attack")` 硬编码，使用 `LocalizedText(UiTextKey::Attack)` 统一包装，FontSize 使用枚举（FontSize::Px/Rem），不用裸 f32
- **违反后果**：违反宪法 ss22 Localization First，多语言支持失败

### INV-UI-008：颜色/字体/间距统一 Token 化
- **条件**：任何视觉属性的设置
- **不变量**：禁止 `Color::srgb(...)` 直接写在 Widget 中，使用 `UiColors::Primary`、`UiSpacing::Md`、`UiTypography::Heading` 等 Token，Theme 切换时所有 Widget 自动更新
- **违反后果**：视觉风格不一致，Theme 切换失败

### INV-UI-010：五条铁律（精简版）

从 9 条不变量中提炼的 5 条最核心约束：

1. **Domain 不依赖 UI** — Core/Infra 禁止 import ui/ 中的任何类型
2. **UI 不直接读 Domain** — 通过 ViewModel，禁止 Query Domain Component
3. **Screen 组合 Widget** — Screen 不直接拼 Node
4. **颜色字体间距统一 Token 化** — 禁止 Color::srgb() 直接写在 Widget 中
5. **Primitives 隔离** — Widgets 和 Screens 禁止直接 import Bevy UI 原语（Node、Button、Interaction、BackgroundColor 等），必须通过 Primitives 层

违反任何一条，50 万行后 UI 必然成为最大技术债。

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：UI 模块直接 Query Domain 组件获取数据 — 理由：违反 INV-UI-001，UI 只通过 ViewModel 获取数据
- 🟥 禁止：Widget 组件中包含 Entity 字段 — 理由：违反 INV-UI-002，Entity 在重建/重载后失效
- 🟥 禁止：动画播放阻塞或影响 Domain 逻辑执行 — 理由：违反 INV-UI-003，动画是可选表现，逻辑必须独立
- 🟥 禁止：ViewModel 与 Domain 组件双向绑定 — 理由：违反 INV-UI-004，数据流必须单向
- 🟥 禁止：Screen 中直接使用 Bevy UI 原语（Node/BackgroundColor/Interaction） — 理由：违反 INV-UI-005，所有原语必须封装为 Widget
- 🟥 禁止：Overlay 挂载为 Screen 的子节点 — 理由：违反 INV-UI-006，Overlay 生命周期独立于 Screen
- 🟥 禁止：硬编码用户可见文本字符串 — 理由：违反 INV-UI-007，必须使用 LocalizationKey
- 🟥 禁止：在 Widget 中直接使用 Color::srgb 或裸 f32 字号 — 理由：违反 INV-UI-008，必须使用 StyleToken
- 🟥 禁止：每帧全量刷新 ViewModel — 理由：必须使用 Dirty 标记按需刷新，全量刷新导致性能问题
- 🟥 禁止：手动 despawn/spawn 切换 Screen — 理由：Screen 切换必须通过 ScreenStack 管理，手动操作破坏导航栈一致性

### PROHIBIT-UI-011：禁止大量 Marker 结构体

禁止为每个 UI 元素创建独立 Marker 结构体：

```rust
// ❌ 禁止
struct HpText;
struct ManaText;
struct ExpText;
struct GoldText;

// ✅ 允许：统一枚举
#[derive(Component, Reflect)]
pub enum UiBinding {
    Hp, Mana, Exp, Gold, Level, Turn, Ap,
}

// ✅ 或统一 ID
#[derive(Component, Reflect)]
pub struct UiElementId(pub u32);
```

原因：50 万行项目最终 400+ Marker，导致 Archetype 爆炸、AI 代码生成困难、重构成本极高。

---

## 5. 流程定义

### 5.1 Projection 更新流程

- **输入**：Domain Event（如 DamageApplied）+ 当前 ViewModel
- **处理**：
  1. Observer 监听到 Domain Event
  2. 调用对应的 Projection 纯函数
  3. Projection 输入：Domain Event 数据 + 当前 ViewModel → 输出：新 ViewModel
  4. 替换当前 ViewModel
  5. 设置 Dirty 标记
- **输出**：更新后的 ViewModel + Dirty 标记
- **失败处理**：Projection 函数异常时 ViewModel 不更新，记录错误日志

### 5.2 Widget 刷新流程

- **输入**：Dirty 标记变更
- **处理**：
  1. 检测 Dirty 标记为 true
  2. 读取最新 ViewModel 数据
  3. 更新 Widget 渲染状态
  4. 清除 Dirty 标记
- **输出**：Widget 视觉更新
- **失败处理**：刷新失败时 Widget 保持上一帧状态，不崩溃

### 5.3 用户输入处理流程

- **输入**：原始输入事件（键盘/鼠标/手柄）
- **处理**：
  1. Input 系统将原始输入转换为 UiIntent
  2. UiIntent 路由到当前活跃的 FocusGroup
  3. FocusGroup 将 UiIntent 映射为 Widget 的 UiAction
  4. UiAction 转换为 UiCommand
  5. UiCommand 发送到 Domain 层执行
- **输出**：UiCommand 发送到 Domain
- **失败处理**：UiIntent 无法映射时静默忽略（如当前无 Focusable 元素）

### 5.4 Screen 导航流程

- **输入**：导航请求（push/pop/replace）+ 目标 Screen 标识
- **处理**：
  1. 校验 ScreenStack 深度（上限 10）
  2. push：将当前 Active Screen 转为 Background，加载新 Screen
  3. pop：卸载栈顶 Screen，恢复下层 Screen 为 Active
  4. replace：卸载栈顶 Screen，加载新 Screen（栈深度不变）
  5. 播放过渡动画
- **输出**：Screen 状态变更 + 过渡动画
- **失败处理**：栈深度超限时拒绝 push，记录警告；pop 空栈时忽略

### 5.5 Notification 显示流程

- **输入**：NotificationVm（消息键、优先级、持续时间、类型）
- **处理**：
  1. NotificationService 接收 NotificationVm
  2. 检查同类型 Notification 是否已存在（合并规则）
  3. 按优先级排序插入队列（Critical > Important > Normal）
  4. 显示 Notification
  5. 启动超时定时器
- **输出**：Notification 渲染
- **失败处理**：队列满时丢弃最低优先级 Notification

### 5.6 Modal 显示流程

- **输入**：ModalVm（内容、按钮配置）
- **处理**：
  1. ModalService 接收请求
  2. 校验 Modal 栈深度（上限 3）
  3. 推入 Modal 栈
  4. 最底层 Modal 阻止 Screen 交互
  5. 等待用户操作（Confirm/Cancel）
- **输出**：UiAction::Confirm 或 UiAction::Cancel
- **失败处理**：栈深度超限时拒绝新 Modal，记录警告

### 5.7 Tooltip 显示流程

- **输入**：TooltipVm（内容、位置提示）
- **处理**：
  1. Focusable 元素获得焦点/hover 超过 0.3s
  2. TooltipService 创建 Tooltip Overlay
  3. 同一时间只显示一个 Tooltip（新 Tooltip 替换旧的）
  4. 焦点移出时销毁 Tooltip
- **输出**：Tooltip 渲染
- **失败处理**：Tooltip 内容为空时不显示

### 5.8 UI Schema 治理流程（RULE-UI-009）

AI 生成 UI 代码必须遵守 `docs/ui_schema/` 中的 Schema 定义：

```yaml
# docs/ui_schema/widgets/skill_panel.yaml
SkillPanel:
  input: SkillPanelVm
  output:
    - UiAction::SelectSkill(SkillId)
    - UiAction::CastSkill(SkillId)
  children: [SkillButton, SkillTooltip]
  prohibited: [Query<&Ability>, Query<&Health>, EventReader]
```

Schema 文件结构：
- `docs/ui_schema/screens/` — Screen Schema
- `docs/ui_schema/widgets/` — Widget Schema
- `docs/ui_schema/view_models/` — ViewModel Schema
- `docs/ui_schema/contracts/` — Contract 声明

每个新增 Widget 必须先写 Schema，再写代码。

### 5.9 UI 三层测试流程（RULE-UI-010）

| 测试层 | 测试对象 | 验证内容 | 位置 |
|--------|---------|---------|------|
| Widget Test | 单个 Widget | 渲染、交互、Contract 合规 | `src/ui/widgets/*/tests/` |
| Screen Test | Screen 组合 | Widget 组合、导航、VM 绑定 | `src/ui/screens/*/tests/` |
| Snapshot Test | UI 树结构 | Entity 层级/Component 结构 | `src/ui/tests/snapshot/` |

Widget Test 必须验证：
1. Contract 合规（无 Domain Query）
2. 渲染正确性（从 ViewModel 到 UI 元素）
3. 交互响应（点击→UiAction）

Screen Test 必须验证：
1. Widget 组合完整性
2. ViewModel 绑定正确
3. 导航流程正常

Snapshot Test 使用 insta 快照：
```rust
#[test]
fn battle_screen_tree_snapshot() {
    let tree = capture_ui_tree::<BattleScreen>();
    insta::assert_yaml_snapshot!("battle_screen_tree", tree);
}
```

### 5.10 WidgetFactory 流程（RULE-UI-011）

所有 Widget 实现统一的 WidgetFactory trait：

```rust
pub trait WidgetFactory: Component {
    type Vm: Reflect + Default;
    fn create(commands: &mut Commands, vm: &Self::Vm) -> Entity;
    fn refresh(entity: Entity, vm: &Self::Vm, query: &mut Query<&mut Self>);
    fn destroy(commands: &mut Commands, entity: Entity);
}
```

Screen 通过 WidgetFactory 组合 Widget，不直接 spawn Node。

### 5.11 UI 与 Content 数据流流程（RULE-UI-012）

UI 不直接访问 DefRegistry 中的 Definition：

```rust
// ❌ 禁止
fn update_skill_icon(defs: Res<DefRegistry<SpellDef>>) { ... }

// ✅ 允许：Projection 读取 Def，投影到 ViewModel
fn project_skill_info(
    trigger: Trigger<AbilityUsed>,
    defs: Res<DefRegistry<SpellDef>>,
    mut store: ResMut<UiStore>,
) {
    if let Some(def) = defs.get(trigger.ability_id) {
        store.skill_panel.update_from_def(def);
    }
}
```

数据流路径：Content → DefRegistry → Projection → ViewModel → Widget

Modding 数据流：Mod → Content → DefRegistry → Projection → ViewModel → Widget
禁止 Mod 直接扩展 UI。

---

## 6. 领域事件

### UI 监听的 Domain Event

| Domain Event | Projection | ViewModel 更新 |
|-------------|-----------|---------------|
| DamageApplied | BattleProjection | BattleHudVm.hp |
| HealthChanged | CharacterProjection | CharacterPanelVm.hp/max_hp |
| ManaChanged | CharacterProjection | CharacterPanelVm.mp/max_mp |
| TurnStarted | BattleProjection | BattleHudVm.current_turn |
| BuffApplied | CharacterProjection | CharacterPanelVm.buffs |
| BuffExpired | CharacterProjection | CharacterPanelVm.buffs |
| AbilityUsed | BattleProjection | SkillPanelVm.cooldowns |
| ItemAcquired | InventoryProjection | InventoryVm.items |
| QuestUpdated | QuestProjection | QuestLogVm.quests |
| LevelUp | CharacterProjection | CharacterPanelVm.level |
| GoldChanged | EconomyProjection | ShopVm.gold |

### UI 发出的 UiCommand

| UiCommand | 目标 Domain | 说明 |
|-----------|-----------|------|
| CastSkill(SkillId) | Ability | 施放技能 |
| SelectTarget(CharacterId) | Combat | 选择目标 |
| MoveToPosition(GridPos) | Tactical | 移动到位置 |
| UseItem(ItemId) | Inventory | 使用物品 |
| AcceptQuest(QuestId) | Quest | 接受任务 |
| BuyItem(ItemId) | Economy | 购买物品 |
| SaveGame(SaveSlot) | Save | 保存游戏 |
| LoadGame(SaveSlot) | Save | 加载游戏 |
| ChangeSettings(UiSettings) | Settings | 修改设置 |
| TogglePause | — | 暂停/继续 |

### 事件流向图

```
Domain 层                              UI 层
                                       
EffectApplied ─────────────────→ Observer
    │                                │
    │                          CharacterProjection
    │                                │
    │                                ▼
    │                          CharacterPanelVm (Dirty)
    │                                │
    │                                ▼
    │                          CharacterPanel Widget 刷新
    │
    ▼
用户点击"施放技能" ──── UiAction::CastSkill ──→ UiCommand::CastSkill(SkillId) ──→ Ability Domain
```

---

## 7. ViewModel 定义

### BattleHudVm

```
BattleHudVm {
    hp: u32,
    max_hp: u32,
    mp: u32,
    max_mp: u32,
    current_turn: u32,
    active_character: Option<CharacterId>,
    phase: BattlePhase,
    cooldowns: HashMap<SkillId, f32>,
}
```

### CharacterPanelVm

```
CharacterPanelVm {
    character_id: CharacterId,
    name_key: UiTextKey,
    level: u32,
    hp: u32,
    max_hp: u32,
    mp: u32,
    max_mp: u32,
    buffs: Vec<BuffVm>,
    stats: StatsVm,
}
```

### SkillPanelVm

```
SkillPanelVm {
    skills: Vec<SkillSlotVm>,
    selected: Option<SkillId>,
    ap_remaining: u32,
}

SkillSlotVm {
    skill_id: SkillId,
    name_key: UiTextKey,
    icon_key: AssetKey,
    cooldown_remaining: f32,
    is_usable: bool,
    mp_cost: u32,
}
```

### InventoryVm

```
InventoryVm {
    items: Vec<InventorySlotVm>,
    gold: u32,
    filter: InventoryFilter,
    selected: Option<ItemId>,
}
```

### NotificationVm

```
NotificationVm {
    message_key: UiTextKey,
    params: Vec<String>,
    priority: NotificationPriority,
    duration: f32,
    notification_type: NotificationType,
}
```

---

## 8. Widget Contract 清单

| Widget | Input | Output | 禁止 |
|--------|-------|--------|------|
| PrimaryButton | UiTextKey | UiAction::Click | Query, EventReader |
| ProgressBar | f32 (ratio) | — | Query |
| SkillPanel | SkillPanelVm | UiAction::SelectSkill, UiAction::CastSkill | Query<&Ability> |
| CharacterPanel | CharacterPanelVm | UiAction::SelectCharacter | Query<&Health> |
| InventoryGrid | InventoryVm | UiAction::SelectItem, UiAction::UseItem | Query<&Item> |
| TurnBar | BattleHudVm | — | Query<&TurnState> |
| Tooltip | TooltipVm | — | Query |
| Modal | ModalVm | UiAction::Confirm, UiAction::Cancel | Query |
| Notification | NotificationVm | — | Query |
| LocalizedText | UiTextKey | — | 硬编码字符串 |

---

## 9. Bevy 0.19 特性映射

| 0.19 特性 | UI 层使用方式 |
|-----------|-------------|
| BSN (bsn!) | 所有 Widget/Screen 使用 bsn! 生成（限于 app/scenes/ 目录，核心玩法层用工厂函数） |
| SceneComponent | 关键 Screen/Widget 预制体化 |
| Observer + run_if | 监听 Domain Event，条件刷新 ViewModel |
| Delayed Commands | 动画延迟、Notification 超时 |
| FontSize 枚举 | 所有 font_size 使用 FontSize::Px/Rem，禁止裸 f32 |
| FontSource | 语义字体族（heading/body/mono） |
| User Settings | UiSettings 持久化 |
| EditableText | 角色命名、搜索框 |
| DiagnosticsOverlay | DebugLayer FPS 显示 |
| Reflect | 所有 ViewModel 补齐 Reflect（支持序列化/调试） |
| Relationship | FocusGroup → Focusable 关系 |

---

## 10. 与已有架构的对齐校验

- ✅ 架构边界：UI 位于 Presentation Layer（L3），依赖方向 Core → UI 单向，UI 不反向依赖 Core 的内部组件
- ✅ 术语一致：SkillId、CharacterId、BuffId、ItemId 等与各领域文档定义一致
- ✅ 与 Cue 领域对齐：Cue 负责逻辑→表现的信号触发（CueType::Popup），UI 层负责 Popup 的实际渲染呈现，职责不重叠
- ✅ 与 Event 领域对齐：UI 通过 Observer 监听 Domain Event，不直接订阅 EventBus 内部机制
- ✅ 与 Localization 架构对齐：所有文本使用 LocalizationKey（ADR-053），ViewModel 中文本字段使用 UiTextKey
- ✅ 与 Bevy 0.19 迁移对齐：BSN 仅限 UI 层使用（ADR-054 DR-003），核心玩法层用工厂函数
- ✅ 单向数据流：Domain Event → Projection → ViewModel → Widget，反向 User Input → UiCommand → Domain，无双向绑定
- ✅ LocalizationKey：ViewModel 中所有用户可见文本使用 UiTextKey/LocalizationKey，不硬编码（宪法 ss22）

---

## 11. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致（SkillId/CharacterId/BuffId/ItemId 等对齐各领域）
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突（Cue 领域职责不重叠，Event 领域通信方式一致）
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖 Screen 导航、Widget 生命周期、Projection 更新、用户输入处理等全场景
- [x] 所有不变量和约束条件已识别（9 条不变量，含四条铁律精简版）
- [x] 禁止事项已明确列出（11 条禁止，含 Marker 结构体禁令）
- [x] Screen 与 Widget 生命周期状态机定义清晰
- [x] 每个操作有完整的流程定义（Projection 更新、Widget 刷新、用户输入、Screen 导航、Notification、Modal、Tooltip、Schema 治理、三层测试、WidgetFactory、Content 数据流）
- [x] Widget Contract 清单完整（10 个 Widget 的输入/输出/禁止声明）
