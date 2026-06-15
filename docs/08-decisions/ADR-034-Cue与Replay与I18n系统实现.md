# ADR-034: Cue / Replay / I18n 系统实现

## 状态
Accepted（2026-06-15）

## 背景

这三个系统在当前项目中存在不同程度的缺失：

### Cue 系统现状（437 行）

Cue 模块已建立基本框架（`cue/types.rs` + `cue/emitter.rs` + `cue/mod.rs`），但存在以下不足：

1. **覆盖率不足** — 当前 9 种 Cue 事件不完全覆盖 Linglan 8 种 EffectType 的所有场景
2. **Cue 发射位置分散** — 部分 Cue 在 `effect/handler.rs` 中直接调用，部分在 `cue/emitter.rs` 中统一下发，无统一执行策略
3. **无 CueDefinition** — Cue 信号只有运行时类型，无静态配置（缺少 cue_id 映射）
4. **无 Cue 过滤/配置** — 无法配置特定 Effect 是否发射特定 Cue

### Replay 系统（不存在，需新建）

当前项目完全缺乏回放系统：

| 需实现 | 现状 |
|--------|------|
| 确定性事件流 | 不存在 |
| ActionType 枚举 | 不存在 |
| ReplayEvent 数据结构 | 不存在 |
| UI 回放控件 | 不存在 |
| 种子快照 | 不存在 |

### I18n 系统（不存在，需新建）

当前项目完全缺乏国际化系统：

| 需实现 | 现状 |
|--------|------|
| FTL 文件加载 | 不存在 |
| FluentBundle 运行时 | 不存在 |
| name_key/desc_key 字段定义 | 缺失 |
| LocalizedText Component | 不存在 |
| 语言回退链 | 不存在 |
| 运行时语言切换 | 不存在 |

### 引用文档

- `docs/04-data/ll/10_Pipeline_Replay_ll.md` — ReplayEvent 数据结构 + 确定性约束
- `docs/04-data/ll/11_I18n_ll.md` — Key 命名规范 + FTL 文件组织 + 回退链
- `docs/04-data/ll/data_relationship_overview.md` — Cue 在 Pipeline 中的位置
- `docs/01-architecture/events_audit_design.md` — 双轨制日志（Command vs Audit）
- `docs/01-architecture/logging_design.md` — 日志是领域事件的消费者
- `docs/08-decisions/ADR-017-国际化架构决策.md` — 国际化架构
- `docs/08-decisions/ADR-018-国际化迁移方案.md` — 国际化迁移

## 决策

### 1. Cue 系统全面强化

#### 1.1 Cue 事件全覆盖

所有 Effect 必须发射对应的 Cue 事件。当前 9 种扩展为 12 种，覆盖 Linglan 完整场景：

```rust
/// Cue 事件类型（所有 Cue 均为独立 Message struct）
pub struct CueDamage {
    pub source: Entity, pub target: Entity,
    pub damage_type: TagId, pub amount: i32,
    pub is_crit: bool, pub is_backstab: bool,
    pub breakdown: DamageBreakdown,
}

pub struct CueHeal {
    pub source: Entity, pub target: Entity,
    pub amount: i32, pub is_crit: bool,
    pub overheal_to_shield: Option<i32>,
}

pub struct CueDeath {
    pub victim: Entity, pub killer: Entity,
    pub death_type: DeathType,  // Normal / Overkill / Sacrifice
}

pub struct CueBuffApply {
    pub target: Entity, pub source: Entity,
    pub effect_id: EffectId, pub stack_count: u32,
    pub remaining_duration: u32,
}

pub struct CueBuffRemove {
    pub target: Entity, pub effect_id: EffectId,
    pub remove_reason: RemoveReason,  // Expired / Dispelled / Overwritten
}

pub struct CueShield {
    pub target: Entity,
    pub shield_type: ShieldType,
    pub amount: i32, pub remaining: i32,
    pub absorbed: i32,
}

pub struct CueSkillCast {
    pub caster: Entity, pub ability_id: AbilityId,
    pub targets: Vec<Entity>,
}

pub struct CueMovement {
    pub unit: Entity,
    pub path: Vec<GridPosition>,
}

pub struct CueStatusChange {
    pub target: Entity,
    pub added_tags: Vec<TagId>,
    pub removed_tags: Vec<TagId>,
}

pub struct CueDisplacement {
    pub target: Entity,
    pub from: GridPosition, pub to: GridPosition,
    pub wall_damage: Option<i32>,
}

pub struct CueSummon {
    pub summoner: Entity,
    pub summoned: Entity,
    pub template_id: UnitId,
}

pub struct CueEnergyChange {
    pub target: Entity,
    pub ap_change: i32, pub cp_change: i32,
    pub reason: EnergyChangeReason,
}
```

#### 1.2 CueDefinition

```ron
// content/cues/cues.ron
(
    cues: [
        (id: "cue.c_001", cue_type: BuffApply,
         env_config: Some((
            trigger_tag: Some("control_hard"),       // 硬控触发
            result_tag: Some("stunned"),             // 表现结果
            vfx_id: "vfx/stun_indicator",
            sfx_id: "sfx/stun_impact",
         ))),
        (id: "cue.c_010", cue_type: Damage,
         vfx_id: "vfx/hit_spark",
         sfx_id: "sfx/hit_impact"),
    ],
)
```

#### 1.3 CueEmitter 强化

```rust
pub struct CueEmitter {
    /// 帧内累积的 Cue 事件（帧末批量下发）
    pending_cues: Vec<Box<dyn CueEvent>>,
}

impl CueEmitter {
    pub fn emit<T: CueEvent + Event>(&mut self, cue: T) {
        self.pending_cues.push(Box::new(cue));
    }

    /// 帧末批量发射（在 PostUpdate Schedule 中执行）
    pub fn flush(&mut self, writer: &mut EventWriter<...>) {
        for cue in self.pending_cues.drain(..) {
            cue.write_to(writer);
        }
    }
}
```

### 2. Replay 系统完整实现

#### 2.1 ReplayEvent 定义

```rust
/// 回放事件（Persistence 层，可序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub turn: u32,                          // 回合号
    pub action_index: u32,                  // 行动序号
    pub actor: EntityId,                    // 行动实体（确定性 ID）
    pub action_type: ReplayActionType,      // 行动类型
    pub target: Option<EntityId>,           // 目标实体
    pub ability_id: Option<AbilityId>,       // 使用的技能
    pub seed_snapshot: u64,                 // 随机数种子快照
    pub result: ReplayActionResult,         // 结果快照
}

pub enum ReplayActionType {
    Move { path: Vec<GridPos> },
    UseAbility { ability_id: AbilityId, target: EntityId },
    Wait,
    EndTurn,
}

pub struct ReplayActionResult {
    pub damage_dealt: Vec<DamageEntry>,
    pub healing_done: Vec<HealEntry>,
    pub effects_applied: Vec<EffectId>,
    pub units_killed: Vec<EntityId>,
    pub units_summoned: Vec<EntityId>,
}
```

#### 2.2 Replay 录制与播放

```rust
/// 回放录制器
pub struct ReplayRecorder {
    pub events: Vec<ReplayEvent>,
    pub initial_seed: u64,
    pub battle_config: BattleConfig,
}

impl ReplayRecorder {
    /// 记录一个行动
    pub fn record_action(&mut self, event: ReplayEvent) {
        // 捕获当前 Prng 种子
        // 记录完整行动事件
        self.events.push(event);
    }

    /// 导出为 Replay 文件
    pub fn export(&self) -> ReplayData { /* JSON 序列化 */ }
}

/// 回放播放器
pub struct ReplayPlayer {
    events: Vec<ReplayEvent>,
    current_index: usize,
    prng: ChaCha8Rng,
}

impl ReplayPlayer {
    /// 初始化回放（重置 Prng 种子）
    pub fn initialize(&mut self, initial_seed: u64) {
        self.prng = ChaCha8Rng::seed_from_u64(initial_seed);
    }

    /// 步进一个行动（逐帧驱动）
    pub fn step(&mut self, world: &mut World) -> bool {
        if self.current_index >= self.events.len() { return false; }
        let event = &self.events[self.current_index];
        // 强制 Prng 状态与录制时一致
        // 执行行动
        // 验证结果与 event.result 一致
        self.current_index += 1;
        true
    }
}
```

#### 2.3 确定性约束

| 约束 | 实现 |
|------|------|
| 固定管线 | 所有 Effect 必须经过同一 Pipeline（ADR-032） |
| 确定性排序 | 速度→阵营→站位固定，无随机 Comparator |
| 统一取整 | 所有百分比计算完成后统一 `floor()` |
| 触发链限制 | `MAX_CHAIN_DEPTH = 0`（反应不连锁） |
| 浮点禁止 | 核心战斗使用 `i32`/`i64`，禁止 `f32` |
| 种子快照 | 每次 Action 前快照 `Prng.seed` |

### 3. I18n 系统完整实现

#### 3.1 基础设施

```rust
/// 本地化 Key（newtype 包装）
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct LocalizedKey(String);

impl LocalizedKey {
    /// 创建 Key（验证格式）
    pub fn new(key: &str) -> Result<Self, KeyValidationError> {
        // 必须匹配 <namespace>.<permanent_id>.<field>
        // 验证 namespace 在合法列表中
        // 禁止语义化名称（如 skill.fireball.name）
    }
}
```

#### 3.2 Fluent 集成

```rust
/// 本地化管理器（Resource）
pub struct LocalizationManager {
    /// 按语言索引的 FluentBundle 缓存
    bundles: HashMap<Language, FluentBundle<FluentResource>>,
    /// 当前语言
    current_language: Language,
    /// LRU 解析缓存
    cache: LruCache<(Language, String), String>,
}

impl LocalizationManager {
    /// 加载 FTL 文件（启动时）
    pub fn load_ftl(&mut self, lang: Language, dir: &str) -> Result<(), LoadError> {
        // 遍历 assets/localization/{lang}/*.ftl
        // 解析为 FluentResource
        // 添加到 bundle
    }

    /// 解析 Key（带回退链）
    pub fn resolve(&mut self, key: &LocalizedKey, args: Option<&FluentArgs>) -> String {
        // 1. 尝试 current_language
        // 2. 回退 zh-CN（项目默认）
        // 3. 回退 en-US（兜底）
        // 4. 回退 key.as_str() (Debug) / "" (Release)
    }
}
```

#### 3.3 LocalizedText Component

```rust
/// 附着在需要本地化的 UI 实体上
#[derive(Component)]
pub struct LocalizedText {
    pub key: LocalizedKey,
    pub args: Option<FluentArgs>,
}

/// 语言切换时触发重新解析
pub fn refresh_localized_text(
    mut query: Query<&mut Text, With<LocalizedText>>,
    localized_query: Query<&LocalizedText>,
    mut manager: ResMut<LocalizationManager>,
) {
    for (mut text, localized) in query.iter_mut() {
        text.0 = manager.resolve(&localized.key, localized.args.as_ref());
    }
}
```

#### 3.4 FTL 文件组织

```
assets/localization/
├── zh-CN/
│   ├── attr.ftl          # 属性名称
│   ├── tag.ftl           # 标签名称
│   ├── buff.ftl          # Effect/Buff 名称和描述
│   ├── skill.ftl         # Ability 名称和描述
│   ├── ui.ftl            # 界面文本
│   └── battle.ftl        # 战斗文本（命中等）
├── en-US/
│   └── ...
└── ja-JP/
    └── ...
```

#### 3.5 Definition 字段迁移

所有 Definition 类型必须将 `name`/`description` 替换为 `name_key`/`desc_key`：

| 领域 | 旧字段 | 新字段 | Key 格式 |
|------|--------|--------|----------|
| Attribute | `name` | `name_key` | `attr.a_XXX.name` |
| Tag | `name` | `name_key` | `tag.t_XXX.name` |
| Effect（ApplyBuff） | `name` / `description` | `name_key` / `desc_key` | `buff.b_XXX.name` / `.desc` |
| Ability | `name` / `description` | `name_key` / `desc_key` | `skill.s_XXX.name` / `.desc` |

## Module Design

```
src/
├── core/
│   ├── cue/
│   │   ├── mod.rs          # CuePlugin + CueEvent trait
│   │   ├── types.rs        # 12 种 Cue struct（Message）
│   │   ├── emitter.rs      # CueEmitter Resource + flush 系统
│   │   └── def.rs          # CueDefinition + CueRegistry（ADR-030）
│   ├── replay/
│   │   ├── mod.rs          # ReplayPlugin
│   │   ├── recorder.rs     # ReplayRecorder Resource
│   │   ├── player.rs       # ReplayPlayer（回放驱动）
│   │   └── types.rs        # ReplayEvent + ReplayActionType + ReplayActionResult
│   └── i18n/（不存在，Core 层仅存 Key，不存文本）
├── infrastructure/
│   └── localization/
│       ├── mod.rs          # LocalizationPlugin
│       ├── manager.rs      # LocalizationManager Resource
│       ├── key.rs          # LocalizedKey newtype
│       └── loader.rs       # FTL 文件加载器
├── ui/
│   └── components/
│       └── localized_text.rs  # LocalizedText Component + refresh 系统
└── assets/localization/    # FTL 文件
```

## Communication Design

### 执行时序

```
PreStartup:
  LocalizationPlugin::build()
    → 加载所有 FTL 文件 → LocalizationManager Resource

Logic Schedule (每帧):
  ReplayRecorder.record_action()
    → 在每个 Action 完成后追加 ReplayEvent

PostUpdate Schedule:
  CueEmitter::flush()
    → 批量发射帧内累积的 Cue → EventWriter<T>

  refresh_localized_text()
    → 语言切换时刷新 Text Component

Startup / 语言切换:
  app.world.resource::<LocalizationManager>().set_language("en-US")
    → 触发 refresh_localized_text 重新解析
```

### 跨层通信

```
Core (Cue) ──→ CueEvent (Message) ──→ UI (EventReader<CueDamage>)
                                       ├── 飘字
                                       ├── HP 条更新
                                       └── 战斗日志追加

Core (Replay) ──→ ReplayEvent (序列化) ──→ 文件系统
                                       ├── 写入磁盘
                                       └── CI 回归验证

Infrastructure (I18n) ←── Core (LocalizedKey)
                        ←── UI (LocalizedText Component)
```

## 边界定义

| 规则 | 允许 | 禁止 |
|------|------|------|
| Core → Cue | 发射 CueEvent Message | Cue 反向影响战斗逻辑 |
| Core → Replay | 记录 ReplayEvent | Replay 修改战斗状态 |
| Core → I18n | 持有 LocalizedKey | Core 直接调用 LocalizationManager |
| UI → Cue | 订阅 CueEvent 驱动表现 | Cue 携带 UI 资源引用 |
| UI → I18n | 持有 LocalizedText Component | 硬编码玩家可见文本 |

## Forbidden（禁止事项）

- 🟥 **禁止** Core 层有任何硬编码的玩家可见文本字符串
- 🟥 **禁止** 任何 Effect 执行后不发射对应的 Cue 事件
- 🟥 **禁止** 使用语义化 Key（如 `skill.fireball.name`）— 必须使用永久 ID（如 `skill.s_1001.name`）
- 🟥 **禁止** Replay 事件流记录任何 UI 状态或翻译文本 — 只记录 ID 和数值
- 🟥 **禁止** 回放时使用实时时钟或系统随机源 — 必须使用确定性 ChaCha8Rng
- 🟥 **禁止** Replay 验证失败后静默修正游戏状态 — 必须报错或断言失败
- 🟥 **禁止** 每帧解析 Fluent AST — 必须使用预编译缓存
- 🟥 **禁止** CueEmitter 在非帧末时间点发射事件 — 必须走 `flush()` 批量通道
- 🟥 **禁止** `f32` 出现在任何 ReplayEvent 数据结构中

## Definition / Instance Design

| 层 | Cue | Replay | I18n |
|----|-----|--------|------|
| Definition | `CueDefinition`（id, cue_type, vfx/sfx 引用） | — | FTL 文件（`assets/localization/*.ftl`） |
| Instance | — | `ReplayRecorder` Resource + `ReplayPlayer` Resource | `LocalizationManager` Resource（FluentBundle 缓存） |
| Runtime | `CueEmitter` Resource（帧内累积队列） | `ReplayEvent` 序列 | `LocalizedText` Component（UI 绑定） |
| Persistence | — | `ReplayData`（JSON 序列化文件） | 仅存储 Key，不存储文本 |

## 后果

### 正面
- 表现层与逻辑层彻底解耦 — 所有 Core→UI 通信通过 Cue 事件
- 确定性回放系统可验证所有战斗 Bug（Bug → Replay 测试 → 修复）
- 国际化使项目具备多语言发布能力
- FTL 回退链保证语言切换的鲁棒性
- 所有硬编码文本被 Key 取代，内容配置化

### 负面
- 新建 3 个重要系统，工作量大（尤其是 Replay 的录制播放基础设施）
- 所有现有 RON 配置文件的 name/description 字段需要迁移到 name_key/desc_key
- 需要额外 CI 步骤验证 Replay 确定性
- FTL 文件的维护工作量随内容增长

## 替代方案（已拒绝）

| 方案 | 拒绝原因 |
|------|----------|
| 使用 bevy_fluent 插件而非自研 Fluent 适配 | 宪法已明确禁止依赖第三方 bevy_fluent 插件，必须基于 fluent-rs 自封装 |
| Replay 只录制 Event Stream 不做结果验证 | 无验证的 Replay 无法转化为 CI 测试用例，价值减半 |
| Cue 消息直接使用 Bevy Event | `CueEmitter.batch` 模式可避免帧内多次事件分发开销 |
| Lazy FTL 加载 | 启动时一次性预加载+缓存对 UX 更优，Lazy 加载导致语言切换卡顿 |
