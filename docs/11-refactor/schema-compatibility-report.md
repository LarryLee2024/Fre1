---
id: 11-refactor.schema-compatibility-report
title: Data Schema Compatibility Report for Screen Spec
status: partial
created: 2026-06-22
owner: data-architect
tags:
  - schema-review
  - ui
  - screen-spec
  - data-architect
---

# Screen Spec 数据 Schema 兼容性报告

## 参考来源

| 来源 | 权威性 | 备注 |
|------|--------|------|
| `src/ui/view_models/battle_hud.rs` | **实际代码** | MVP 实现（3 个 ViewModel） |
| `src/ui/view_models/character_panel.rs` | **实际代码** | |
| `src/ui/view_models/skill_panel.rs` | **实际代码** | |
| `src/ui/binding/ui_binding.rs` | **实际代码** | UiBinding 枚举 |
| `src/ui/binding/dirty_flag.rs` | **实际代码** | Dirty<T> 实现 |
| `src/ui/view_models/mod.rs` | **实际代码** | UiStore（3 个字段） |
| `docs/04-data/capabilities/ui-presentation-schema.md` | **Schema 文档（草稿）** | 完整设计，与代码不一致 |
| `docs/06-ui/04-data-flow/projection-viewmodel.md` | **与代码对齐** | 匹配代码，参考视图 |
| `docs/06-ui/02-design-system/widget-composites.md` | **与代码对齐** | 复合 widget 定义 |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` | **计划** | Screen Spec 结构 + widget-id-map |

---

## 1. ViewModel 类型一致性

### 1.1 关键发现：BattleHudVm 的三个分歧定义

存在三个互不兼容的 `BattleHudVm` 版本。Screen Spec 模板的事件契约引用了**与代码对齐的文档**中的字段，但 **schema 文档**有一个完全不同的结构。

| 字段 | 实际代码 | projection-viewmodel.md | ui-presentation-schema.md | Screen Spec 是否使用？ |
|------|---------|------------------------|--------------------------|----------------------|
| `hp` | `f32` | `f32` | `u32` | 是——事件契约 |
| `max_hp` | `f32` | `f32` | `u32` | 隐式 |
| `mp` | `f32` | `f32` | `u32` | 隐式 |
| `max_mp` | `f32` | `f32` | `u32` | 隐式 |
| `ap` | `f32` | `f32` | **缺失**（使用 `action_points: u32`） | 是——BattleScreen TopBar |
| `max_ap` | `f32` | `f32` | **缺失**（使用 `max_action_points: u32`） | 隐式 |
| `turn_number` | `u32` | `u32` | **缺失**（使用 `current_turn: u32`） | 是——事件契约 |
| `phase_key` | `&'static str` | `&'static str` | **缺失**（使用 `phase: BattlePhaseVm`） | 是——事件契约 |
| `active_character` | **缺失** | **缺失** | `Option<CharacterId>` | 否 |
| `cooldowns` | **缺失** | **缺失** | `HashMap<SkillId, f32>` | 否 |

**裁定**：代码库中三个互不兼容的 BattleHudVm 定义。ui-presentation-schema.md §4.2 是一个**与真实代码不匹配的设计文档**，其 `active_character`/`cooldowns` 字段尚未实现。如果 Screen Spec 遵循实际代码则是安全的，但 schema 文档必须被协调。

### 1.2 Screen Spec 事件契约字段验证

执行计划的事件契约（§3.8）使用了以下字段引用：

```yaml
vm_update: BattleHudVm.hp ← damage_value
vm_update: BattleHudVm.turn_number += 1
vm_update: BattleHudVm.phase_key = "ui.battle.phase.player"
```

所有三个字段（**hp**、**turn_number**、**phase_key**）在实际代码中都存在。这些引用与当前实现**兼容**。

然而，`damage_value` 是一个未定义的变量——Screen Spec 模板应明确指定数据源（例如 `DamageApplied.damage_amount`，类型 `f32`）。

### 1.3 CharacterPanelVm 验证

| 字段 | 实际代码 | Screen Spec 引用 | 兼容？ |
|------|---------|-----------------|--------|
| `character_id: u32` | 是 | 隐式（CharacterCard） | 是 |
| `name_key: &'static str` | 是 | 隐式 | 是 |
| `level: u32` | 是 | 隐式（CharacterCard） | 是 |
| `hp: f32` | 是 | 隐式（HP 条） | 是 |
| `max_hp: f32` | 是 | 隐式 | 是 |
| `mp: f32` | 是 | 隐式（MP 条） | 是 |
| `max_mp: f32` | 是 | 隐式 | 是 |

BattleScreen Widget Tree 引用了 `CharacterCard` 和 `CharacterStatusPanel`。当前的 `CharacterPanelVm` **没有**足够的字段来支持 `CharacterStatusPanel`，后者需要 `buffs: Vec<BuffVm>` 和 `ap_current/ap_max`（参见 widget-composites.md §3.2 Props）。这是一个**缺口**——复合 widget 的 Props 超过了当前 ViewModel 的字段。

### 1.4 SkillPanelVm 验证

| 字段 | 实际代码 | Screen Spec 引用 | 兼容？ |
|------|---------|-----------------|--------|
| `skills: HashMap<u32, SkillSlotVm>` | 是 | Widget Contract 引用 SkillPanel | 是 |
| `selected: Option<SkillId>` | **缺失** | 隐式（技能选择） | **缺口** |
| `ap_remaining: u32` | **缺失** | 隐式（AP 消耗检查） | **缺口** |
| `max_ap: u32` | **缺失** | 隐式 | **缺口** |

代码中的 SkillPanelVm 是 `HashMap<u32, SkillSlotVm>` 的薄包装，没有 `selected` 或 `ap_remaining` 字段，但 widget-composites.md 的 SkillPanel Props 和 SkillSlot Contract 都需要这些字段进行交互验证。

### 1.5 Screen Spec Widget 引用但未定义的 ViewModel

Screen Spec 将定义引用复合 widget 的 Widget Tree。这些 widget 具有需要 ViewModel 的 Props，而这些 ViewModel 当前在代码中不存在：

| 复合 Widget | 所需 ViewModel | 存在？ | 引用位置 |
|------------|----------------|-------|----------|
| CharacterPortrait（分子） | CharacterPortraitVm | **否** | widget-composites.md §2.2 |
| CharacterStatusPanel（有机体） | CharacterStatusPanelVm | **否** | widget-composites.md §3.2 |
| TurnOrderBar（有机体） | TurnOrderBarVm | **否** | widget-composites.md §3.4 |
| TurnIndicator（分子） | TurnIndicatorVm | **否** | widget-composites.md §2.8 |
| InventoryGrid（有机体） | InventoryGridVm | **否** | widget-composites.md §3.5 |
| QuestEntry（分子） | QuestEntryVm | **否** | widget-composites.md §2.4 |
| QuestLog（有机体） | QuestLogVm | **仅 schema** | widget-composites.md §3.6 |
| DialogueChoice（分子） | DialogueChoiceVm | **否** | widget-composites.md §2.5 |
| DialoguePanel（有机体） | DialoguePanelVm | **否** | widget-composites.md §3.7 |
| ShopItemCard（分子） | ShopItemCardVm | **否** | widget-composites.md §2.6 |
| ShopPanel（有机体） | ShopPanelVm（ShopVm 仅存在于 schema） | **仅 schema** | widget-composites.md §3.8 |
| BuffIcon（分子） | BuffVm | **仅 schema** | widget-composites.md §2.7 |

**影响**：BattleScreen、InventoryScreen、ShopScreen、QuestLogScreen 和 SettingsScreen 的 Screen Spec 将引用其 ViewModel 不作为可实现的代码结构体存在的 widget。Spec 仍然可以编写（它们描述的是布局而非实现），但 ViewModel 定义阶段必须在实现之前完成。

---

## 2. UiBinding 枚举完整性

### 2.1 实际代码 vs 文档不匹配

实际代码（`src/ui/binding/ui_binding.rs`）拥有最完整的 UiBinding：

```rust
pub enum UiBinding {
    // Battle HUD: Hp, MaxHp, Mp, MaxMp, Ap, MaxAp, Turn, Phase
    // Character Panel: Level, Exp, Name, CharacterLevel
    // Skill Panel: SkillSlot(u8), Cooldown
    // Inventory: ItemSlot(u8), Gold
    // Quest: QuestEntry(u16)
    // General: Tooltip, Modal, Notification, Text, Icon
}
```

Schema 文档（`ui-presentation-schema.md §23`）**缺失**了 `CharacterLevel`、`Text` 和 `Icon`——这些在实际代码中存在，但不在数据 schema 文档中。

### 2.2 Screen Spec widget-id-map 兼容性

执行计划中的 widget-id-map（§3.9）将 widget_id 映射到 UiBinding 变体：

| widget_id | 映射的 Binding | 存在？ | 问题 |
|-----------|---------------|--------|------|
| `turn_indicator` | `UiBinding::Turn` | 是 | OK |
| `phase_label` | `UiBinding::Phase` | 是 | OK |
| `hp_bar` | `UiBinding::Hp` | 是 | OK |
| `mp_bar` | `UiBinding::Mp` | 是 | OK |
| `title_text` | `UiBinding::Text` | 是（代码中） | Schema 文档中缺失 |
| `buff_icons_0` | `UiBinding::BuffSlot(0)` | **否** | **缺少变体** |
| `char_panel` | `UiBinding::None` | **否** | **缺少变体** |

**问题 1 — `UiBinding::BuffSlot(u8)` 不存在。**
枚举中没有任何地方有 BuffSlot 变体。Buff 图标当前使用 `StatusIcon` widget（widget-atoms.md §11.2）和 `BuffVm` ViewModel，但没有 UiBinding 变体。widget-id-map 需要要么：
- 向 UiBinding 添加 `BuffSlot(u8)`（推荐——与 `SkillSlot(u8)`/`ItemSlot(u8)` 模式一致）
- 或者将 buff 图标映射到通用的 `Icon` 绑定

**问题 2 — `UiBinding::None` 不存在。**
映射表对容器 widget（root、top_bar、battle_area、char_panel、action_menu）广泛使用 `UiBinding::None`。但不存在 `None` 变体——没有绑定的 widget 根本没有 `UiBinding` 组件。widget-id-map 应使用 `(none)` 或 `—` 而不是 `UiBinding::None`。

**问题 3 — 缺少 Screen 级别的绑定。**
Screen Spec 使用 `UiBinding::None` 引用 `end_turn_btn`。但 screens.md（§2.4）将 EndTurnButton 映射到 `BattleAction::EndTurn`。如果需要 UiBinding 变体，可以将 `UiBinding::EndTurn` 添加到 Battle HUD 类别。目前，它由 `BattleAction` 组件标记处理。这是可以接受的——不是每个按钮都需要 UiBinding。

### 2.3 参数化变体充分性

当前的参数化模式（`SkillSlot(u8)`、`ItemSlot(u8)`、`QuestEntry(u16)`）已足够。Screen Spec 不需要新的参数化模式。`Chars(u8)` 维度已由每个实体的绑定覆盖。

---

## 3. Dirty<T> 机制兼容性

### 3.1 当前实现

实际代码（`src/ui/binding/dirty_flag.rs`）将 `Dirty<T>` 作为 Widget 实体上的 `Component` 实现：

```rust
pub struct Dirty<T: Reflect + Default + Clone + Send + Sync + 'static> {
    pub inner: T,
    is_dirty: bool,
}
```

关键行为：
- `get_mut()` 自动标记为 dirty
- `consume()` 返回 true 一次然后清除
- 初始状态为 dirty（触发首次渲染）

### 3.2 Screen Spec 事件契约模式

执行计划的事件契约使用类型参数化的标记模式：
```yaml
side_effect: mark_dirty::<BattleHudVm>()
```

这在**概念上是兼容的**，但当前实现不支持将 `mark_dirty::<BattleHudVm>()` 作为独立函数。模式必须是：

```rust
// Projection 更新特定的 UiStore 字段，而不是通用类型
store.battle_hud.get_mut().hp = trigger.new_hp;
// get_mut() 调用会自动将 Dirty<BattleHudVm> 标记为 dirty（如果该组件
// 存在于 BattleHud 实体上）。
```

事件契约的 `mark_dirty::<BattleHudVm>()` 是文档速记。只要 Screen Spec 理解投影模式（UiStore 上的直接字段突变），就没有不兼容。

### 3.3 按区域状态映射缺口

**重要缺口**：Screen Spec 将按区域状态映射（`Loading/Empty/Normal/Error`）定义为要求（§3.7，§4 Step 4 field 8）。当前的 `Dirty<T>` 机制只有二元的 dirty/clean 标记。**没有用于按区域 Loading/Empty/Error 状态跟踪的机制。**

当前的 ViewModel 没有"此区域正在加载"或"此区域为空"的概念。Widget 当前使用默认的 `Default::default()` ViewModel 值渲染，直到发生真正的投影。

**建议**：这需要单独处理。选项包括：
- 短期：在 Screen Spec 中记录 Loading/Empty/Error 状态尚不被数据层支持，当前行为是渲染默认 ViewModel 值。
- 长期：向 ViewModel 字段添加 `OptionalVm<T>` 包装器或 `RegionState` 枚举，可以是 Loading/Empty/Error。

### 3.4 Projection→Dirty 数据路径

现有的数据路径是：
```
Domain Event → Observer → Projection → UiStore.field.get_mut() → auto mark_dirty()
```

这与所有 Screen Spec 要求兼容。无需修改。

---

## 4. UiStore 结构完整性

### 4.1 当前 UiStore（实际代码）

```rust
pub struct UiStore {
    pub battle_hud: BattleHudVm,       // 存在
    pub character_panel: CharacterPanelVm,  // 存在
    pub skill_panel: SkillPanelVm,      // 存在
}
```

### 4.2 Screen Spec 需求

6 个 Screen Spec 将引用以下 UiStore 字段：

| Screen | 需要的 UiStore 字段 | 存在？ | 优先级 |
|--------|-------------------|--------|--------|
| BattleScreen | `battle_hud` | 是 | P0 |
| BattleScreen | `skill_panel` | 是 | P0 |
| BattleScreen | `character_panel` | 是 | P0 |
| MainMenuScreen | 无 | 不适用 | P0 |
| InventoryScreen | `inventory` | **否** | P1 |
| SettingsScreen | 无（直接读取 UiSettings） | 不适用 | P1 |
| ShopScreen | `shop` | **否（仅 schema）** | P1 |
| SaveLoadScreen | 无（读取 Save Domain） | 不适用 | P1 |
| QuestLogScreen | `quest_log` | **否（仅 schema）** | P1 |

### 4.3 缺口分析

| 字段 | 状态 | 影响 |
|------|------|------|
| `inventory: InventoryVm` | 不在代码中，在 schema 文档中定义 | InventoryScreen Spec 可以编写但无法实现。InventoryVm 必须在实现前编码。 |
| `shop: ShopVm` | 不在代码中，在 schema 文档中定义 | ShopScreen Spec 可以编写（Spec 先于代码是设计意图），但 ShopVm 必须在实现前编码。 |
| `quest_log: QuestLogVm` | 不在代码中，在 schema 文档中定义 | QuestLogScreen Spec 可以编写，但 QuestLogVm 必须在实现前编码。 |
| `notification_queue: Vec<NotificationVm>` | NotificationVm 在代码中存在（`src/ui/overlay/notification.rs`），但不在 UiStore 中 | NotificationVm 作为 overlay 组件存在，但 UiStore 中没有队列。schema 文档的 `notification_queue` 字段未实现。 |
| `modal_stack: Vec<ModalVm>` | ModalVm 仅存在于 schema 文档中 | 任何地方都未实现。Modal overlay 使用 ModalService 模式，而非 UiStore。 |

**关键发现**：Schema 文档 `ui-presentation-schema.md` §4.1 定义了包含 8 个字段的更丰富的 UiStore。实际代码有 3 个字段。Screen Spec 计划假设完整的 UiStore 存在，但实际上不存在。

---

## 5. 兼容性结论

### 5.1 兼容（无需修改）

- **Screen Spec 事件契约使用的 BattleHudVm 字段**（`hp`、`turn_number`、`phase_key`）——全部在实际代码中存在
- **CharacterPanelVm 结构**——匹配 Screen Spec 隐式的 CharacterCard 需求
- **UiBinding 核心变体**（Hp、Mp、Ap、Turn、Phase、SkillSlot、Level、Exp、Name、Text）——全部存在
- **Dirty<T> 机制**——Projection → ViewModel → mark_dirty 路径兼容
- **UiStore 字段访问模式**——所有 3 个现有字段的直接结构体字段访问可用
- **Projection→Observer 模式**——与事件契约连线完全兼容

### 5.2 需要文档澄清

| 项目 | 操作 |
|------|------|
| widget-id-map 中的 UiBinding `None` 用法 | 将 `UiBinding::None` 替换为 `(none)` 或 `—`，以避免暗示不存在的枚举变体 |
| 事件契约中的 `mark_dirty::<T>()` 泛型语法 | 记录实际模式是 `store.field.get_mut() → auto mark_dirty()`，而不是泛型函数调用 |
| 代码与 schema 文档之间的 ViewModel 类型分歧（`f32` vs `u32`、缺失/枚举字段） | 更新 `ui-presentation-schema.md` 以匹配实际代码，或者如果计划基于枚举的阶段则规划迁移 |
| schema 文档中缺失的 UiBinding `Text`/`Icon`/`CharacterLevel` | 向 `ui-presentation-schema.md §23` 添加缺失的变体 |

### 5.3 Screen Spec 实现前需要的增量变更

| 需求 | 优先级 | 操作 |
|------|--------|------|
| `UiBinding::BuffSlot(u8)` | **P0**（widget-id-map 引用它） | 在 widget-id-map 定稿前向 `src/ui/binding/ui_binding.rs` 添加变体 |
| `SkillPanelVm.selected: Option<SkillId>` | **P1**（widget-composites §3.1 Props） | 向 `src/ui/view_models/skill_panel.rs` 添加字段 |
| `SkillPanelVm.ap_remaining: u32` + `max_ap: u32` | **P1** | 添加技能可用性检查字段 |
| `CharacterPanelVm.buffs: Vec<BuffVm>` | **P1**（CharacterStatusPanel 需要） | 要么添加到现有 Vm，要么创建 CharacterStatusPanelVm |

### 5.4 Screen Spec 实现前需要的新实现

| ViewModel | 被谁需要 | 参考 |
|-----------|---------|------|
| CharacterStatusPanelVm | BattleScreen CharacterCard/StatusPanel | widget-composites.md §3.2 |
| TurnOrderBarVm + TurnIndicatorVm | BattleScreen TurnOrderBar | widget-composites.md §3.4、§2.8 |
| CharacterPortraitVm | BattleScreen CharacterCard | widget-composites.md §2.2 |
| InventoryVm + InventorySlotVm + 过滤器 | InventoryScreen | schema doc §4.5 |
| ShopVm + ShopSlotVm | ShopScreen | schema doc §4.6 |
| QuestLogVm + QuestSlotVm + 过滤器 | QuestLogScreen | schema doc §4.7 |
| BuffVm | 多个 widget | schema doc §4.3（在 CharacterPanelVm 内） |

### 5.5 数据 Schema 文档操作项

| 文档 | 问题 | 操作 |
|-----|------|------|
| `docs/04-data/capabilities/ui-presentation-schema.md` §4.2 | BattleHudVm 使用 `u32` + `BattlePhaseVm` + 缺少 ap 字段 | **与代码协调**：代码使用 `f32` + `&'static str phase_key`。决定是否迁移到枚举。 |
| `docs/04-data/capabilities/ui-presentation-schema.md` §4.3 | CharacterPanelVm 有 11 个字段，代码有 7 个 | 同步到实际代码，将扩展字段移到 CharacterStatusPanelVm |
| `docs/04-data/capabilities/ui-presentation-schema.md` §23 | 缺少 CharacterLevel、Text、Icon 变体 | 添加缺失的变体 |
| `docs/06-ui/04-data-flow/projection-viewmodel.md` §3.4 | `&'static str` 类型的 phase_key 有问题 | `&'static str` 无法序列化。应为 `UiTextKey`（即 `String`）以支持未来的保存/回放兼容性。 |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` §3.9 | `UiBinding::None` 用法 | 替换为 `(no binding)` 标记 |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` §3.9 | `UiBinding::BuffSlot(0)` 用法 | 首先向 UiBinding 添加 BuffSlot 变体 |

### 5.6 风险总结

| 风险 | 严重程度 | 缓解措施 |
|------|---------|---------|
| Screen Spec 事件契约引用了复合 widget 不存在的 ViewModel 字段 | **高**——事件契约是 P1，但复合 widget 是 P0 | 分阶段交付 Screen Spec：先写 Spec（P0），然后实现缺失的 ViewModel（P0.5），再实现代码（P1） |
| 按区域 Loading/Empty/Error 状态没有数据机制 | **中**——Screen Spec 要求它，数据层不支持 | 在 Spec 中记录为"未来：暂不支持"，短期内使用默认值 |
| UiStore 只有 3 个字段，Screen Spec 需要 6 个以上 | **低**——加法字段模型，UiStore 是单个 Resource | 在实现每个 Screen 时逐个添加字段 |
| ViewModel 使用 `&'static str` 作为文本 key（不可序列化） | **低**——ViewModel 不会持久化，但这会阻碍未来的调试工具 | 在单独的重构中替换为 `UiTextKey`（基于 String） |
| 三个分歧的 BattleHudVm 定义 | **中**——导致 AI 代码生成混淆 | 将实际代码定义为 SSOT，将 schema 文档标记为需要更新的草稿 |

### 5.7 Screen Spec 阶段的总体建议

**现在可以编写 Screen Spec**，不会因数据层缺口而阻塞，因为 Spec 描述的是布局和契约，而非实现。然而，在 Spec 实现之前必须完成以下数据前置条件：

1. **在任何 Spec 实现之前**：添加 `UiBinding::BuffSlot(u8)` 并记录 `(no binding)` 是正确的容器标记。
2. **在 BattleScreen 实现之前**：确保定义了 CharacterStatusPanelVm、TurnOrderBarVm（即使是桩实现）。
3. **在 InventoryScreen/ShopScreen 实现之前**：必须编码 InventoryVm、ShopVm 及相关过滤/排序类型（使用 schema 文档作为蓝图）。
4. **在 QuestLogScreen 实现之前**：必须编码 QuestLogVm。
5. **解决 schema 文档分歧**：协调 `ui-presentation-schema.md` 与实际代码，以确保从文档生成 AI 代码时产生正确的实现。

---

## 附录：ViewModel 字段交叉引用表

### 当前实际代码 ViewModel

```
BattleHudVm
  hp: f32
  max_hp: f32
  mp: f32
  max_mp: f32
  ap: f32
  max_ap: f32
  turn_number: u32
  phase_key: &'static str

CharacterPanelVm
  character_id: u32
  name_key: &'static str
  level: u32
  hp: f32
  max_hp: f32
  mp: f32
  max_mp: f32

SkillPanelVm
  skills: HashMap<u32, SkillSlotVm>

SkillSlotVm
  skill_id: u32
  name_key: &'static str
  cooldown_remaining: u32
  max_cooldown: u32
  is_usable: bool
  ap_cost: u32

TooltipVm          （位于 src/ui/overlay/tooltip.rs）
NotificationVm     （位于 src/ui/overlay/notification.rs）
```
