# ADR: 战役管线架构（Campaign Pipeline）

## 状态
Proposed

---

## 背景

项目完成了完整的战斗系统（Effect Pipeline、Turn 状态机、胜负条件检查），但缺少"战役/内容管线"层——玩家启动游戏后直接进入战斗，无法选择关卡，没有战役流程的概念。

@domain-designer 已产出 `docs/domain/campaign_rules_v1.md`，定义了 Campaign、Stage、Level 三层术语体系。本 ADR 基于该领域模型，设计对应的 ECS 架构。

### 关键问题

1. 战役配置如何加载？谁负责加载？
2. 关卡选择的数据从哪里来？
3. 关卡完成后，进度/解锁状态如何更新？
4. 新增模块 `campaign/` 与现有 `map/`、`turn/`、`ui/` 的边界如何划分？

---

## 引用的领域规则

- `docs/domain/campaign_rules_v1.md` — Campaign / Stage / Level 术语、不变量、流程定义
- `docs/domain/level_rules_v1.md` — LevelConfig / LevelRegistry 现有机制
- `docs/domain/victory_condition_rules_v1.md` — GameOverState / LevelCompleted Message
- `docs/domain/ui_rules_v2.md` — UI 层只读 ViewModel，不操作 ECS

---

## 决策

### 决策 1：新增 `campaign/` 模块

新增 `src/campaign/` 作为独立 Feature 模块。

**理由**：
- Campaign 是一个独立的业务概念（战役编排），有独立的数据、流程和生命周期
- 当前 `src/map/` 负责关卡数据加载和运行时地图，不应再承担战役编排职责
- `src/turn/` 负责回合流程和胜负检查，不负责战役层面的进度管理
- 符合 Feature First 原则（`campaign/` 表达业务含义）

### 决策 2：Campaign 使用双类型模式（Def / Resource）

| 类型 | 性质 | 文件 | 说明 |
|------|------|------|------|
| `CampaignDef` | Definition | `src/campaign/def.rs` | RON 反序列化中间类型 |
| `CampaignRegistry` | Registry (Resource) | `src/campaign/registry.rs` | 运行时只读注册表 |
| `CampaignProgress` | Resource | `src/campaign/progress.rs` | 运行时可变进度状态 |

`CampaignDef` 是轻量引用层，只包含 `stages: Vec<StageDef>`，其中 `StageDef = { id, level_id }`。不内嵌任何 Level 数据。

### 决策 3：CampaignRegistry 只读取 `assets/campaigns/` 目录

```
assets/
  campaigns/
    campaign_001.ron   ← CampaignDef（只包含 level_id 引用）
  maps/
    tutorial.ron        ← LevelConfigDef（已有，不变）
```

Campaign 加载流程：
1. `CampaignPlugin` 启动时读取 `assets/campaigns/*.ron`
2. 反序列化为 `CampaignDef`，验证所有 `level_id` 在 `LevelRegistry` 中存在
3. 构建 `CampaignRegistry`（`HashMap<campaign_id, CampaignDef>`）

### 决策 4：CampaignProgress 跟踪运行时进度

`CampaignProgress` 是运行时 Resource，不是持久化数据。

```rust
pub struct CampaignProgress {
    pub campaign_id: String,
    pub current_stage: Option<String>,
    pub stages: HashMap<String, StageStatus>, // stage_id → Locked/Unlocked/Completed
}

pub enum StageStatus {
    Locked,
    Unlocked,
    Completed,
}
```

初始状态：第一个 stage = Unlocked，其余 = Locked。

### 决策 5：关卡完成后通过 Message 更新进度

胜利/失败检查在 `turn/victory_check.rs` 完成后，发送 `LevelCompleted` Message：

```
LevelCompleted { level_id, result, turn_number }
    ↓
CampaignPlugin 监听 LevelCompleted Message
    ↓
更新 CampaignProgress：
  - Victory → 当前 Stage = Completed，下一个 Stage = Unlocked
  - Defeat  → 当前 Stage 保持 Unlocked（可重玩）
```

### 决策 6：不引入 MapAsset / EncounterDef 分离

当前 Level 保持"一切合一"模式（地形 + 单位 + 胜负条件在一个文件中）。Campaign 只通过 `level_id` 引用 LevelConfig。

此决策在 `assets/campaigns/*.ron` 中只有 1 个关卡时完全够用。Map/Encounter 分离推迟到满足以下任一条件时：
- 同一张地图需要用于 ≥2 个不同关卡
- 同一个敌人编队需要用于 ≥2 个不同关卡

---

## Module Design

### 文件组织

```
src/campaign/
├── mod.rs           // CampiagnPlugin 注册
├── def.rs           // CampaignDef, StageDef 数据结构
├── registry.rs      // CampaignRegistry 定义与加载
├── progress.rs      // CampaignProgress 运行时状态
├── loader.rs        // 从 assets/campaigns/ 加载的系统
└── progression.rs   // 监听 LevelCompleted，更新 CampaignProgress
```

### 插件注册顺序（src/main.rs）

当前插件顺序：

```
Core Layer:     EffectPlugin, ModifierRulePlugin, AttributeDefPlugin, TagDefPlugin
Data Layer:     SkillPlugin, BuffPlugin, AiBehaviorPlugin, EquipmentPlugin, InventoryPlugin
Logic Layer:    AssetsPlugin, TurnPlugin, MapPlugin, CharacterPlugin, BattlePlugin, AiPlugin
Presentation:   UiPlugin, InputPlugin, DebugPlugin
```

新增 `CampaignPlugin` 的注册位置：**Data Layer，在 AssetsPlugin 之前**

同理：
- `CampaignRegistry` 在 `LevelRegistry` 加载之后构建
- AssetsPlugin 负责加载 UnitTemplateRegistry、LevelRegistry 等基础数据
- CampaignPlugin 基于已加载的 LevelRegistry 验证 level_id 引用

```rust
// 数据层插件（CampaignPlugin 加在这里）
.add_plugins((
    SkillPlugin,
    BuffPlugin,
    AiBehaviorPlugin,
    EquipmentPlugin,
    InventoryPlugin,
    CampaignPlugin,    // ← 新增，在 LevelRegistry 就绪后注册
))
```

### 启动流程变更

当前 `src/main.rs` 第 87-89 行：

```rust
.add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
    next.set(AppState::InGame);
});
```

变更为：

```rust
.add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
    next.set(AppState::MainMenu);
});
```

InGame 不再作为默认启动状态。由 UI/玩家操作从 MainMenu 进入。

---

## Communication Design

### Message（跨功能通信）

| Message | 发送方 | 接收方 | 说明 |
|---------|--------|--------|------|
| `LevelCompleted` | `turn/victory_check` | `campaign/progression`, `ui/screens`, `battle/record` | 已有，扩展接收方 |

### Observer（同功能局部响应）

不涉及。

### Hook（组件添加/删除副作用）

不涉及。

### 函数调用（模块内）

- `CampaignRegistry::load_from_dir(dir)` — 从目录加载 CampaignDef
- `CampaignProgress::initialize(registry)` — 根据 CampaignRegistry 初始化进度
- `CampaignProgress::on_level_completed(level_id, result)` — 处理关卡完成事件

---

## 边界定义

### 允许

- `campaign/loader` 读取 `map::LevelRegistry`（验证 level_id 有效性）
- `campaign/progression` 读取 `LevelCompleted` Message
- `ui/screens` 读取 `campaign::CampaignRegistry` 和 `campaign::CampaignProgress`（展示关卡列表和进度）
- `campaign/` 写入 `CampaignProgress` Resource

### 禁止

- 🟥 禁止：Campaign 模块内嵌 Level 数据（地形/单位/胜负条件） — 应通过 level_id 引用
- 🟥 禁止：Campaign 模块参与战斗逻辑（Effect Pipeline、TurnPhase 等）
- 🟥 禁止：`campaign/` 模块直接依赖 `ui/` 或 `input/` — 表现层隔离
- 🟥 禁止：CampaignProgress 写入持久化存储 — 存档系统暂不实现
- 🟥 禁止：跳过 LevelRegistry 直接加载 LevelConfig — 必须统一通过注册表
- 🟥 禁止：在运行时修改 CampaignRegistry（它是 Definition） — 只读

---

## Forbidden（禁止事项）

- 🟥 **FORBIDDEN-1** — 禁止：Campaign 配置中内嵌 Level 数据 — 理由：必须通过 level_id 引用 LevelConfig（Definition 分离）
- 🟥 **FORBIDDEN-2** — 禁止：CampaignProgress 持久化到文件 — 理由：存档系统暂不实现（只解决当前复杂度）
- 🟥 **FORBIDDEN-3** — 禁止：跳过 Campaign 流程直接进入 InGame — 理由：必须通过 MainMenu → LevelSelect → InGame 流程
- 🟥 **FORBIDDEN-4** — 禁止：在 `campaign/` 中编写 UI 代码 — 理由：Logic/Presentation 分离
- 🟥 **FORBIDDEN-5** — 禁止：`campaign/` 参与战斗逻辑 — 理由：边界清晰
- 🟥 **FORBIDDEN-6** — 禁止：新增 Campaign 数据类型时修改 `map::LevelConfig` — 理由：Campaign 只引用，不修改
- 🟥 **FORBIDDEN-7** — 禁止：在没有验证 level_id 有效性的情况下将 Campaign 标记为可用 — 理由：无效引用导致关卡加载失败

---

## Definition / Instance Design

### Definition（不可变配置）

| 类型 | 存储位置 | 加载来源 | 不可变性保证 |
|------|----------|----------|-------------|
| `CampaignDef` | `CampaignRegistry` | `assets/campaigns/*.ron` | 加载后不可修改 |
| `StageDef` | `CampaignDef.stages` | 同上 | 同上 |
| `LevelConfig` | `LevelRegistry` | `assets/maps/*.ron` | 已有，不变 |

### Instance（运行时状态）

| 类型 | 存储方式 | 写入方 | 读取方 |
|------|----------|--------|--------|
| `CampaignProgress` | Resource（CampaignPlugin 注册） | `campaign/progression`（响应 LevelCompleted） | `ui/screens/level_select` |
| `StageStatus` | `CampaignProgress.stages` 值 | 同上 | 同上 |

---

## 后果

### 正面

- **零成本解耦**：新增一个关卡 = 在 `assets/campaigns/` 中新增一个 Stage 条目 + 在 `assets/maps/` 中新增 Level 配置。已有代码无需修改。
- **独立可测**：Campaign 模块不依赖战斗系统，可独立测试加载/解锁/进度逻辑。
- **最小侵入**：现有 `map/`、`turn/`、`ui/` 模块不需要为 Campaign 做任何修改。新增模块只追加，不重构。
- **预留扩展**：MapAsset/EncounterDef 分离可在需要时引入，不影响 Campaign 的 level_id 引用模式。

### 负面

- **模块数量增加**：新增 `campaign/` 模块 + 5 个文件，增加项目总模块数到 13 个。
- **启动流程变更**：需要将 Startup 从 InGame 改为 MainMenu，现有测试可能依赖旧启动行为。
- **运行时状态管理**：CampaignProgress 只在内存中，退出后丢失。未来需要存档系统来持久化进度。

---

## 替代方案

### 替代方案 A：扩展 MapPlugin 而非新建模块

将 Campaign 逻辑放在 `src/map/campaign.rs`。

**放弃理由**：
- Map 的职责是地图数据和寻路，战役编排是独立概念
- 违反"一个领域只负责一件事"原则
- 已有 `level_rules_v1.md` 和 `campaign_rules_v1.md` 分别定义了 Level 和 Campaign，代码模块应反映这种分离

### 替代方案 B：用 YAML/TOML 替换 RON

放弃理由：项目已有完整 RON 管线（RegistryLoader trait、双类型模式、serde 配置），引入新格式增加维护成本。Campaign 配置极简单（`Vec<StageDef>`），RON 完全够用。

---

## 架构合规性自检

- [x] 符合 ECS 约束 — CampaignRegistry/Progress 是 Resource，无 Componene 或 Entity 滥用
- [x] 符合 Plugin 注册顺序 — CampaignPlugin 注册在 Data Layer
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs） — 按业务命名（def/registry/progress/loader/progression）
- [x] 没有绕过 Effect/Modifier Pipeline — Campaign 不参与战斗
- [x] 符合"定义与实例分离" — CampaignRegistry 是 Definition，CampaignProgress 是 Instance
- [x] 符合"规则与内容分离" — 新 Campaign = 新 RON 文件
- [x] 符合"逻辑与表现分离" — Campaign 模块不包含 UI 代码
- [x] 所有禁止事项已明确列出（FORBIDDEN-1 到 FORBIDDEN-7）
- [x] 已检查 `docs/domain/` 相关文档（campaign_rules_v1、level_rules_v1）

---

## 与现有 Message 表的关系

新增监听方（Message 本身已有，扩展接收者）：

| Message | 新增接收方 | 职责 |
|---------|-----------|------|
| `LevelCompleted` | `campaign/progression` | 更新 CampaignProgress |
