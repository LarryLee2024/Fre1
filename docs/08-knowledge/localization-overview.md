# Localization 国际化深度解析 — 从宪法到代码

> Fre 是一个需要支持中文、英文、日文、韩文的 SRPG 项目。50 万行代码里如果每个人都在代码里直接写 `"攻击"` 或 `"Attack"`，那切换语言就是一场噩梦。本文从宪法原则出发，一步步拆解 Fre 的国际化方案是如何设计、如何落地、如何在运行时工作的。

---

## 1. 为什么需要一套 Localization 体系？

假设你是一个玩家，打开游戏设置，把语言从简体中文切到英文——你希望**所有**文本瞬间变成英文，包括按钮、技能名、对话、战斗日志、错误提示。但如果代码里的一半文本是写死的：

```rust
// ❌ 看看这种写法有多普遍
commands.spawn(Text::new("结束回合"));
commands.spawn(Text::new("背包"));
let msg = format!("{} 造成了 {} 点伤害", name, damage);
```

那么语言切换时这些写死的文本会保持中文，用户体验割裂。更糟的是，对于需要支持多语言的 SRPG，翻译团队需要在分散的代码中翻找每一处文本，改错或遗漏是必然的。

Fre 的应对方案是一条铁律：**任何用户可见的文本，代码中都只存一个 Key。** 真正被看到的是什么文字，由运行时根据当前语言决定。

---

## 2. 从宪法到底层代码：五层约束

Localization 体系不是某一天拍脑袋引入的，它是从项目最高规则层层落地的结果：

```
宪法 §22 (Localization First) ── 最高原则：所有文本必须走 Key
    │
    ▼
Data Law 013/014 ── 数据架构铁律：Key 的格式和标准
    │
    ▼
ADR-053 ── 架构决策：用 Fluent (.ftl)、放在 Infra 层、代码生成
    │
    ▼
Schema ── 数据结构定义：LocalizationDatabase、LocalizedText、LocaleId
    │
    ▼
代码 ── plugin.rs、loader.rs、database.rs、validator.rs……
```

每一层都约束下一层，上层不关心下层实现细节。这就是说"宪法最高"的真正含义——如果你跳过了宪法直接写代码，很可能在某处违反规则而不自知。

---

## 3. 三个核心概念

在深入代码之前，先建立三个核心概念，理解了它们整个体系就很清楚：

### 3.1 LocalizationKey — 文本的"身份证"

每个用户可见文本对应一个**唯一的不透明 Key**，像身份证号一样。你看不到文本内容，只能看到 Key：

```
ability.abl_000042.name   →   翻译后可能是 "Fireball" 或 "火球术"
core.yes                  →   翻译后可能是 "Yes" 或 "是"
ui.battle.end_turn        →   翻译后可能是 "End Turn" 或 "结束回合"
```

为什么要用无意义的 ID（`abl_000042`）而不是语义化命名（`fireball`）？想象一下：策划把技能"火球术"改名为"烈焰爆"。如果用 `ability.fireball.name`，改名后 Key 变得毫无意义。用 `ability.abl_000042.name`，不管技能改成什么名字，Key 始终有效。

### 3.2 Locale — 语言标识

Fre 当前支持 4 种语言，用 BCP-47 标准格式标识：

| Locale | 含义 | 默认 |
|--------|------|------|
| `en-US` | 美式英语 | **标准模板**——所有 Key 必须先在这定义 |
| `zh-CN` | 简体中文 | 运行时默认 |
| `ja-JP` | 日语 | 可选 |
| `ko-KR` | 韩语 | 可选 |

Locales 作为编译期枚举定义在 `src/shared/localization_key.rs`：

```rust
pub enum Locale {
    ZhCn,  // BCP-47: "zh-CN"
    EnUs,  // BCP-47: "en-US"
    JaJp,  // BCP-47: "ja-JP"
    KoKr,  // BCP-47: "ko-KR"
}
```

默认语言是 `zh-CN`（因为产品面向中文玩家），但设计上 `en-US` 是**标准模板**——所有 Key 必须先在英文 .ftl 文件中定义，其他语言可以缺失（缺失时会回退到英文）。

### 3.3 Fluent (.ftl) — Mozilla 设计的本地化格式

Fre 用 `.ftl` 文件存储翻译文本。这是 Mozilla 推出的 ICU 消息格式替代方案，比 JSON/YAML 强的地方在于：

```ftl
### 文件：ability.ftl
-ability-abl-000042-name = Fireball
    .desc = Deals {$damage} fire damage to all targets in radius
```

- `-ability-abl-000042-name` 是消息 ID，对应 LocalizationKey `ability.abl_000042.name`
- `.desc` 是属性（attribute），对应 Key `ability.abl_000042.name.desc`
- `{$damage}` 是变量插值——运行时传入实际数值

用 Fluent 而不是 JSON 的原因是 Fluent 原生支持变量插值、复数规则、性别选择，这些都是本地化中常见的需求。

---

## 4. 架构归属：为什么 Localization 是 Infrastructure 层？

这个问题在设计阶段是经过争论的。有人觉得 UI 应该管翻译，有人觉得翻译属于内容配置，但最终 ADR-053 确认：

**Localization 是 Infrastructure 层（L2），不属于 UI，不属于 Capabilities，不属于任何业务域。**

理由很简单：
1. 游戏里**所有地方**都用本地化文本：UI、战斗日志、技能描述、对话、错误消息、成就……放在任何一个业务域都会导致依赖方向违规
2. 它不包含任何游戏规则——把 Fluent 换成 JSON 或其他格式，游戏玩法不变
3. 它被 UI 层、Domain 层、Content 层同时依赖，天然是基础设施

对应的代码位置：`src/infra/localization/`

```
infra/localization/
├── mod.rs              # 模块入口，公开 LocalizedText、LocalizationDatabase
├── plugin.rs           # LocalizationPlugin —— 注册到 Bevy App
├── database.rs         # LocalizationDatabase —— 核心数据库 ECS Resource
├── loader.rs           # .ftl 文件加载器和解析器
├── cache.rs            # LocalizedTextCache —— 运行时缓存
├── components.rs       # LocalizedText Component —— UI 文本组件
├── validator.rs        # 启动校验 —— 缺失 Key → panic
├── audit.rs            # 运行时审计 —— 覆盖率报告
├── error.rs            # 错误类型（KeyNotFound、MissingParameter）
└── generated/
    └── keys.rs         # 自动生成（build.rs）：loc::ability::ABL_000042_NAME
```

---

## 5. 数据流：一个文本从配置到屏幕的全过程

假设游戏要显示"结束回合"按钮。这个文本完整生命周期如下：

### Step 1: .ftl 文件定义

文件 `assets/localization/en-US/ui.ftl` 中有：

```ftl
-ui-battle-end-turn = End Turn
```

### Step 2: build.rs 代码生成

每次构建时，build.rs 扫描 `assets/localization/en-US/*.ftl`，自动生成 Rust 常量文件 `generated/keys.rs`：

```rust
// 自动生成的部分
pub mod loc {
    pub mod ui {
        pub const BATTLE_END_TURN: &str = "ui.battle.end.turn";
    }
}

pub const ALL_KEYS: &[&str] = &[
    "ui.battle.end.turn",
    // ……所有其他 key
];
```

这提供了**编译期检查**——如果你在代码里写 `loc::ui::BATTLE_END_TURN`，但 .ftl 中删除了这个 key，build.rs 会重新生成常量，编译器会报"未找到常量"。错误在编译阶段就被捕获，不会留到运行时才显示 ??? 文本。

### Step 3: 启动时 FTL 加载

`LocalizationPlugin::build()` 注册了 `load_all_ftl_system` 作为 PreStartup System：

```rust
fn load_all_ftl_system(mut db: ResMut<LocalizationDatabase>) {
    // 扫描 assets/localization/ 下所有 locale 目录
    // 读取每个目录下的所有 .ftl 文件
    // 解析为 key → Pattern 映射
    // 写入 LocalizationDatabase
}
```

`parse_ftl()` 函数解析简单的 .ftl 格式（消息 ID + 属性 + 变量提取），转成 `HashMap<String, Pattern>` 存入数据库。目前用的是简化解析器（直接字符串替换 `{$var}`），后续可升级到完整的 `fluent-rs` 库。

### Step 4: 启动时 Key 校验

加载完成后，`validation_system` (Startup System) 启动校验：

- **缺失 Key**：`ALL_KEYS` 中的 key 在 en-US .ftl 中不存在 → **panic，阻止启动**。这意味着任何新增的 Key 如果在 .ftl 中漏定义了，游戏根本跑不起来。
- **Orphan Key**：.ftl 中存在但 `ALL_KEYS` 中没有 → 输出 **WARN**。这是提醒可能有废弃的 Key 需要清理。
- **覆盖率低**：当前 locale 对 en-US 的翻译覆盖率低于 80% → 输出 **WARN**。

### Step 5: UI 使用 LocalizedText

UI 代码中，用 `LocalizedText` 组件声明文本：

```rust
commands.spawn((
    LocalizedText::static_text(loc::ui::BATTLE_END_TURN),
    ButtonBundle { /* …… */ },
));
```

这里的关键是**代码里完全没有"End Turn"或"结束回合"字样**，只有一个编译期常量。

### Step 6: 运行时渲染

`render_localized_text` 系统（PreUpdate System）监测 `Changed<LocalizedText>` 组件，自动解析并更新 Text：

```rust
fn render_localized_text(
    db: Res<LocalizationDatabase>,
    mut cache: ResMut<LocalizedTextCache>,
    mut query: Query<(&LocalizedText, &mut Text), Changed<LocalizedText>>,
) {
    for (loc_text, mut text) in query.iter_mut() {
        let params: Vec<(&str, &str)> = /* 转换参数格式 */;
        match db.resolve_cached(loc_text.key, &params, &mut cache) {
            Ok(resolved) => text.0 = resolved,
            Err(e) => text.0 = format!("[LOC_ERR: {}]", e),
        }
    }
}
```

`resolve_cached` 先查缓存，未命中才走 `resolve`，然后写入缓存。

### Step 7: 三级回退

```rust
pub fn resolve(&self, key: &str, params: &[(&str, &str)]) -> Result<String, LocError> {
    // 1. 当前 locale → 有 pattern → 返回
    if let Some(pattern) = self.get_pattern(current, key) {
        return Ok(self.format_pattern(pattern, params));
    }
    // 2. Fallback 到 en-US
    if current != "en-US" && let Some(pattern) = self.get_pattern("en-US", key) {
        return Ok(self.format_pattern(pattern, params));
    }
    // 3. 兜底 — 返回 key 本身
    Ok(key.to_string())
}
```

三级回退链：
```
当前语言（如 zh-CN） → 有翻译    → 显示翻译文本
                  → 没翻译    → 降级到 en-US
en-US              → 有翻译    → 显示英文
                  → 没翻译    → 显示 Key 本身（兜底）
```

这意味着 zh-CN 或 ja-JP 可以只完成部分翻译。如果某个技能还没翻译成中文，玩家看到的是英文而非乱码。如果团队连英文都忘了写，至少开发者能通过 Key 值（如 `ability.abl_000042.name`）推断出问题。

---

## 6. 缓存系统：LocalizedTextCache

UI 文本每帧都需要渲染，每次都去解析 .ftl 模式是不现实的。`LocalizedTextCache` 做了三件事：

1. **缓存已解析文本**：以 `(locale, key, params_hash)` 为键，存储最终渲染字符串
2. **容量控制**：最多 500 条，超限时驱逐 1/4 最早的条目（防止无限内存增长）
3. **自动失效**：`detect_locale_change_and_clear_cache` 系统检测 `LocalizationDatabase.current_locale` 的变化，一旦语言切换就清空整个缓存

---

## 7. Fake Locale：捕获"漏网之鱼"

总有些硬编码文本藏在代码深处。Fake Locale 就是用来捕捉它们的：

```
# 启动方式
cargo run --features fake-locale
```

这会启用 `zz-ZZ` locale，所有通过 LocalizationKey 走的文本显示为伪翻译：

```ftl
### zz-ZZ/core.ftl
-core-yes = [Ýéś]
-core-no = [Ñó]
-ability-abl-000042-name = [Fírébáll]
```

**效果**：已国际化的文本变成 `[Fírébáll]` 这种形式，一眼能认出来。没走 Key 的硬编码文本（如直接 `Text::new("攻击")`）保持原样，在伪翻译中暴露无遗。

测试团队拿到 Fake Locale 构建，扫一遍所有界面，任何没被 `[]` 包着的文本都是有问题的。

---

## 8. 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                       开发期                                       │
│                                                                  │
│  assets/localization/en-US/*.ftl                                 │
│    │                                                             │
│    ├──→ build.rs 扫描 → generated/keys.rs (编译期常量)            │
│    │                  → 代码中用 loc::ability::XXX 引用            │
│    │                                                             │
│    └──→ PreStartup: load_all_ftl_system → LocalizationDatabase   │
│                                                                  │
│  Startup: validation_system (缺失 Key → panic)                   │
│                                                                  │
│  Debug 构建: create_locale_watcher + hot_reload_system           │
│    (.ftl 文件变化时自动重载)                                       │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                       运行期                                       │
│                                                                  │
│  UI Widget 创建:                                                 │
│    commands.spawn(LocalizedText { key: loc::ui::BATTLE_END })    │
│                                                                  │
│  PreUpdate: render_localized_text                               │
│    │                                                             │
│    ├──→ LocalizationDatabase.resolve(key)                       │
│    │      ├── 当前 locale? → 返回                                │
│    │      ├── en-US? → 返回 (fallback)                          │
│    │      └── raw key → 返回 (兜底)                             │
│    │                                                             │
│    └──→ LocalizedTextCache.get_or_insert(locale, key, params)   │
│           │                                                      │
│           ▼                                                      │
│    Text 组件更新 (text.0 = resolved)                            │
│                                                                  │
│  语言切换:                                                      │
│    db.set_locale("zh-CN") → detect_locale_change 检测变化        │
│    → cache.clear() → 下一帧所有 LocalizedText 自动重新渲染       │
│                                                                  │
│  运行时审计 (每 5 分钟):                                         │
│    audit_system → 输出覆盖率报告                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. UI 层的接入方式

Presentation 层（`docs/06-ui/`）定义了 UiTextKey 枚举和 LocalizedText Widget：

```rust
/// UiTextKey 是 UI 层对 LocalizationKey 的二次封装
pub struct UiTextKey(pub &'static str);

impl UiTextKey {
    pub const END_TURN: Self = Self(loc::ui::BATTLE_END_TURN);
    pub const HP: Self = Self(loc::ui::HP);
    // ...
}
```

ViewModel 只存 Key，不存翻译文本——这是宪法级别的约束：

```rust
// ✅ 正确：ViewModel 存 Key
struct BattleScreenVm {
    end_turn_key: UiTextKey,
    hp_label_key: UiTextKey,
}

// ❌ 错误：ViewModel 存翻译文本
struct BattleScreenVm {
    end_turn_text: String,   // 翻成中文后存了"结束回合"
}
```

Widget 通过 `LocalizedText` 组件直接使用 Key，不需要手动调用翻译：

```rust
// ✅ 正确
commands.spawn(LocalizedText::static_text(loc::ui::BATTLE_END_TURN));

// ❌ 错误
commands.spawn(Text::new("结束回合"));
```

---

## 10. 热重载（Debug 构建）

正在开发时，如果你修改了 .ftl 文件，不需要重启游戏。`hot_reload_system` 使用 `notify` crate 监控 `assets/localization/` 目录：

1. 检测到 .ftl 文件变化（修改或新增）
2. 解析变化的文件内容
3. 更新 `LocalizationDatabase` 中对应 locale 的 pattern
4. 清空 `LocalizedTextCache`
5. 下一帧所有 UI 文本自动刷新

这个机制让翻译调整的迭代速度非常快。

---

## 11. 当前的实现状态

Fre 的 Localization 体系部分已经实现，部分还是设计方案：

| 组件 | 状态 | 说明 |
|------|------|------|
| `LocalizationKey` 类型定义 | ✅ 已实现 | `src/shared/localization_key.rs` — 强类型包装器 |
| `LocalizationDatabase` | ✅ 已实现 | 三级回退链、set_locale、resolve/resolve_cached |
| `LocalizedTextCache` | ✅ 已实现 | 最大 500 条、locale 切换时清空 |
| `LocalizedText` Component | ✅ 已实现 | key + params，自动渲染系统 |
| loader（FTL 解析） | ✅ 已实现 | 简化版解析（`{$var}` 替换），非完整 fluent-rs |
| 启动校验 | ✅ 已实现 | 缺失 Key → panic，Orphan → warn |
| 运行时审计 | ✅ 已实现 | 每 5 分钟输出覆盖率报告 |
| 热重载 | ✅ 已实现 | debug 构建 + 非 wasm 平台 |
| Fake Locale (zz-ZZ) | ✅ 已实现 | feature-gated |
| `fake-locale` cargo feature | ✅ 已实现 | 启用时 default_locale 切换为 zz-ZZ |
| build.rs Key 代码生成 | ✅ 已实现 | 扫描 .ftl → 生成 `generated/keys.rs` |
| en-US .ftl 文件 | ✅ 已实现 | 8 个文件覆盖 core/ui/ability/buff/item/quest/tutorial/gameplay |
| zz-ZZ .ftl 文件 | ⚠️ 仅 core.ftl | 其余 namespace 待提供伪翻译 |
| zh-CN .ftl 翻译 | ❌ 未实现 | 计划中 |
| ja-JP / ko-KR .ftl 翻译 | ❌ 未实现 | 计划中 |
| 完整 `fluent-rs` 集成 | ⏸️ 待定 | 当前用简化解析器，未来可升级 |
| Mod 覆盖链 | ❌ 未实现 | 设计已完成，等待 Mod 系统 |
| 文本长度预算校验 | ❌ 未实现 | 设计已完成 |
| 覆盖率 CI 检查 | ❌ 未实现 | 校验规则已定义 |

---

## 12. 常见问题

### Q: 为什么 Key 不包含语言信息？
`core.yes_zh` 是反模式。语言是运行时维度，不是 Key 的维度。Key 应该只标识"哪个文本"，不关心"哪种语言"。同一个 Key 在所有 locale 中共享，locale 切换由数据库层处理。

### Q: 新增一个文本需要几步？
1. 在 `assets/localization/en-US/` 对应 .ftl 中添加消息
2. rebuild → build.rs 自动生成常量
3. 在代码中用 `loc::xxx::XXX` 引用

如果漏了第 1 步，启动时 `validation_system` 会 panic 告诉你。如果漏了第 3 步，启动时会有 Orphan Key 警告。

### Q: 可以只在某些 locale 定义 Key 吗？
可以。如果一个 Key 只在 en-US 中定义，zh-CN 和 ja-JP 会回退到英文。但缺失过多会导致覆盖率低于 80%，启动时会输出 WARN。

### Q: 为什么不把翻译结果存入存档？
因为存档必须与语言无关。如果玩家用中文存档，用日文读档，存档里存的如果是中国话，日文环境下就无法显示。所以存档只存 `current_locale` 这个枚举值，语言切换由运行时的 `LocalizationDatabase` 处理。

### Q: 现在的简化解析器够用吗？
当前仅支持 `{$var}` 变量替换，不支持 Fluent 的复数规则、性别选择、选择表达式。对于 Fre 当前阶段够用了，后续需要时可以升级到 `fluent-rs` 完整解析，接口不变。


*本文覆盖了从宪法 §22 到 `src/infra/localization/` 每一行代码的完整链路。Architecture 层面的规范详见 `docs/01-architecture/40-cross-cutting/ADR-053-localization-architecture.md`，数据 Schema 详见 `docs/04-data/infrastructure/localization_schema.md`。*
