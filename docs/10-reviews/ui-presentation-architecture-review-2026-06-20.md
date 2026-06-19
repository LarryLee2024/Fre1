---
id: 10-reviews.ui-presentation-architecture-review
title: UI 表现层架构评审报告
status: active
owner: presentation-architect
created: 2026-06-20
reviewed_documents:
  - docs/01-architecture/40-cross-cutting/ADR-055-ui-presentation-architecture.md
  - docs/02-domain/capabilities/ui-presentation.md
  - docs/04-data/capabilities/ui-presentation-schema.md
  - docs/09-planning/bevy-0.19-migration-v3-aggressive.md
  - docs/09-planning/done/new_bevy-0.19-migration-master-plan.md
  - docs/09-planning/done/new_bevy-0.19-phase1-aggressive.md
  - docs/09-planning/done/new_bevy-0.19-phase2-deep-refactor.md
reference_standard: docs/ai_ignore_this_dir/10ui.md
---

# UI 表现层架构评审报告

> **评审角色**: Presentation Architect（表现层架构师）
> **评审基准**: `docs/ai_ignore_this_dir/10ui.md`（50万行+ / 5年+维护 / AI深度参与 / Bevy 0.19+ / SRPG）
> **评审范围**: ADR-055、UI 领域规则、UI 数据 Schema、迁移计划中 UI 相关部分
> **评审日期**: 2026-06-20

---

## 0. 评审总评

**总体评级**: 🟢 B+（良好，有改进空间）

文档体系在架构方向上与 `10ui.md` 高度对齐，核心原则（单向数据流、Projection 防火墙、Widget Contract、Screen/Widget 分离、Overlay 独立、Style Token 化）均已正确采纳。但在以下维度存在差距：

| 维度 | 评级 | 说明 |
|------|------|------|
| 核心原则对齐 | 🟢 A | 四条铁律、单向数据流、Projection 防火墙完整覆盖 |
| 架构层次完整性 | 🟢 A- | L3 UI 层定位、目录结构、依赖方向均正确 |
| Bevy 0.19 特性采用 | 🟡 B+ | BSN/SceneComponent/FontSize 有提及但深度不足 |
| 服务层设计 | 🟡 B | TooltipService/ModalService/NotificationService 有提及但缺设计细节 |
| UI Application Layer | 🟡 B- | 10ui.md 第13层"UI 应有自己的 Application Layer"吸收不完整 |
| 输入与显示分离 | 🟡 C+ | 10ui.md 第14层"UI 输入与 UI 显示彻底分离"吸收不足 |
| UI Debug Overlay | 🟡 C | 10ui.md 第23层提及，文档中仅在 DevTools 中简单提及 |
| UI Cache Layer | 🟡 C+ | 10ui.md 第21层提及，ViewModel 缓存概念有但未显式设计 |
| 迁移计划 UI 部分 | 🟡 C | 迁移计划中 UI 层迁移步骤严重不足 |

---

## 1. 逐文档评审

### 1.1 ADR-055 评审

#### 🟢 正确采纳的内容

| # | 10ui.md 原则 | ADR-055 对应 | 评价 |
|---|-------------|-------------|------|
| 1 | UI 不直接读 Domain | §5.1 UI Query 禁止规则 | 完整采纳 |
| 2 | Widget 不持有 Entity | §5.2 | 完整采纳 |
| 3 | UI 动画不写业务逻辑 | §5.3 | 完整采纳 |
| 4 | Screen 组合 Widget | §5.5 | 完整采纳 |
| 5 | Overlay 独立 | §6 UI Root 分层 | 完整采纳 |
| 6 | Screen Stack 导航 | §2 navigation/ | 完整采纳 |
| 7 | Style Token 化 | §2 theme/ | 完整采纳 |
| 8 | Localization | §2 localization/ | 完整采纳 |
| 9 | Focus System | §2 focus/ | 完整采纳 |
| 10 | Dirty Flag | §2 binding/dirty_flag.rs | 完整采纳 |
| 11 | Persistent Widget | §8 | 完整采纳 |
| 12 | UiBinding 反 Marker | 后续补充 | 完整采纳 |
| 13 | WidgetFactory trait | 后续补充 | 完整采纳 |
| 14 | UI Schema 治理 | 后续补充 | 完整采纳 |
| 15 | UI 三层测试 | 后续补充 | 完整采纳 |
| 16 | UI 与 Content 数据流 | 后续补充 | 完整采纳 |

#### 🔴 CRITICAL：遗漏的关键设计

**C1: UI Application Layer 缺失**

10ui.md 第13层明确指出：

> UI 应该有自己的 Application Layer
> ```
> UI → UI Application → Game Application → Domain
> ```
> 而不是：
> ```
> UI → Domain
> ```

ADR-055 中 `application/` 目录包含 `command.rs`、`event.rs`、`intent.rs`，但**缺少 `ui_application.rs`**——即 UI 层自己的 Application 层，负责将 UiAction 路由到正确的 Domain Application。

当前设计中 `UiCommand → GameCommand` 转换器直接在 `command.rs` 中，缺少中间的 UI Application 层来处理：
- 多输入源复用（鼠标/键盘/手柄/AI自动战斗）
- UiAction 的业务语义验证（如"当前能否施放技能"）
- UI 层内部的状态编排（如"打开技能面板→选择技能→选择目标"的多步流程）

**建议**：在 `src/ui/application/` 中新增 `ui_application.rs`，定义 `UiApplication` trait 或枚举，作为 UiAction → UiCommand 的中间层。

---

**C2: 输入与显示彻底分离不足**

10ui.md 第14层明确指出：

> ```
> Input Layer → Intent Layer → Presentation Layer
> ```
> 这样以后键盘/手柄/触摸/SteamDeck 不用重写 UI

ADR-055 中定义了 `UiIntent`（`application/intent.rs`），但：
- 缺少 **Input → Intent 的映射规范**：硬件输入如何转为 UiIntent 没有详细设计
- 缺少 **Intent → Action 的路由规范**：UiIntent 如何路由到当前 FocusGroup 中的 Widget
- `UiIntent` 与 `InputAction`（Infra/Input 层）的关系仅一句话带过

**建议**：在 ADR-055 或领域规则中新增 §"输入三层架构"：
1. InputAction（硬件语义：Space/Click/GamepadA）→ Infra/Input 层
2. UiIntent（业务语义：OpenSkillPanel/SelectTarget/ConfirmAction）→ UI Application 层
3. UiAction（Widget 语义：Click/Select/Hover）→ Widget 层

---

#### 🟡 HIGH：设计深度不足

**H1: TooltipService 设计缺失**

10ui.md 第15层明确指出：

> Tooltip 不属于 Widget
> 正确：TooltipService 统一管理，类似浏览器 ContextMenu

ADR-055 将 `tooltip/` 放在 `widgets/` 下，这与 10ui.md 的建议矛盾。Tooltip 是跨 Widget 的全局服务，不应是某个 Widget 的子组件。

**建议**：将 `tooltip/` 从 `widgets/` 移到 `overlay/` 或 `services/`，TooltipService 作为独立服务。

**H2: NotificationService / ModalService 设计缺失**

10ui.md 第16-17层强调：
- NotificationService 统一入口，所有 Toast/Banner/Popup 走一个入口
- ModalService 统一管理，防止"确认框套确认框"

ADR-055 将 `modal/` 和 `notification/` 放在 `widgets/` 下，但它们是**全局服务**而非可复用 Widget。领域规则文档（§5.5/5.6）有流程定义，但缺少服务接口设计。

**建议**：在 `src/ui/overlay/` 或新增 `src/ui/services/` 中定义 NotificationService 和 ModalService 的接口。

**H3: UI Debug Overlay 设计不足**

10ui.md 第23层：

> 建议第一天就做。显示当前 Screen/Modal/Tooltip/Focus/VM 实时状态

ADR-055 §6 中 `DebugLayer` 仅一句话提及，迁移计划中 DiagnosticsOverlay 只显示 FPS。缺少：
- 当前 ScreenStack 状态显示
- 当前 ViewModel 快照
- 当前 Focus 路径
- 当前 Modal/Tooltip 状态

**建议**：在 `src/ui/overlay/debug.rs` 中设计 `UiDebugOverlay`，显示上述信息。

---

#### 🟡 MEDIUM：设计细节待完善

**M1: UiStore 与独立 Resource 的取舍**

10ui.md 第2层推荐 `UiStore` 统一管理所有 ViewModel（类似 Redux Store）。ADR-055 采纳了 UiStore，但数据 Schema 中同时存在 `Res<BattleHudVm>` 和 `Res<UiStore>` 两种访问模式，需要明确：
- Widget 到底从 `Res<UiStore>` 读取还是从 `Res<XXXVm>` 读取？
- 如果从 `Res<UiStore>` 读取，`Res<XXXVm>` 是否多余？

**建议**：统一为 `Res<UiStore>` 单一入口，Widget 通过 `ui_store.battle_hud` 访问。删除独立的 `Res<XXXVm>` 注册。

**M2: BSN 在 UI 层的使用规范不足**

ADR-055 §4.1 提到 BSN 全面使用，但缺少：
- BSN 与 WidgetFactory 的关系：WidgetFactory::create 是用 bsn! 还是 spawn_scene？
- BSN 与 SceneComponent 的关系：Screen 是否用 SceneComponent 定义？
- BSN 的嵌套限制：bsn! 内能否引用其他 Widget 的 bsn!？

**建议**：新增 §"BSN UI 层使用规范"，明确 bsn! 在 Widget/Screen/Overlay 中的使用边界。

**M3: Theme 切换机制缺失**

10ui.md 第12层提到 Theme 系统（Light/Dark/Pixel/HD2D），数据 Schema 定义了 `ThemeName` 枚举，但缺少：
- Theme 切换时 Widget 如何自动更新？
- Theme 配置文件格式（RON？）
- Theme 与 bevy_settings 的关系

**建议**：在 ADR-055 或数据 Schema 中新增 Theme 切换流程设计。

---

### 1.2 UI 领域规则评审

#### 🟢 正确采纳的内容

- 15 个术语定义完整，与项目术语对齐
- 8 条不变量 + 4 条铁律精简版完整
- 10 条禁止事项覆盖核心红线
- 11 个流程定义（Projection/Widget/输入/导航/Notification/Modal/Tooltip/Schema/测试/WidgetFactory/Content 数据流）
- Screen 生命周期状态机完整
- Widget 生命周期（Persistent/Ephemeral）完整

#### 🔴 CRITICAL

**C3: 缺少 UI Application Layer 的领域规则**

领域规则中 `UiCommand` 直接从 `UiAction` 转换，缺少中间的 UI Application 层规则：
- UiAction 的语义验证规则（如"技能冷却中时 CastSkill 被拒绝"）
- 多步操作的状态编排规则（如"选择技能→选择目标→确认"的流程状态机）
- 多输入源统一规则（鼠标点击和手柄确认产生相同的 UiAction）

**建议**：新增 RULE-UI-013 "UI Application Layer 规则"。

**C4: 缺少输入三层架构的领域规则**

领域规则中 `UiIntent` 仅在术语表中定义，缺少：
- InputAction → UiIntent 的映射规则
- UiIntent 的路由规则（路由到当前 FocusGroup）
- UiIntent 与 UiAction 的区别和转换规则

**建议**：新增 RULE-UI-014 "输入三层架构规则"。

#### 🟡 HIGH

**H4: Widget Contract 清单不完整**

领域规则 §8 Widget Contract 清单仅有 2 个 Widget（PrimaryButton、ProgressBar），而 ADR-055 目录结构中定义了 8+ 个 Widget 目录。50 万行项目需要完整的 Widget Contract 清单。

**建议**：补齐所有 Widget 的 Contract：ProgressBar、Tooltip、Panel、VirtualList、LocalizedText、Modal、Notification、SkillButton 等。

**H5: 缺少 UI 动画规则**

10ui.md 第19层强调"UI 动画不允许写业务逻辑"，领域规则 INV-UI-003 有提及，但缺少：
- 动画系统的架构位置（在 `src/ui/` 的哪个子目录？）
- 动画与 Delayed Commands 的关系
- 动画跳过/加速/自动战斗的处理规则

**建议**：新增 RULE-UI-015 "UI 动画规则"。

---

### 1.3 UI 数据 Schema 评审

#### 🟢 正确采纳的内容

- UiStore 统一容器设计合理
- 8 个 ViewModel 定义完整（BattleHud/CharacterPanel/SkillPanel/Inventory/Shop/QuestLog/Notification/Modal）
- StyleToken 体系完整（UiColors/UiSpacing/UiTypography/Theme）
- UiSettings + SettingsGroup 对接 Bevy 0.19
- ScreenStack 导航栈设计合理
- Focus Schema 完整
- Dirty<T> 机制设计合理
- UiCommand 枚举完整
- UiBinding 反 Marker 模式
- WidgetFactory trait
- UI 与 Content 数据流
- Replay/Save 兼容性规则清晰

#### 🟡 HIGH

**H6: UiStore 字段平铺 vs 分模块的权衡**

当前 UiStore 将所有 ViewModel 平铺在一个 struct 中：

```rust
pub struct UiStore {
    pub battle_hud: BattleHudVm,
    pub character_panel: CharacterPanelVm,
    pub skill_panel: SkillPanelVm,
    pub inventory: InventoryVm,
    pub shop: ShopVm,
    pub quest_log: QuestLogVm,
    pub notification_queue: Vec<NotificationVm>,
    pub modal_stack: Vec<ModalVm>,
}
```

50 万行项目最终可能有 30+ 个 ViewModel。平铺在一个 struct 中会导致：
- 每次修改任何 ViewModel 都触发 UiStore 的 Changed 过滤器
- 编译时间增长（大 struct 的 derive 宏）
- 模块间耦合（inventory 模块修改 inventory 字段需要 import UiStore）

**建议**：评估是否将 UiStore 拆分为多个子 Store（BattleUiStore、InventoryUiStore 等），或使用 `Dirty<T>` Component 替代集中式 UiStore。

**H7: ScreenStack 中 Entity 引用与 Save 兼容性**

ScreenInfo 包含 `entity: Entity` 字段，但 Entity 在 Save/Load 后会失效。Schema 中缺少 Entity Remapping 的说明。

**建议**：明确 ScreenStack 的 Save/Load 策略——是保存 ScreenType 然后重建，还是通过 EntityRemapper 映射？

**H8: Dirty<T> 作为 Component 的性能问题**

当前 `Dirty<T>` 设计为泛型 Component，每个 ViewModel 类型需要单独注册 `register_type::<Dirty<BattleHudVm>>()`。50 万行项目可能有 30+ 个 Dirty<T> 实例化，每个都是独立的 Archetype。

**建议**：评估是否改用 `DirtyFlag(UiBinding)` 统一枚举 + `HashMap<UiBinding, bool>` 的方案，减少 Archetype 数量。

---

### 1.4 迁移计划 UI 部分评审

#### 🔴 CRITICAL

**C5: 迁移计划中 UI 层迁移步骤严重不足**

`bevy-0.19-migration-v3-aggressive.md` 是当前活跃的迁移计划，其中 UI 相关内容：
- Phase 1 中无任何 UI 层迁移步骤
- Phase 2 中 BSN UI 层试点仅 3-5 文件
- 完全没有 `src/ui/` 模块的创建计划
- 完全没有 Projection/ViewModel/Widget/Screen 的实现计划

ADR-055 定义了完整的 UI 架构，但迁移计划中没有对应的落地步骤。这意味着 UI 架构文档是"空中楼阁"——有设计无执行。

**建议**：在迁移计划中新增 Phase 2.x "UI Presentation Layer 落地"，包含：
1. 创建 `src/ui/` 目录结构
2. 实现 Theme/StyleToken 基础设施
3. 实现 UiStore + Dirty<T> 机制
4. 实现第一个 Projection（BattleProjection）
5. 实现第一个 Widget（HpBarWidget）
6. 实现第一个 Screen（BattleScreen）
7. 实现 ScreenStack 导航
8. 实现 Overlay 层（Tooltip/Notification/Modal）

---

## 2. 10ui.md 逐层覆盖度检查

| # | 10ui.md 层次 | 文档覆盖 | 评级 | 差距 |
|---|-------------|---------|------|------|
| 1 | Projection Layer | ADR-055 §3 + 领域规则 §5.1 | 🟢 | 完整 |
| 2 | ViewModel Store | 数据 Schema §4.1 | 🟢 | 完整 |
| 3 | UI Command Bus | ADR-055 §3 反向流 + 数据 Schema §10 | 🟢 | 完整 |
| 4 | UI Event Bus | ADR-055 §3 正向流 | 🟢 | 完整 |
| 5 | Widget 生命周期 | 领域规则 §2.2 + ADR-055 §8 | 🟢 | 完整 |
| 6 | Screen Stack | ADR-055 §2 navigation/ + 数据 Schema §7 | 🟢 | 完整 |
| 7 | Reactive Projection | ADR-055 §3 + 领域规则 §5.1 | 🟢 | 完整 |
| 8 | Widget Tree | ADR-055 §5.5 + WidgetFactory | 🟢 | 完整 |
| 9 | UI Query 禁止 | 领域规则 INV-UI-001 | 🟢 | 完整 |
| 10 | UI 状态分级 | ADR-055 §7 | 🟢 | 完整 |
| 11 | Localization Layer | ADR-055 §2 localization/ + 领域规则 INV-UI-007 | 🟢 | 完整 |
| 12 | Theme Layer | ADR-055 §2 theme/ + 数据 Schema §5 | 🟢 | 完整 |
| 13 | UI Application Layer | ADR-055 application/ 但缺 ui_application.rs | 🔴 | **缺失** |
| 14 | 输入与显示分离 | UiIntent 有定义但缺三层架构 | 🔴 | **不足** |
| 15 | Tooltip 不属于 Widget | 放在 widgets/tooltip/ 下 | 🟡 | 位置不当 |
| 16 | Notification Center | 放在 widgets/notification/ 下 | 🟡 | 位置不当 |
| 17 | Modal Manager | 放在 widgets/modal/ 下 | 🟡 | 位置不当 |
| 18 | Focus System | ADR-055 §2 focus/ + 数据 Schema §8 | 🟢 | 完整 |
| 19 | UI 动画不写业务逻辑 | 领域规则 INV-UI-003 | 🟢 | 完整 |
| 20 | Widget 不持有 Entity | 领域规则 INV-UI-002 | 🟢 | 完整 |
| 21 | UI Cache Layer | ViewModel 即缓存，但未显式设计 | 🟡 | 隐式存在 |
| 22 | UI Dirty Flag | 数据 Schema §9 | 🟢 | 完整 |
| 23 | UI Debug Overlay | ADR-055 §6 DebugLayer 一句话 | 🟡 | **不足** |
| 24 | Widget Contract | 领域规则 §8 + UI Schema | 🟡 | 清单不完整 |
| 25 | UI 测试体系 | 领域规则 §5.9 RULE-UI-010 | 🟢 | 完整 |
| 26 | UI Schema | 领域规则 §5.8 RULE-UI-009 | 🟢 | 完整 |
| 27 | UI 作为独立顶层模块 | ADR-055 §1 L3 UI | 🟢 | 完整 |
| 28 | UI 与 Content 关系 | 领域规则 §5.11 RULE-UI-012 | 🟢 | 完整 |
| 29 | UI 与 Modding 关系 | 领域规则 §5.11 | 🟢 | 完整 |
| 30 | 四条铁律 | 领域规则 INV-UI-009 | 🟢 | 完整 |

**覆盖率统计**：
- 🟢 完整覆盖：22/30（73%）
- 🟡 部分覆盖：6/30（20%）
- 🔴 缺失/不足：2/30（7%）

---

## 3. 架构合规性检查

### 3.1 十条原则合规检查

| # | 原则 | 合规 | 说明 |
|---|------|------|------|
| 1 | UI 也是领域（UI 不是 Domain） | ✅ | UI 作为 L3 Presentation Layer，与 Domain 分离 |
| 2 | Core 不依赖 UI | ✅ | 铁律明确，依赖方向 UI → Core |
| 3 | UI 不直接读 Domain | ✅ | INV-UI-001 + Projection 防火墙 |
| 4 | 无 MVVM 双向绑定 | ✅ | 单向数据流明确，禁止双向绑定 |
| 5 | Widget 不持有 Entity | ✅ | INV-UI-002，使用业务 ID |
| 6 | Widget 纯展示 | ✅ | Widget Contract 禁止业务逻辑 |
| 7 | Screen 只组合 | ✅ | INV-UI-005，Screen 不直接拼 Node |
| 8 | Projection 是唯一防火墙 | ✅ | Domain Event → Projection → ViewModel |
| 9 | ViewModel 是唯一数据源 | ✅ | UiStore 统一管理 |
| 10 | UI 状态分级 | ✅ | Level 1/2/3 三级分类 |

### 3.2 架构模式合规检查

| 模式 | 合规 | 说明 |
|------|------|------|
| ScreenStack 导航 | ✅ | push/pop/replace |
| Overlay 独立 | ⚠️ | Tooltip/Modal/Notification 放在 widgets/ 下，应移到 overlay/ 或 services/ |
| Theme Token | ✅ | UiColors/UiSpacing/UiTypography |
| Localization | ✅ | UiTextKey + LocalizationService |
| Input → Intent → Action | ⚠️ | Intent 有定义但三层架构不完整 |
| Persistent Widget | ✅ | Visibility 切换模式 |
| Dirty Flag | ✅ | Dirty<T> 机制 |
| Widget Contract | ⚠️ | 框架有但清单不完整 |

---

## 4. 问题清单与优先级

### CRITICAL（必须修复）

| ID | 问题 | 影响 | 修复建议 |
|----|------|------|---------|
| C1 | UI Application Layer 缺失 | 多输入源无法复用，UiAction → UiCommand 缺少中间验证层 | 在 `src/ui/application/` 新增 `ui_application.rs`，定义 UiApplication trait |
| C2 | 输入三层架构不完整 | 手柄/键盘/触摸输入无法统一路由 | 新增 RULE-UI-014，定义 InputAction → UiIntent → UiAction 三层 |
| C5 | 迁移计划缺 UI 落地步骤 | UI 架构文档是空中楼阁，无执行路径 | 在迁移计划中新增 UI Presentation Layer 落地阶段 |

### HIGH（应该修复）

| ID | 问题 | 影响 | 修复建议 |
|----|------|------|---------|
| H1 | Tooltip 放在 widgets/ 下 | Tooltip 是全局服务，不应是 Widget 子组件 | 移到 `overlay/` 或 `services/` |
| H2 | Notification/Modal 放在 widgets/ 下 | 同上，全局服务不应是 Widget | 移到 `overlay/` 或 `services/` |
| H3 | UI Debug Overlay 设计不足 | 大型项目后期调试困难 | 设计 UiDebugOverlay 显示 ScreenStack/VM/Focus/Modal 状态 |
| H4 | Widget Contract 清单不完整 | AI 生成代码无约束，容易违反原则 | 补齐所有 Widget 的 Contract |
| H5 | 缺少 UI 动画规则 | 动画跳过/加速/自动战斗可能出 Bug | 新增 RULE-UI-015 |
| H6 | UiStore 集中式设计权衡 | 30+ ViewModel 平铺导致性能和编译问题 | 评估拆分为子 Store 或改用 Dirty<T> Component |
| H7 | ScreenStack Entity 引用 Save 兼容 | Save/Load 后 Entity 失效 | 明确 ScreenStack Save 策略 |
| H8 | Dirty<T> 泛型 Component 性能 | 30+ Archetype 实例化 | 评估统一枚举方案 |

### MEDIUM（建议修复）

| ID | 问题 | 影响 | 修复建议 |
|----|------|------|---------|
| M1 | UiStore vs 独立 Resource 访问模式不一致 | Widget 访问方式混乱 | 统一为 `Res<UiStore>` 单一入口 |
| M2 | BSN UI 层使用规范不足 | WidgetFactory 与 bsn! 关系不明 | 新增 §"BSN UI 层使用规范" |
| M3 | Theme 切换机制缺失 | 皮肤系统无法实现 | 设计 Theme 切换流程 |
| M4 | UI Cache Layer 未显式设计 | ViewModel 缓存策略不明确 | 新增 §"UI Cache Layer 设计" |

---

## 5. 目录结构调整建议

基于 10ui.md 的建议和当前文档的差距，建议调整 `src/ui/` 目录结构：

```
src/ui/
├── application/              # UI 应用层
│   ├── ui_application.rs     # [新增] UiApplication trait（C1 修复）
│   ├── command.rs            # UiCommand 枚举 + 分发
│   ├── event.rs              # UiEvent 枚举 + 广播
│   ├── intent.rs             # UiIntent + 输入三层架构（C2 修复）
│   └── mod.rs
├── projections/              # 投影层（不变）
├── view_models/              # ViewModel 定义（不变）
├── widgets/                  # 可复用 Widget
│   ├── button/               # PrimaryButton, SecondaryButton, DangerButton
│   ├── progress_bar/         # ProgressBar
│   ├── panel/                # Panel, CardPanel
│   ├── list/                 # VirtualList
│   ├── text/                 # LocalizedText
│   ├── skill_button/         # [新增] SkillButton（SRPG 特有）
│   └── mod.rs
├── screens/                  # 页面（不变）
├── overlay/                  # 独立叠加层
│   ├── tooltip.rs            # [从 widgets/ 移入] TooltipService（H1 修复）
│   ├── modal.rs              # [从 widgets/ 移入] ModalService（H2 修复）
│   ├── notification.rs       # [从 widgets/ 移入] NotificationService（H2 修复）
│   ├── damage_text.rs        # DamageTextOverlay
│   ├── loading.rs            # LoadingOverlay
│   ├── debug.rs              # [增强] UiDebugOverlay（H3 修复）
│   └── mod.rs
├── navigation/               # 导航栈（不变）
├── theme/                    # 主题系统（不变）
├── localization/             # 国际化（不变）
├── focus/                    # 焦点系统（不变）
├── binding/                  # 数据绑定（不变）
├── animation/                # [新增] UI 动画系统（H5 修复）
│   ├── animation_service.rs  # UI 动画服务
│   ├── tween.rs              # 补间动画
│   └── mod.rs
├── tests/                    # UI 测试（不变）
├── plugin.rs                 # UiPlugin
└── mod.rs
```

关键变更：
1. `widgets/tooltip/` → `overlay/tooltip.rs`（Tooltip 是全局服务）
2. `widgets/modal/` → `overlay/modal.rs`（Modal 是全局服务）
3. `widgets/notification/` → `overlay/notification.rs`（Notification 是全局服务）
4. 新增 `application/ui_application.rs`（UI Application Layer）
5. 新增 `animation/`（UI 动画系统）
6. 增强 `overlay/debug.rs`（UI Debug Overlay）

---

## 6. 迁移计划 UI 部分补充建议

当前迁移计划 `bevy-0.19-migration-v3-aggressive.md` 中 UI 相关内容严重不足。建议新增：

### Phase 2.x：UI Presentation Layer 落地

| Step | 任务 | 依赖 | 预估文件数 |
|------|------|------|-----------|
| 2.x.1 | 创建 `src/ui/` 目录结构 + mod.rs + plugin.rs | Phase 1 完成 | 15 |
| 2.x.2 | 实现 Theme/StyleToken 基础设施 | 2.x.1 | 5 |
| 2.x.3 | 实现 UiStore + Dirty<T> 机制 | 2.x.1 | 3 |
| 2.x.4 | 实现 UiApplication + UiCommand + UiIntent | 2.x.1 | 4 |
| 2.x.5 | 实现 BattleProjection（第一个 Projection） | 2.x.3 + 2.x.4 | 2 |
| 2.x.6 | 实现 HpBarWidget（第一个 Widget） | 2.x.2 + 2.x.3 | 3 |
| 2.x.7 | 实现 BattleScreen（第一个 Screen） | 2.x.5 + 2.x.6 | 2 |
| 2.x.8 | 实现 ScreenStack 导航 | 2.x.7 | 2 |
| 2.x.9 | 实现 Overlay 层（Tooltip/Notification/Modal） | 2.x.2 | 4 |
| 2.x.10 | 实现 UiDebugOverlay | 2.x.8 + 2.x.9 | 1 |
| 2.x.11 | BSN UI 层试点（BattleScreen 用 bsn! 重写） | 2.x.7 | 2 |

---

## 7. 结论

### 7.1 核心优势

1. **架构方向正确**：单向数据流、Projection 防火墙、Widget Contract 等核心原则与 10ui.md 高度对齐
2. **文档体系完整**：ADR + 领域规则 + 数据 Schema 三层文档覆盖
3. **Bevy 0.19 对接**：BSN/Observer run_if/Delayed Commands/User Settings 均有考虑
4. **AI 可维护性**：UI Schema 治理、Widget Contract、四条铁律为 AI 编码提供了明确约束

### 7.2 核心风险

1. **UI Application Layer 缺失**：多输入源复用和 UiAction 验证缺少中间层
2. **服务与 Widget 混放**：Tooltip/Modal/Notification 是全局服务，不应放在 widgets/ 下
3. **迁移计划缺 UI 落地**：架构设计无执行路径，文档是空中楼阁
4. **输入三层架构不完整**：手柄/键盘/触摸统一路由缺少设计

### 7.3 行动建议（按优先级）

1. **P0**：在迁移计划中新增 UI Presentation Layer 落地步骤（C5）
2. **P0**：设计 UI Application Layer（C1）
3. **P1**：设计输入三层架构（C2）
4. **P1**：将 Tooltip/Modal/Notification 从 widgets/ 移到 overlay/（H1/H2）
5. **P1**：增强 UI Debug Overlay 设计（H3）
6. **P2**：补齐 Widget Contract 清单（H4）
7. **P2**：新增 UI 动画规则（H5）
8. **P2**：评估 UiStore 拆分方案（H6）
9. **P3**：设计 Theme 切换机制（M3）
10. **P3**：明确 BSN UI 层使用规范（M2）

---

> **评审人**: Presentation Architect
> **下次评审触发条件**: UI Presentation Layer 落地步骤完成后，或 ADR-055 重大修订后
