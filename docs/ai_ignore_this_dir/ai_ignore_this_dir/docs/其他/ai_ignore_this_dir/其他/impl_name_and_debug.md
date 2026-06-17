# 实施方案：Name 规范化 + 业务级 Debug UI

> 依据：`docs/15.md`（Name 使用规范）+ `docs/16.md`（Inspector ≠ Debug UI）
> 原则：Name 用于调试不用于业务；Inspector 看 ECS，Debug Panel 看游戏逻辑

---

## 第一部分：现状评估

### Name 组件现状

| 项目 | 现状 | 15.md 建议 | 差距 |
|------|------|-----------|------|
| Bevy `Name` 组件 | **完全未使用** | 重要实体必加 | 全部缺失 |
| `UnitName` 组件 | 已有，仅用于显示/日志 | 显示名称与业务ID分离 | 符合 |
| `UnitId` 业务ID | **不存在** | 推荐添加 | 缺失 |
| Name 用于业务查询 | 无（合规） | 铁律：禁止 | 合规 |
| Inspector 可读性 | 所有实体显示为 `Entity(N)` | 重要实体应有名称 | 极差 |

### Debug UI 现状

| 层级 | 16.md 定义 | 项目现状 | 差距 |
|------|-----------|---------|------|
| Level 1: Inspector | WorldInspectorPlugin | ✅ 已有（F12） | 无 |
| Level 2: 业务 Debug Panel | Buff/AI/Grid/Equipment Viewer | ✅ 已有 4 个 | 缺 3 个核心面板 |
| Level 3: GM Tools | 运行时修改游戏状态 | ❌ 不存在 | 全部缺失 |

**现有面板详情**：

| 面板 | 快捷键 | 文件 | 功能 |
|------|--------|------|------|
| Buff Viewer | 无 | `debug/buff_viewer.rs` | Buff 状态查看 |
| AI Viewer | 无 | `debug/ai_viewer.rs` | AI 决策状态 |
| Grid Viewer | 无 | `debug/grid_viewer.rs` | 地形网格与占用 |
| Equipment Viewer | 无 | `debug/equipment_viewer.rs` | 装备与背包 |
| Gizmos Overlay | F3 | `debug/overlay.rs` | 4 种 Gizmos 开关 |
| Stepping Control | F6/F7 | `debug/stepping_control.rs` | System 单步调试 |
| Settings Viewer | 无 | `debug/settings_viewer.rs` | GameSettings 修改 |
| World Inspector | F12 | `debug/inspector.rs` | ECS 全局检查 |

**缺失的核心面板**：

| 面板 | 16.md 优先级 | 说明 |
|------|-------------|------|
| Damage Breakdown Viewer | ★★★★★ | 伤害来源分解（基于实际效果管线） |
| Turn Queue Viewer | ★★★★☆ | 完整回合队列预览 |
| Attribute Modifier Viewer | ★★★★★ | 属性修饰符来源分解（Trait+装备+Buff） |

**快捷键分配**：

| 快捷键 | 当前 | 方案 |
|--------|------|------|
| F1 | 未用 | Battle Debugger（回合状态+当前阶段+行动单位） |
| F2 | 未用 | Buff Viewer（已有，绑定快捷键切换显隐） |
| F3 | Gizmos Overlay | 保持 |
| F4 | 未用 | Damage & Attribute Debugger（Tab 切换两个子面板） |
| F5 | 未用 | Turn Queue Viewer（行动队列预览） |
| F6 | Debug Stepping | 保持 |
| F7 | Stepping 单步 | 保持 |
| F12 | Inspector | 保持 |

> **F1 与 F5 区分**：F1 显示当前战斗状态快照（回合号、阶段、当前行动单位、已发生事件数），F5 显示完整行动队列预览（所有单位按 Initiative 排序 + 当前指针位置）。

---

## 第二部分：Name 规范化方案

### 铁律

1. **Name 用于调试，不用于业务逻辑**
2. **重要实体必须命名，海量实体不要命名**
3. **业务ID（UnitId）与 Name 分离**
4. **显示名称（UnitName）与 Name 分离**：Name 用英文标识（Inspector 可读），UnitName 用本地化显示名
5. **不要通过 Name 查找实体**

### Name 添加规则

| 实体类型 | Name（英文标识） | UnitId | UnitName（显示名） | 优先级 |
|----------|-----------------|--------|-------------------|--------|
| 战斗单位 | ✅ `"knight"` | ✅ `"knight_001"` | ✅ `"艾琳"` | P0 |
| UI 根节点 | ✅ `"UnitInfoPanel"` | ❌ | ❌ | P0 |
| 系统根实体 | ✅ `"BattleCamera"` | ❌ | ❌ | P1 |
| Debug 实体 | ✅ `"GridDebug"` | ❌ | ❌ | P1 |
| 地形格子 | ❌ | ❌ | ❌ | — |
| 范围标记 | ❌ | ❌ | ❌ | — |
| HP 条 | ❌ | ❌ | ❌ | — |
| 飘字/粒子 | ❌ | ❌ | ❌ | — |

> **注意**：战斗单位的 `Name` 使用 `template.id`（英文标识符如 `"knight"`），而非 `template.name`（中文显示名如 `"骑士"`）。`UnitName` 继续使用 `template.name`。三者职责完全分离：Name→Inspector、UnitId→业务逻辑、UnitName→UI显示。

### 任务清单

#### N1：添加 UnitId 业务ID 组件

```rust
// src/character/components.rs
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct UnitId(pub String);
```

- 在 `spawn_unit_from_template` 中注入 `UnitId(template.id.clone())`
- 日志中 **并存** `entity` 和 `unit_id`：`entity=?entity, unit_id=%unit_id.0`（保留 ECS 层定位能力）

#### N2：为战斗单位添加 Name

```rust
// character/spawn.rs — spawn_unit_from_template
commands.spawn((
    Unit,
    Name::new(&template.id),       // Inspector 显示 "knight" 而非 Entity(3)
    UnitId(template.id.clone()),   // 业务逻辑标识 "knight_001"
    UnitName(template.name.clone()), // UI 显示 "艾琳"
    // ...
));
```

> **关键**：`Name::new(&template.id)` 而非 `Name::new(&template.name)`。Name 用英文 ID，UnitName 用中文显示名。

#### N3：为 UI 根节点添加 Name

| UI 面板 | Name | 实际文件 |
|---------|------|---------|
| 角色信息面板 | `"UnitInfoPanel"` | `ui/panels/unit_info.rs` |
| 行动菜单 | `"ActionMenu"` | `ui/action_menu.rs` |
| 战斗日志面板 | `"CombatLogPanel"` | `battle/log.rs`（标记组件在此定义） |
| 背包面板 | `"InventoryPanel"` | `ui/panels/inventory_panel.rs` |
| 行动提示 | `"ActionHint"` | `ui/panels/action_hint.rs` |
| 回合指示器 | `"TurnIndicator"` | `ui/panels/turn_indicator.rs` |
| 战斗预览 | `"CombatPreview"` | `ui/combat_preview.rs`（不在 panels/ 下） |

> **注意**：`CombatLogPanel` 的标记组件定义在 `battle/log.rs`，`CombatPreview` 的实体在 `ui/combat_preview.rs`（不在 `ui/panels/` 下）。添加 Name 时需定位到正确的 spawn 位置。

#### N4：为系统/Debug 实体添加 Name

| 实体 | Name | 文件 |
|------|------|------|
| 摄像机 | `"BattleCamera"` | `ui/camera.rs` |

#### N5：确保 Name 不参与业务逻辑

- 在代码审查中检查：任何 `Query<&Name>` 或对 `Name` 的查询都是违规
- `UnitId` 用于业务身份识别
- `UnitName` 用于 UI 显示
- `Name` 仅用于 Inspector 可读性

---

## 第三部分：业务级 Debug UI 方案

### 架构原则（16.md）

```
Inspector（F12）→ ECS 层调试
Debug Panels（F1-F5）→ 游戏逻辑调试
GM Tools（未来）→ 运行时修改游戏状态
```

**不改 Inspector，不把业务调试塞进 Inspector。**

### 数据源说明

项目中有两套战斗日志系统：

| 系统 | 位置 | 用途 | Debug Panel 应使用 |
|------|------|------|-------------------|
| `CombatLog` | `battle/log.rs` | 文本日志，UI 显示 | ❌ 不适合 |
| `BattleRecord` | `battle/record.rs` | 结构化记录，回放/调试 | ✅ 数据源 |

所有 Debug Panel 统一读取 `BattleRecord`，不读取 `CombatLog`。

### 任务清单

#### D1：Damage Breakdown Viewer（★★★★★ 最高优先级）

**痛点**：Inspector 只能看到 HP 从 200 变成 48，看不到为什么。

**实际管线分析**：

当前伤害管线并非"基础攻击→武器加成→Buff加成→暴击"的线性流程，而是：

```
CombatIntent
  → generate（生成 Effect 列表）
  → modify（ModifierRule 修改效果值）
  → execute（应用最终效果）
```

伤害值在 `execute` 阶段已是最终值，中间无分解步骤。因此 `DamageBreakdown` 需要重新设计，匹配实际管线：

```rust
/// 伤害分解记录（匹配实际效果管线）
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct DamageBreakdown {
    /// 原始效果值（generate 阶段产出）
    pub base_amount: i32,
    /// 修饰符列表（modify 阶段应用的每条规则）
    pub modifiers: Vec<ModifierEntry>,
    /// 最终效果值（modify 后）
    pub modified_amount: i32,
    /// 实际扣血（execute 后，含防御减免等）
    pub actual_damage: i32,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct ModifierEntry {
    /// 修饰符来源（Trait/Equipment/Buff）
    pub source: ModifierSource,
    /// 修饰符操作类型
    pub op: ModifierOp,
    /// 修饰符值
    pub value: f32,
}
```

**实现策略**：在 `battle/pipeline/execute.rs` 的 `apply_damage_effect` 中，记录 `generate` 产出值 → `modify` 后值 → `execute` 后实际值的完整链路。

**面板设计**：

```rust
// src/debug/viewers/damage_viewer.rs
/// 伤害来源分解面板
pub fn damage_breakdown_panel(
    mut contexts: EguiContexts,
    battle_record: Res<BattleRecord>,
    units: Query<&UnitName>,
) {
    // 从 BattleRecord 中提取最近 N 条 DamageApplied 记录
    // 每条记录展开显示：
    // ┌──────────────────────────┐
    // │ Damage #3 → Goblin_02    │
    // ├──────────────────────────┤
    // │ 原始效果值   80          │
    // │ + Trait加成  5 (战士精通) │
    // │ + 装备加成  15 (铁剑)    │
    // │ × Buff倍率  1.5 (狂暴)  │
    // │ = 修饰后伤害 150         │
    // │ 实际扣血    120（防御后） │
    // └──────────────────────────┘
}
```

**快捷键**：F4（Tab 1）

#### D2：Turn Queue Viewer（★★★★☆）

**痛点**：当前只能看到"轮到谁"，看不到完整行动队列。

**方案**：

```rust
// src/debug/viewers/turn_queue_viewer.rs
/// 回合队列查看器
pub fn turn_queue_viewer(
    mut contexts: EguiContexts,
    turn_order: Res<TurnOrder>,
    turn_state: Res<TurnState>,
    units: Query<(&UnitName, &Unit, &Attributes), Without<Dead>>,
) {
    // 空队列处理
    if turn_order.queue.is_empty() {
        // 显示 "等待回合开始"
        return;
    }
    // ┌──────────────────────────┐
    // │ Turn Queue (Round 5)     │
    // ├──────────────────────────┤
    // │ ▶ 艾琳  AGI=18  [友]     │  ← 当前
    // │   哥布林 AGI=12  [敌]     │
    // │   骑士  AGI=10  [友]     │
    // │   暗骑士 AGI=8  [敌]     │
    // └──────────────────────────┘
}
```

**空队列处理**：当 `TurnOrder.queue` 为空时显示"等待回合开始"，而非空白面板。

**快捷键**：F5

#### D3：Attribute Modifier Viewer（★★★★★）

**痛点**：Inspector 只能看到最终属性值，看不到修饰符来源。

**方案**：

```rust
// src/debug/viewers/attribute_viewer.rs
/// 属性修饰符来源分解面板
pub fn attribute_viewer(
    mut contexts: EguiContexts,
    selected: Res<SelectedUnitView>,
    units: Query<(&UnitName, &Attributes, &ActiveBuffs, &EquipmentSlots, &TraitCollection)>,
) {
    // ┌──────────────────────────┐
    // │ Attribute: 艾琳          │
    // ├──────────────────────────┤
    // │ Attack = 45              │
    // │   基础    20             │
    // │   + Trait  5 (战士精通)   │
    // │   + 装备  15 (铁剑)      │
    // │   + Buff   5 (攻击提升)  │
    // ├──────────────────────────┤
    // │ Defense = 18             │
    // │   基础    12             │
    // │   + 装备   6 (铁盾)      │
    // │   - Buff  -2 (防御下降)  │  ← 红色标注
    // └──────────────────────────┘
}
```

**数据源**：`Attributes.modifiers` 中的 `ModifierSource` 区间（精确值）：

| 来源 | 区间 | 判断方法 |
|------|------|---------|
| Trait | `u64::MAX ~ u64::MAX-999` | `ModifierSource::is_trait()` |
| Equipment | `u64::MAX-1000 ~ u64::MAX-1999` | `ModifierSource::is_equipment()` |
| Buff | `1 ~ u64::MAX-2001` | `ModifierSource::is_buff()` |

> **注意**：Buff 区间为 `1 ~ u64::MAX-2001`，远大于之前错误标注的 `1 ~ 999999`。使用 `is_trait()`/`is_equipment()`/`is_buff()` 方法判断，不要硬编码区间。

**快捷键**：F4（Tab 2，与 Damage Breakdown 合并为一个面板，Tab 切换）

#### D4：为现有面板绑定快捷键

| 面板 | 快捷键 | 当前状态 | 行为 |
|------|--------|---------|------|
| Battle Debugger | F1 | 新建 | 显示回合号、阶段、当前行动单位、已发生事件数 |
| Buff Viewer | F2 | 已有，需加快捷键 | 切换面板显隐 |
| Gizmos Overlay | F3 | 已有 | 保持 |
| Damage & Attribute Debugger | F4 | 新建 | Tab 切换两个子面板 |
| Turn Queue Viewer | F5 | 新建 | 显示完整行动队列 |

#### D5：Debug 面板目录重组

当前 `debug/` 目录结构中 viewer 和基础设施混放，按 16.md 建议重组：

```
debug/
├── mod.rs                    # DebugPlugin 统一管理
├── inspector.rs              # WorldInspector（Level 1）
├── overlay.rs                # Gizmos 可视化开关
├── gizmos_viz.rs             # Gizmos 绘制系统
├── stepping_control.rs       # System 单步调试
├── settings_viewer.rs        # GameSettings 面板
├── viewers/                  # 业务 Debug Panels（Level 2）
│   ├── mod.rs
│   ├── buff_viewer.rs        # F2 Buff 调试器
│   ├── ai_viewer.rs          # AI 决策查看器
│   ├── grid_viewer.rs        # 地形网格查看器
│   ├── equipment_viewer.rs   # 装备查看器
│   ├── damage_viewer.rs      # F4 伤害分解查看器（新建）
│   ├── attribute_viewer.rs   # F4 属性修饰符查看器（新建）
│   └── turn_queue_viewer.rs  # F5 回合队列查看器（新建）
└── gm/                       # GM Tools（Level 3，未来）
    └── mod.rs
```

> **影响范围**：移动 4 个现有 viewer 到 `viewers/` 子目录，需修改 `debug/mod.rs` 的 `mod` 声明、所有 `use` 路径、以及测试中的 import，预计影响 8+ 文件。

---

## 第四部分：实施步骤

### 阶段 A：Name 规范化（P0）

| 步骤 | 内容 | 影响文件 |
|------|------|---------|
| A1 | 添加 `UnitId` 组件 + Reflect 注册 | `character/components.rs`, `character/plugin.rs` |
| A2 | 单位生成时添加 `Name` + `UnitId` | `character/spawn.rs` |
| A3 | UI 根节点添加 `Name`（7 个面板） | `ui/panels/unit_info.rs`, `ui/action_menu.rs`, `battle/log.rs`, `ui/panels/inventory_panel.rs`, `ui/panels/action_hint.rs`, `ui/panels/turn_indicator.rs`, `ui/combat_preview.rs` |
| A4 | 系统/Debug 实体添加 `Name` | `ui/camera.rs` |
| A5 | 日志中 **并存** `entity` 和 `unit_id` | `battle/*.rs`, `ai/*.rs`, `buff/*.rs` |
| A6 | 补充 `UnitId` 单元测试 | `character/components.rs` (#[cfg(test)]) |

### 阶段 B：Damage Breakdown 数据结构（P0）

| 步骤 | 内容 | 影响文件 |
|------|------|---------|
| B1 | 新建 `DamageBreakdown` + `ModifierEntry` 结构 | `battle/record.rs` |
| B2 | 在效果管线中记录伤害分解（generate→modify→execute） | `battle/pipeline/execute.rs`, `battle/pipeline/modify.rs` |
| B3 | `BattleEntry::DamageApplied` 携带 `Option<DamageBreakdown>` | `battle/record.rs` |
| B4 | 补充 `DamageBreakdown` 单元测试 | `battle/record.rs` (#[cfg(test)]) |

### 阶段 C：新增 Debug 面板（P1）

| 步骤 | 内容 | 影响文件 |
|------|------|---------|
| C1 | 重组 debug 目录（移动 4 个 viewer 到 `viewers/`） | `debug/mod.rs` + 4 个 viewer + 测试文件（8+ 文件） |
| C2 | 新建 `damage_viewer.rs` | `debug/viewers/damage_viewer.rs` |
| C3 | 新建 `attribute_viewer.rs` | `debug/viewers/attribute_viewer.rs` |
| C4 | 新建 `turn_queue_viewer.rs` | `debug/viewers/turn_queue_viewer.rs` |
| C5 | 新建 `battle_debugger.rs`（F1 战斗状态面板） | `debug/viewers/battle_debugger.rs` |
| C6 | 绑定快捷键 F1/F2/F4/F5 | `debug/mod.rs` |

### 阶段 D：现有面板增强（P2）

| 步骤 | 内容 | 影响文件 |
|------|------|---------|
| D1 | Buff Viewer 添加触发链追踪 | `debug/viewers/buff_viewer.rs` |
| D2 | AI Viewer 添加评分详情 | `debug/viewers/ai_viewer.rs` |
| D3 | 所有 Viewer 使用 `UnitId` 替代 `entity.index()` | `debug/viewers/*.rs` |

### 阶段 E：GM Tools 基础（P3，待 16.md 补充 GM 规范后再细化）

| 步骤 | 内容 |
|------|------|
| E1 | GM 面板框架（添加Buff/修改属性/切换职业） |
| E2 | bevy_remote HTTP 传输层（等 0.19） |
| E3 | 战斗回放器（基于 BattleRecord） |

---

## 第五部分：验收标准

### Name 规范化

- [ ] Inspector 中重要实体显示可读名称（不再是 `Entity(N)`）
  - 手动验证：运行游戏 → F12 Inspector → 检查 Unit/UI 实体名称
- [ ] `UnitId` 组件存在于所有战斗单位
- [ ] 日志中 `entity` 和 `unit_id` 字段并存
- [ ] 无任何代码通过 `Name` 做业务查询
- [ ] `Name` 使用英文标识（`template.id`），`UnitName` 使用中文显示名（`template.name`）

### Debug UI

- [ ] F1 打开 Battle Debugger（回合状态+当前阶段+行动单位）
- [ ] F2 切换 Buff Viewer 显隐
- [ ] F4 打开 Damage & Attribute Debugger（Tab 切换）
- [ ] F5 打开 Turn Queue Viewer
- [ ] Damage Breakdown 显示完整伤害计算链（原始值→修饰符→最终值→实际扣血）
- [ ] Attribute Viewer 显示修饰符来源分解（使用 `is_trait()`/`is_equipment()`/`is_buff()` 判断）
- [ ] Turn Queue Viewer 显示完整行动顺序（空队列时显示"等待回合开始"）

### 测试

- [ ] `cargo test` 全部通过
- [ ] 新增 `DamageBreakdown` 相关单元测试
- [ ] 新增 `UnitId` 相关单元测试
- [ ] 目录重组后所有测试 import 正确
