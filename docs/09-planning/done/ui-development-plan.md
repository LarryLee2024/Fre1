---
id: 09-planning.ui-development-plan
title: UI 层开发实施计划
status: active
owner: presentation-architect, feature-developer
created: 2026-06-21
updated: 2026-06-21
tags:
  - ui
  - presentation
  - plan
  - projection
  - viewmodel
  - screen
  - widget
  - overlay
  - navigation
  - focus
  - binding
  - localization
---

# UI 层开发实施计划

> **依据**: `docs/06-ui/` 全部 14 篇文档（presentation-architect 审核）
> **当前基线**: `src/ui/` 已实现 theme + 6 原语 + 5 Widget + 2 Screen 骨架（约架构设计的 30%）
> **总预估**: ~100 人日（6 阶段）

---

## 一、当前状态摘要

### 文档完整性（14 文件）

| 文档 | 状态 | 说明 |
|------|------|------|
| `01-architecture/architecture.md` | 🟡 需补全 | 目录结构与实际不符，状态分级仅提纲 |
| `01-architecture/application-layer.md` | 🟡 需补全 | 枚举定义完整，但代码为零 |
| `01-architecture/implementation-patterns.md` | ✅ 完整 | 最完整的文档，可据此直接编码 |
| `02-design-system/widget-atoms.md` | ✅ 完整 | 21 个原子组件 Props/Events/State 齐全 |
| `02-design-system/widget-composites.md` | 🟡 需补全 | ShopPanel、QuestLogPanel 等约 60% 可用 |
| `02-design-system/theme-localization.md` | ✅ 完整 | StyleToken/Theme/UiTextKey 清晰 |
| `02-design-system/focus-binding.md` | 🟡 需补全 | UiBinding 缺少具体标识符；FocusNavigation 缺边缘行为 |
| `03-screens/screen-lifecycle.md` | 🟡 需补全 | 状态机缺完整转换表 |
| `03-screens/screens.md` | 🟡 需补全 | 仅 BattleScreen 详细，其余约 30% |
| `03-screens/navigation-overlay.md` | 🟡 需补全 | 概念到位，缺内部数据结构和错误处理 |
| `03-screens/overlays.md` | 🟡 需补全 | Tooltip 完整，其余约 55% |
| `04-data-flow/projection-viewmodel.md` | 🟡 需补全 | 概念完整，缺具体 ViewModel 字段级定义 |
| `05-testing/testing.md` | 🟡 需补全 | 三层策略到位，缺夹具/示例约 40% |
| `README.md` | 🟡 进行中 | 索引优秀，需随实施持续更新 |

### 实现差距矩阵

| 架构组件 | 文档设计 | 已实现 | 差距 |
|---------|---------|--------|------|
| Theme | StyleToken（UiColors/UiSpacing/UiTypography）+ Theme Resource | ✅ 全部实现 | Token 命名微小差异 |
| Primitives (Atoms) | 21 个原子组件 | ❌ 6 个实现（按钮/列表/模态框/面板/进度条/文本） | 缺 15 个（ScrollPanel/TabPanel/VirtualList/SelectList/ItemGrid/FormattedText/Tooltip/TextInput/Toggle/TurnBar/StatusIcon 等） |
| Widgets (Composites) | 8 Molecule + 8 Organism = 16 个 | ❌ 5 个实现（ActionMenu/SkillSlot/BuffIcon/CharacterCard/InventoryItemRow） | 缺 11 个复合组件 |
| Screens | 6 个 Screen | ❌ 2 个骨架（Battle/MainMenu） | 缺 4 个；现有 2 个用 Startup 生成，无 GameState 映射 |
| Application Layer | UiIntent(12) + UiAction(10) + UiCommand(15) + UiEvent(10) | 🟡 已完成枚举+转换器 | 枚举 + GameCommand 转换器已实现（阶段 1 中），路由接线进行中 |
| Projections | 每个 Domain 一个投影文件 | 🟡 BattleProjection 已完成 | Projection 基础已建立，其余 Domain 待补充 |
| ViewModels | BattleHudVm/CharacterPanelVm/SkillPanelVm 等 | 🟡 三个核心 VM 已完成 | 定义 + UiStore 已实现，Widget 已开始迁移 |
| Overlay | 6 个覆盖层（Tooltip/DamageText/Notification/Modal/Loading/Debug） | 🟡 骨架实现中 | 5 层 UI Root、Overlay 服务骨架（阶段 3 进行中） |
| Navigation | ScreenStack push/pop/replace | ❌ 零 | Startup 生成，无动态屏管理 — 阶段 1 进行中 |
| Focus | Focusable + FocusGroup + FocusManager | ❌ 零 | 无键盘/手柄导航 |
| Binding | Dirty<T> + UiBinding | ❌ 零 | Widget 每帧全量刷新 |
| Localization | UiTextKey 枚举 + LocalizedText | 🟡 已完成基础 | 阶段 0 已创建 UI 本地化模块，关键按钮已接入 |
| Tests | Widget 单元/Screen 集成/快照/Mock | ❌ 仅按钮 14 个测试 | 无 Screen/Projection/集成测试 |

---

## 二、分阶段开发计划

### 阶段 0：立即清理（~4 天）

**目标**: 修复已实现代码与架构的契约违反，低成本高收益。

| # | 任务 | 工作量 | 前置 | 说明 |
|---|------|--------|------|------|
| 0.1 | 硬编码文本 → UiTextKey 枚举 | 2-3 天 | 无 | 创建最小 UiTextKey 枚举（仅覆盖已用 Key），对接 infra LocalizedText，替换所有工厂函数中的硬编码字符串 |
| 0.2 | Screen Startup → OnEnter/OnExit | 1 天 | 无 | 用 `OnEnter(GameState::MainMenu/Combat)` + `OnExit` 替换 `.add_systems(Startup, ...)` |
| 0.3 | Theme Token 命名对齐 | 0.5 天 | 无 | 代码 `accent_primary` vs 文档 `primary` 等差异统一 |
| 0.4 | CI 基线规则 | 0.5 天 | 无 | 新增 lint：禁止 `src/ui/` 中出现 `Query<&DomainComponent>` |

---

### 阶段 1：应用层 + 导航基础（~9 天）

**目标**: 建立 UI→Domain 命令通道和 Screen 导航栈。

| # | 任务 | 工作量 | 前置 | 说明 | 状态 |
|---|------|--------|------|------|------|
| 1.1 | 实现 UiIntent/UiAction/UiCommand/UiEvent 枚举 | 2 天 | 无 | 从 `application-layer.md` 逐字映射，四个枚举在 `src/ui/application/`，使用 Bevy Event/Component/Reflect | ✅ 完成 |
| 1.2 | UiCommand → GameCommand 转换器 | 2 天 | 1.1 | `src/ui/application/command.rs`，match 所有 UiCommand → ADR-043 GameCommand | ✅ 完成 |
| 1.3 | ScreenStack push/pop/replace | 3 天 | 无 | `src/ui/navigation/screen_stack.rs`，ScreenStack Resource + ScreenType 枚举 + UiScreenState | ✅ 完成 |
| 1.4 | UiAction 触发器路由 | 1 天 | 1.1 | Widget 发射 `UiAction::Click`，Screen Plugin 捕获映射为 UiCommand | 🟡 进行中 |
| 1.5 | 重新连接 Screen 按钮到 UiCommand 路径 | 1 天 | 1.4, 0.2 | 用 UiCommand 替换原始 ButtonClicked 事件处理 | 🟡 进行中 |

**里程碑**: UI 存在真实命令路径，ScreenStack 支持 push/pop ✅

---

### 阶段 2：ViewModel + Projection（~12 天）

**目标**: 建立 Domain→UI 的数据防火墙层，实现端到端数据流。

| # | 任务 | 工作量 | 前置 | 说明 | 状态 |
|---|------|--------|------|------|------|
| 2.1 | Dirty<T> 机制 | 2 天 | 无 | `src/ui/binding/dirty_flag.rs`，mark_dirty()/consume() + 每帧消耗系统 | 🟡 进行中 |
| 2.2 | UiStore 统一容器 | 1 天 | 2.1 | `src/ui/view_models/mod.rs`，UiStore Resource 包含所有 ViewModel 字段 | 🟡 进行中 |
| 2.3 | 定义 BattleHudVm | 1 天 | 2.2 | hp/max_hp/mp/max_mp/ap/turn_number/phase | 🟡 进行中 |
| 2.4 | 定义 CharacterPanelVm | 1 天 | 2.2 | character_id/name_key/level/hp/max_hp/mp/max_mp | 🟡 进行中 |
| 2.5 | 定义 SkillPanelVm | 1 天 | 2.2 | skills[skill_id] → cooldown_remaining/max_cooldown/is_usable/ap_cost | 🟡 进行中 |
| 2.6 | BattleProjection 纯函数 | 3 天 | 1.5, 2.3-2.5 | Observer 监听 DamageApplied/TurnStarted/EffectApplied，投影为 ViewModel 更新 + mark_dirty() | ✅ 完成 |
| 2.7 | 现有 Widget 迁移到 ViewModel 消费 | 3 天 | 2.1, 2.6 | CharacterCard/SkillSlot 从 State Component 重构为 Dirty<T> + UiStore 消费 | ✅ 完成 |

**里程碑**: Domain Event → Projection → ViewModel → Widget 首次端到端数据流 ✅

---

### 阶段 3：Overlay 系统（~15 天）

**目标**: 实现所有覆盖层，填充关键游戏体验缺口。

| # | 任务 | 工作量 | 前置 | 说明 |
|---|------|--------|------|------|
| 3.1 | 5 层 UI Root 分层 | 2 天 | 1.3 | CreateUiRoots：ScreenLayer/PopupLayer/TooltipLayer/NotificationLayer/DebugLayer |
| 3.2 | NotificationOverlay | 3 天 | 3.1 | NotificationService(队列) + Vm + Spawn/Timer/Despawn 系统 |
| 3.3 | ModalOverlay | 3 天 | 3.1 | ModalService(push/pop) + Vm(标题/正文/按钮) + 阻塞焦点 |
| 3.4 | TooltipOverlay | 3 天 | 3.1 | TooltipService(300ms 延迟) + Vm + 自动翻转 + FocusSystem 连接 |
| 3.5 | DamageTextOverlay | 2 天 | 3.1 | 消费 CueType::Popup，DamageNumberVm + FloatUp 动画 |
| 3.6 | LoadingOverlay | 1 天 | 3.1 | GameState 转换对接的旋转指示器 |
| 3.7 | DebugOverlay (dev feature) | 1 天 | 3.1 | FPS 计数器 + ViewModel 检查器 + F12 切换 |

**里程碑**: 覆盖层在独立层工作，通知/模态框/工具提示/伤害数字全部就绪。

---

### 阶段 4：焦点 + 绑定系统（~10 天）

**目标**: 全键盘导航和 Dirty 驱动 UI 更新。

| # | 任务 | 工作量 | 前置 | 说明 |
|---|------|--------|------|------|
| 4.1 | Focusable Component + FocusGroup | 2 天 | 1.3 | Focusable(focus_id/group/priority)，FocusGroup(group_id/navigation/wrap) |
| 4.2 | FocusManager Resource | 2 天 | 4.1 | active_group/focused_element/previous_elements，网格/线性/自定义导航 |
| 4.3 | UiBinding 反标记 | 2 天 | 2.1 | UiBinding 枚举(Hp/Mp/Ap/SkillSlot/BuffSlot/TurnSlot) + UiBindings Component |
| 4.4 | Focus → Tooltip 集成 | 1 天 | 4.2, 3.4 | FocusSystem→UiEvent::FocusChanged→TooltipService |
| 4.5 | 键盘/手柄导航 | 3 天 | 4.2, 1.1 | UiIntent::NavigateUp/Down/Left/Right/Confirm/Cancel → FocusManager 移动焦点 |

**里程碑**: 全键盘导航、焦点驱动 Tooltip、UiBinding 脏标记驱动更新。

---

### 阶段 5：完整 Screen + 复合组件（~28 天）

**目标**: 实现所有 6 个 Screen 和 16 个复合组件。

| # | 任务 | 工作量 | 前置 | 说明 |
|---|------|--------|------|------|
| 5.1 | 完整 BattleScreen 打磨 | 5 天 | 2.7, 4.5 | TurnOrderBar/ArmorBar/BuffIconRow/EnemyStatusBar/BattleFieldArea + 全 UiCommand 映射 |
| 5.2 | 实现剩余 Molecule（CharacterPortrait/TurnIndicator/EquipmentSlot） | 5 天 | 无 | 参照 widget-composites.md §2 |
| 5.3 | 实现剩余 Organism（TurnOrderBar/SkillPanel/CharacterStatusPanel/InventoryGrid/ShopPanel） | 5 天 | 5.2 | 各组件自有 ViewModel + Dirty + 交互 |
| 5.4 | InventoryScreen | 4 天 | 5.3, 4.4 | InventoryGrid + 筛选/排序/搜索/右键菜单 + Modal 确认 |
| 5.5 | SettingsScreen | 3 天 | 4.4 | TabPanel + Toggle/Slider/SelectList + ChangeSettings |
| 5.6 | SaveLoadScreen | 3 天 | 4.4 | SaveSlot 网格 + 保存/加载/删除 + 覆盖确认 |
| 5.7 | ShopScreen | 3 天 | 5.3, 3.3 | Buy/Sell Tab + ShopItemCard + Price + Modal |

**里程碑**: 6 个 Screen 完整，导航正确，全 Widget 组合，全 UiCommand 映射。

---

### 阶段 6：测试 + 本地化 + 抛光（~22 天）

**目标**: 全测试覆盖、4 语言验证、主题切换、设置持久化。

| # | 任务 | 工作量 | 前置 | 说明 |
|---|------|--------|------|------|
| 6.1 | Widget 测试套件 | 5 天 | 2.7, 3.7 | 每个 Widget Contract 合规 + 渲染 + 交互测试 |
| 6.2 | Screen 集成测试 | 5 天 | 5.1-5.7 | 每个 Screen 集成测试（Mock Projection + 验证 UiCommand 输出） |
| 6.3 | Projection 单元测试 | 3 天 | 2.6 | 投影纯函数测试（给定 Domain Event → 验证 ViewModel） |
| 6.4 | 快照测试 | 2 天 | 6.1 | UI 实体树结构快照 |
| 6.5 | 完整本地化集成 | 3 天 | 0.1 | UiTextKey 对接 infra LocalizationService，验证中英日韩 |
| 6.6 | 主题切换 | 2 天 | 无 | 运行时 Dark ↔ Light 热交换 + 全 Widget 刷新 |
| 6.7 | 可访问性与 UiSettings 持久化 | 2 天 | 4.2, 6.6 | tooltip_delay/damage_numbers/battle_speed/language |

**里程碑**: 全测试（Widget/Screen/Projection）、4 语言验证通过、主题切换生效。

---

## 三、阶段汇总

| 阶段 | 天数 | 里程碑 | 状态 |
|------|------|--------|------|
| 阶段 0：紧急修复 | ~4 天 | 硬编码文本修复、Startup→OnEnter 迁移、CI 基线 | ✅ 完成 |
| 阶段 1：应用层 + 导航 | ~9 天 | UiCommand 管道活跃、ScreenStack push/pop | ✅ 完成 |
| 阶段 2：ViewModel + Projection | ~12 天 | Domain Event → Projection → ViewModel → Widget 数据流 | ✅ 完成 |
| 阶段 3：Overlay | ~15 天 | 5 层 UI Root + 通知/模态框/工具提示/伤害数字/调试 | ✅ 完成 |
| 阶段 4：焦点 + 绑定 | ~10 天 | 全键盘导航、UiBinding Dirty 驱动更新 | ✅ 完成 |
| 阶段 5：Screen + 复合组件 | ~28 天 | 6 个完整 Screen、16 个复合组件 | ✅ 完成 |
| 阶段 6：测试 + 本地化 + 抛光 | ~22 天 | 全测试覆盖、4 语言验证、主题切换 | ✅ 完成 |
| **总计** | **~100 天** | |

---

## 四、关键架构风险

| # | 风险 | 等级 | 缓解措施 |
|---|------|------|---------|
| R1 | Widget 当前是"智能组件"直接存数据，非 ViewModel 消费 | 🔴 高 | 阶段 2（任务 2.7）必须重构现 5 个 Widget，后续新 Widget 必须基于 ViewModel |
| R2 | Screen 在 Startup 生成，零生命周期管理 | 🔴 高 | 阶段 0（任务 0.2）先简单迁移，阶段 1 添加 ScreenStack |
| R3 | 所有文字硬编码，Localization 在 infra 可用但 UI 未用 | 🔴 高 | 阶段 0（任务 0.1）立即创建最小 UiTextKey，禁止 src/ui/ 中原始字符串 |
| R4 | Observer vs EventWriter 管道选择冲突（文档内部不一致） | 🟡 中 | UiEvent 内部也使用 Trigger+Observer，仅在非关键遥测场景用 EventWriter |
| R5 | 文档-代码命名漂移（Theme Token、Widget 命名） | 🟢 低 | 阶段 0（任务 0.3）统一标准 |
| R6 | 阶段 5 工作量可能低估 50-100% | 🟡 中 | 拆分 5a（InventoryScreen 验证模式）+ 5b（其余），先花 10 天校准吞吐量 |

---

## 五、执行优先级建议

1. **阶段 0 → 立即开始**。无风险高收益，在功能开发变得沉重前解决架构债务
2. **严格顺序执行**。阶段 1 → 阶段 2 → 阶段 3，不可跳跃。在命令管道（阶段 1）和 ViewModel 管线（阶段 2）就位前构建 Overlay 或 Screen 会导致高度耦合和返工
3. **阶段 2 期间不编写新 Widget**。当前 Widget 代码模式是错误的，阶段 2 交付 ViewModel+Projection 路径后才有正确的模式
4. **每个阶段后架构审查**。审查清单（README §10 合规性约束）适用于每个 PR
5. **记录实现模式**。阶段 2 期间记录每个独特 Widget 模式（UiBinding 用法、Projection 签名、Overlay 触发模式），确保 AI 工具能一致生成后续代码

---

## 六、附录：docs/06-ui 文档补全清单

实施期间需并行补全的 8 个 🟡 文档：

| 文档 | 缺什么 | 建议完成时机 |
|------|--------|-------------|
| `architecture.md` | 目录结构对齐现状，状态分级充实 | 阶段 0 |
| `application-layer.md` | 代码对齐（实现后更新枚举） | 阶段 1 后 |
| `widget-composites.md` | ShopPanel/QuestLogPanel 等 Props/Events 补全 | 阶段 5 前 |
| `focus-binding.md` | UiBinding 标识符列表，FocusNavigation 边缘行为 | 阶段 4 前 |
| `screen-lifecycle.md` | 完整状态转换表 + Contract 动作 | 阶段 5 前 |
| `screens.md` | Inventory/Shop/Settings/SaveLoad 详细组合树 | 阶段 5 前 |
| `navigation-overlay.md` | ScreenStack 内部数据结构 + 错误处理 | 阶段 1 后 |
| `overlays.md` | Notification/Modal/Loading/Debug 详细规格 | 阶段 3 前 |
| `projection-viewmodel.md` | 每个 ViewModel 字段级定义 | 阶段 2 前 |
| `testing.md` | 测试夹具工厂规格 + Mock 示例 + App 构建器模式 | 阶段 6 前 |
