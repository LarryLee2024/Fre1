---
id: 06-ui.focus-binding
title: Focus and Binding Architecture — 焦点导航与数据绑定
status: code-aligned
updated: 2026-06-21
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - focus
  - binding
  - dirty-flag
  - navigation
  - uibinding
---

# Focus and Binding Architecture — 焦点导航与数据绑定

> **职责**: @presentation-architect | **上游**: ADR-055 §4.2 (Dirty<T>), §11 (UiBinding 反 Marker 模式) | domain rules §1 (Focusable/FocusGroup/Dirty/UiBinding 定义), §5.3 (用户输入处理流程), §PROHIBIT-UI-011 | schema §8 (Focus Schema), §9 (Dirty Flag Schema), §23 (UiBinding Schema)

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 1. 设计目的

本文档覆盖两个紧密关联的子系统——焦点导航和数据绑定，它们的核心设计目标都是**减少重复劳动和提高确定性**：

- **焦点系统**（Focus）为键盘/手柄导航提供确定性数据结构，确保 Widget 焦点移动可预测
- **数据绑定系统**（Binding）将 ViewModel 的变化传递到 Widget，避免每帧全量刷新

两个系统都服务于"50 万行代码规模下 UI 不退化"的目标。

---

## 2. Focus System — 焦点导航系统

### 2.1 体系结构

```
InputSystem（原始输入）
    │
    ▼
UiIntent::NavigateUp/Down/Left/Right
    │
    ▼
FocusSystem（navigation/screen_state.rs）
    │
    ├── FocusManager Resource（全局焦点状态）
    │   ├── active_group: Option<FocusGroupId>    // 当前活跃的焦点组
    │   ├── focused_element: Option<FocusId>      // 当前聚焦的元素
    │   └── previous_elements: HashMap<FocusGroupId, FocusId> // 每组之前聚焦的元素
    │
    ├── Focusable Component（标记可聚焦元素）
    ├── FocusGroup Component（定义组内导航规则）
    └── FocusInputHandler System（处理 Navigate 意图）
```

### 2.2 Focusable Component

```rust
/// 可聚焦组件 — 标记 UI 元素可被手柄/键盘选中
#[derive(Component, Reflect)]
pub struct Focusable {
    /// 焦点 ID（Screen 内唯一，用于焦点定位）
    pub focus_id: FocusId,
    /// 所属焦点组 ID（决定该 Focusable 属于哪个导航组）
    pub group: FocusGroupId,
    /// 优先级（数值越大越优先——Screen 激活时，优先级最高的 Focusable 自动获得焦点）
    pub priority: u32,
}
```

**设计规则**：
- 每个 Focusable 必须在创建时分配 Screen 内唯一的 focus_id
- focus_id 使用递增计数器分配（`FocusId(u32)`），FocusManager 维护计数
- 一个 Widget 可以有 0 个或 1 个 Focusable（部分 Widget 如 LocalizedText 不需要聚焦）
- Focusable 的 group 字段不可为 None——每个 Focusable 必须属于一个 FocusGroup

### 2.3 FocusGroup Component

```rust
/// 焦点组 — 定义一组 Focusable 的导航规则
#[derive(Component, Reflect)]
pub struct FocusGroup {
    /// 组 ID
    pub group_id: FocusGroupId,
    /// 导航模式
    pub navigation: FocusNavigation,
    /// 是否循环导航（越界时回绕到另一端）
    pub wrap: bool,
}

#[derive(Clone, Reflect, Default, PartialEq)]
pub enum FocusNavigation {
    /// 网格导航 — 支持方向键上下左右
    #[default]
    Grid { cols: u32 },
    /// 线性导航 — 仅支持上下（或左右）
    Linear,
    /// 自定义导航规则
    Custom,
}
```

**FocusGroup 创建规则**：

| 场景 | Navigation | wrap | cols | 说明 |
|------|-----------|------|------|------|
| 技能面板（K 个技能单列） | Grid { cols: 1 } | true | 1 | 上下切换技能，循环 |
| 物品网格（4 列物品） | Grid { cols: 4 } | true | 4 | 方向键导航网格，循环 |
| 设置列表 | Linear | false | — | 上下切换设置项 |
| Modal 按钮行 | Grid { cols: N } | true | N 个按钮 | 左右切换按钮 |
| 主菜单按钮列 | Grid { cols: 1 } | true | 1 | 上下切换菜单项 |

### 2.4 焦点导航规则

**Grid 模式导航**（FocusNavigation::Grid）：

```
设当前聚焦元素在虚拟网格中的位置为 (row, col)，
其中 row = index / cols, col = index % cols，
index 是 Focusable 在 FocusGroup 的子元素列表中的顺序索引。

操作:
  上移: 新行 = row - 1（若 row == 0，根据 wrap 决定）
        若 wrap: 新行 = max_row（跳到最底行）
        若不 wrap: 停留在当前行
  下移: 新行 = row + 1（若 row == max_row，根据 wrap 决定）
        若 wrap: 新行 = 0（跳到最顶行）
        若不 wrap: 停留在当前行
  左移: 新列 = col - 1（若 col == 0，根据 wrap 决定）
  右移: 新列 = col + 1（若 col == cols - 1，根据 wrap 决定）
```

**Linear 模式导航**（FocusNavigation::Linear）：

```
操作:
  上移/左移: 选中前一个元素（index - 1）
        若 index == 0:
          若 wrap: 选中最后一个元素
          若不 wrap: 停留在当前
  下移/右移: 选中后一个元素（index + 1）
        若 index == 最后一个:
          若 wrap: 选中第一个元素
          若不 wrap: 停留在当前
```

**Custom 模式**（FocusNavigation::Custom）：

Custom 模式使用自定义的焦点映射表 `HashMap<(FocusId, Direction), FocusId>`。适用于非规则布局（如战斗场地上散落的角色选择）。Custom 映射表由 Screen 在创建时注册到 FocusManager。

### 2.5 Focus 优先级规则

| 规则 | 描述 |
|------|------|
| 同组优先级 | 同一 FocusGroup 内 priority 数值越大越优先 |
| 激活焦点 | Screen 从 Background → Active 时，FocusGroup 中优先级最高的 Focusable 自动获得焦点 |
| 退出恢复 | FocusGroup 失去活跃时记录当前 focused_element，恢复活跃时回到该元素 |
| 跨组 | 焦点导航不跨组，除非显式的 FocusGroup 切换（如关闭 Modal 后恢复底层 Screen 的 FocusGroup） |

### 2.6 Focus 生命周期

```
Screen 激活
    │
    ▼
FocusSystem 激活 Screen 的根 FocusGroup
    │
    ▼
查找 FocusGroup 内 priority 最高的 Focusable
    │
    ▼
设置该 Focusable 为当前焦点
    │
    ▼
...焦点在组内通过 Navigation 意图移动...
    │
    ▼
UiEvent::FocusChanged(focus_id) 广播
    │
    ▼
TooltipService 监听 → 显示新焦点元素的 Tooltip
    │
    ▼
Screen 转为 Background
    │
    ▼
FocusSystem 记录当前 focused_element → FocusManager.previous_elements
    │
    ▼
Screen 恢复 Active
    │
    ▼
FocusSystem 从 previous_elements 恢复焦点（如果存在）
    否则返回组内优先级最高的 Focusable
```

### 2.7 Focus 状态不持久化

| 场景 | 行为 |
|------|------|
| 存档加载后 | 焦点状态重新初始化（Screen 重建后从最高优先级开始） |
| Screen 重建（replace） | 焦点状态丢失，新 Screen 的 FocusGroup 从 priority 最高开始 |
| Overlay（Modal）弹出 | Modal FocusGroup 获得活跃，底层 Screen FocusGroup 记录当前焦点 |
| Overlay 关闭 | 底层 Screen FocusGroup 恢复之前记录的焦点 |

---

## 3. ViewModel → Widget 数据绑定系统

### 3.1 体系结构

```
Projection（更新 ViewModel）
    │
    │  store.battle_hud.get_mut().hp = 85;  // 自动 mark_dirty
    ▼
UiStore（Dirty<T> 标记为 true）
    │
    │  System 检测 Dirty 标记
    ▼
BindingSystem
    │
    │  filter: Query<&Dirty<BattleHudVm>>
    │  if dirty.consume() { /* 刷新 Widget */ }
    ▼
Widget 刷新（仅在 Dirty 为 true 时）
```

### 3.2 Dirty<T> 机制

```rust
/// 脏标记机制 — ViewModel 更新后设置 Dirty，Widget 只在 Dirty 时刷新
#[derive(Component, Debug, Clone, Reflect)]
pub struct Dirty<T: Reflect + Default + Clone + Send + Sync + 'static> {
    /// 内部数据
    pub inner: T,
    /// 是否已被标记为脏（私有，通过 mark_dirty / get_mut 修改）
    is_dirty: bool,
}

impl<T: Reflect + Default + Clone + Send + Sync + 'static> Dirty<T> {
    /// 创建新的 Dirty 包装（初始状态为 dirty，触发首次刷新）
    pub fn new(inner: T) -> Self {
        Self { inner, is_dirty: true }
    }

    /// 手动标记为脏 — 由 Projection 在更新 ViewModel 后调用
    pub fn mark_dirty(&mut self) { self.is_dirty = true; }

    /// 消费脏标记：如果脏则清除并返回 true，否则返回 false
    /// Widget 系统调用，确保每帧最多刷新一次
    pub fn consume(&mut self) -> bool {
        if self.is_dirty {
            self.is_dirty = false;
            true
        } else {
            false
        }
    }

    /// 获取内部数据引用（不触发 dirty）
    pub fn get(&self) -> &T { &self.inner }

    /// 获取内部数据可变引用（自动标记 dirty）
    /// Projection 应通过此方法更新 ViewModel 字段
    pub fn get_mut(&mut self) -> &mut T {
        self.is_dirty = true;
        &mut self.inner
    }
}
```

### 3.3 Dirty<T> 的标准消费流程

```
System 在每个 ViewModel 类型对应的 Dirty<T> 上运行：

fn refresh_hp_bar(
    mut query: Query<&mut Dirty<BattleHudVm>>,
    vm_store: Res<UiStore>,
    mut hp_bar_query: Query<(&mut UiBinding, &mut Style)>,
) {
    for mut dirty_battle in query.iter_mut() {
        if dirty_battle.consume() {
            // ViewModel 已变更，需要刷新
            for (binding, mut style) in hp_bar_query.iter_mut() {
                if *binding == UiBinding::Hp {
                    let ratio = vm_store.battle_hud.hp as f32
                        / vm_store.battle_hud.max_hp as f32;
                    style.width = Val::Percent(ratio * 100.0);
                } else if *binding == UiBinding::Mp {
                    let ratio = vm_store.battle_hud.mp as f32
                        / vm_store.battle_hud.max_mp as f32;
                    style.width = Val::Percent(ratio * 100.0);
                }
            }
        }
    }
}
```

### 3.4 Dirty<T> 全局刷新策略

| 场景 | 行为 |
|------|------|
| 增量更新（Projection 更新单个 ViewModel 字段） | 仅对应的 Dirty<T> 标记为 true |
| Screen 恢复（Background → Active） | 所有关联的 Dirty<T> 标记为 true |
| 存档加载后 | 全部 Dirty<T> 标记为 true，触发首次全量刷新 |
| 主题切换 | 全局 Dirty（不涉及 Widget 内容，Widget 自动读取新 Theme） |

### 3.5 Dirty<T> 注册要求

每个 ViewModel 类型的 `Dirty<T>` 必须在 UiPlugin 中注册：

```rust
app.register_type::<Dirty<BattleHudVm>>();
app.register_type::<Dirty<CharacterPanelVm>>();
app.register_type::<Dirty<SkillPanelVm>>();
app.register_type::<Dirty<InventoryVm>>();
app.register_type::<Dirty<ShopVm>>();
app.register_type::<Dirty<QuestLogVm>>();
```

### 3.6 Dirty<T> 使用规则

| 规则 | 描述 | 违反后果 |
|------|------|---------|
| Projection 更新后必须 mark_dirty() | Projection 写入 UiStore 字段后调用 | Widget 不刷新，显示过时数据 |
| Widget 只在 consume() == true 时刷新 | Widget 读取 Dirty<T> 并判断 | 性能浪费（每帧全量刷新） |
| 禁止手动设置 Dirty 标记 | 只能通过 Projection 设置 dirty | 数据流不可追踪 |
| 加载存档后所有 Dirty = true | 确保首次刷新 | 初始 UI 空白 |

---

## 4. UiBinding — 反 Marker 模式

### 4.1 问题

50 万行规模的项目，如果每个 UI 元素使用一个独立 Marker 结构体（如 `struct HpText;`、`struct ManaText;`、`struct SkillSlot0;`），最终会产生 400+ 结构体，导致：
- Archetype 爆炸（每个 Marker 结构体产生一个新 Archetype）
- AI 代码生成困难（400+ 结构体中选择正确的）
- 重构成本极高（批量重命名 Marker 困难）

### 4.2 UiBinding 枚举设计

```rust
/// UI 绑定标识 — 替代大量独立 Marker 结构体
/// 单一 Archetype 查询 `Query<&UiBinding>` 即可覆盖所有 UI 绑定
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum UiBinding {
    // ── Battle HUD ──
    /// HP 值/条
    Hp,
    /// 最大 HP
    MaxHp,
    /// MP 值/条
    Mp,
    /// 最大 MP
    MaxMp,
    /// 行动点
    Ap,
    /// 最大行动点
    MaxAp,
    /// 回合数
    Turn,
    /// 战斗阶段
    Phase,

    // ── Character Panel ──
    /// 等级
    Level,
    /// 经验值
    Exp,
    /// 角色名称
    Name,
    /// 角色等级文本
    CharacterLevel,

    // ── Skill Panel ──
    /// 第 N 个技能槽（0~N-1）
    SkillSlot(u8),
    /// 冷却
    Cooldown,

    // ── Inventory ──
    /// 第 N 个物品槽（0~N-1）
    ItemSlot(u8),
    /// 金币
    Gold,

    // ── Quest ──
    /// 第 N 个任务条目（0~N-1）
    QuestEntry(u16),

    // ── General ──
    /// 工具提示区域
    Tooltip,
    /// 模态弹窗
    Modal,
    /// 通知
    Notification,
    /// 通用文本
    Text,
    /// 通用图标
    Icon,
}
```

### 4.3 UiBinding 使用模式

**Pattern 1：静态绑定的 Widget（如 HP 条、回合指示器）**

```rust
// BattleScreen 中创建 HP 条
fn create_hp_bar(commands: &mut Commands) {
    commands.spawn((
        Node { .. },
        UiBinding::Hp,          // 标识这是 HP 条元素
        Dirty::<BattleHudVm>::default(),
        // ... 其他组件
    ));
}
```

**Pattern 2：动态绑定的 Widget（如技能槽位、物品槽位）**

```rust
// SkillPanel 中创建第 i 个技能槽
fn create_skill_slot(commands: &mut Commands, index: u8, skill: &SkillSlotVm) {
    commands.spawn((
        Node { .. },
        UiBinding::SkillSlot(index),  // 带参数变体，标识第 i 个技能槽
        // ...
    ));
}
```

**Pattern 3：查询特定绑定**

```rust
// 查询所有 HP 相关的 UI 元素
fn refresh_hp(
    hp_bindings: Query<&UiBinding, (With<UiBinding>, Changed<UiBinding>)>,
) {
    for binding in hp_bindings.iter() {
        match binding {
            UiBinding::Hp => { /* 刷新 HP 值显示 */ }
            UiBinding::MaxHp => { /* 刷新最大 HP 显示 */ }
            _ => {}
        }
    }
}
```

### 4.4 UiBinding 的命名规范

| 绑定名 | UI 元素类型 | 典型 Widget |
|--------|-----------|------------|
| Hp | ProgressBar 的填充 | HpBar |
| MaxHp | LocalizedText（最大值文本） | HpBar label |
| Mp | ProgressBar 的填充 | MpBar |
| SkillSlot(i) | IconButton | SkillSlot |
| ItemSlot(i) | IconButton + 数量文本 | InventorySlot |
| Gold | LocalizedText | GoldDisplay |
| CharacterLevel | LocalizedText | CharacterPanel |
| Name | LocalizedText | CharacterPanel |
| Tooltip | Panel | TooltipOverlay |
| Modal | Panel | ModalOverlay |
| Notification | Panel | NotificationOverlay |
| Text | LocalizedText | 通用文本组件 |
| Icon | Image | 通用图标组件 |

### 4.5 UiBinding 使用约束

| 规则 | 描述 |
|------|------|
| 同一 Screen 内不重复 | 同一 Screen 内的 UiBinding 值不重复（带参数变体使用不同索引） |
| 静态 Widget 使用无参变体 | Hp、Mp、Turn 等固定数量的绑定使用无参变体 |
| 动态 Widget 使用有参变体 | SkillSlot(u8)、ItemSlot(u8) 等动态数量的绑定使用有参变体 |
| 禁止为特定 UI 创建新 Marker | 新需求优先使用现有 UiBinding 变体或新增枚举变体，而非独立 Marker |

---

## 5. Focus 与 Binding 的交互

### 5.1 两者关系

Focus 系统和 Binding 系统是正交的：
- Focus 系统关注**交互导航**（用户如何从一个 Widget 移动到另一个 Widget）
- Binding 系统关注**数据流向**（ViewModel 如何传递给 Widget）

一个 Widget 可以同时具备 Focusable 和 UiBinding 两个组件。

### 5.2 典型交互场景

| 场景 | Focus | Binding |
|------|-------|---------|
| 用户移动焦点到 HP 条 | FocusSystem 更新 active Focusable | 无（HP 条不可交互，Focusable 不存在） |
| 用户移动焦点到技能按钮 | FocusSystem 更新 active Focusable | 技能按钮持有 UiBinding::SkillSlot(i) |
| 用户选择技能 | FocusSystem 将 Confirm 意图 → Widget | Widget 通过 UiBinding::SkillSlot(i) 找到对应 ViewModel 数据 |
| ViewModel 更新（冷却变更） | 无影响 | Dirty<SkillPanelVm> 标记 → Widget 刷新 |

---

## 6. 验证规则

### 6.1 Focus 系统规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| FOC-VAL-01 | 每个 FocusGroup 至少 1 个 Focusable | Screen 构建时检查组内 Focusable 数量 |
| FOC-VAL-02 | FocusId 在 Screen 内唯一 | 创建 Focusable 时确保 focus_id 不重复 |
| FOC-VAL-03 | 焦点导航不跨组 | FocusManager 不处理跨组焦点切换（除非显式切换） |
| FOC-VAL-04 | Screen 退出时记录焦点 | Screen Background 时保存当前 focused_element |

### 6.2 Binding 系统规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| BIND-VAL-01 | Widget 刷新仅在 consume() == true 时 | Widget 刷新系统调用 consume() 后才执行刷新逻辑 |
| BIND-VAL-02 | 禁止手动设置 is_dirty | 只有 Projection 通过 UiStore.mark_dirty() 设置 dirty |
| BIND-VAL-03 | UiBinding 使用枚举而非独立 Marker | 代码审查，禁止 `struct HpText;` 等独立 Marker 结构体 |
| BIND-VAL-04 | 静态 Widget 使用无参变体 | 固定数量的 UI 元素（HP/MP 条）不使用带参数 UiBinding |

---

*本文档由 @presentation-architect 维护。新增 Focus 或 Binding 机制需经过架构审查。*
