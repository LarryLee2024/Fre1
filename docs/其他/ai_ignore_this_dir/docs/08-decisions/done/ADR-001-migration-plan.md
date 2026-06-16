---
id: 08-decisions.ADR-001-migration-plan
title: ADR 001 Migration Plan
status: stable
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - adr
---

# ADR-001: 七层架构迁移计划

## 状态: Accepted

**版本**: 1.0  
**日期**: 2026-06-14  
**范围**: 157 个 Rust 源文件（33,771 行）从扁平结构迁移到七层架构  
**预估工期**: 30 个工作日（7 个执行阶段 Phase 0-6，外加 Phase 7-8 为未来规划）  
**关键约束**: 每个 Phase 必须保持项目可编译、可运行、所有测试通过

---

## 背景

### 当前状态

`src_待迁移` 目录包含 157 个 Rust 文件、33,771 行代码，采用扁平的 Feature First 结构。虽然遵循了 Feature First 原则，但存在以下关键问题：

| # | 问题 | 严重度 | 影响范围 |
|---|------|--------|---------|
| 1 | **Unit God Component**: `Unit` 通过 `#[require(...)]` 强制依赖 10 个组件 | 高 | character/components.rs |
| 2 | **跨模块循环依赖**: `character → ui::events::MovementIntent` | 高 | character/mod.rs, ui/events.rs |
| 3 | **Double-Event 模式**: `infrastructure/logging/events.rs` 镜像了 12 个领域事件 | 中 | infrastructure/logging/ |
| 4 | **core/ 超载**: 同时承担 shared + content + infra 职责 | 高 | core/ 全目录 |
| 5 | **ID 位置错误**: `core/id/` 应在 `shared/ids/` | 中 | core/id/ |
| 6 | **错误位置错误**: `core/error/` 应在各领域 `domain/` + `shared/error/` | 中 | core/error/ |
| 7 | **String::new() 占位符**: 58 处空字符串作为实体名称占位 | 低 | 18 个文件 |
| 8 | **Entity::PLACEHOLDER**: 15 处占位符用于默认 Entity | 中 | 4 个文件 |
| 9 | **13 个资源手动重置**: `turn/mod.rs` cleanup_ingame 手动重置 13 个 Resource | 中 | turn/mod.rs |
| 10 | **缺少 App 层**: main.rs 直接注册所有 Plugin，无统一装配层 | 高 | main.rs |
| 11 | **缺少 Content 层**: 38 个 RON 配置文件散落在 `assets/` 中 | 高 | assets/*.ron |
| 12 | **assets.rs 在根目录**: 资源加载应在 infrastructure/ | 低 | assets.rs |
| 13 | **input.rs 在根目录**: 输入处理应在 ui/ | 低 | input.rs |

### 当前模块统计

| 类别 | 数量 | 说明 |
|------|------|------|
| Rust 文件 | 157 | 总计 33,771 行 |
| 顶层模块 | 14 | ai, battle, buff, campaign, character, core, debug, equipment, infrastructure, input, inventory, map, skill, turn, ui |
| Plugin 实现 | 39 | 需合并/重组为 22 个 |
| Component 类型 | 66 | 需清理 God Component |
| Resource 类型 | 56 | 需统一重置机制 |
| Message 类型 | 35 | 需去重（logging 12 个是镜像） |
| RON 配置文件 | 38 | 需迁移到 content/ |

### 迁移目标

1. 建立七层架构（App / Core / Shared / Infrastructure / Content / Modding / Tools）
2. 消除所有已知的架构违规
3. 保持所有现有测试通过
4. 使 4 个并行 feature-developer agents 能独立执行

### 架构规范引用

本 ADR 遵循以下文档的最高优先级约束：

- `docs/01-architecture/README.md` — 七层架构总纲（Version 4.0）
- `docs/01-architecture/00-overview/layer-contracts.md` — 各层边界定义
- `docs/01-architecture/04-events-logging-error/error-architecture.md` — 三层错误模型
- `docs/01-architecture/03-data-config-asset/content-pipeline.md` — Content/Core 分离
- `docs/01-architecture/00-overview/plugin-design.md` — Plugin 注册顺序
- `docs/AI开发宪法完整版.md` — AI 开发最高约束

---

## 决策

### 核心决策

**采用分阶段迁移策略，共 7 个执行阶段（Phase 0-6），Phase 7-8 为未来规划。每个 Phase 保持项目可编译运行。**

迁移顺序遵循**依赖图自底向上**原则：
```
Shared → Core 重组 → Error → Infrastructure → Content → App → Modding → Tools
```

### 架构目标（迁移完成后）

```
src/
├── app/                    # Layer 1: 游戏启动与装配
├── core/                   # Layer 2: 游戏规则（纯领域逻辑）
├── shared/                 # Layer 3: 基础能力（通用工具）
├── infrastructure/         # Layer 4: 技术实现
├── content/                # Layer 5: 内容桥接（配置 → 规则）
├── modding/                # Layer 6: MOD 支持
├── ui/                     # 表现层
└── debug/                  # 调试工具

content/                    # 项目根目录：RON 配置数据
├── characters/
├── skills/
├── buffs/
├── equipments/
├── items/
├── terrains/
├── stages/
├── ai_behaviors/
├── formulas/
└── classes/
```

### 层间依赖规则（迁移后必须满足）

```
App      → 任意层           ✅（仅注册，不含逻辑）
Core     → Shared           ✅（唯一允许的外部依赖）
Shared   → 无               ✅（叶子节点，零外部依赖）
Infra    → Core, Shared     ✅
Content  → Core, Infra, Shared  ✅
UI       → ViewModel only   ✅
Debug    → Core（只读）      ✅
Modding  → Core, Shared, Infra, Content  ✅
Tools    → Core, Shared      ✅
```

**严格禁止**:
- Core → Infra / Content / UI / Modding
- Shared → Core / Infra / UI
- Infra → UI

---

## Module Design

### 七层目标模块组织

#### Layer 1: App — 游戏启动与装配

```
src/app/
├── mod.rs              # 公开导出
├── app_plugin.rs       # 主 Plugin，注册所有子 Plugin
├── game_state.rs       # AppState 定义（MainMenu / InGame / GameOver）
├── schedules.rs        # Schedule 定义
├── sets.rs             # SystemSet 定义
├── startup.rs          # 启动逻辑
├── shutdown.rs          # 关闭逻辑
└── plugins.rs          # Plugin 汇集注册（22 个 Plugin 的最终注册表）
```

**迁移来源**: `main.rs`（Plugin 注册逻辑）、`turn/state.rs`（AppState, TurnPhase）

#### Layer 2: Core — 游戏规则

```
src/core/
├── mod.rs
│
├── battle/                    # 从 src/battle/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # BattlePlugin
│   ├── combat.rs              # 战斗辅助函数
│   ├── record.rs              # BattleRecord + DamageBreakdown
│   ├── log.rs                 # CombatLog
│   ├── domain/
│   │   ├── mod.rs
│   │   └── battle_error.rs    # BattleError (B001-B006) ← 从 core/error/ 迁入
│   └── pipeline/
│       ├── mod.rs             # CombatEventPlugin
│       ├── intent.rs          # CombatIntent
│       ├── generate.rs        # 步骤1：生成效果
│       ├── modify.rs          # 步骤2：修饰效果
│       ├── execute.rs         # 步骤3：执行效果
│       └── trait_trigger.rs   # Trait 触发器
│
├── character/                 # 从 src/character/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # CharacterPlugin
│   ├── unit.rs                # Unit 组件（拆分自 components.rs）
│   ├── faction.rs             # Faction 组件
│   ├── spawn.rs               # UnitPlugin（生成系统）
│   ├── template.rs            # UnitTemplateRegistry
│   ├── marker.rs              # PlayerControlled, AiControlled, Selected
│   ├── movement.rs            # 移动动画
│   ├── movement_execution.rs  # 移动执行
│   ├── domain/
│   │   ├── mod.rs
│   │   └── character_error.rs # CharacterError（新增）
│   └── traits/
│       ├── mod.rs             # TraitPlugin
│       ├── types.rs           # TraitTrigger, TraitEffect, TraitDefinition
│       ├── handlers.rs        # TraitEffectHandler trait 分发
│       └── trait_collection.rs # TraitCollection 组件
│
├── skill/                     # 从 src/skill/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # SkillPlugin
│   ├── skill_def.rs           # SkillDef + SkillData
│   ├── skill_slots.rs         # SkillSlots + SkillCooldowns
│   ├── skill_preview.rs       # 技能效果预览
│   ├── id.rs                  # SkillId（从 core/id/ 合并）
│   └── domain/
│       ├── mod.rs
│       ├── skill_error.rs     # SkillError (S001-S005) ← 从 core/error/ 迁入
│       └── defaults.rs        # 默认技能
│
├── buff/                      # 从 src/buff/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # BuffPlugin
│   ├── buff_def.rs            # BuffData + BuffDef + BuffRegistry
│   ├── buff_instance.rs       # ActiveBuffs + BuffInstance
│   ├── buff_apply.rs          # 穿戴/移除 Buff
│   ├── buff_tick.rs           # 持续效果结算
│   ├── id.rs                  # BuffId（从 core/id/ 合并）
│   └── domain/
│       ├── mod.rs
│       └── buff_error.rs      # BuffError (BF001-BF004) ← 从 core/error/ 迁入
│
├── equipment/                 # 从 src/equipment/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # EquipmentPlugin
│   ├── equipment_def.rs       # EquipmentDef + EquipmentRegistry
│   ├── equipment_slots.rs     # EquipmentSlots 组件
│   ├── equipment_instance.rs  # EquipmentInstance
│   ├── equip.rs               # 穿脱逻辑
│   ├── requirements.rs        # 装备需求检查
│   └── domain/
│       ├── mod.rs
│       └── equipment_error.rs # EquipmentError（新增）
│
├── inventory/                 # 从 src/inventory/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # InventoryPlugin
│   ├── item_def.rs            # ItemDef + ItemRegistry
│   ├── item_instance.rs       # ItemInstance + ItemStack
│   ├── container.rs           # Container 组件
│   ├── battle_bag.rs          # BattleInventory
│   ├── transfer.rs            # 物品转移
│   ├── use_item.rs            # 消耗品使用
│   ├── resources.rs           # InventoryResources
│   ├── id.rs                  # ItemId（从 core/id/ 合并）
│   └── domain/
│       ├── mod.rs
│       └── inventory_error.rs # InventoryError (I001-I005) ← 从 core/error/ 迁入
│
├── ai/                        # 从 src/ai/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # AiPlugin
│   ├── behavior.rs            # AiBehaviorRegistry
│   ├── strategy.rs            # TargetSelector/MoveSelector/SkillSelector
│   ├── decision.rs            # AI 主系统
│   ├── targeting.rs           # 目标选择
│   ├── movement.rs            # 移动选择
│   ├── skill_select.rs        # 技能选择
│   └── domain/
│       ├── mod.rs
│       └── ai_error.rs        # AiError（新增）
│
├── map/                       # 从 src/map/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # MapPlugin
│   ├── terrain_grid.rs        # TerrainGrid
│   ├── occupancy_grid.rs      # OccupancyGrid
│   ├── game_map.rs            # GameMap 坐标转换
│   ├── hp_bar.rs              # HP 条
│   └── pathfinding/
│       ├── mod.rs
│       ├── algorithms.rs      # BFS 寻路
│       └── cost.rs            # 地形消耗计算
│
├── turn/                      # 从 src/turn/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # TurnPlugin
│   ├── turn_order.rs          # TurnOrder + TurnState
│   ├── turn_phase.rs          # TurnPhase SubState（从 state.rs 拆分）
│   └── victory.rs             # 胜利条件检查
│
├── campaign/                  # 从 src/campaign/ 迁入
│   ├── mod.rs
│   ├── plugin.rs              # CampaignPlugin
│   ├── def.rs                 # CampaignDef
│   ├── loader.rs              # CampaignLoader
│   ├── progress.rs            # CampaignProgress
│   ├── progression.rs         # CampaignProgression
│   └── registry.rs            # CampaignRegistry
│
├── attribute/                 # 现有 core/attribute/（保留）
│   ├── mod.rs                 # Attributes Component
│   └── types.rs               # AttributeKind, ModifierSource 等
│
├── effect/                    # 现有 core/effect/（保留）
│   ├── mod.rs                 # EffectPlugin
│   ├── handler.rs             # EffectHandlerRegistry
│   └── types.rs               # EffectDef, PendingEffect 等
│
├── modifier_rule.rs           # 现有（保留）
├── registry_loader.rs         # 现有（保留）
├── snapshot.rs                # 现有（保留）
├── tag.rs                     # 现有 GameplayTags（保留）
├── tag_def.rs                 # → 迁至 content/tag_def.rs（见 ADR-002 Debt 4）
└── attribute_def.rs           # → 迁至 content/attribute_def.rs（见 ADR-002 Debt 4）
```

**迁移来源**: `src/battle/`, `src/character/`, `src/skill/`, `src/buff/`, `src/equipment/`, `src/inventory/`, `src/ai/`, `src/map/`, `src/turn/`, `src/campaign/`

#### Layer 3: Shared — 基础能力

```
src/shared/
├── mod.rs
├── shared_plugin.rs           # SharedPlugin（统一入口）
│
├── ids/                       # 从 src/core/id/ 迁入
│   ├── mod.rs
│   ├── unit_id.rs             # UnitId(String)
│   ├── skill_id.rs            # SkillId(String)
│   ├── buff_id.rs             # BuffId(String)
│   └── item_id.rs             # ItemId(String)
│
├── error/                     # 从 src/core/error/game_result.rs 迁入
│   ├── mod.rs
│   ├── result.rs              # GameResult<T> + InfrastructureError
│   ├── context.rs             # ErrorContext trait
│   └── extensions.rs          # LogIfError trait
│
├── events/                    # 跨模块领域事件白名单
│   ├── mod.rs
│   └── event_whitelist.rs     # 从 infrastructure/audit/whitelist.rs 迁入
│
├── audit/                     # 审计轨迹白名单（从 infrastructure/audit/ 部分迁入）
│   ├── mod.rs
│   └── whitelist.rs           # EventWhitelist
│
├── random/                    # 确定性随机数（空壳）
│   └── mod.rs
├── math/                      # 游戏数学工具（空壳）
│   └── mod.rs
├── time/                      # 时间工具（空壳）
│   └── mod.rs
├── collections/               # 通用集合（空壳）
│   └── mod.rs
├── validation/                # 校验工具（空壳）
│   └── mod.rs
├── constants/                 # 全局常量（空壳）
│   └── mod.rs
├── traits/                    # 核心 trait（空壳）
│   └── mod.rs
├── testing/                   # 测试工具（空壳）
│   ├── mod.rs
│   └── spawns.rs              # spawn_test_battle() 等
└── versioning/                # 版本管理（空壳）
    └── mod.rs
```

**迁移来源**: `src/core/id/`, `src/core/error/game_result.rs`, `src/infrastructure/audit/whitelist.rs`

#### Layer 4: Infrastructure — 技术实现

```
src/infrastructure/
├── mod.rs
│
├── assets/                    # 从 src/assets.rs 迁入
│   ├── mod.rs
│   ├── plugin.rs              # AssetsPlugin
│   ├── asset_error.rs         # AssetError（新增）
│   ├── game_assets.rs         # GameAssets Resource
│   └── loaders/
│       ├── mod.rs
│       └── ron_loader.rs      # RON 加载工具
│
├── logging/                   # 现有（保留，去除 double-event）
│   ├── mod.rs                 # LogPlugin
│   ├── observer.rs            # 日志观察者（消费领域事件，不镜像）
│   └── plugin.rs
│
├── audit/                     # 现有（保留基础设施部分）
│   ├── mod.rs                 # AuditPlugin
│   ├── trail.rs               # AuditTrail Resource
│   └── event.rs               # AuditEvent + AuditMetadata
│
├── persistence/               # 未来扩展（空壳）
│   ├── mod.rs
│   └── plugin.rs
├── localization/              # 未来扩展（空壳）
│   ├── mod.rs
│   └── plugin.rs
├── replay/                    # 未来扩展（空壳）
│   ├── mod.rs
│   └── plugin.rs
├── config/                    # 未来扩展（空壳）
│   ├── mod.rs
│   └── plugin.rs
└── hot_reload/                # 未来扩展（空壳）
    ├── mod.rs
    └── plugin.rs
```

**迁移来源**: `src/assets.rs`, `src/infrastructure/logging/`, `src/infrastructure/audit/`

#### Layer 5: Content — 内容桥接

```
src/content/
├── mod.rs
├── content_plugin.rs          # ContentPlugin（统一入口）
│
├── characters/                # 角色内容加载
│   ├── mod.rs
│   └── character_content.rs   # RON → UnitTemplate → UnitTemplateRegistry
│
├── skills/                    # 技能内容加载
│   ├── mod.rs
│   └── skill_content.rs       # RON → SkillDef → SkillData → SkillRegistry
│
├── buffs/                     # Buff 内容加载
│   ├── mod.rs
│   └── buff_content.rs        # RON → BuffDef → BuffData → BuffRegistry
│
├── equipments/                # 装备内容加载
│   ├── mod.rs
│   └── equipment_content.rs   # RON → EquipmentDef → EquipmentRegistry
│
├── items/                     # 物品内容加载
│   ├── mod.rs
│   └── item_content.rs        # RON → ItemDef → ItemRegistry
│
├── terrains/                  # 地形内容加载
│   ├── mod.rs
│   └── terrain_content.rs     # RON → TerrainDef → TerrainRegistry
│
├── stages/                    # 关卡内容加载
│   ├── mod.rs
│   └── stage_content.rs       # RON → LevelConfig → LevelRegistry
│
├── ai_behaviors/              # AI 行为内容加载
│   ├── mod.rs
│   └── ai_behavior_content.rs # RON → AiBehavior → AiBehaviorRegistry
│
├── classes/                   # 职业内容加载
│   ├── mod.rs
│   └── class_content.rs       # RON → ClassData（Trait + Modifier 集合）
│
└── formulas/                  # 公式内容加载
    ├── mod.rs
    └── formula_content.rs     # RON → ModifierRule → ModifierRuleRegistry
```

**来源**: 全新创建，桥接 `content/*.ron` → `core/` Registry

#### Layer 6: Modding — MOD 支持（未来）

```
src/modding/
├── mod.rs
├── api/                       # MOD API 暴露
├── registry/                  # MOD 注册表
├── loaders/                   # MOD 加载器
├── validators/                # MOD 校验器
├── sandbox/                   # MOD 沙箱环境
└── compatibility/             # MOD 兼容性
```

#### Layer 7: Tools — 开发工具（未来）

```
tools/
├── content_editor/
├── data_validator/
├── balance_checker/
└── replay_inspector/
```

#### 表现层（跨层，保留顶层）

```
src/ui/                        # UI 表现层（保留顶层）
├── mod.rs
├── plugin.rs                  # UiPlugin
├── input.rs                   # 从 src/input.rs 迁入
├── view_models.rs
├── command_handler.rs
├── events.rs                  # UiCommand Message
├── panels/                    # 功能面板
├── screens/                   # 屏幕
├── widgets/                   # 基础组件
└── ...

src/debug/                     # 调试工具（保留顶层）
├── mod.rs
├── plugin.rs                  # DebugPlugin（#[cfg(feature = "dev")]）
└── viewers/
```

---

## Communication Design

### 迁移后的 Message 注册表

| Message | 发送方（迁移后位置） | 接收方（迁移后位置） | 用途 |
|---------|---------------------|---------------------|------|
| `UiCommand` | `ui/input.rs` | `ui/command_handler.rs` | UI→Logic 意图 |
| `DamageApplied` | `core/battle/pipeline/execute.rs` | `ui/combat_vfx_handler.rs`, `ui/combat_log_handler.rs`, `core/battle/record.rs` | 伤害通知 |
| `HealApplied` | `core/battle/pipeline/execute.rs` | `ui/combat_log_handler.rs`, `core/battle/record.rs` | 治疗通知 |
| `CharacterDied` | `core/battle/pipeline/execute.rs` | `core/battle/events.rs`, `ui/combat_log_handler.rs`, `core/battle/record.rs` | 死亡通知 |
| `StunApplied` | `core/buff/buff_tick.rs` | `ui/combat_log_handler.rs`, `core/battle/record.rs` | 晕眩通知 |
| `DotApplied` | `core/buff/buff_tick.rs` | `ui/combat_log_handler.rs`, `core/battle/record.rs` | DoT 通知 |
| `HotApplied` | `core/buff/buff_tick.rs` | `ui/combat_log_handler.rs`, `core/battle/record.rs` | HoT 通知 |
| `EquipItem` | `ui/panels/inventory_panel.rs` | `core/equipment/equip.rs` | 穿戴装备 |
| `UnequipItem` | `ui/panels/inventory_panel.rs` | `core/equipment/equip.rs` | 脱卸装备 |
| `ItemEquipped` | `core/equipment/equip.rs` | `ui/combat_log_handler.rs` | 装备已穿 |
| `ItemUnequipped` | `core/equipment/equip.rs` | `ui/combat_log_handler.rs` | 装备已脱 |
| `UseItem` | `ui/panels/inventory_panel.rs` | `core/inventory/use_item.rs` | 使用物品 |
| `TransferItem` | `ui/panels/inventory_panel.rs` | `core/inventory/transfer.rs` | 物品转移 |
| `TurnStarted` | `core/turn/turn_order.rs` | `core/battle/record.rs` | 回合开始 |
| `TurnEnded` | `core/turn/turn_order.rs` | `core/battle/record.rs` | 回合结束 |
| `ForceEndTurn` | `ui/command_handler.rs` | `core/turn/turn_order.rs` | 强制结束回合 |

### 消除 Double-Event 模式

**当前问题**: `infrastructure/logging/events.rs` 定义了 12 个镜像 Message（`ConfigLoaded`, `BuffApplied`, `BuffRemoved` 等），与领域事件重复。

**迁移方案**:
1. 删除 `infrastructure/logging/events.rs` 中的 12 个镜像 Message
2. `infrastructure/logging/observer.rs` 直接监听领域 Message（`DamageApplied`, `CharacterDied` 等）
3. 领域事件成为唯一事实源（`shared/events/event_whitelist.rs` 管理白名单）

### 通信机制分配

| 通信方式 | 迁移后使用场景 | 示例 |
|----------|---------------|------|
| **Hook** | 组件固有行为 | `Dead` 标签添加时移除移动组件 |
| **Observer** | 同 Feature 内局部响应 | Pointer\<Click\> → UiCommand |
| **Message** | 跨 Feature 广播 | `DamageApplied` → battle → UI |
| **函数调用** | 模块内部 | `apply_buff()`, `remove_buff()` |

### 移除 Entity 当对象使用

**当前违规**: `character → ui::events::MovementIntent` 的跨模块直接依赖。

**迁移方案**:
1. 将 `MovementIntent` 从 `ui/events.rs` 移到 `core/movement/events.rs`（movement 是移动关注点，CombatIntent 留在 `core/battle/pipeline/intent.rs`）
2. `character` 模块不再引用 `ui` 模块的任何类型
3. UI 通过 `UiCommand::MoveUnit` → command_handler → 发送 `MovementIntent` Message

---

## 边界定义

### 各层依赖矩阵（迁移后必须满足）

| 依赖方 ↓ \ 被依赖方 → | App | Core | Shared | Infra | Content | UI | Debug | Modding |
|------------------------|-----|------|--------|-------|---------|-----|-------|---------|
| **App** | - | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Core** | ❌ | - | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Shared** | ❌ | ❌ | - | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Infra** | ❌ | ✅ | ✅ | - | ✅ | ❌ | ❌ | ❌ |
| **Content** | ❌ | ✅ | ✅ | ✅ | - | ❌ | ❌ | ❌ |
| **UI** | ❌ | ❌ | ✅ | ❌ | ❌ | - | ❌ | ❌ |
| **Debug** | ❌ | ✅(只读) | ✅ | ✅(只读) | ❌ | ❌ | - | ❌ |
| **Modding** | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | - |

### Core 内部模块边界

Core 内部模块之间通过 Message 通信，禁止直接访问内部组件：

```
core/battle   → core/skill      只能通过 CombatIntent / DamageApplied Message
core/skill    → core/buff       只能通过 SkillCastStarted / SkillCastFinished Message
core/equipment → core/buff      只能通过 EquipItem / ItemEquipped Message
core/character → core/battle    只能通过 CharacterDied / UnitMoved Message
core/turn     → core/battle     只能通过 TurnStarted / TurnEnded Message
core/ai       → core/battle     只能通过 CombatIntent（AI 与玩家共享 Pipeline）
```

### 错误归属边界

| 错误类型 | 放置位置 | 示例 |
|---------|---------|------|
| 领域错误 | `core/xxx/domain/xxx_error.rs` | SkillError, BattleError, BuffError |
| 基础设施错误 | `infrastructure/xxx/xxx_error.rs` | SaveError, AssetError |
| 共享错误工具 | `shared/error/` | GameResult\<T\>, ErrorContext, LogIfError |

**严格禁止**:
- 全局统一 `AppError` 大枚举
- `anyhow::Error` 或 `Box<dyn Error>` 作为业务层返回类型
- 领域错误放在 infrastructure/ 或 shared/
- 基础设施错误包含领域语义

---

## Forbidden（禁止事项）

### 迁移期间的禁止事项

| # | 禁止事项 | 理由 | 违反后果 |
|---|---------|------|---------|
| F1 | 🟥 **迁移期间禁止新增功能** | 迁移是纯重构，不引入新业务逻辑 | 立即回滚 |
| F2 | 🟥 **每个 Phase 必须通过 `cargo build` 和 `cargo test`** | 保持项目可运行 | 暂停迁移，修复后继续 |
| F3 | 🟥 **禁止一次性移动所有文件** | 增量迁移，每模块独立验证 | 拆分为更小的步骤 |
| F4 | 🟥 **禁止修改 Definition 配置数据** | Definition/Instance 分离不可豁免 | 回滚配置变更 |
| F5 | 🟥 **禁止在迁移中引入新的跨层依赖** | 架构违规不可累积 | 重新设计模块边界 |
| F6 | 🟥 **禁止删除测试** | 测试是迁移的安全网 | 恢复测试 |
| F7 | 🟥 **禁止绕过 Effect Pipeline** | 统一管线是架构铁律 | 停止并修复 |
| F8 | 🟥 **禁止使用 `anyhow::Error`** | 业务层错误必须分领域定义 | 替换为领域错误 |
| F9 | 🟥 **禁止在 Core 层 use Infrastructure** | Core 只依赖 Shared | 移动到正确层级 |
| F10 | 🟥 **禁止在 Shared 层 use Core** | Shared 零外部依赖 | 移动到 Core |

### 架构宪法强制约束

以下约束在整个迁移期间不可豁免：

1. **Feature First**: 按业务拆模块，不按技术拆模块
2. **Definition / Instance 分离**: 配置与运行时状态完全隔离
3. **Rule / Content 分离**: 代码只实现规则，配置只定义内容
4. **Logic / Presentation 分离**: 业务逻辑与表现层完全隔离
5. **Entity 仅为 ID**: 禁止 EntityManager、禁止在 Entity 上调用方法
6. **Tag Component 优先**: 禁止 `is_xxx: bool` 字段
7. **Effect Pipeline 统一**: 所有战斗效果必须走 generate → modify → execute

---

## Definition / Instance Design

### 迁移后的 Definition / Instance 分离

| 模块 | Definition（不可变配置） | Instance（运行时状态） | 层级归属 |
|------|------------------------|----------------------|---------|
| Buff | `BuffDef` / `BuffData` | `BuffInstance` / `ActiveBuffs` | Core |
| Skill | `SkillDef` / `SkillData` | `SkillSlots` / `SkillCooldowns` | Core |
| Equipment | `EquipmentDef` | `EquipmentInstance` / `EquipmentSlots` | Core |
| Item | `ItemDef` | `ItemInstance` / `ItemStack` | Core |
| Unit | `UnitTemplate` | `Unit` + `Attributes` + `ActiveBuffs` | Core |
| AI | `AiBehavior` | `AiStrategyState` | Core |
| Terrain | `TerrainDef` | `TerrainGrid` | Core |
| Level | `LevelConfig` | `MapConfig` / `MapDataState` | Core |

### Content 层的 Definition 加载流

```
content/*.ron
    ↓ [AssetServer 加载]
XxxDef（RON 反序列化类型，TagName 字符串）
    ↓ [impl From<XxxDef> for XxxData]
XxxData（运行时类型，GameplayTag 位掩码）
    ↓ [Registry.insert()]
XxxRegistry（全局注册表，不可变）
    ↓ [System 查询]
运行时业务逻辑
```

---

## 迁移阶段详解

### Phase 0: 脚手架搭建（Day 1）

**目标**: 创建目标目录结构骨架，不移动任何代码。

**步骤**:
1. 创建 `src/app/` 目录（空 mod.rs）
2. 创建 `src/shared/` 目录结构（所有子目录 + 空 mod.rs）
3. 创建 `src/content/` 目录结构（所有子目录 + 空 mod.rs）
4. 创建 `src/modding/` 目录结构（空 mod.rs）
5. 创建项目根目录 `content/`（RON 配置目标位置）
6. 更新 `.gitignore`（添加 `content/` 到 Git LFS）
7. 在 `src/lib.rs` 中添加新的 `pub mod` 声明（空模块）

**验证清单**:
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] 所有目标目录已创建
- [ ] 现有代码未修改

---

### Phase 1: Shared 层迁移（Day 2-4, 3 天）

**目标**: 建立 Shared 层，迁移通用工具。

#### 1.1 迁移强类型 ID（Day 2）

**迁移内容**: `src/core/id/` → `src/shared/ids/`

| 源文件 | 目标文件 | 类型 |
|--------|---------|------|
| `core/id/unit_id.rs` | `shared/ids/unit_id.rs` | UnitId(String) |
| `core/id/skill_id.rs` | `shared/ids/skill_id.rs` | SkillId(String) |
| `core/id/buff_id.rs` | `shared/ids/buff_id.rs` | BuffId(String) |
| `core/id/item_id.rs` | `shared/ids/item_id.rs` | ItemId(String) |
| `core/id/mod.rs` | `shared/ids/mod.rs` | 模块导出 |

**步骤**:
1. 创建 `src/shared/ids/` 目录
2. 移动 4 个 ID 文件 + mod.rs
3. 创建 `src/shared/ids/mod.rs` 导出所有 ID 类型
4. 在 `src/shared/mod.rs` 中添加 `pub mod ids;`
5. 全局搜索替换：`use crate::core::id::` → `use crate::shared::ids::`
6. 验证编译通过

**影响范围**: 所有引用 `UnitId`, `SkillId`, `BuffId`, `ItemId` 的文件（约 30+ 文件）

**技术债务修复**:
- ✅ **修复 #5**: ID 从 core/ 迁移到 shared/ids/

#### 1.2 迁移共享错误工具（Day 2-3）

**迁移内容**: `src/core/error/game_result.rs` → `src/shared/error/`

| 源文件 | 目标文件 | 说明 |
|--------|---------|------|
| `core/error/game_result.rs` | `shared/error/result.rs` | GameResult\<T\> + InfrastructureError |
| （新增） | `shared/error/context.rs` | ErrorContext trait |
| （新增） | `shared/error/extensions.rs` | LogIfError trait |

**步骤**:
1. 创建 `src/shared/error/` 目录
2. 移动 `game_result.rs` → `result.rs`
3. 创建 `context.rs`（ErrorContext trait，带 `with_context` 方法）
4. 创建 `extensions.rs`（LogIfError trait，带 `log_if_error` 方法）
5. 创建 `mod.rs` 导出所有公共类型
6. 更新所有 `use crate::core::error::game_result::` 引用
7. 验证编译通过

**影响范围**: 所有引用 `GameResult`, `InfrastructureError` 的文件

**技术债务修复**:
- ✅ **修复 #6 (部分)**: GameResult 从 core/error/ 迁移到 shared/error/

#### 1.3 创建 Shared 层其他模块（Day 3-4）

**步骤**:
1. 创建 `src/shared/events/mod.rs`（领域事件白名单）
2. 创建 `src/shared/audit/`（从 `infrastructure/audit/whitelist.rs` 迁入）
3. 创建空壳模块：`random/`, `math/`, `time/`, `collections/`, `validation/`, `constants/`, `traits/`, `testing/`, `versioning/`
4. 创建 `src/shared/shared_plugin.rs`（SharedPlugin 统一入口）
5. 在 `shared/mod.rs` 中导出所有子模块
6. 验证编译通过

**验证清单**:
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `src/shared/ids/` 包含 4 个 ID 类型
- [ ] `src/shared/error/` 包含 GameResult + 工具
- [ ] 所有 `use crate::core::id::` 已替换为 `use crate::shared::ids::`
- [ ] 所有 `use crate::core::error::game_result::` 已替换

---

### Phase 2: Core 层重组（Day 5-11, 7 天）

**目标**: 将业务模块从顶层迁移到 `src/core/` 下。

**迁移顺序**（按依赖图自底向上）:
```
map → skill → buff → equipment → inventory → character → battle → ai → turn → campaign
```

#### 2.1 迁移 Map 模块（Day 5）

**迁移内容**: `src/map/` → `src/core/map/`

| 源文件 | 目标文件 |
|--------|---------|
| `map/mod.rs` | `core/map/mod.rs` |
| `map/data.rs` | `core/map/data.rs` |
| `map/grid.rs` | `core/map/game_map.rs` |
| `map/hp_bar.rs` | `core/map/hp_bar.rs` |
| `map/pathfinding/` | `core/map/pathfinding/` |
| `map/runtime/` | `core/map/runtime/` |

**步骤**:
1. 在 `src/core/` 下创建 `map/` 目录
2. 移动所有文件
3. 更新 `core/mod.rs` 添加 `pub mod map;`
4. 全局搜索替换：`use crate::map::` → `use crate::core::map::`
5. 移除 `src/lib.rs` 中的 `pub mod map;`
6. 验证编译通过

#### 2.2 迁移 Skill 模块（Day 5-6）

**迁移内容**: `src/skill/` → `src/core/skill/`

**步骤**:
1. 创建 `core/skill/` 目录
2. 移动所有文件
3. 创建 `core/skill/domain/` 子目录
4. 迁移 `core/error/skill_error.rs` → `core/skill/domain/skill_error.rs`
5. 更新 `core/mod.rs`
6. 全局搜索替换：`use crate::skill::` → `use crate::core::skill::`
7. 全局搜索替换：`use crate::core::error::SkillError` → `use crate::core::skill::domain::SkillError`
8. 验证编译通过

#### 2.3 迁移 Buff 模块（Day 6）

**迁移内容**: `src/buff/` → `src/core/buff/`

**步骤**:
1. 创建 `core/buff/` 目录
2. 移动所有文件
3. 创建 `core/buff/domain/` 子目录
4. 迁移 `core/error/buff_error.rs` → `core/buff/domain/buff_error.rs`
5. 更新 `core/mod.rs`
6. 全局搜索替换
7. 验证编译通过

#### 2.4 迁移 Equipment 模块（Day 6-7）

**迁移内容**: `src/equipment/` → `src/core/equipment/`

**步骤**:
1. 创建 `core/equipment/` 目录
2. 移动所有文件
3. 创建 `core/equipment/domain/` 子目录
4. 创建 `equipment_error.rs`（新增 EquipmentError 枚举）
5. 更新 `core/mod.rs`
6. 全局搜索替换
7. 验证编译通过

#### 2.5 迁移 Inventory 模块（Day 7）

**迁移内容**: `src/inventory/` → `src/core/inventory/`

**步骤**:
1. 创建 `core/inventory/` 目录
2. 移动所有文件
3. 创建 `core/inventory/domain/` 子目录
4. 迁移 `core/error/inventory_error.rs` → `core/inventory/domain/inventory_error.rs`
5. 更新 `core/mod.rs`
6. 全局搜索替换
7. 验证编译通过

#### 2.6 迁移 Character 模块（Day 7-8）

**迁移内容**: `src/character/` → `src/core/character/`

**步骤**:
1. 创建 `core/character/` 目录
2. 移动所有文件
3. 创建 `core/character/domain/` 子目录
4. 创建 `character_error.rs`（新增 CharacterError 枚举）
5. **修复跨模块循环依赖**: 将 `MovementIntent` 从 `ui/events.rs` 移到 `core/movement/events.rs`
6. 更新 `character/mod.rs` 中的 `use crate::ui::events::MovementIntent` → `use crate::core::movement::events::MovementIntent`
7. 更新 `core/mod.rs`
8. 全局搜索替换
9. 验证编译通过

**技术债务修复**:
- ✅ **修复 #2**: 消除 character → ui 循环依赖

#### 2.7 迁移 Battle 模块（Day 8-9）

**迁移内容**: `src/battle/` → `src/core/battle/`

**步骤**:
1. 创建 `core/battle/` 目录
2. 移动所有文件
3. 创建 `core/battle/domain/` 子目录
4. 迁移 `core/error/battle_error.rs` → `core/battle/domain/battle_error.rs`
5. 更新 `core/mod.rs`
6. 全局搜索替换
7. 验证编译通过

#### 2.8 迁移 AI 模块（Day 9-10）

**迁移内容**: `src/ai/` → `src/core/ai/`

**步骤**:
1. 创建 `core/ai/` 目录
2. 移动所有文件
3. 创建 `core/ai/domain/` 子目录
4. 创建 `ai_error.rs`（新增 AiError 枚举）
5. 更新 `core/mod.rs`
6. 全局搜索替换
7. 验证编译通过

#### 2.9 迁移 Turn 模块（Day 10）

**迁移内容**: `src/turn/` → `src/core/turn/`

**步骤**:
1. 创建 `core/turn/` 目录
2. 移动所有文件
3. 更新 `core/mod.rs`
4. 全局搜索替换
5. 验证编译通过

#### 2.10 迁移 Campaign 模块（Day 10-11）

**迁移内容**: `src/campaign/` → `src/core/campaign/`

**步骤**:
1. 创建 `core/campaign/` 目录
2. 移动所有文件
3. 更新 `core/mod.rs`
4. 全局搜索替换
5. 验证编译通过

#### 2.11 清理旧位置（Day 11）

**步骤**:
1. 删除 `src/core/error/`（已全部分拆到各领域 domain/）
2. 删除 `src/core/id/`（已迁移到 shared/ids/）
3. 删除 `src/lib.rs` 中所有已迁移模块的 `pub mod` 声明
4. 验证编译通过
5. 运行 `cargo clippy` 检查警告

**验证清单**:
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `src/` 下无顶层业务模块（battle/, skill/, buff/ 等都在 core/ 下）
- [ ] `src/core/error/` 不存在
- [ ] `src/core/id/` 不存在
- [ ] Core 层不依赖 Infrastructure 层
- [ ] 无跨模块循环依赖

---

### Phase 3: 错误架构迁移（Day 12-13, 2 天）

**目标**: 完成错误三层模型的最终形态。

#### 3.1 验证领域错误归属（Day 12）

Phase 2 中已将错误迁移到各领域 domain/ 子目录。本步骤验证：

| 错误类型 | 当前位置 | 目标位置 | 状态 |
|---------|---------|---------|------|
| BattleError | `core/battle/domain/battle_error.rs` | ✅ 正确 | 已完成 |
| SkillError | `core/skill/domain/skill_error.rs` | ✅ 正确 | 已完成 |
| BuffError | `core/buff/domain/buff_error.rs` | ✅ 正确 | 已完成 |
| InventoryError | `core/inventory/domain/inventory_error.rs` | ✅ 正确 | 已完成 |
| EquipmentError | `core/equipment/domain/equipment_error.rs` | ✅ 正确 | 已完成 |
| CharacterError | `core/character/domain/character_error.rs` | ✅ 正确 | 已完成 |
| AiError | `core/ai/domain/ai_error.rs` | ✅ 正确 | 已完成 |

#### 3.2 完善 shared/error/ 工具（Day 12）

确保 `shared/error/` 包含：

```rust
// shared/error/result.rs
pub type GameResult<T> = Result<T, InfrastructureError>;

#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    #[error("[INF001] 持久化错误: {0}")]
    Persistence(String),
    #[error("[INF002] 资源加载错误: {0}")]
    Asset(String),
    #[error("[INF003] 网络错误: {0}")]
    Network(String),
    #[error("[INF004] 配置错误: {0}")]
    Config(String),
}
```

#### 3.3 统一错误码规范（Day 13）

为新增的领域错误添加错误码：

| 领域 | 前缀 | 新增错误 |
|------|------|---------|
| Character | C | C001-C003 |
| Equipment | E | E001-E003 |
| AI | AI | AI001-AI003 |

**验证清单**:
- [ ] 每个领域有独立的错误枚举
- [ ] 所有错误变体携带完整上下文
- [ ] 无全局 AppError 大枚举
- [ ] 无 `anyhow::Error` 或 `Box<dyn Error>`
- [ ] 所有错误码格式统一（领域前缀 + 3 位序号）

---

### Phase 4: Infrastructure 层扩展（Day 14-16, 3 天）

**目标**: 扩展基础设施层，迁移资源加载。

#### 4.1 迁移资源加载（Day 14）

**迁移内容**: `src/assets.rs` → `src/infrastructure/assets/`

**步骤**:
1. 创建 `src/infrastructure/assets/` 目录
2. 移动 `assets.rs` → `infrastructure/assets/game_assets.rs`
3. 创建 `infrastructure/assets/plugin.rs`（AssetsPlugin）
4. 创建 `infrastructure/assets/asset_error.rs`（AssetError 枚举）
5. 创建 `infrastructure/assets/loaders/ron_loader.rs`（RON 加载工具）
6. 更新 `infrastructure/mod.rs`
7. 全局搜索替换：`use crate::assets::` → `use crate::infrastructure::assets::`
8. 验证编译通过

**技术债务修复**:
- ✅ **修复 #10**: assets.rs 从根目录迁移到 infrastructure/assets/

#### 4.2 消除 Double-Event 模式（Day 14-15）

**当前问题**: `infrastructure/logging/events.rs` 定义了 12 个镜像 Message。

**迁移步骤**:
1. 删除 `infrastructure/logging/events.rs` 中的 12 个 Message 定义
2. 修改 `infrastructure/logging/observer.rs`：直接监听领域 Message
3. 验证所有日志输出仍然正常

**影响范围**: `infrastructure/logging/` 目录

**技术债务修复**:
- ✅ **修复 #3**: 消除 Double-Event 模式

#### 4.3 创建 Infrastructure 新模块骨架（Day 15-16）

**步骤**:
1. 创建 `infrastructure/persistence/`（空壳）
2. 创建 `infrastructure/localization/`（空壳）
3. 创建 `infrastructure/replay/`（空壳）
4. 创建 `infrastructure/config/`（空壳）
5. 创建 `infrastructure/hot_reload/`（空壳）
6. 验证编译通过

#### 4.4 统一 Resource 重置机制（Day 15-16）

**当前问题**: `turn/mod.rs` 中 `cleanup_ingame()` 手动重置 13 个 Resource。

**迁移方案**:
1. 创建 `shared/resettable.rs`（ResettableResource trait）
2. 让所有需要重置的 Resource 实现该 trait
3. 在 `app/shutdown.rs` 中统一调用重置逻辑
4. 从 `turn/mod.rs` 中移除手动重置代码

**技术债务修复**:
- ✅ **修复 #9**: 统一 Resource 重置机制

**验证清单**:
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `infrastructure/assets/` 包含资源加载逻辑
- [ ] 无 Double-Event 模式
- [ ] 13 个 Resource 通过 ResettableResource trait 统一重置

---

### Phase 5: Content 层（Day 17-23, 7 天）— 最高风险

**目标**: 建立内容桥接层，分离数据与规则。

#### 5.1 创建 Content 层代码骨架（Day 17）

**步骤**:
1. 创建 `src/content/` 目录结构
2. 创建 `content/content_plugin.rs`（ContentPlugin 统一入口）
3. 为每个内容类型创建子目录和空 mod.rs
4. 验证编译通过

#### 5.2 创建项目根目录 content/（Day 17）

**步骤**:
1. 创建 `content/` 目录
2. 创建子目录：`characters/`, `skills/`, `buffs/`, `equipments/`, `items/`, `terrains/`, `stages/`, `ai_behaviors/`, `formulas/`, `classes/`
3. 更新 `.gitignore`

#### 5.3 迁移 RON 配置文件（Day 18-21）— 逐步迁移

**内容迁移映射表**:

| 源路径 | 目标路径 | 文件数 | Registry |
|--------|---------|--------|----------|
| `assets/units/*.ron` | `content/characters/*.ron` | 6 | UnitTemplateRegistry |
| `assets/skills/*.ron` | `content/skills/*.ron` | 6 | SkillRegistry |
| `assets/buffs/*.ron` | `content/buffs/*.ron` | 8 | BuffRegistry |
| `assets/terrains/*.ron` | `content/terrains/*.ron` | 4 | TerrainRegistry |
| `assets/ai/*.ron` | `content/ai_behaviors/*.ron` | 3 | AiBehaviorRegistry |
| `assets/maps/*.ron` | `content/stages/*.ron` | 1 | LevelRegistry |
| `assets/traits/*.ron` | `content/classes/*.ron` | 5 | TraitRegistry |
| `assets/definitions/*.ron` | `content/definitions/*.ron` | 2 | AttributeDefRegistry, TagDefRegistry | → 参见 ADR-004 for authoritative content paths |
| `assets/rules/*.ron` | `content/modifiers/*.ron` | 1 | ModifierRuleRegistry | → 参见 ADR-004 for authoritative content paths |
| `assets/settings.ron` | 保留 `assets/settings.ron` | 1 | GameSettings | → 参见 ADR-004 for authoritative content paths |

**总计**: 37 个 RON 文件需要迁移

**迁移步骤（每个类别）**:
1. 复制 RON 文件到新位置
2. 创建对应的 Content 加载模块（`src/content/xxx/xxx_content.rs`）
3. 更新 `AssetServer` 加载路径
4. 验证游戏正常运行
5. 删除旧位置的配置文件

**风险缓解**:
- 每迁移一类配置就验证一次
- 保留旧路径的兼容层（`pub use` 重新导出）直到所有引用更新
- 使用编译器警告发现遗漏的引用

#### 5.4 更新 AssetServer 路径（Day 21-22）

**影响范围**: 所有引用 RON 文件路径的代码

**步骤**:
1. 全局搜索 `assets/units/` → 替换为 `content/characters/`
2. 全局搜索 `assets/skills/` → 替换为 `content/skills/`
3. 全局搜索 `assets/buffs/` → 替换为 `content/buffs/`
4. 全局搜索 `assets/terrains/` → 替换为 `content/terrains/`
5. 全局搜索 `assets/ai/` → 替换为 `content/ai_behaviors/`
6. 全局搜索 `assets/maps/` → 替换为 `content/stages/`
7. 全局搜索 `assets/traits/` → 替换为 `content/classes/`
8. 全局搜索 `assets/rules/` → 替换为 `content/modifiers/`
9. 验证游戏正常运行

#### 5.5 实现 RegistryLoader（Day 22-23）

为每个 Content 模块实现加载逻辑：

```rust
// content/skills/skill_content.rs
pub fn load_skills(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<SkillRegistry>,
) {
    let skill_paths = discover_ron_files("content/skills/");
    for path in skill_paths {
        let def: SkillDef = asset_server.load(&path);
        let data: SkillData = def.into();
        registry.insert(data.id.clone(), data);
    }
}
```

**验证清单**:
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `content/` 目录包含所有 RON 配置
- [ ] `assets/` 不包含 RON 配置数据（只有二进制资源）
- [ ] 所有 Registry 加载路径已更新
- [ ] 游戏正常运行

---

### Phase 6: App 层（Day 24-25, 2 天）

**目标**: 建立游戏装配层。

#### 6.1 创建 App 层（Day 24）

**步骤**:
1. 创建 `src/app/` 目录结构
2. 创建 `app/game_state.rs`（从 `turn/state.rs` 迁入 AppState, TurnPhase）
3. 创建 `app/schedules.rs`（Schedule 定义）
4. 创建 `app/sets.rs`（SystemSet 定义）
5. 创建 `app/startup.rs`（启动逻辑）
6. 创建 `app/shutdown.rs`（关闭逻辑）
7. 创建 `app/plugins.rs`（22 个 Plugin 汇集注册）

#### 6.2 创建 AppPlugin（Day 24）

**迁移内容**: `main.rs` 的 Plugin 注册逻辑 → `app/app_plugin.rs`

**步骤**:
1. 创建 `app/app_plugin.rs`
2. 将 `main.rs` 中的 Plugin 注册逻辑迁移到 `app_plugin.rs`
3. 更新 `main.rs` 只保留入口点：
   ```rust
   fn main() {
       App::new()
           .add_plugins(AppPlugin)
           .run();
   }
   ```
4. 验证编译通过

#### 6.3 清理 main.rs（Day 25）

**步骤**:
1. 移除 `main.rs` 中所有非入口点代码
2. 更新 `lib.rs` 只导出必要的模块
3. 验证编译通过

**技术债务修复**:
- ✅ **修复 #10**: main.rs 只保留入口点

**验证清单**:
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `src/app/` 包含游戏装配逻辑
- [ ] `main.rs` 只有入口点代码
- [ ] Plugin 注册顺序正确（Shared → Infra → Core → Content → UI → Debug）

---

### Phase 7: Modding 层（Day 26-40, 15 天）— 未来

**目标**: 建立 MOD 支持框架。

**步骤**:
1. 创建 `src/modding/` 目录结构
2. 实现 `ModApi` trait 定义
3. 实现 `ModContext` 上下文
4. 实现 `ModPlugin` 注册
5. 实现 MOD 加载器
6. 实现 MOD 校验器
7. 实现 MOD 沙箱环境

---

### Phase 8: Tools 层（Day 41-55, 15 天）— 未来

**目标**: 建立开发工具链。

**步骤**:
1. 创建 `tools/` 目录结构
2. 实现内容编辑器
3. 实现数据校验器
4. 实现数值检查器
5. 实现回放检查器

---

## 技术债务修复追踪

### 10 个已知违规的修复计划

| # | 违规 | 修复 Phase | 修复方式 | 影响文件 | 验证方式 |
|---|------|-----------|---------|---------|---------|
| 1 | Unit God Component（10 个 require） | Phase 2.6 | 拆分 Unit 组件，移除不必要的 require | character/components.rs | 编译通过 + 测试 |
| 2 | 跨模块循环依赖（character → ui） | Phase 2.6 | 将 MovementIntent 移到 core/movement/events.rs | character/mod.rs, ui/events.rs | cargo check 无循环 |
| 3 | Double-Event 模式 | Phase 4.2 | 删除 infrastructure/logging/events.rs 镜像 | infrastructure/logging/ | 日志正常输出 |
| 4 | core/ 超载 | Phase 2 | 将业务模块迁移到 core/xxx/ | core/ 全目录 | 依赖图检查 |
| 5 | ID 位置错误 | Phase 1.1 | 迁移到 shared/ids/ | core/id/ → shared/ids/ | 编译通过 |
| 6 | 错误位置错误 | Phase 2 + Phase 3 | 迁移到各领域 domain/ + shared/error/ | core/error/ | 错误码验证 |
| 7 | String::new() 占位符 | Phase 2+ | 实现 UnitName 查询，替换占位符 | 18 个文件 | 编译通过 + 测试 |
| 8 | Entity::PLACEHOLDER | Phase 2+ | 实现正确的 Entity 查询逻辑 | 4 个文件 | 编译通过 + 测试 |
| 9 | 13 个资源手动重置 | Phase 4.4 | 创建 ResettableResource trait | turn/mod.rs | 统一重置逻辑 |
| 10 | 缺少 App 层 | Phase 6 | 创建 app/ 目录，迁移 Plugin 注册 | main.rs | Plugin 注册顺序验证 |

---

## Plugin 注册顺序（最终目标）

迁移完成后，22 个 Plugin 的注册顺序：

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 1: App                                                    │
│  └── AppPlugin（注册所有子 Plugin）                               │
├─────────────────────────────────────────────────────────────────┤
│  Layer 3: Shared                                                 │
│  1. SharedPlugin                                                 │
├─────────────────────────────────────────────────────────────────┤
│  Layer 4: Infrastructure                                         │
│  2. LogPlugin                                                    │
│  3. AuditPlugin                                                  │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2: Core（基础）                                           │
│  4. EffectPlugin                                                 │
│  5. ModifierRulePlugin                                           │
│  6. AttributeDefPlugin                                           │
│  7. TagDefPlugin                                                 │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2: Core（数据注册表）                                     │
│  8. SkillPlugin                                                  │
│  9. BuffPlugin                                                   │
│  10. AiBehaviorPlugin                                            │
│  11. EquipmentPlugin                                             │
│  12. InventoryPlugin                                             │
├─────────────────────────────────────────────────────────────────┤
│  Layer 5: Content                                                │
│  13. ContentPlugin（统一加载入口）                                │
├─────────────────────────────────────────────────────────────────┤
│  Layer 2: Core（逻辑）                                           │
│  14. TurnPlugin                                                  │
│  15. MapPlugin                                                   │
│  16. CharacterPlugin                                             │
│  17. BattlePlugin                                                │
│  18. AiPlugin                                                    │
│  19. CampaignPlugin                                              │
├─────────────────────────────────────────────────────────────────┤
│  Layer 4: Infrastructure（资源加载）                              │
│  20. AssetsPlugin                                                │
├─────────────────────────────────────────────────────────────────┤
│  表现层                                                          │
│  21. UiPlugin + InputPlugin                                      │
├─────────────────────────────────────────────────────────────────┤
│  调试层                                                          │
│  22. DebugPlugin（#[cfg(feature = "dev")]）                      │
└─────────────────────────────────────────────────────────────────┘
```

**注册顺序规则**:
- 🟥 表现层插件禁止在数据层之前注册
- 🟥 逻辑层插件禁止在核心层之前注册
- 🟥 禁止跳过任何注册顺序

---

## 风险缓解

### 逐 Phase 风险评估

| Phase | 风险 | 概率 | 影响 | 缓解措施 |
|-------|------|------|------|---------|
| Phase 0 | 无风险 | - | - | - |
| Phase 1 | ID 迁移遗漏引用 | 中 | 高 | 全局搜索替换 + 编译器警告 |
| Phase 2 | 模块循环依赖 | 高 | 高 | 逐模块迁移，每迁一个就 `cargo check` |
| Phase 2 | 错误迁移后类型不兼容 | 中 | 中 | 保持错误枚举结构不变，只改位置 |
| Phase 3 | 错误码冲突 | 低 | 低 | 使用唯一前缀 + 序号 |
| Phase 4 | 日志系统中断 | 中 | 中 | 保留旧日志系统直到新系统验证通过 |
| Phase 5 | 配置路径变更导致加载失败 | 高 | 高 | 兼容层 + 逐步迁移 + 每类验证 |
| Phase 5 | RON 文件内容不兼容 | 中 | 高 | 迁移前备份，迁移后完整测试 |
| Phase 6 | Plugin 注册顺序错误 | 中 | 高 | 严格按依赖图注册，使用元组批量注册 |
| Phase 7-8 | 未来阶段 | 低 | 低 | 独立于 Phase 0-6 |

### 关键路径

```
Phase 0 → Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6
                                                        ↓
                                                   Phase 7 (独立)
                                                        ↓
                                                   Phase 8 (独立)
```

### 回滚策略

每个 Phase 完成后：
1. 创建 Git tag：`migration-phase-N-complete`
2. 如果下一 Phase 失败，可回滚到上一个 tag
3. 每个 Phase 的迁移范围最小化，回滚影响可控

---

## 验证清单（总览）

### Phase 0 验证
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] 所有目标目录已创建
- [ ] 现有代码未修改

### Phase 1 验证
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `src/shared/ids/` 包含 4 个 ID 类型
- [ ] `src/shared/error/` 包含 GameResult + 工具
- [ ] 所有 `use crate::core::id::` 已替换

### Phase 2 验证
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `src/` 下无顶层业务模块
- [ ] `src/core/error/` 不存在
- [ ] `src/core/id/` 不存在
- [ ] Core 层不依赖 Infrastructure 层
- [ ] 无跨模块循环依赖

### Phase 3 验证
- [ ] 每个领域有独立的错误枚举
- [ ] 所有错误变体携带完整上下文
- [ ] 无全局 AppError 大枚举

### Phase 4 验证
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] 无 Double-Event 模式
- [ ] 13 个 Resource 统一重置

### Phase 5 验证
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `content/` 包含所有 RON 配置
- [ ] `assets/` 不包含 RON 配置数据
- [ ] 游戏正常运行

### Phase 6 验证
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] `src/app/` 包含游戏装配逻辑
- [ ] `main.rs` 只有入口点代码
- [ ] Plugin 注册顺序正确

### 最终验证
- [ ] `src/` 下无顶层业务模块（battle/、skill/ 等都在 core/ 下）
- [ ] `src/core/id/` 不存在（已迁移到 `src/shared/ids/`）
- [ ] `src/core/error/` 不存在（错误已迁移到各领域 `domain/` + `src/shared/error/`）
- [ ] `src/app/` 存在且包含游戏装配逻辑
- [ ] `src/content/` 存在且包含数据加载逻辑
- [ ] `content/` 项目根目录存在且包含 RON 配置
- [ ] `assets/` 不包含 RON 配置数据（只有二进制资源）
- [ ] `src/shared/` 不包含任何业务逻辑
- [ ] `src/core/` 不依赖 `src/infrastructure/`
- [ ] `src/shared/` 不依赖任何其他层
- [ ] 所有测试通过
- [ ] 编译无警告
- [ ] 架构 Review 完成

---

## 后果

### 正面后果

1. **架构清晰**: 七层架构明确各层职责，新人可快速理解项目结构
2. **依赖可控**: 严格的层间依赖规则防止架构腐化
3. **可测试性**: Core 层不依赖 Infrastructure，可独立测试
4. **可扩展性**: Content 层分离数据与规则，新增内容不改代码
5. **并行开发**: 4 个 feature-developer agents 可独立开发不同模块
6. **MOD 支持**: Modding 层为未来 MOD 系统奠定基础

### 负面后果

1. **迁移成本**: 30 个工作日的迁移工期
2. **临时双重结构**: Phase 1-3 期间两种结构并存，增加认知负担
3. **学习曲线**: 团队需要学习七层架构的约束
4. **过度设计风险**: 部分空壳模块（Modding, Tools）可能永远不会实现

### 替代方案

| 方案 | 优点 | 缺点 | 决策 |
|------|------|------|------|
| **A: 不迁移** | 零成本 | 架构违规累积，长期维护成本高 | ❌ 拒绝 |
| **B: 渐进式重构** | 风险低，可随时停止 | 架构不一致期更长 | ❌ 拒绝（已决定七层架构） |
| **C: 一次性重写** | 架构一致性好 | 风险极高，可能丢失功能 | ❌ 拒绝 |
| **D: 分阶段迁移（本 ADR）** | 风险可控，每阶段可验证 | 工期较长 | ✅ 接受 |

---

## 附录

### A. 迁移映射表（完整）

| 当前位置 | 目标位置 | 说明 |
|----------|---------|------|
| `src/core/id/` | `src/shared/ids/` | 强类型 ID |
| `src/core/error/game_result.rs` | `src/shared/error/result.rs` | GameResult |
| `src/core/error/battle_error.rs` | `src/core/battle/domain/battle_error.rs` | 领域错误 |
| `src/core/error/skill_error.rs` | `src/core/skill/domain/skill_error.rs` | 领域错误 |
| `src/core/error/buff_error.rs` | `src/core/buff/domain/buff_error.rs` | 领域错误 |
| `src/core/error/inventory_error.rs` | `src/core/inventory/domain/inventory_error.rs` | 领域错误 |
| `src/battle/` | `src/core/battle/` | 战斗规则 |
| `src/skill/` | `src/core/skill/` | 技能规则 |
| `src/buff/` | `src/core/buff/` | Buff 规则 |
| `src/character/` | `src/core/character/` | 角色规则 |
| `src/equipment/` | `src/core/equipment/` | 装备规则 |
| `src/inventory/` | `src/core/inventory/` | 背包规则 |
| `src/ai/` | `src/core/ai/` | AI 规则 |
| `src/map/` | `src/core/map/` | 地图规则 |
| `src/turn/` | `src/core/turn/` | 回合规则 |
| `src/campaign/` | `src/core/campaign/` | 战役规则 |
| `src/assets.rs` | `src/infrastructure/assets/` | 资源加载 |
| `src/input.rs` | `src/ui/input.rs` | 输入处理 |
| `src/infrastructure/audit/whitelist.rs` | `src/shared/events/event_whitelist.rs` | 事件白名单 |
| `main.rs` Plugin 注册 | `src/app/app_plugin.rs` | 游戏装配 |

### B. RON 配置迁移表（完整）

| 源路径 | 目标路径 | 文件数 | Registry |
|--------|---------|--------|----------|
| `assets/units/player_warrior.ron` | `content/characters/player_warrior.ron` | 1 | UnitTemplateRegistry |
| `assets/units/player_mage.ron` | `content/characters/player_mage.ron` | 1 | UnitTemplateRegistry |
| `assets/units/player_archer.ron` | `content/characters/player_archer.ron` | 1 | UnitTemplateRegistry |
| `assets/units/enemy_goblin.ron` | `content/characters/enemy_goblin.ron` | 1 | UnitTemplateRegistry |
| `assets/units/enemy_dark_knight.ron` | `content/characters/enemy_dark_knight.ron` | 1 | UnitTemplateRegistry |
| `assets/units/enemy_goblin_leader.ron` | `content/characters/enemy_goblin_leader.ron` | 1 | UnitTemplateRegistry |
| `assets/skills/basic_attack.ron` | `content/skills/basic_attack.ron` | 1 | SkillRegistry |
| `assets/skills/fireball.ron` | `content/skills/fireball.ron` | 1 | SkillRegistry |
| `assets/skills/charge.ron` | `content/skills/charge.ron` | 1 | SkillRegistry |
| `assets/skills/cleanse_skill.ron` | `content/skills/cleanse_skill.ron` | 1 | SkillRegistry |
| `assets/skills/heal.ron` | `content/skills/heal.ron` | 1 | SkillRegistry |
| `assets/skills/pierce.ron` | `content/skills/pierce.ron` | 1 | SkillRegistry |
| `assets/buffs/attack_up.ron` | `content/buffs/attack_up.ron` | 1 | BuffRegistry |
| `assets/buffs/attack_down.ron` | `content/buffs/attack_down.ron` | 1 | BuffRegistry |
| `assets/buffs/defense_up.ron` | `content/buffs/defense_up.ron` | 1 | BuffRegistry |
| `assets/buffs/defense_down.ron` | `content/buffs/defense_down.ron` | 1 | BuffRegistry |
| `assets/buffs/burn.ron` | `content/buffs/burn.ron` | 1 | BuffRegistry |
| `assets/buffs/poison.ron` | `content/buffs/poison.ron` | 1 | BuffRegistry |
| `assets/buffs/regen.ron` | `content/buffs/regen.ron` | 1 | BuffRegistry |
| `assets/buffs/stun.ron` | `content/buffs/stun.ron` | 1 | BuffRegistry |
| `assets/terrains/plain.ron` | `content/terrains/plain.ron` | 1 | TerrainRegistry |
| `assets/terrains/mountain.ron` | `content/terrains/mountain.ron` | 1 | TerrainRegistry |
| `assets/terrains/water.ron` | `content/terrains/water.ron` | 1 | TerrainRegistry |
| `assets/terrains/forest.ron` | `content/terrains/forest.ron` | 1 | TerrainRegistry |
| `assets/traits/warrior_mastery.ron` | `content/classes/warrior_mastery.ron` | 1 | TraitRegistry |
| `assets/traits/mage_mastery.ron` | `content/classes/mage_mastery.ron` | 1 | TraitRegistry |
| `assets/traits/archer_mastery.ron` | `content/classes/archer_mastery.ron` | 1 | TraitRegistry |
| `assets/traits/fire_affinity.ron` | `content/classes/fire_affinity.ron` | 1 | TraitRegistry |
| `assets/traits/heavy_armor.ron` | `content/classes/heavy_armor.ron` | 1 | TraitRegistry |
| `assets/ai/default.ron` | `content/ai_behaviors/default.ron` | 1 | AiBehaviorRegistry |
| `assets/ai/cautious.ron` | `content/ai_behaviors/cautious.ron` | 1 | AiBehaviorRegistry |
| `assets/ai/aggressive.ron` | `content/ai_behaviors/aggressive.ron` | 1 | AiBehaviorRegistry |
| `assets/maps/tutorial.ron` | `content/stages/tutorial.ron` | 1 | LevelRegistry |
| `assets/campaigns/campaign_001.ron` | `content/campaigns/campaign_001.ron` | 1 | CampaignRegistry | → 参见 ADR-004 for authoritative content paths |
| `assets/rules/element_interactions.ron` | `content/formulas/element_interactions.ron` | 1 | ModifierRuleRegistry |
| `assets/settings.ron` | 保留 `assets/settings.ron` | 1 | GameSettings | → 参见 ADR-004 for authoritative content paths |
| **总计** | | **37** | |

### C. 文件大小参考

| Phase | 预估新增文件数 | 预估修改文件数 | 预估删除文件数 |
|-------|--------------|--------------|--------------|
| Phase 0 | 15 | 1 | 0 |
| Phase 1 | 12 | 30+ | 5 |
| Phase 2 | 20 | 80+ | 15 |
| Phase 3 | 7 | 10 | 0 |
| Phase 4 | 10 | 15 | 1 |
| Phase 5 | 20 | 25 | 0 |
| Phase 6 | 7 | 5 | 0 |
| Phase 7 | 15 | 0 | 0 |
| Phase 8 | 10 | 0 | 0 |

---

**文档版本**: 1.0  
**生成日期**: 2026-06-14  
**项目**: Bevy SRPG  
**架构师**: Chief Architect Agent
