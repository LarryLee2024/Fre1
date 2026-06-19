# User Settings 用户设置系统

## 1. 新特性概述

Bevy 0.19 新增 `bevy_settings` crate，提供标准化的用户偏好持久化方案。在此之前，开发者需要自行实现配置文件的读写逻辑，涉及序列化、文件路径管理、平台差异处理等繁琐工作。`bevy_settings` 将这些通用需求统一封装，让开发者只需定义一个普通的 Rust struct 即可获得完整的持久化能力。

**核心概念：Settings Group**

Settings Group 是普通 Rust struct，derive `Resource` + `SettingsGroup` + `Reflect` 即可成为可持久化的用户设置组。每个 Settings Group 自动获得：

- 启动时从磁盘加载（如文件不存在则使用 `Default` 值）
- 作为 Bevy `Resource` 注入 World，可在系统中正常读取
- 通过 `SavePreferencesDeferred` / `SavePreferencesSync` 触发保存
- 跨平台存储路径自动管理

---

## 2. API 详解

### 2.1 定义 Settings Group

```rust
use bevy::prelude::*;
use bevy::settings::{SettingsGroup, PreferencesPlugin, SavePreferencesDeferred, SavePreferencesSync};

#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct AudioSettings {
    music_volume: f32,
    sfx_volume: f32,
}
```

**要点：**

- 必须同时 derive `Resource`、`SettingsGroup`、`Reflect`、`Default`
- `#[reflect(...)]` 中必须包含 `Resource`、`SettingsGroup`、`Default`
- 字段类型必须是实现了 `Reflect` + `Serialize` + `DeserializeOwned` 的类型
- `Default` 值即为首次启动时的默认设置

### 2.2 注册 Plugin

```rust
fn main() {
    App::new()
        .add_plugins(PreferencesPlugin::new("com.fre.srpg"))
        .register_type::<AudioSettings>()
        .add_plugins(DefaultPlugins)
        .run();
}
```

**行为说明：**

- `PreferencesPlugin::new("com.fre.srpg")` 接收应用标识符，用于确定存储路径
- 插件启动时自动扫描所有已注册的 `SettingsGroup` 类型，从磁盘加载对应文件
- 若文件不存在或解析失败，回退到 `Default` 值
- 加载完成后自动将 Settings Group 作为 `Resource` 插入 World
- **必须**调用 `register_type::<T>()` 注册每个 Settings Group 类型，否则 Reflect 系统无法识别

### 2.3 读取设置

```rust
fn adjust_volume(audio: Res<AudioSettings>, mut music: ResMut<AudioSink>) {
    music.set_volume(audio.music_volume);
}
```

Settings Group 注册后就是普通的 `Resource`，可以使用 `Res<T>` / `ResMut<T>` 在系统中正常读取。无需任何特殊 API。

### 2.4 保存设置

#### 延迟保存（推荐）

```rust
fn save_settings_on_volume_changed(
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    if !settings.is_changed() { return; }
    commands.queue(SavePreferencesDeferred(Duration::from_secs_f32(0.5)));
}
```

**`SavePreferencesDeferred` 机制：**

- 接收一个 `Duration` 参数作为防抖延迟
- 在延迟期间若有新的保存请求，会重置计时器（防抖）
- 延迟结束后才真正执行写盘操作
- 适合 UI 滑块拖动等高频修改场景，避免频繁 I/O
- 非阻塞，不影响帧率

#### 同步保存

```rust
fn on_app_exit(mut exit: EventReader<AppExit>, settings: Res<AudioSettings>) {
    // 使用 SavePreferencesSync::IfChanged，阻塞直到写入完成
}
```

**`SavePreferencesSync` 机制：**

- 阻塞式保存，确保数据写入磁盘后才返回
- `IfChanged` 变体仅在数据有变更时才写盘
- 适用于应用退出等必须确保数据落盘的场景

### 2.5 退出时保存

```rust
// 在应用退出流程中，使用 SavePreferencesSync::IfChanged
// 阻塞等待写入完成，确保用户设置不会丢失
```

退出时保存是保障用户设置不丢失的关键步骤。推荐在 `AppExit` 事件处理中触发同步保存。

### 2.6 存储位置

| 平台 | 路径 |
|------|------|
| Linux | `$XDG_CONFIG_HOME/<app_name>/` |
| macOS | `~/Library/Preferences/<app_name>/` |
| Windows | `%LOCALAPPDATA%\<app_name>\` |
| WASM | browser `localStorage` |

其中 `<app_name>` 即 `PreferencesPlugin::new()` 时传入的标识符。例如 `"com.fre.srpg"` 在 macOS 上的完整路径为 `~/Library/Preferences/com.fre.srpg/`。

每个 Settings Group 对应一个独立的配置文件，文件名基于类型名自动生成。

---

## 3. 对 SRPG 项目的应用

### 3.1 推荐的 Settings Group

根据 SRPG 项目的需求，建议定义以下 Settings Group：

```rust
/// 音频设置
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct AudioSettings {
    /// 背景音乐音量 [0.0, 1.0]
    music_volume: f32,
    /// 音效音量 [0.0, 1.0]
    sfx_volume: f32,
    /// 语音音量 [0.0, 1.0]（战棋游戏常有剧情语音）
    voice_volume: f32,
}

/// 视频设置
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct VideoSettings {
    /// 是否全屏
    fullscreen: bool,
    /// 垂直同步
    vsync: bool,
    /// 分辨率缩放 [0.5, 2.0]
    resolution_scale: f32,
}

/// 游戏玩法设置
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct GameplaySettings {
    /// 自动存档
    auto_save: bool,
    /// 战斗速度倍率 [1.0, 3.0]
    battle_speed: f32,
    /// 显示伤害数字
    show_damage_numbers: bool,
    /// 显示网格
    show_grid: bool,
}

/// 操控设置
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct ControlSettings {
    // 键位映射
    // 具体结构待 @data-architect 设计
}
```

**设计原则：**

- 每个 Settings Group 职责单一，按功能域划分
- 字段使用语义清晰的名称，配合文档注释
- 数值范围建议在注释中标注，UI 层负责 clamp
- `ControlSettings` 的键位映射结构较复杂，需 @data-architect 专门设计

### 3.2 Config vs Settings 分离

这是项目架构中的重要区分，必须严格遵守：

| 维度 | Config（配置） | Settings（设置） |
|------|---------------|-----------------|
| 性质 | 游戏内容定义 | 用户偏好 |
| 可变性 | 只读 | 可读写 |
| 来源 | 开发者/策划 | 用户 |
| 存储 | `assets/config/` | 系统偏好目录 |
| 设计者 | @data-architect | @feature-developer |
| 示例 | 职业属性、技能数据、Buff定义 | 音量、画质、语言、键位 |

**红线：不要在 `assets/config` 中混入 user settings。**

违反此原则会导致：
- 用户修改无法持久化（assets 目录不应被运行时写入）
- 打包后设置丢失
- 违反数据架构的 Definition/Runtime 分离原则

---

## 4. 与现有 Config 系统的关系

项目已有数据驱动配置系统（content/ 层，定义态数据），User Settings 是补充而非替代：

```
┌─────────────────────────────────────────────────┐
│                  数据层总览                       │
├─────────────────────┬───────────────────────────┤
│   Config（定义态）    │   Settings（运行态）       │
├─────────────────────┼───────────────────────────┤
│ 职业定义             │ 音量偏好                   │
│ 技能定义             │ 画质偏好                   │
│ Buff定义             │ 战斗速度偏好               │
│ 地图数据             │ 键位映射                   │
│ 怪物数据             │ 语言偏好                   │
├─────────────────────┼───────────────────────────┤
│ assets/config/      │ 系统偏好目录               │
│ 只读                │ 可读写                     │
│ @data-architect     │ @feature-developer         │
└─────────────────────┴───────────────────────────┘
```

**交互规则：**

- Config 数据在运行时通过 Effect/Modifier 管线生效，Settings 数据直接影响 UI/系统行为
- Settings 不应绕过 Effect/Modifier 管线直接修改战斗数值（如 `battle_speed` 应通过 Modifier 管线生效）
- Config 与 Settings 之间不存在依赖关系，各自独立加载

---

## 5. 迁移步骤

### 步骤 1：添加 bevy_settings 依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
bevy = { version = "0.19", features = ["settings"] }
```

> 注意：具体 feature 名称以 Bevy 0.19 正式发布为准，`bevy_settings` 可能作为默认 feature 或需要显式启用。

### 步骤 2：定义 Settings Group

创建 `src/settings/` 模块，定义各 Settings Group：

```
src/settings/
├── mod.rs           # 模块导出 + Plugin 注册
├── audio.rs         # AudioSettings
├── video.rs         # VideoSettings
├── gameplay.rs      # GameplaySettings
└── control.rs       # ControlSettings
```

### 步骤 3：注册 PreferencesPlugin

在 App 构建时注册：

```rust
app.add_plugins(PreferencesPlugin::new("com.fre.srpg"))
    .register_type::<AudioSettings>()
    .register_type::<VideoSettings>()
    .register_type::<GameplaySettings>()
    .register_type::<ControlSettings>();
```

### 步骤 4：迁移现有的硬编码设置到 SettingsGroup

检查项目中所有硬编码的用户偏好值（如音量默认值、画质选项等），将其迁移到对应的 Settings Group 中。迁移时注意：

- 保留现有默认值，确保向后兼容
- 若已有自定义的配置文件格式，编写一次性迁移工具
- 确保迁移后现有功能不受影响

### 步骤 5：添加保存触发系统

```rust
/// 通用设置保存系统：任何 Settings Group 变更时延迟保存
fn auto_save_preferences(
    audio: Res<AudioSettings>,
    video: Res<VideoSettings>,
    gameplay: Res<GameplaySettings>,
    control: Res<ControlSettings>,
    mut commands: Commands,
) {
    let any_changed = audio.is_changed()
        || video.is_changed()
        || gameplay.is_changed()
        || control.is_changed();

    if any_changed {
        commands.queue(SavePreferencesDeferred(Duration::from_secs_f32(0.5)));
    }
}
```

将此系统添加到 `PostUpdate` schedule 中。

---

## 6. 注意事项

### 6.1 必须的 Derive 宏

Settings Group 必须同时 derive `Resource` + `SettingsGroup` + `Reflect` + `Default`，缺少任何一个都会导致编译错误或运行时异常。`#[reflect(...)]` 属性也必须完整：

```rust
#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
struct MySettings {
    // ...
}
```

### 6.2 应用标识符格式

`app_name` 建议使用反向域名格式（如 `com.fre.srpg`），这能：

- 避免与其他应用的设置冲突
- 符合各平台的命名惯例
- 在 macOS/Linux 上生成合理的目录结构

### 6.3 防抖机制

`SavePreferencesDeferred` 内置防抖机制，频繁修改不会频繁写盘。例如用户拖动音量滑块时，每帧都会触发 `is_changed()`，但实际写盘只在最后一次修改后的延迟时间到期时执行一次。

### 6.4 WASM 环境

WASM 环境下使用浏览器 `localStorage` 存储，无文件系统。需注意：

- `localStorage` 有容量限制（通常 5-10MB）
- 不支持二进制格式，使用 JSON 序列化
- 同步保存 (`SavePreferencesSync`) 在 WASM 中可能表现不同

### 6.5 未来扩展

- Bevy 编辑器工具可以直接利用 Settings 系统，提供可视化的设置编辑界面
- Settings Group 的 Reflect 支持使得运行时 UI 自动生成成为可能
- 可通过 Reflect 实现设置的导入/导出功能

### 6.6 与项目宪法的对齐

- Settings 属于运行态数据，不违反"严禁修改定义态配置数据"的红线
- Settings 的变更不应绕过 Effect/Modifier 管线直接影响战斗数值
- Settings Group 的数据结构设计需经 @data-architect 审查，确保与现有数据架构一致
