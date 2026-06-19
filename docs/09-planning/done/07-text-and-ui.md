# 文本系统增强与 Feathers UI

> Bevy 0.19 迁移系列 — 第 7 篇
>
> 本文档覆盖 Bevy 0.19 中文本系统的重大增强（FontSource / Variable Font / Responsive Font / LetterSpacing / 文本引擎迁移）、新增的 EditableText 文本输入组件、Feathers Widget 更新，以及对本 SRPG 项目的迁移建议。

---

## 1. 文本系统增强

### 1.1 FontSource — 更好的字体选择

Bevy 0.19 引入 `FontSource` 枚举，提供三种字体标识方式，摆脱了过去只能使用资产句柄的局限：

```rust
// 1. 资产句柄（与以前相同，向后兼容）
TextFont::default().with_font(asset_server.load("fonts/FiraMono.ttf"))

// 2. 字体族名（从字体数据库解析）
TextFont { font: FontSource::Family("FiraMono".into()), ..default() }

// 3. 语义类别
TextFont { font: FontSource::Monospace, ..default() }
```

#### 语义类别

`FontSource` 支持以下语义类别，与 CSS 通用字体族概念对齐：

| 类别 | 说明 |
|------|------|
| `Serif` | 衬线体 |
| `SansSerif` | 无衬线体 |
| `Cursive` | 手写体 |
| `Fantasy` | 装饰体 |
| `Monospace` | 等宽体 |
| `SystemUi` | 系统默认 UI 字体 |
| `Emoji` | 表情符号字体 |
| `Math` | 数学符号字体 |

#### 自定义默认值

通过 `FontCx` Resource 可以为每个语义类别指定默认字体族名：

```rust
fn configure_fonts(mut font_cx: ResMut<FontCx>) {
    font_cx.set_serif_family("Merriweather");
    font_cx.set_monospace_family("JetBrains Mono");
}
```

#### Feature 要求

- `FontSource::Family` 按名称查找系统字体，需要启用 `bevy/system_font_discovery` feature。
- `FontSource::Serif` 等语义类别同样依赖系统字体发现功能。
- 如果未启用该 feature，语义类别和族名查找将回退到默认字体。

### 1.2 Variable Font Properties

`TextFont` 新增 `weight`、`width`、`style` 字段，支持可变字体属性，无需为每种字重/宽度加载单独的字体文件：

```rust
TextFont {
    font: FontSource::SansSerif,
    weight: FontWeight::BOLD,
    style: FontStyle::Italic,
    width: FontWidth::CONDENSED,
    ..default()
}
```

#### FontWeight

字重值范围 1–1000，与 CSS `font-weight` 对齐：

| 常量 | 数值 | 说明 |
|------|------|------|
| `THIN` | 100 | 极细 |
| `EXTRA_LIGHT` | 200 | 特轻 |
| `LIGHT` | 300 | 轻 |
| `NORMAL` | 400 | 常规（默认） |
| `MEDIUM` | 500 | 中等 |
| `SEMI_BOLD` | 600 | 半粗 |
| `BOLD` | 700 | 粗体 |
| `EXTRA_BOLD` | 800 | 特粗 |
| `BLACK` | 900 | 极粗 |

#### FontStyle

| 值 | 说明 |
|----|------|
| `Normal` | 正常（默认） |
| `Italic` | 斜体 |
| `Oblique` | 倾斜 |

#### FontWidth

字宽值范围 50%–200%，与 CSS `font-stretch` 对齐：

| 常量 | 百分比 | 说明 |
|------|--------|------|
| `ULTRA_CONDENSED` | 50% | 极窄 |
| `EXTRA_CONDENSED` | 62.5% | 特窄 |
| `CONDENSED` | 75% | 窄 |
| `SEMI_CONDENSED` | 87.5% | 半窄 |
| `NORMAL` | 100% | 常规（默认） |
| `SEMI_EXPANDED` | 112.5% | 半宽 |
| `EXPANDED` | 125% | 宽 |
| `EXTRA_EXPANDED` | 150% | 特宽 |
| `ULTRA_EXPANDED` | 200% | 极宽 |

#### 对项目的意义

- 减少字体资产数量：一个可变字体文件可覆盖多种字重/宽度组合。
- 战棋 UI 中常见的"标题粗体 + 正文常规"可用同一字体文件实现，只需切换 `weight`。

### 1.3 Responsive Font Sizing

`font_size` 从 `f32` 变为 `FontSize` 枚举，支持响应式字体大小：

```rust
TextFont::from_font_size(FontSize::Px(24.0))   // 固定像素
TextFont::from_font_size(FontSize::Vh(5.0))    // 视口高度 5%
TextFont::from_font_size(FontSize::Rem(1.5))   // 相对于 RemSize
```

#### 变体一览

| 变体 | 说明 | 计算基准 |
|------|------|----------|
| `Px(f32)` | 固定像素值 | 物理像素 |
| `Vw(f32)` | 视口宽度百分比 | 主窗口宽度 |
| `Vh(f32)` | 视口高度百分比 | 主窗口高度 |
| `VMin(f32)` | 视口较小边百分比 | min(宽, 高) |
| `VMax(f32)` | 视口较大边百分比 | max(宽, 高) |
| `Rem(f32)` | 相对于 RemSize | RemSize Resource 值 |

#### RemSize — 全局缩放旋钮

`RemSize` 是一个 Resource，`FontSize::Rem` 值会随它缩放。修改 `RemSize` 即可一键控制所有使用 Rem 单位的文本大小：

```rust
fn adjust_text_scale(mut rem_size: ResMut<RemSize>) {
    rem_size.0 = 20.0; // 默认 16.0，调大则所有 Rem 文本变大
}
```

#### 对项目的意义

- `FontSize::Rem`：战棋 UI 中大量文本（血量、状态名、对话）可统一用 Rem 单位，一个旋钮控制全局缩放。
- `FontSize::Vh/Vw`：标题或全屏 HUD 元素可根据窗口大小自适应。
- `FontSize::Px`：需要精确像素控制的场景（如像素风字体）仍可使用。

### 1.4 LetterSpacing

新增 `LetterSpacing` 组件，控制字符间距：

```rust
commands.spawn((
    Text::new("SPACED OUT"),
    LetterSpacing::Px(4.0),
));
```

- 正值增大间距，负值缩小间距。
- 目前仅支持 `Px` 单位。

#### 对项目的意义

- 战斗伤害数字：增大间距让数字更醒目。
- 标题文字：适当加大字间距提升可读性。
- 紧凑布局：负值间距可在有限空间内塞入更多文字。

### 1.5 文本引擎迁移

Bevy 0.19 将底层文本引擎从 `cosmic_text` 迁移到 `parley`：

| 对比项 | cosmic_text | parley |
|--------|-------------|--------|
| 文档质量 | 较差 | 更好 |
| 易用性 | 一般 | 更易用 |
| 维护活跃度 | 一般 | 更活跃 |
| Bevy 集成 | 旧方案 | 官方推荐 |

这是内部实现变更，上层 API 不受影响。迁移后文本布局行为可能存在细微差异，需要视觉回归测试。

---

## 2. EditableText 文本输入

### 2.1 基本用法

`EditableText` 是 Bevy 0.19 新增的文本输入组件，可直接 spawn 使用：

```rust
commands.spawn((
    Node { width: px(200), border: px(2).all(), padding: px(8).all(), ..default() },
    BorderColor::from(Color::WHITE),
    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
    EditableText::default(),
    TextFont { font_size: FontSize::Px(24.0), ..default() },
    TextCursorStyle::default(),
));
```

### 2.2 功能列表

| 功能 | 说明 |
|------|------|
| 键盘输入 | 基本字符输入 |
| 光标导航 | 方向键 / Home / End / Ctrl+箭头 |
| 选择 | Shift+箭头、点击拖拽 |
| 多击 | 双击选词、三击选行 |
| 退格/删除 | 单字符和词级删除 |
| 剪贴板 | 复制/粘贴/剪切（需 `system_clipboard` feature） |
| Unicode 感知 | 导航和编辑按 Unicode grapheme cluster |
| 双向文本 | 支持 RTL/LTR 混排 |
| IME 支持 | CJK 等输入法 |
| 多行支持 | 换行与多行编辑 |
| 水平滚动 | 长文本水平滚动 |
| 输入过滤 | `EditableTextFilter` 限制可输入字符 |
| 聚焦全选 | `SelectAllOnFocus` 聚焦时自动全选 |
| 最大字符限制 | `max_characters` 限制输入长度 |

### 2.3 读取提交

通过 `InputFocus` Resource 和 `ButtonInput<KeyCode>` 监听回车提交：

```rust
fn on_submit(
    input_focus: Res<InputFocus>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inputs: Query<&mut EditableText>,
) {
    if keyboard.just_pressed(KeyCode::Enter)
        && let Some(entity) = input_focus.get()
        && let Ok(mut input) = inputs.get_mut(entity)
    {
        println!("Submitted: {}", input.value());
        input.clear();
    }
}
```

### 2.4 FeathersTextInput

如果使用 Feathers UI，有预构建的 `FeathersTextInput`，自动处理以下细节：

- 聚焦环（Focus ring）
- 光标/选择颜色跟随 `UiTheme`
- 鼠标悬停文本光标
- `TabIndex` 键盘导航

```rust
bsn! {
    :FeathersTextInputContainer
    Children [
        (:FeathersTextInput { @max_characters: 20usize }, MyMarker, on(on_text_change))
    ]
}
```

> **注意**：`FeathersTextInput` 是 BSN-only 的，需要启用 BSN 相关 feature。

---

## 3. Feathers Widget 更新

### 3.1 新增 Widget

Bevy 0.19 为 Feathers 新增了以下 Widget：

| Widget | 说明 |
|--------|------|
| Text input | 文本输入框 |
| Number input | 数字输入框 |
| Dropdown menu button | 下拉菜单按钮 |
| Menu divider | 菜单分割线 |
| Disclosure toggle | 折叠/展开切换（chevron 箭头） |
| Icon | 图标显示原语 |
| Label | 文本标签显示原语 |
| Pane | 装饰性面板框架 |
| Subpane | 子面板框架 |
| Group | 分组框架 |

### 3.2 Feathers + BSN

- 旧 Widget（button / checkbox / slider）已有 `bsn!` 定义，原 Rust API 已废弃。
- 新 Widget 从一开始就是 **BSN-only**，不再提供传统 Rust API。

### 3.3 对项目的意义

Feathers 是 Bevy 的 **Editor Widget 标准库**，定位类似 Unity Editor UI / Unreal Slate：

- **如果未来做编辑器**（地图编辑器 / 技能编辑器 / 数据调试面板），优先关注 Feathers。
- **当前阶段不需要关注**：SRPG 游戏本体 UI 不依赖 Feathers Widget。

---

## 4. 对 SRPG 项目的迁移建议

### 4.1 立即可用

| 特性 | 用途 |
|------|------|
| `FontSource` | 按名称/语义类别选择字体，不再需要硬编码资产路径 |
| `FontSize::Rem` | 响应式字体大小，方便全局缩放 |
| `LetterSpacing` | 战斗数字/标题文字间距调整 |

### 4.2 逐步采用

| 特性 | 用途 |
|------|------|
| Variable Font Properties | 减少字体资产数量，一个可变字体覆盖多种字重 |
| `EditableText` | 角色命名、搜索框等文本输入场景 |

### 4.3 暂不采用

| 特性 | 原因 |
|------|------|
| Feathers Widget | 项目不做编辑器，暂不需要 |
| BSN UI | 等第二阶段 UI 层试点 |

### 4.4 font_size 迁移注意

`font_size` 从 `f32` 变为 `FontSize` 枚举，这是**破坏性变更**：

```rust
// 0.18 — f32
TextFont { font_size: 24.0, ..default() }

// 0.19 — FontSize 枚举
TextFont { font_size: FontSize::Px(24.0), ..default() }
```

项目中所有使用 `TextFont` 的地方都需要更新此字段。建议迁移时统一替换：

1. 全局搜索 `font_size:` 并逐一替换为 `font_size: FontSize::Px(...)`。
2. 考虑将频繁使用的尺寸值提取为常量或工具函数。
3. 评估哪些场景适合改用 `FontSize::Rem` 以获得响应式缩放能力。

---

## 5. 注意事项

| 事项 | 说明 |
|------|------|
| `FontSource::Family` 需要 feature | 需启用 `bevy/system_font_discovery` feature 才能按名称查找系统字体 |
| 视口单位基准 | `Text2d` 的视口单位（Vw/Vh/VMin/VMax）基于主窗口，不是 render target |
| `LetterSpacing` 仅支持 Px | 目前仅支持像素单位，不支持 Rem 等响应式单位 |
| `EditableText` 功能尚不完整 | 无 placeholder / undo-redo / password masking，需等待后续版本 |
| `FeathersTextInput` 是 BSN-only | 需要 BSN feature，不提供传统 Rust API |
| 文本引擎迁移 | 从 cosmic_text 到 parley，上层 API 不变但布局行为可能有细微差异 |
| `FontSource` 语义类别回退 | 未启用 `system_font_discovery` 时，语义类别会回退到默认字体 |
