# Config System Design — 配置系统设计

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第三节 — 配置系统设计

本文档定义统一的运行时配置管理体系，涵盖引擎配置、游戏规则配置、用户设置和调试开关的分层设计。

交叉引用：
- `docs/architecture/infrastructure-design.md` — Infrastructure 层 config 模块设计
- `docs/architecture.md` — 数据驱动总纲（配置三级分离）
- `docs/architecture/content-pipeline.md` — 内容管线（Definition 加载）
- `docs/AI开发宪法完整版.md` — AI 开发宪法（最高约束力），本文档对应条款：1.1.2（定义与实例分离）、12.1.1-12.1.6（数据驱动核心）、12.2.1（Schema）、12.4.1（平衡参数全配置化）、14.0.1（统一设置管理）

---

## 1. 配置分层模型

> **宪法 §1.1.2（定义与实例分离）**：所有配置层（EngineConfig、GameRulesConfig、UserSettings）均为 Definition 数据，运行时只读，不可变。DebugSwitches 为运行时内存状态，不持久化。

### 1.1 四层配置架构

```
┌─────────────────────────────────────────┐
│  EngineConfig        引擎层配置          │  Bevy 引擎参数、窗口设置、渲染选项
├─────────────────────────────────────────┤
│  GameRulesConfig     游戏规则配置        │  平衡性参数、伤害公式、回合规则
├─────────────────────────────────────────┤
│  UserSettings        用户偏好设置        │  分辨率、音量、语言、控制绑定
├─────────────────────────────────────────┤
│  DebugSwitches       调试开关            │  God Mode、速度切换、无冷却
└─────────────────────────────────────────┘
```

### 1.2 各层职责

| 层 | 职责 | 修改频率 | 热重载 | 存储位置 |
|----|------|---------|--------|---------|
| EngineConfig | Bevy 引擎参数 | 极低 | 🟥 需重启 | `assets/config/engine.ron` |
| GameRulesConfig | 平衡性参数 | 中等（策划调优） | ✅ 支持 | `content/rules/` |
| UserSettings | 用户偏好 | 低（用户主动修改） | 🟨 部分支持 | 用户目录 |
| DebugSwitches | 开发调试 | 高（开发调试时频繁切换） | ✅ 支持 | 运行时内存 |

### 1.3 层间优先级

当多层配置同时定义同一参数时，优先级从高到低：

```
DebugSwitches > UserSettings > GameRulesConfig > EngineConfig（默认值）
```

---

## 2. 配置存储

### 2.1 EngineConfig

**格式**：RON 文件
**路径**：`assets/config/engine.ron`
**内容**：

```rust
pub struct EngineConfig {
    pub window: WindowConfig,
    pub rendering: RenderingConfig,
    pub physics: PhysicsConfig,
    pub asset: AssetConfig,
}

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

pub struct RenderingConfig {
    pub max_fps: u32,
    pub msaa: bool,
    pub shadow_quality: ShadowQuality,
}
```

**规则**：
- 🟥 EngineConfig 修改后必须重启才能生效
- 🟥 禁止运行时修改 EngineConfig 的窗口参数
- 🟩 修改 EngineConfig 需要用户确认

### 2.2 GameRulesConfig

**格式**：RON 文件
**路径**：`content/rules/`（每个领域一个文件）
**内容**：

```rust
pub struct GameRulesConfig {
    pub battle: BattleConfig,
    pub character: CharacterConfig,
    pub skill: SkillConfig,
    pub buff: BuffConfig,
}

pub struct BattleConfig {
    pub damage_floor: i32,           // 伤害下限（≥1）
    pub heal_floor: i32,             // 治疗下限（≥0）
    pub max_buffs_per_unit: usize,   // 每单位最大 Buff 数
    pub crit_multiplier: f32,        // 暴击倍率
}

pub struct SkillConfig {
    pub base_cooldown: u32,          // 基础冷却回合数
    pub max_skill_slots: usize,      // 最大技能槽位
}
```

> **优化来源**：`docs/其他/44.md` — 反"上帝配置"：将单一 GameRulesConfig 拆分为细粒度的独立 Resource

**反上帝配置（Anti-God-Config）规则**：

🟥 **禁止将所有游戏规则塞入单一的 `GameRulesConfig` Resource**。

Bevy 中所有 System 读取同一个 `Res<GameRulesConfig>` 会产生全局读锁竞争，阻碍 System 并行执行。且修改任一子模块会触发整个 Resource 的 `Changed` 状态，导致所有依赖系统重新执行。

**正确做法 — 按领域拆分为细粒度 Resource**：

```rust
// ✅ 正确：拆分为独立 Resource，各自可独立热重载
#[derive(Resource)]
pub struct BattleConfig { /* ... */ }

#[derive(Resource)]
pub struct SkillConfig { /* ... */ }

#[derive(Resource)]
pub struct BuffConfig { /* ... */ }

// 或使用 Bevy Asset（Handle<T>）天然支持热重载
// 游戏逻辑通过 Assets<BattleConfig> 访问
```

**性能收益**：
- System 只声明所需 `Res<SkillConfig>`，其他 System 可并行
- 修改 SkillConfig 不触发 BattleConfig 的 `Changed` 检测
- MOD 可精确替换单个配置文件而不影响其他

**规则**：
- 🟥 GameRulesConfig 是 Definition 数据，运行时只读
- 🟩 GameRulesConfig 支持热重载（战斗外）
- 🟥 战斗中禁止热重载 GameRulesConfig（破坏确定性）
- 🟥 所有平衡性参数必须在 GameRulesConfig 中，禁止硬编码

### 2.3 UserSettings

**格式**：RON 文件
**路径**：`~/.config/srpg/settings.ron`（平台相关）
**内容**：

```rust
pub struct UserSettings {
    pub audio: AudioSettings,
    pub display: DisplaySettings,
    pub input: InputSettings,
    pub localization: LocalizationSettings,
}

pub struct AudioSettings {
    pub master_volume: f32,       // 0.0 ~ 1.0
    pub bgm_volume: f32,
    pub sfx_volume: f32,
}

pub struct DisplaySettings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub ui_scale: f32,
}
```

> **优化来源**：`docs/其他/44.md` — 引入防抖机制（Debounce），避免 UI 滑块拖动时的高频 IO 卡顿

**防抖写入机制**：

用户在设置界面拖动"音量滑块"或"UI 缩放滑块"时，每次微小变更都会触发磁盘写入，导致严重 IO 阻塞和卡顿。

```
UI 层：滑块拖动时只修改内存中的 UserSettings（立即生效）
    ↓
防抖 Timer：停止操作 200ms 后触发写入
    ↓
异步写入：使用 AsyncComputeTaskPool 将配置写入磁盘
```

**实现要点**：

```rust
#[derive(Resource)]
pub struct UserSettingsDebounce {
    pub timer: Timer,  // 200ms 防抖窗口
    pub needs_write: bool,
}

// 音量等轻量设置：内存修改立即生效，防抖写盘
// 分辨率/全屏等重型设置：UI 显示"待应用"状态，点击"Apply"按钮才生效
```

**规则**：
- 🟩 UserSettings 加载一次后缓存在内存中
- 🟩 修改后立即生效（音量、分辨率等）
- 🟩 UserSettings 修改时自动保存到用户目录（防抖 200ms）
- 🟥 UserSettings 不影响游戏逻辑正确性

### 2.4 DebugSwitches

**格式**：纯内存 Resource（不持久化）
**路径**：运行时
**内容**：

```rust
#[derive(Resource, Default)]
pub struct DebugSwitches {
    pub god_mode: bool,              // 无敌模式
    pub one_hit_kill: bool,          // 一击必杀
    pub no_cooldown: bool,           // 无冷却
    pub speed_multiplier: f32,       // 速度倍率（1.0 = 正常）
    pub show_hitboxes: bool,         // 显示碰撞框
    pub show_pathfinding: bool,      // 显示寻路路径
    pub skip_ai_animation: bool,     // 跳过 AI 动画
}
```

**规则**：
- 🟥 DebugSwitches 仅在 Dev 构建中可用
- 🟥 DebugSwitches 不参与 Release 构建
- 🟩 通过 Feature Flag 控制编译
- 🟥 DebugSwitches 修改不触发日志记录

---

## 3. 配置加载时机

### 3.1 加载时间线

```
游戏启动
  ↓
加载 EngineConfig          ← 启动时立即加载
  ↓
加载 UserSettings          ← 启动时加载并缓存
  ↓
AppState::MainMenu
  ↓
用户选择关卡
  ↓
OnEnter(AppState::InGame)
  ↓
加载 GameRulesConfig       ← 进入游戏时加载
  ↓
加载关卡内容配置
  ↓
战斗开始
  ↓
DebugSwitches 初始化       ← Dev 构建初始化
```

### 3.2 加载规则

| 配置类型 | 加载时机 | 加载方式 | 缓存策略 |
|---------|---------|---------|---------|
| EngineConfig | 游戏启动 | 同步加载 | 全局缓存 |
| UserSettings | 游戏启动 | 同步加载 | 全局缓存 |
| GameRulesConfig | OnEnter(InGame) | 同步加载 | 场景生命周期 |
| DebugSwitches | 插件注册时 | init_resource | 全局缓存 |

### 3.3 加载失败处理

- 🟥 EngineConfig 加载失败：使用硬编码默认值 + 记录 ERROR 日志
- 🟥 UserSettings 加载失败：使用默认设置 + 记录 WARN 日志
- 🟥 GameRulesConfig 加载失败：使用硬编码默认值 + 记录 ERROR 日志
- 🟥 任何配置加载失败都禁止 crash

---

## 4. 热重载

### 4.1 热重载边界

| 配置类型 | 可热重载 | 时机 | 说明 |
|---------|---------|------|------|
| EngineConfig | 🟥 | — | 需要重启 |
| GameRulesConfig | ✅ | 战斗外 | Definition 数据，安全替换 |
| UserSettings | 🟨 | 部分 | 音量等立即生效，分辨率需重启 |
| DebugSwitches | ✅ | 任意 | 运行时内存，随时可改 |

> **优化来源**：`docs/其他/44.md` — 战斗锁规则强化 + 热重载事件风暴缓解

**战斗锁（Battle Lock）规则**：

🟥 **在战斗中（AppState::InGame）期间，禁止 ALL 配置热重载**。

如果在战斗中途修改了伤害公式或暴击率，当前的 Replay 录像将彻底作废。这把锁是确定性架构的底线。

```rust
// 实现：在热重载系统中检查 AppState
fn config_hot_reload_system(
    state: Res<State<AppState>>,
    // ...
) {
    if *state.get() == AppState::InGame {
        return; // 战斗中跳过所有热重载
    }
    // 正常热重载逻辑...
}
```

**热重载事件风暴缓解**：

策划保存包含 500 个技能配置的 RON 文件时，可能在一帧内触发大量变更事件，导致下游 System 卡顿。

```
优化方案：
1. ConfigReloaded 事件只传递 config_type（如 SkillConfig），不传递 changes 列表
2. 下游 System 收到事件后标记 NeedsRebuild<SkillConfig> Marker
3. 在下一帧的特定 Phase（如 PreparePhase）统一执行重建
```

### 4.2 热重载流程

```
检测文件变更
  ↓
验证新配置合法性
  ↓
├── 合法 → 替换内存中的配置 → 通知依赖系统
└── 非法 → 回退到旧配置 → 记录 WARN 日志
```

### 4.3 热重载通知

热重载完成后，通过 Message 通知依赖系统：

```rust
pub struct ConfigReloaded {
    pub config_type: ConfigType,
    pub changes: Vec<ConfigChange>,
}
```

---

## 5. 平衡参数管理

### 5.1 参数存放约定

🟥 **所有平衡性参数必须在 GameRulesConfig 中，禁止硬编码在 Rust 代码中**。

**必须在配置中的参数**：

- 伤害公式系数（攻击倍率、防御减免率）
- 冷却时间默认值
- 属性成长率
- Buff 持续时间、叠加上限
- 技能范围、消耗
- 经验值曲线、升级属性增长
- 装备属性加成

### 5.2 参数验证

加载 GameRulesConfig 时必须验证：

- 🟥 伤害下限 ≥ 1
- 🟥 治疗下限 ≥ 0
- 🟥 Buff 叠加上限 > 0
- 🟥 冷却时间 ≥ 0
- 🟥 暴击倍率 > 1.0
- 🟥 属性成长率在合理范围内（0.0 ~ 10.0）

验证失败时使用默认值并记录 ERROR 日志。

### 5.3 配置加载失败分级处理

> **优化来源**：`docs/其他/44.md` — 按配置类型分级处理加载失败，EngineConfig 失败直接 panic，其他优雅降级

不同配置类型的加载失败应有不同的处理策略：

| 配置类型 | 失败行为 | 理由 |
|---------|---------|------|
| EngineConfig | 🟥 ERROR 日志 + 使用硬编码默认值 | 引擎参数缺失时必须优雅降级，禁止 panic（宪法 §13.9.4） |
| GameRulesConfig | 🟥 ERROR 日志 + 使用默认值 | 游戏规则可降级，不能因配置错误导致闪退 |
| UserSettings | 🟨 WARN 日志 + 重置为默认值 | 用户设置非关键，重置即可恢复 |

```rust
// 实现示例
fn load_config<T: ConfigResource>(path: &str) -> T {
    match load_from_path(path) {
        Ok(config) => match config.validate() {
            Ok(()) => config,
            Err(e) => {
                error!("配置验证失败: {:?}，使用默认值", e);
                T::default_config()
            }
        },
        Err(e) => {
            error!("配置加载失败: {:?}，使用默认值", e);
            T::default_config()
        }
    }
}
```

---

## 6. MOD 配置覆盖机制

> **优化来源**：`docs/其他/44.md` — MOD 配置使用 Patch 语义（深度合并）而非 Override（整体替换）

### 6.1 Patch vs Override

SRPG 的 MOD 生态要求配置覆盖采用 **Patch 语义**（深度合并），而非 Override（整体替换）：

```
❌ Override（整体替换）：MOD 需要复制整个 skill.ron 文件，两个 MOD 修改同一文件只能二选一
✅ Patch（深度合并）：MOD 只写修改的字段，系统自动合并到基础配置
```

### 6.2 实现方式

```
content/rules/
├── base/                    # 基础配置
│   ├── battle.ron
│   ├── skill.ron
│   └── buff.ron
└── overrides/               # 覆盖配置（MOD 或策划微调）
    ├── mod_balance/
    │   └── skill.ron        # 只包含修改的字段
    └── mod_new_skills/
        └── skill.ron        # 新增或覆盖特定技能
```

**加载流程**：
1. 加载 `base/skill.ron` 作为基础
2. 遍历 `overrides/` 下所有 RON 文件
3. 按 `load_order` 排序，后加载的覆盖先加载的
4. 执行深度合并（Deep Merge）：数值字段直接替换，数组字段可选 Append 或 Replace

### 6.3 合并策略标记

```ron
// MOD 配置中可指定合并策略
(
    id: "fireball",
    base_damage: 60,                    // 数值字段：直接替换
    buff_effects: Append(["stun"]),     // 数组字段：追加而非替换
    // buff_effects: Replace(["stun"]), // 如需整体替换，显式声明
)
```

**冲突日志**：当发生 MOD 覆盖时，系统必须在 debug.log 中输出 Warning，方便 MOD 作者排查兼容性问题。

---

## 7. 调试开关实现

### 7.1 Feature Gate

```rust
// Cargo.toml
[features]
default = []
dev = ["debug_switches"]

[dependencies]
# ...
```

```rust
// src/app/plugins.rs
#[cfg(feature = "dev")]
app.add_plugins(DebugSwitchPlugin);
```

### 7.2 调试面板集成

- 🟩 通过 `bevy_egui` 调试面板控制 DebugSwitches
- 🟩 快捷键切换常用开关（如 F8 = God Mode）
- 🟩 调试面板中显示当前所有开关状态

---

## 8. 禁止事项

- 🟥 **硬编码平衡性数字在 Rust 代码中**（必须在 GameRulesConfig 中）
- 🟥 **运行时修改 Config 定义**（Config 是只读的）
- 🟥 **配置加载失败时 crash**（必须使用默认值）
- 🟥 **战斗中热重载 GameRulesConfig**（破坏确定性）
- 🟥 **DebugSwitches 参与 Release 构建**（Feature Gate 控制）
- 🟥 **配置结构频繁变更**（稳定性优先于优雅性）
- 🟥 **UserSettings 包含游戏逻辑数据**（只存用户偏好）
- 🟥 **EngineConfig 被运行时修改**（需要重启）

---

## 9. 实现备注

### 9.1 ConfigResource 统一接口

```rust
pub trait ConfigResource: Resource + Default + Clone + 'static {
    /// 配置文件路径
    fn file_path() -> &'static str;
    /// 验证配置合法性
    fn validate(&self) -> Result<(), ConfigValidationError>;
    /// 获取默认值
    fn default_config() -> Self;
}
```

### 9.2 配置管理 Plugin

```rust
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // 1. 加载 EngineConfig（启动时）
        // 2. 加载 UserSettings（启动时）
        // 3. 注册 GameRulesConfig Resource（进入游戏时加载）
        // 4. 注册 DebugSwitches（Dev 构建）
        // 5. 启用热重载监听（战斗外）
    }
}
```

---

## 10. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `infrastructure-design.md` | 本文档的 config 模块在 Infrastructure 层实现 |
| `content-pipeline.md` | GameRulesConfig 是 Content 数据的一部分 |
| `asset_lifecycle_rules.md` | 配置文件的资源加载遵循资源生命周期规则 |
| `hot_reload_rules.md` | 配置热重载是热重载系统的核心场景 |
| `architecture.md` | 本文档是"数据驱动"章节的详细补充 |
