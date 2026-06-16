# 战棋项目完全重构任务书

> 基于《项目铁律》，对 `tactical_rpg` 进行全面重构。
> 每个阶段独立可编译、可运行，确保项目始终处于可用状态。

---

## 阶段总览

| 阶段 | 主题 | 核心目标 |
|------|------|----------|
| 一 | 架构重组与模块边界 | 按业务重新划分模块，消除跨层耦合 |
| 二 | ECS 现代化 | 引入 Hook / Observer / Required Components / Message |
| 三 | 逻辑与表现分离 | 伤害计算、战斗日志、VFX 彻底解耦 |
| 四 | 属性与 Trait 体系统一 | 统一 Modifier 管线，消除散落加成逻辑 |
| 五 | 数据驱动升级 | 注册表迁移至 AssetServer，配置热重载 |
| 六 | UI 架构重构 | ViewModel 完善，Widget 库封装 |
| 七 | 函数与文件瘦身 | 参数精简、函数拆分、文件职责收拢 |
| 八 | 测试与调试体系 | 补充集成测试，调试入口可视化 |

---

## 阶段一：架构重组与模块边界

**目标**：消除 `gameplay` 跨业务依赖，将表现层逻辑从业务模块中剥离，确保模块只暴露公共接口。

### 1.1 拆分 `gameplay` 模块

**问题**：`gameplay` 作为"基础设施"模块，包含了 `attribute`、`effect`、`modifier_rule`、`tag` 四个子模块，但这些子模块被不同业务模块依赖的方式不同，导致 `gameplay` 成了"什么都往里塞"的容器。

**方案**：

```
src/
├── core/                    # 原 gameplay/，纯基础设施
│   ├── mod.rs               # CorePlugin
│   ├── attribute/           # 属性系统（保持不变）
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── attribute_def.rs     # 属性定义注册表
│   ├── effect/              # 效果管道（保持不变）
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   └── handler.rs
│   ├── modifier_rule.rs     # 修饰规则注册表
│   ├── tag.rs               # GameplayTag
│   └── tag_def.rs           # 标签定义注册表
```

- [ ] 将 `src/gameplay/` 重命名为 `src/core/`
- [ ] 更新 `lib.rs` 中的模块声明
- [ ] 更新所有 `use crate::gameplay::` 为 `use crate::core::`
- [ ] `CorePlugin` 替代原各独立 Plugin（`AttributeDefPlugin`、`TagDefPlugin`、`EffectPlugin`、`ModifierRulePlugin`），统一注册
- [ ] `main.rs` 中 `CorePlugin` 一次性注册，不再分散

### 1.2 从 `character/components.rs` 剥离表现逻辑

**问题**：`Faction::unit_color()` 和 `clear_markers` 属于表现层，放在了组件定义文件中。

**方案**：

- [ ] 将 `Faction::unit_color()` 移至 `ui/` 模块，作为 UI 颜色映射函数
- [ ] 将 `clear_markers` 移至 `character/movement.rs` 或新建 `character/marker.rs`
- [ ] `components.rs` 只保留纯数据定义：`Unit`、`UnitName`、`UnitRace`、`UnitClass`、`GridPosition`、`Selected`、`AiBehaviorId`
- [ ] Tag Component（`MovableRange`、`AttackRange`、`HpBarBg`、`HpBarFg`、`SelectionHighlight`、`PathArrow`）移至各自的"拥有者"模块：
  - `MovableRange` / `AttackRange` → `character/marker.rs`
  - `HpBarBg` / `HpBarFg` → `ui/` 模块
  - `SelectionHighlight` → `ui/` 模块
  - `PathArrow` → `character/movement.rs`

### 1.3 从 `input.rs` 剥离表现逻辑

**问题**：`show_move_range`、`show_attack_range`、`spawn_selection_highlight` 是 UI 表现逻辑，放在了输入处理模块。

**方案**：

- [ ] 将 `show_move_range` / `show_attack_range` 移至 `ui/` 模块（如 `ui/highlight.rs`）
- [ ] 将 `spawn_selection_highlight` 移至 `ui/highlight.rs`
- [ ] `input.rs` 只负责：读取输入 → 发送 `UiCommand` Message
- [ ] UI 模块通过 Observer 监听 `UiCommand`，自行决定如何高亮

### 1.4 `turn.rs` 拆分

**问题**：`turn.rs` 包含了状态定义（`AppState`、`TurnPhase`）、Resource 定义（`TurnOrder`、`TurnState`、`AiTimer` 等）、System 集合定义（`GameSet`），职责过多。

**方案**：

```
src/
├── turn/                    # 回合模块
│   ├── mod.rs               # TurnPlugin
│   ├── state.rs             # AppState / TurnPhase / GameSet
│   ├── order.rs             # TurnOrder / TurnState / init_turn_order
│   ├── phase.rs             # 各阶段 System（execute_action_on_enter 等）
│   └── resource.rs          # AiTimer / NeedsResolve / ForceEndFaction
```

- [ ] 拆分 `turn.rs` 为上述文件结构
- [ ] `state.rs` 只放状态枚举和 SystemSet
- [ ] `order.rs` 只放行动队列相关
- [ ] `phase.rs` 放各阶段转换逻辑
- [ ] `resource.rs` 放辅助 Resource

### 1.5 `camera.rs` 归入表现层

**问题**：`camera.rs` 是表现层逻辑，但与业务模块同级。

**方案**：

- [ ] 将 `camera.rs` 移入 `ui/camera.rs` 或新建 `presentation/camera.rs`
- [ ] `CameraPlugin` 由 `UiPlugin`（或 `PresentationPlugin`）统一注册

### 1.6 模块可见性审计

- [ ] 逐模块检查 `pub` 声明，能不 `pub` 的全部降级为 `pub(crate)` 或私有
- [ ] 每个模块的 `mod.rs` 只 re-export 外部需要的类型
- [ ] 确保没有跨模块直接访问内部细节的情况（如 Battle 直接修改 Inventory）

---

## 阶段二：ECS 现代化

**目标**：全面引入 Bevy 0.18 的 Hook、Observer、Required Components、Message 机制，替代手写状态标记和轮询。

### 2.1 引入 Required Components

**问题**：当前 `spawn_units` 手动添加所有组件（`Unit`、`Attributes`、`SkillSlots`、`ActiveBuffs` 等），依赖关系隐式。

**方案**：

- [ ] 为 `Unit` 添加 Required Components：
  ```rust
  #[derive(Component, Require)]
  #[require(Attributes, SkillSlots, ActiveBuffs, GameplayTags, TraitCollection, GridPosition)]
  pub struct Unit { ... }
  ```
- [ ] 为 `Player` Tag 添加 Required Components：
  ```rust
  #[derive(Component, Require)]
  #[require(Unit, Faction(|| Faction::Player))]
  pub struct Player;
  ```
- [ ] 为 `Enemy` Tag 添加 Required Components：
  ```rust
  #[derive(Component, Require)]
  #[require(Unit, Faction(|| Faction::Enemy), AiBehaviorId(|| AiBehaviorId::default()))]
  pub struct Enemy;
  ```
- [ ] 简化 `spawn_units`，只需 `commands.spawn(Player)` 即可自动附带所有必要组件

### 2.2 引入 Hook 处理组件固有行为

**问题**：当前 `Dead` 状态通过系统轮询检测，没有自动清理关联组件。

**方案**：

- [ ] 新增 `Dead` Tag Component，添加 Hook：
  ```rust
  #[derive(Component)]
  pub struct Dead;

  // Hook: Dead 添加时自动移除 MovableRange、AttackRange、Selected
  app.world_mut().register_component_hooks::<Dead>()
      .on_add(|mut world, entity| {
          world.entity_mut(entity).remove::<(MovableRange, AttackRange, Selected)>();
      });
  ```
- [ ] 移除 `update_acted_unit_color` 中的手动状态检查，改用 `Added<Dead>` 或 Observer
- [ ] `Dead` Hook 自动设置 `Unit.acted = true`

### 2.3 引入 Observer 处理外部响应

**问题**：角色死亡时直接 `despawn`，其他系统无法响应（UI 更新、任务追踪、掉落等）。

**方案**：

- [ ] 新增 `CharacterDied` Message：
  ```rust
  #[derive(Message)]
  pub struct CharacterDied { pub entity: Entity }
  ```
- [ ] `apply_damage_effect` 中 HP<=0 时：添加 `Dead` Tag + 发送 `CharacterDied` Message，不再直接 despawn
- [ ] 各模块通过 Observer 响应 `CharacterDied`：
  - UI：播放死亡动画、更新面板
  - Turn：从行动队列移除
  - Battle：更新胜负判定
  - Map：更新 OccupancyGrid
- [ ] 死亡动画播放完毕后，由表现层 Observer 执行 despawn

### 2.4 扩展 Message 体系

**问题**：当前只有 `UiCommand` 一个 Message，跨模块通信不够充分。

**方案**：

- [ ] 新增核心 Message：
  ```rust
  #[derive(Message)] pub struct TurnStarted { pub turn: u32 }
  #[derive(Message)] pub struct TurnEnded { pub turn: u32 }
  #[derive(Message)] pub struct UnitSelected { pub entity: Entity }
  #[derive(Message)] pub struct UnitDeselected;
  #[derive(Message)] pub struct DamageApplied { pub target: Entity, pub amount: i32, pub damage_type: DamageType }
  #[derive(Message)] pub struct BuffApplied { pub target: Entity, pub buff_id: String }
  #[derive(Message)] pub struct BuffExpired { pub target: Entity, pub buff_id: String }
  ```
- [ ] UI 模块通过 Observer 监听这些 Message，自行刷新
- [ ] 移除 ViewModel 的轮询式更新（`update_selected_unit_view` 等），改为 Observer 响应式

### 2.5 用 Added/Changed/Removed 替代手动状态标记

- [ ] 移除 `NeedsResolve` Resource，改用 `Changed<ActiveBuffs>` 或 `Added<TurnPhase::SelectUnit>` 触发
- [ ] 移除 `ForceEndFaction` Resource，改用 Message 通知
- [ ] 审计所有 `is_dirty` 风格的手动标记，用 ECS 变更检测替代

---

## 阶段三：逻辑与表现分离

**目标**：伤害计算、战斗日志、VFX 彻底解耦，逻辑层只发 Message，表现层自行响应。

### 3.1 `execute_effects` 纯逻辑化

**问题**：`apply_damage_effect` 同时执行扣血逻辑和 `vfx::spawn_damage_popup`。

**方案**：

- [ ] `apply_damage_effect` 只做：扣血 → 判定死亡 → 发送 `DamageApplied` Message
- [ ] 移除 `apply_damage_effect` 中的 `vfx::spawn_damage_popup` 调用
- [ ] 移除 `apply_damage_effect` 中的 `CombatLog` 写入
- [ ] VFX 模块通过 Observer 监听 `DamageApplied`，自行生成飘字
- [ ] CombatLog 通过 Observer 监听 `DamageApplied`，自行格式化并写入日志

### 3.2 `resolve_status_effects` 纯逻辑化

**问题**：DoT/HoT 结算同时调用 `vfx::spawn_damage_popup` 和 `CombatLog`。

**方案**：

- [ ] `resolve_status_effects` 只做：计算 DoT/HoT 伤害 → 扣血/回血 → 发送 Message
- [ ] VFX 和 CombatLog 通过 Observer 响应，同 3.1

### 3.3 CombatLog 格式化移至 UI 层

**问题**：战斗日志的格式化（颜色、分段）属于 Presentation，但直接在逻辑层构建。

**方案**：

- [ ] 逻辑层只发送结构化 Message（`DamageApplied`、`BuffApplied` 等）
- [ ] `ui/panels/combat_log_panel.rs` 中的 Observer 接收 Message，自行格式化
- [ ] `CombatLog` Resource 改为纯事件列表，不含格式化信息
- [ ] 颜色、分段、图标等表现细节全部在 UI 层处理

### 3.4 角色颜色/高亮移至 UI 层

- [ ] `Faction::unit_color()` → `ui/theme.rs` 中的颜色映射
- [ ] `update_acted_unit_color` → UI Observer 监听 `Added<Dead>` 或 `Unit.acted` 变更
- [ ] 所有 Sprite 颜色修改由 UI 模块负责

---

## 阶段四：属性与 Trait 体系统一

**目标**：统一 Modifier 管线，消除散落加成逻辑，确保属性公式集中管理。

### 4.1 属性计算管线统一

**问题**：虽然已有 `AttributeModifierInstance` 统一管理，但部分地方仍可能直接修改属性值。

**方案**：

- [ ] 审计所有 `attributes.xxx += value` 的直接修改，统一走 `add_modifier` / `remove_modifier`
- [ ] 确保不存在两套属性计算路径（UI 一套、战斗一套）
- [ ] 衍生属性（攻击力、防御力等）全部通过 `calculate_derived()` 实时计算，不缓存

### 4.2 Trait 效果处理器统一

**问题**：`TraitEffectHandler` 和 `EffectHandler` 是两套独立的处理器体系。

**方案**：

- [ ] 评估是否可以将 `TraitEffectHandler` 统一到 `EffectHandler` 体系
- [ ] 如果 Trait 效果本质上就是"被动效果"，则统一入口：
  - Trait 授予时 → 注册对应 EffectHandler
  - Trait 移除时 → 注销对应 EffectHandler
- [ ] 如果 Trait 效果与主动效果差异过大，保持两套但确保 Modifier 管线统一

### 4.3 Buff 修饰符生命周期绑定

- [ ] `apply_buff` 时通过 `add_modifier` 添加修饰符，返回 `BuffInstanceId`
- [ ] `remove_buff` 时通过 `remove_modifiers_from(buff_instance_id)` 清理
- [ ] 确保 Buff 过期时修饰符一定被清理（当前已有，需验证）

### 4.4 属性公式集中管理

- [ ] 新建 `core/attribute/formula.rs`，集中所有衍生属性计算公式
- [ ] 公式函数签名统一：`fn calculate_attack(base: i32, modifiers: &[AttributeModifierInstance]) -> i32`
- [ ] 移除散落在各文件中的属性计算逻辑

---

## 阶段五：数据驱动升级

**目标**：注册表迁移至 Bevy AssetServer，支持配置热重载，消除硬编码文件路径。

### 5.1 注册表迁移至 AssetServer

**问题**：所有 `load_from_dir("assets/xxx")` 使用 `std::fs` 读取，依赖工作目录，不支持热重载。

**方案**：

- [ ] 为每种注册表定义 Bevy Asset：
  ```rust
  #[derive(Asset, TypePath, Deserialize)]
  pub struct SkillDefsAsset { pub skills: Vec<SkillDef> }
  ```
- [ ] 使用 `AssetServer::load::<SkillDefsAsset>("skills/fireball.ron")` 加载
- [ ] 使用 `Assets<SkillDefsAsset>` Resource 访问已加载资源
- [ ] 注册表在 `AssetEvent::Loaded` 时自动更新

### 5.2 配置热重载

- [ ] 启用 Bevy 的 `AssetPlugin` 文件监视
- [ ] `AssetEvent::Modified` 时自动更新注册表
- [ ] 开发期修改 RON 文件无需重启游戏

### 5.3 配置兼容性

- [ ] 为所有 RON 配置添加版本号字段：`version: u32 = 1`
- [ ] 加载时检查版本号，不兼容时输出明确错误而非静默失败
- [ ] 存档系统预留配置版本兼容检查

### 5.4 注册表默认值策略

- [ ] `register_defaults()` 保留作为 fallback
- [ ] 但优先从 Asset 加载，fallback 只在 Asset 不存在时使用
- [ ] 移除硬编码在代码中的默认数据，改为内嵌 RON 字符串或内置 Asset

---

## 阶段六：UI 架构重构

**目标**：ViewModel 完善，Widget 库封装，UI 全部 Observer 响应式。

### 6.1 ViewModel 响应式重构

**问题**：当前 ViewModel 通过 System 轮询更新（`update_selected_unit_view` 等）。

**方案**：

- [ ] `SelectedUnitView` 改为 Observer 响应：
  - 监听 `UnitSelected` / `UnitDeselected` Message
  - 监听 `DamageApplied` / `BuffApplied` 等变更
- [ ] `TurnInfoView` 改为 Observer 响应：
  - 监听 `TurnStarted` / `TurnEnded` Message
- [ ] `CombatPreviewView` 改为 Observer 响应：
  - 监听 `UiCommand::SelectTarget` 等
- [ ] 移除所有 `update_xxx_view` 轮询 System

### 6.2 Widget 库封装

**问题**：当前 UI 大量手写 Node 树，维护困难。

**方案**：

- [ ] 新建 `ui/widgets/` 目录，封装通用 Widget：
  - `VBox` / `HBox`：弹性布局容器
  - `Panel`：带边框和背景的面板
  - `Window`：可拖拽窗口
  - `ScrollView`：滚动视图
  - `ResourceBar`：血条/蓝条
  - `Popup`：弹出菜单
- [ ] 所有 UI 面板使用 Widget 库构建，不再手写 Node 树
- [ ] Widget 库支持主题（`UiTheme`）

### 6.3 UI Feature 模块化

- [ ] 每个 UI 面板作为独立 Feature 模块：
  - `ui/panels/unit_info.rs` → `UnitInfoPlugin`
  - `ui/panels/combat_log_panel.rs` → `CombatLogPanelPlugin`
  - `ui/panels/turn_indicator.rs` → `TurnIndicatorPlugin`
  - `ui/panels/action_hint.rs` → `ActionHintPlugin`
  - `ui/panels/combat_preview.rs` → `CombatPreviewPlugin`
- [ ] 每个面板 Plugin 只暴露构建函数，内部实现私有
- [ ] 面板之间通过 Message 通信，不直接引用

### 6.4 UI 与逻辑完全解耦

- [ ] UI 模块不 `use crate::battle::*`，不 `use crate::character::*`
- [ ] UI 只依赖 `core::attribute`、`core::tag` 和 Message 类型
- [ ] UI 通过 ViewModel 和 Message 与逻辑层交互

---

## 阶段七：函数与文件瘦身

**目标**：参数精简、函数拆分、文件职责收拢，消除超长函数和超多参数。

### 7.1 `execute_effects` 参数精简

**问题**：`execute_effects_inline` 有 13 个参数，`apply_damage_effect` 有 12 个参数。

**方案**：

- [ ] 引入 `EffectContext` 结构体封装常用参数：
  ```rust
  pub struct EffectContext<'a> {
      pub commands: &'a mut Commands,
      pub effect_queue: &'a EffectQueue,
      pub attribute_registry: &'a AttributeRegistry,
      pub modifier_registry: &'a ModifierRuleRegistry,
      pub tag_registry: &'a TagRegistry,
      pub buff_registry: &'a BuffRegistry,
      pub skill_registry: &'a SkillRegistry,
  }
  ```
- [ ] 各 `apply_xxx_effect` 函数签名改为 `(context: &EffectContext, effect: &PendingEffect)`
- [ ] 通过 `EffectContext` 访问注册表，不再逐个传参

### 7.2 长函数拆分

- [ ] 审计所有超过 100 行的函数，逐个拆分
- [ ] 每个函数只保留一个主要职责
- [ ] 函数名描述意图，不描述过程

### 7.3 文件职责收拢

- [ ] 审计所有超过 500 行的文件，评估是否需要拆分
- [ ] 确保一个文件一个主题
- [ ] 消除"垃圾桶"文件（如 `utils.rs`）

---

## 阶段八：测试与调试体系

**目标**：补充集成测试，调试入口可视化，确保重构不引入回归。

### 8.1 重构前：补充回归测试

- [ ] 在重构前，为所有核心流程补充集成测试：
  - 角色生成 → 属性计算 → 伤害计算 → Buff 施加/移除 → 回合流转
- [ ] 确保现有测试全部通过
- [ ] 这些测试将作为重构的回归保障

### 8.2 每阶段重构后：运行测试

- [ ] 每完成一个阶段，运行全部测试
- [ ] 修复测试失败，确保阶段结束时项目可编译、可运行、测试通过

### 8.3 调试入口可视化

- [ ] 保留 `bevy-inspector-egui` Inspector
- [ ] 新增调试面板：
  - 属性面板：选中单位的所有属性 + Modifier 来源
  - 事件追踪器：最近的 Message/Observer 触发记录
  - Buff 面板：选中单位的所有 Buff + 剩余回合
- [ ] 关键 Message 链路可追踪（日志输出）

### 8.4 战斗回放（可选）

- [ ] 记录每回合的 Message 序列
- [ ] 支持回放指定回合的战斗过程
- [ ] 用于调试复杂战斗链

---

## 执行原则

1. **每阶段独立可编译**：完成一个阶段后，项目必须能 `cargo build` + `cargo test` 通过
2. **先测试后重构**：阶段八的回归测试在阶段一开始前补充
3. **小步提交**：每个子任务完成后单独提交，便于回滚
4. **不新增功能**：重构只改善架构，不添加新游戏功能
5. **铁律优先**：任何决策冲突时，以《项目铁律》为准
6. **定期复盘**：每完成一个阶段，审视哪些规则需要调整

---

## 预期成果

| 指标 | 重构前 | 重构后 |
|------|--------|--------|
| Logic/Presentation 混合点 | 4 处 | 0 处 |
| 手动状态标记 | 2 个（NeedsResolve、ForceEndFaction） | 0 个 |
| Hook/Observer 使用 | 0 个 | 全覆盖（Dead、Buff、Turn 等） |
| Message 类型 | 1 个（UiCommand） | 8+ 个 |
| 函数参数 > 8 个 | 2 个 | 0 个 |
| ViewModel 更新方式 | 轮询 | Observer 响应式 |
| 配置加载方式 | std::fs | AssetServer |
| 热重载 | 不支持 | 支持 |
| UI Node 树手写 | 大量 | Widget 库封装 |
| 模块 pub 审计 | 未审计 | 全部审计 |
