---
id: 06-ui.theme-localization
title: Theme and Localization Architecture — 主题与本地化架构
status: code-aligned
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - theme
  - localization
  - styletoken
  - uitextkey
---

# Theme and Localization Architecture — 主题与本地化架构

> **职责**: @presentation-architect | **上游**: ADR-055 §4.3-§4.4 (FontSize/FontSource), §7 (状态分级) | domain rules §1 (StyleToken, Theme 定义), §INV-UI-007, §INV-UI-008 | schema §5 (StyleToken), §6 (UiSettings), §13 (ID 策略) | ADR-053 (Localization)

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 1. 设计目的

大型 SRPG 的 UI 往往需要支持多种主题（Light/Dark/Pixel/HD2D）和多种语言。没有架构约束时：

- **P2**：无 StyleToken 体系，视觉属性散落在 Widget 代码中 → 主题切换困难（schema §2 P2）
- **硬编码泛滥**：50 万行代码中有 1000+ 处 `Color::srgb(0.2, 0.3, 0.5)` 和 `Text::new("Attack")` → 视觉不一致，多语言失败
- **主题切换灾难**：要搜索所有 Widget 逐一修改颜色

本文档定义 StyleToken 体系、Theme 系统、LocalizedText + UiTextKey 使用规范，确保"一次主题切换，全 UI 自动更新"。

---

## 2. StyleToken 体系

### 2.1 设计目的

StyleToken 是**语义化的视觉属性引用**。Widget 不直接使用具体的颜色值/尺寸/字体，而是引用 StyleToken。Theme 切换时通过替换 StyleToken 的配置实现全 UI 自动更新。

### 2.2 三层 StyleToken

```
StyleToken
├── UiColors    — 颜色体系（Primary/Danger/PanelBg/TextPrimary...）
├── UiSpacing   — 间距体系（xs/sm/md/lg/xl/xxl...）
└── UiTypography— 排版体系（heading_font/body_font/heading_size/body_size...）
```

**关键规则**：StyleToken 是 Definition（不可变配置），运行时只读，通过 Theme 配置文件加载。

（引用：domain rules §1 — StyleToken 定义；schema §5 — StyleToken Schema）

### 2.3 UiColors（颜色体系）

| Token 名称 | 语义 | 用途 |
|-----------|------|------|
| primary | 主色 | 主要按钮、高亮元素 |
| secondary | 辅助色 | 次要按钮、辅助元素 |
| danger | 危险色 | 删除/警告按钮 |
| success | 成功色 | 正向反馈 |
| warning | 警告色 | 警告提示 |
| panel_bg | 面板背景 | 所有 Panel 的背景色 |
| panel_border | 面板边框 | 所有 Panel 的边框色 |
| text_primary | 主文本 | 正文颜色 |
| text_secondary | 次要文本 | 注释/说明文字 |
| text_disabled | 禁用文本 | 不可交互的文字 |
| hp_bar | HP 条 | 血条颜色 |
| mp_bar | MP 条 | 魔条颜色 |
| exp_bar | 经验条 | 经验条颜色 |
| buff_icon | 增益图标 | 正面效果图标 |
| debuff_icon | 减益图标 | 负面效果图标 |

**禁止事项**：
- 🟥 禁止在 Widget 中直接使用 `Color::srgb(r, g, b)`
- 🟥 禁止在 Widget 中直接使用 `Color::WHITE` / `Color::BLACK` 等预定义常量
- 🟥 禁止在 Widget 中硬编码任何十六进制颜色值

（引用：schema §5.1 — UiColors）

### 2.4 UiSpacing（间距体系）

| Token | 值 | 用途 |
|-------|-----|------|
| xs | 4px | 最小间距 |
| sm | 8px | 元素间小间距 |
| md | 16px | 标准间距 |
| lg | 24px | 段落间距 |
| xl | 32px | 区域间距 |
| xxl | 48px | 大区域间距 |
| panel_padding | 16px | 面板内边距 |
| button_padding | 8px 16px | 按钮内边距 |
| icon_size | 32px | 图标尺寸 |
| border_radius | 4px | 圆角半径 |

**禁止事项**：
- 🟥 禁止在 Widget 中使用裸 `Val::Px(16.0)` 等间距值
- 🟥 禁止硬编码边距、内边距数值

（引用：schema §5.2 — UiSpacing）

### 2.5 UiTypography（排版体系）

| Token | 语义 | 说明 |
|-------|------|------|
| heading_font | 标题字体族 | FontSource::Family("heading") |
| body_font | 正文字体族 | FontSource::Family("body") |
| mono_font | 等宽字体族 | FontSource::Family("mono") |
| heading_size | 标题字号 | FontSize::Rem(1.5) |
| body_size | 正文字号 | FontSize::Rem(1.0) |
| caption_size | 注释字号 | FontSize::Rem(0.8) |
| button_size | 按钮字号 | FontSize::Rem(0.9) |

**Bevy 0.19 字体规范**：
- FontSize 使用枚举：`FontSize::Px(14.0)`（固定像素）或 `FontSize::Rem(1.2)`（响应式），禁止裸 f32
- FontSource 使用语义类别：`FontSource::Family("heading")`，避免每个 Widget 独立选择字体文件

（引用：ADR-055 §4.3 — FontSize 枚举；§4.4 — FontSource 语义类别；domain rules §9 — Bevy 0.19 特性映射）

---

## 3. Theme 系统

### 3.1 数据结构

```rust
/// 主题定义 — 聚合所有 StyleToken
#[derive(Resource, Reflect, Clone, Default)]
pub struct Theme {
    pub colors: UiColors,
    pub spacing: UiSpacing,
    pub typography: UiTypography,
    pub name: ThemeName,
}

#[derive(Clone, Reflect, Default, PartialEq)]
pub enum ThemeName {
    #[default]
    Dark,
    Light,
    Pixel,
    HD2D,
}
```

(引用：schema §5.4 — Theme)

### 3.2 Theme → StyleToken 映射机制

```
Theme::Dark → UiColors::dark(), UiSpacing::default(), UiTypography::default()
Theme::Light → UiColors::light(), UiSpacing::default(), UiTypography::default()
Theme::Pixel → UiColors::pixel(), UiSpacing::pixel(), UiTypography::pixel()
Theme::HD2D → UiColors::hd2d(), UiSpacing::hd2d(), UiTypography::hd2d()
```

**Theme 切换流程**：
1. 用户在设置中选择新 Theme（或程序通过 UiCommand::ChangeSettings 切换）
2. UiSettings.theme 更新
3. Theme 系统加载对应 Theme 配置
4. Theme Resource 替换
5. 全局 Dirty 触发所有 Widget 刷新（使用新 StyleToken）

**Widget 从不缓存 StyleToken 值**——每次渲染时通过 Res<Theme> 读取当前值，保证切换即时生效。

### 3.3 Theme 配置来源

Theme 配置文件存放在 `assets/config/ui/themes/`：

```
assets/config/ui/themes/
├── dark.theme.ron       # Dark 主题
├── light.theme.ron      # Light 主题
├── pixel.theme.ron      # Pixel 主题
└── hd2d.theme.ron       # HD2D 主题
```

运行时仅加载当前激活的 Theme。Theme 文件在 Asset 热重载时自动刷新。

### 3.4 Theme 自定义（未来扩展）

用户自定义颜色覆盖通过 `UiSettings.custom_colors` 实现（未来功能），覆盖 Theme 中的特定 Token：

```rust
// 未来扩展
pub struct ThemeCustomization {
    pub custom_colors: Option<HashMap<String, Color>>,
    pub custom_spacing: Option<HashMap<String, Val>>,
}
```

自定义值优先级：Theme → 自定义覆盖。

(引用：schema §19 — Future Extension，Theme 自定义)

---

## 4. Localization 体系

### 4.1 核心原则

宪法 §22 Localization First 要求**所有用户可见文本必须使用 LocalizationKey**。

### 4.2 UiTextKey 命名规范

```
格式：ui.<scope>.<id>.<suffix>

示例：
ui.battle.end_turn           — 战斗：结束回合
ui.battle.victory            — 战斗：胜利
ui.inventory.empty_slot      — 背包：空槽位
ui.shop.buy_confirm          — 商店：购买确认
ui.quest.abandon_confirm     — 任务：放弃确认
ui.settings.show_grid        — 设置：显示网格
ui.notification.item_acquired— 通知：获得物品
```

**命名约定**：
- 全小写
- 点号分隔
- scope 对应功能域（battle/inventory/shop/quest/settings/notification/...）
- suffix 描述具体文本用途
- 遵循 `docs/04-data/foundation/id_strategy.md` §5 规范
- ID 分类体系详见 `docs/04-data/foundation/id-taxonomy.md`

(引用：schema §13 — ID 策略)

### 4.3 UiTextKey 使用规范

**规范 L-TEXT-01：ViewModel 只存 Key，不存文本**

```rust
// ❌ 禁止
struct SkillSlotVm {
    name: String,            // 翻译后的文本
}

// ✅ 允许
struct SkillSlotVm {
    name_key: UiTextKey,     // 本地化 Key
}
```

**规范 L-TEXT-02：Widget 使用 LocalizedText，不直接创建 Text**

```rust
// ❌ 禁止
commands.spawn(Text::new("Attack"));

// ✅ 允许
commands.spawn(LocalizedText(UiTextKey::Attack));
```

**规范 L-TEXT-03：参数化文本使用 params**

```rust
// 需要参数插值的文本
let key = UiTextKey::ItemAcquired;  // "获得了 {item_name}"
let params = HashMap::from([
    ("item_name".into(), item_def.name_key),
]);
```

(引用：domain rules §INV-UI-007 — 所有文本走 LocalizationKey；ADR-055 — Forbidden 列表中"UI 存储翻译后的文本字符串"）

### 4.4 LocalizedText Widget

LocalizedText 是统一的文本包装 Widget，对接 ADR-053：

```
LocalizedText
├── 输入：UiTextKey + 可选参数
├── 行为：从 Localization 系统获取翻译文本，自动渲染
├── 刷新：语言切换时自动更新
└── 位置：src/ui/widgets/text/localized_text.rs
```

**对接 ADR-053 的实现路径**：
- LocalizedText Component 持有 UiTextKey
- 系统每帧检测语言变化 → 重新获取翻译 → 更新 Text 内容
- Widget 代码不需要手动调用翻译函数

**Primitives 层豁免**: `spawn_text` 等 Primitives 工厂函数使用 `Text::new` 是允许的，因为 Primitives 层不感知本地化。本地化责任在调用方（Widget/Screen 层）。Widget/Screen 层应使用 `spawn_localized_text` 或 `spawn_localized_button`。

(引用：ADR-055 — LocalizedText Component 对接 ADR-053)

### 4.5 硬编码禁止

| 禁止用例 | 反面示例 | 规范用法 |
|---------|---------|---------|
| 按钮文本 | `Text::new("结束回合")` | `LocalizedText(UiTextKey::EndTurn)` |
| 面板标题 | `Text::new("背包")` | `LocalizedText(UiTextKey::Inventory)` |
| 提示文字 | `Text::new("确认删除？")` | `LocalizedText(UiTextKey::DeleteConfirm)` |
| 状态名称 | `Text::new("中毒")` | `LocalizedText(UiTextKey::StatusPoisoned)` |
| 数值单位 | `Text::new("HP")` | `LocalizedText(UiTextKey::HpLabel)` |

### 4.6 文本所有权
- UI文本展示应使用 `Cow<'static, str>` 避免静态文本堆分配
- LocalizedText 组件存储 Key+参数，渲染时解析为 Cow 文本

---

## 5. UiSettings 持久化

### 5.1 数据结构

```rust
#[derive(Resource, SettingsGroup, Reflect, Clone, Default)]
pub struct UiSettings {
    // 显示偏好
    pub show_damage_numbers: bool,
    pub show_minimap: bool,
    pub show_grid: bool,

    // 战斗设置
    pub battle_speed: f32,          // 0.5 ~ 3.0
    pub auto_battle: bool,

    // 交互设置
    pub tooltip_delay_secs: f32,    // ≥ 0.0
    pub notification_duration_secs: f32,

    // 主题/语言
    pub theme: ThemeName,
    pub language: LanguageVm,
}
```

(引用：schema §6 — UiSettings Schema)

### 5.2 持久化策略

- 使用 Bevy 0.19 的 `SettingsGroup` 自动持久化
- 不进入游戏 Save 文件（独立配置文件）
- 跨会话保持
- SettingsGroup 的三个必须 derive：`Resource` + `SettingsGroup` + `Reflect`

### 5.3 验证规则

| 字段 | 规则 |
|------|------|
| battle_speed | 0.5 ≤ speed ≤ 3.0 |
| tooltip_delay_secs | ≥ 0.0 |
| notification_duration_secs | > 0.0 |

### 5.4 版本迁移

| 迁移场景 | 策略 |
|---------|------|
| 新增字段 | SettingsGroup 自动使用 Default 值填充 |
| 删除字段 | Reflect 反序列化自动忽略 |
| 字段类型变更 | 需要显式迁移函数，在 UiPlugin 中注册 |

(引用：schema §17.3 — 版本迁移；schema §17.4 — Save 版本号)

---

## 6. 四条铁律中的 Token/文本规则

### 6.1 铁律 4：颜色字体间距统一 Token 化

```
INV-UI-008：禁止 Color::srgb() 直接写在 Widget 中
所有视觉属性必须使用 StyleToken（UiColors::Primary / UiSpacing::Md / UiTypography::Heading）
Theme 切换时所有 Widget 自动更新
```

### 6.2 铁律 3（来自 INV-UI-007）：所有文本走 LocalizationKey

```
INV-UI-007：禁止用户可见文本硬编码
符合宪法 §22 Localization First
FontSize 使用枚举，FontSource 使用语义类别
```

(引用：domain rules §INV-UI-009 — 四条铁律精简版)

---

## 7. 与 ADR-053 的对齐校验

| ADR-053 要求 | UI 层实现 |
|-------------|----------|
| 使用 LocalizationKey 引用文本 | UiTextKey 枚举 |
| 禁止存翻译后文本 | ViewModel 只存 Key，不存 String |
| 支持参数插值 | UiTextKey + params HashMap |
| 文本组件统一管理 | LocalizedText Widget |
| 语言切换自动刷新 | Observer 监听语言变更事件 |

---

*本文档由 @presentation-architect 维护。新增 Theme 或 UiTextKey 需经过架构审查。*
