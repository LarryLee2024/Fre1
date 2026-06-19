---
id: 03-technical.localization-design
title: Localization（国际化）技术设计
status: draft
owner: architect
created: 2026-06-19
tags:
  - localization
  - i18n
  - infrastructure
  - technical-design
---

# Localization（国际化）技术设计

> **上游依据**: ADR-053-localization-architecture.md | **数据 Schema**: `docs/04-data/infrastructure/localization_schema.md` | **实施计划**: `docs/09-planning/localization-implementation-plan.md`
>
> **目标读者**: @feature-developer — 本文档指导如何编写 `src/infra/localization/` 下的代码。

---

## 1. 目录结构

`src/infra/localization/` 目录布局及每个文件的职责：

```
src/infra/localization/
├── mod.rs                          # 模块导出: pub use plugin::LocalizationPlugin;
├── plugin.rs                       # LocalizationPlugin — Bevy Plugin 注册点
├── database.rs                     # LocalizationDatabase — ECS Resource，核心文本数据库
├── loader.rs                       # 从 assets/localization/{locale}/*.ftl 加载文本
├── cache.rs                        # LocalizedTextCache — 运行时解析文本缓存
├── components.rs                   # LocalizedText Component — UI 系统消费
├── validator.rs                    # 启动时校验 System
├── audit.rs                        # 运行时审计（orphan key 检测、覆盖率报告）
├── error.rs                        # LocError 错误类型定义
├── generated/
│   └── keys.rs (auto-gen)          # build.rs 自动生成，不出现在 git 中？
│                                   # 建议: 提交 generated/keys.rs 到版本控制，
│                                   # 避免开发环境没有 build.rs 工具链时编译失败
└── test.rs                         # 单元测试与集成测试
```

### 文件依赖关系（从底向上）

```
error.rs                   — 零依赖，仅 Rust 标准库
generated/keys.rs          — 依赖 build.rs 输出，纯常量定义

database.rs                — 依赖 error.rs, generated/keys.rs
loader.rs                  — 依赖 database.rs, error.rs
cache.rs                   — 依赖 database.rs, error.rs
components.rs              — 依赖 generated/keys.rs

validator.rs               — 依赖 database.rs, generated/keys.rs, error.rs
audit.rs                   — 依赖 database.rs, generated/keys.rs, error.rs

plugin.rs                  — 依赖以上所有，组装 Bevy Plugin
test.rs                    — 依赖 plugin.rs（通过集成测试）
```

### 模块导出 (`mod.rs`)

```rust
mod plugin;
mod database;
mod loader;
mod cache;
mod components;
mod validator;
mod audit;
mod error;
pub mod generated {
    include!("generated/keys.rs");
}

pub use plugin::LocalizationPlugin;
pub use database::LocalizationDatabase;
pub use components::LocalizedText;
pub use error::LocError;
```

---

## 2. Fluent 集成方案

### 2.1 crate 选择与版本

**推荐**: 直接解析 .ftl 关键字段（轻量方案），而非完整引入 `fluent-rs` crate。

| 方案 | 优点 | 缺点 |
|------|------|------|
| **A: 完整 fluent-rs** | 完整 Fluent 语法支持（复数、性别等） | 编译时间增加，API 复杂度高，Bevy 0.19 兼容性未验证 |
| **B: 轻量解析（推荐）** | 零额外依赖，编译快，满足当前需求 | 不支持复数规则（可后续升级） |
| **C: 自研解析器** | 完全可控 | 工作量较大，不必要的定制 |

**推荐方案 B — 轻量解析**，因为当前阶段只用到变量插值 `{$value}`，暂不需要复数/性别选择。后续可无缝升级到方案 A。

**当复数/性别成为需求时**，引入 `fluent-rs` 作为替代解析引擎，`LocalizationDatabase` 的接口不变，只需替换 `resolve()` 内部实现。

### 2.2 .ftl 文件解析规则

解析器只需支持 Fluent 的一个子集：

```ftl
### 注释行（以 ### 开头，整行忽略）

-ability-abl-000042-name = Fireball
    .desc = Deals {$damage} fire damage in radius
```

**解析产出**：`HashMap<String, (String, Vec<String>)>`，每个 entry 的 key 是 message ID（带 attribute），value 是（pattern_text, 变量名列表）。

**Message ID 映射规则**（Fluent 中 `-` 开头 → Rust 点分 key）：

| Fluent Message ID | 生成 key | 说明 |
|-------------------|----------|------|
| `-ability-abl-000042-name` | `ability.abl_000042.name` | 主 ID |
| `-ability-abl-000042-name.desc` | `ability.abl_000042.desc` | 属性 ID |

### 2.3 变量插值解析

当 `resolve()` 发现 pattern 中包含 `{$var_name}` 时：

```rust
/// 从 pattern 文本中提取变量名列表
fn extract_vars(pattern: &str) -> Vec<String> {
    // 正则: \{\$([a-zA-Z_][a-zA-Z0-9_]*)\}
    // 匹配 {$value}, {$damage}, {$target} 等
    VARS_REGEX
        .captures_iter(pattern)
        .map(|c| c[1].to_string())
        .collect()
}

/// 变量插值替换
fn interpolate(pattern: &str, params: &HashMap<&str, &str>) -> String {
    INTERPOLATE_REGEX // \{\$([a-zA-Z_][a-zA-Z0-9_]*)\}
        .replace_all(pattern, |caps: &Captures| {
            params
                .get(&caps[1])
                .copied()
                .unwrap_or(&caps[0]) // 未提供变量则保留原始 {$var}
        })
        .into_owned()
}
```

### 2.4 启动时 FTL 加载流程

```
App::new()
  .add_plugins(LocalizationPlugin)
    │
    ▼  [LocalizationPlugin::build()]
  register_type::<LocalizationDatabase>()
  init_resource::<LocalizationDatabase>()
    │
    ▼  [on_startup_system: load_locale_data]
  │
  ├── 1. 扫描 assets/localization/ 目录下所有 locale 目录
  ├── 2. 对每个 locale 目录:
  │     ├── 读取该目录下所有 *.ftl 文件
  │     └── 解析为 flat HashMap<String, Pattern>
  ├── 3. 写入 LocalizationDatabase.patterns
  ├── 4. 设置默认 locale（由 AppConfig 控制，默认 en-US）
  │
    ▼  [validation_system (Startup, after load)]
  ├── 1. 检测缺失 Key → panic
  ├── 2. 检测 Orphan Key → warn
  └── 3. 覆盖率检查 → warn if < 80%
```

### 2.5 变量名约定

所有 .ftl 文件中使用的变量名统一在 `docs/03-technical/localization-design.md` 登记（即本文档）：

| 变量名 | 来源字段 | 适用场景 |
|--------|---------|----------|
| `{$value}` | EffectDef.value / 通用数值 | 通用数值展示 |
| `{$damage}` | DamageEffectDef.damage | 伤害值 |
| `{$heal}` | HealEffectDef.value | 治疗值 |
| `{$turns}` | EffectDef.duration | 持续回合 |
| `{$count}` | 复数计数 | 数量表示 |
| `{$target}` | target.name / target.LocalizedText | 目标名称 |
| `{$source}` | source.name / source.LocalizedText | 来源名称 |
| `{$item}` | ItemDef.name_key | 物品名称 |
| `{$skill}` | AbilityDef.name_key | 技能名称 |

**禁止**在 .ftl 中自行发明变量名。新增变量名需要更新本文档。

---

## 3. LocalizationDatabase 实现

### 3.1 类型定义

```rust
use std::collections::HashMap;
use bevy::prelude::*;

/// Locale ID: 使用 2-letter 或 4-letter 代码，如 "en-US", "ja-JP", "zz-ZZ"
pub type LocaleId = String;

/// 解析后的 Pattern，含原始文本和预提取的变量名列表
#[derive(Debug, Clone)]
pub struct Pattern {
    /// 原始模式文本（带 {$var} 占位符）
    pub template: String,
    /// 从 template 中提取的变量名（按出现顺序）
    pub variables: Vec<String>,
}

/// 核心 Localization 数据库，全局唯一 ECS Resource
#[derive(Resource)]
pub struct LocalizationDatabase {
    /// 当前 locale
    current_locale: LocaleId,
    /// 按 (locale, key) 索引的原始 pattern 映射
    patterns: HashMap<LocaleId, HashMap<String, Pattern>>,
}

impl LocalizationDatabase {
    /// 创建空数据库（默认 locale = en-US）
    pub fn new() -> Self {
        Self {
            current_locale: "en-US".into(),
            patterns: HashMap::new(),
        }
    }

    /// 为指定 locale 批量插入 pattern
    pub fn load_patterns(
        &mut self,
        locale: &LocaleId,
        entries: HashMap<String, Pattern>,
    ) {
        let locale_entry = self.patterns.entry(locale.clone()).or_default();
        locale_entry.extend(entries);
    }
}
```

### 3.2 `set_locale()` 实现

```rust
impl LocalizationDatabase {
    /// 切换当前语言，同时触发缓存清理事件
    pub fn set_locale(
        &mut self,
        locale: LocaleId,
        commands: &mut Commands,
    ) {
        self.current_locale = locale.clone();
        // 发出缓存清理事件，通知所有 LocalizedTextCache 失效
        commands.trigger(LocaleChangedEvent(locale));
    }

    /// 获取当前 locale
    pub fn current_locale(&self) -> &LocaleId {
        &self.current_locale
    }
}

/// 语言切换事件 — cache.rs 中的系统监听此事件清理缓存
#[derive(Event)]
pub struct LocaleChangedEvent(pub LocaleId);
```

**关键设计点**：
- `set_locale()` 通过 Event 通知缓存失效，而非直接操作缓存 — 保持 `database.rs` 不依赖 `cache.rs`
- `commands.trigger()` 使用 Bevy 的 Observer 模式，支持多处监听

### 3.3 `resolve()` 三级回退链

```rust
#[derive(Debug)]
pub enum LocError {
    /// Key 在所有 locale 中均不存在
    KeyNotFound {
        key: String,
        locale: LocaleId,
        fallbacks_attempted: Vec<LocaleId>,
    },
    /// 参数缺失（key 使用了 {$var} 但未提供）
    MissingParameter {
        key: String,
        missing: Vec<String>,
    },
    /// 内部错误（如 Pattern 解析异常）
    Internal(String),
}

impl std::fmt::Display for LocError { /* ... */ }
impl std::error::Error for LocError {}

impl LocalizationDatabase {
    /// 解析文本 —— 三级回退链
    ///
    /// # 回退链
    /// 1. 当前 locale → 有 pattern → 解析后返回
    /// 2. 当前 locale → 无 pattern → fallback 到 "en-US"
    /// 3. en-US → 无 pattern → 返回 raw_key 字符串（兜底）
    ///
    /// # 参数
    /// - `key`: 完整的 LocalizationKey，如 "ability.abl_000042.name"
    /// - `params`: 插值参数。无参数时传空 slice
    pub fn resolve(
        &self,
        key: &str,
        params: &[(&str, &str)],
    ) -> Result<String, LocError> {
        // Step 1: 尝试当前 locale
        let current = self.current_locale.as_str();
        if let Some(pattern) = self.get_pattern(current, key) {
            return Ok(self.format_pattern(pattern, params));
        }

        // Step 2: Fallback 到 en-US
        if current != "en-US" {
            if let Some(pattern) = self.get_pattern("en-US", key) {
                return Ok(self.format_pattern(pattern, params));
            }
        }

        // Step 3: 兜底 — 返回 key 本身
        // key 本身是开发者能理解的描述性字符串
        Ok(key.to_string())
    }
}
```

**关于 step 3 的说明**：`docs/04-data/README.md` §3.2 定义的 Key 格式 `<namespace>.<ID>.<suffix>`（如 `ability.abl_000042.name`）本身是人类可读的，兜底返回 key 比 panic 或返回空字符串友好得多。但如果需要严格模式（如发布构建），可通过 feature flag 或 startup 参数开启 `strict` 模式使缺失 key 触发 panic。

### 3.4 辅助方法

```rust
impl LocalizationDatabase {
    /// 检查 key 在指定 locale 中是否存在
    pub fn has_key(&self, locale: &str, key: &str) -> bool {
        self.patterns
            .get(locale)
            .is_some_and(|m| m.contains_key(key))
    }

    /// 获取指定 locale 的所有 key
    pub fn all_keys(&self, locale: &str) -> Vec<&str> {
        self.patterns
            .get(locale)
            .map(|m| m.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default()
    }

    /// 获取当前 locale 的所有缺失 key（相对 en-US）
    pub fn missing_keys(&self) -> Vec<&str> {
        let Some(en) = self.patterns.get("en-US") else {
            return vec![];
        };
        let current = self.patterns.get(&self.current_locale);
        en.keys()
            .filter(|k| !current.is_some_and(|m| m.contains_key(*k)))
            .map(|k| k.as_str())
            .collect()
    }

    /// 覆盖率: 当前 locale 相对 en-US 的翻译完成度
    pub fn coverage(&self) -> f64 {
        let Some(en) = self.patterns.get("en-US") else {
            return 1.0;
        };
        let Some(current) = self.patterns.get(&self.current_locale) else {
            return 0.0;
        };
        if en.is_empty() {
            return 1.0;
        }
        let translated = en.keys().filter(|k| current.contains_key(*k)).count();
        translated as f64 / en.len() as f64
    }
}
```

---

## 4. LocalizedText Component

### 4.1 定义

```rust
use bevy::prelude::*;

/// UI 组件：本地化文本，携带 Key 和参数
///
/// UI 系统将此组件渲染为对应语言的最终文本。
/// 组件本身不存储翻译结果 — 这是缓存层的职责。
///
/// # 为什么用 &'static str 而非 String？
/// - Key 是编译期已知的常量（由 build.rs 生成），无需运行时分配
/// - 确保所有 key 引用都经过编译检查
#[derive(Component, Debug, Clone)]
pub struct LocalizedText {
    /// Localization Key（编译期常量，来自 generated/keys.rs）
    pub key: &'static str,
    /// Fluent 参数: (参数名, 值) 列表
    /// 参数名 &'static str = 编译期已知
    /// 参数值 String = 运行时动态构建
    pub params: Vec<(&'static str, String)>,
    /// 文本样式
    pub style: TextStyle,
}

impl LocalizedText {
    /// 创建无参数的静态文本
    pub fn static_text(key: &'static str, style: TextStyle) -> Self {
        Self {
            key,
            params: vec![],
            style,
        }
    }

    /// 创建带参数的动态文本
    pub fn with_params(
        key: &'static str,
        params: Vec<(&'static str, String)>,
        style: TextStyle,
    ) -> Self {
        Self { key, params, style }
    }
}
```

### 4.2 UI 系统使用示例

```rust
/// UI 渲染系统：监听 LocalizedText 组件变化，自动更新文本
fn render_localized_text(
    db: Res<LocalizationDatabase>,
    mut query: Query<(&LocalizedText, &mut Text), Changed<LocalizedText>>,
) {
    for (loc_text, mut text) in query.iter_mut() {
        // 将 params 转为 resolve() 接受的格式
        let params: Vec<(&str, &str)> = loc_text
            .params
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        match db.resolve(loc_text.key, &params) {
            Ok(resolved) => text.sections[0].value = resolved,
            Err(e) => {
                text.sections[0].value = format!("[LOC_ERR: {}]", e);
                warn!("Localization error: {}", e);
            }
        }
    }
}
```

**关键设计**：
- 使用 `Changed<LocalizedText>` Filter，只有 key/params 变化时才重新解析
- 不依赖 `Deref` 或 `From` 等隐式转换 — 显式调用 `resolve()`

### 4.3 在 UI 构建中使用

```rust
// 在 bundle spawn 时使用
commands.spawn((
    LocalizedText::with_params(
        loc::battle::damage_dealt,       // 编译期常量
        vec![
            ("source", source_name),
            ("target", target_name),
            ("damage", damage.to_string()),
        ],
        TextStyle {
            font_size: 16.0,
            color: Color::WHITE,
            ..default()
        },
    ),
    TextBundle {
        style: Style { ..default() },
        ..default()
    },
));
```

### 4.4 参数传递约定

| 场景 | params 构造方式 | 示例 |
|------|----------------|------|
| 静态文本（名称/描述） | `vec![]` | `LocalizedText::static_text(loc::ability::abl_000042::NAME, style)` |
| 动态文本（战斗日志） | 运行时构造 | `vec![("damage", dmg.to_string())]` |
| 嵌套 Key（物品名 + 目标名） | 预先 resolve 子项 | 外部已 resolve 为 String 再传参 |

**禁止**在 params 中传递未解析的 LocalizationKey。如果某个参数本身也是本地化文本，先 resolve 再传值。

---

## 5. build.rs Key 代码生成

### 5.1 扫描路径配置

```rust
// build.rs

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let ftl_dir = Path::new("assets/localization/en-US");

    // 确保目录存在
    assert!(
        ftl_dir.exists(),
        "assets/localization/en-US/ directory not found"
    );

    // 扫描所有 .ftl 文件
    let entries = fs::read_dir(ftl_dir)
        .expect("Failed to read localization directory");
    let mut keys: Vec<String> = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "ftl") {
            let content = fs::read_to_string(&path).unwrap();
            extract_keys(&content, &mut keys);
        }
    }

    // 生成 Rust 常量文件
    generate_keys_file(&keys);

    // 增量编译控制
    println!("cargo:rerun-if-changed=assets/localization/en-US/");
    println!("cargo:rerun-if-changed={}", ftl_dir.display());
}
```

### 5.2 Key 提取逻辑

从 .ftl 内容中提取所有 message ID 并转换为点分 key：

```rust
fn extract_keys(content: &str, keys: &mut Vec<String>) {
    // Fluent message ID = 以 "-" 开头（非缩进），
    // 截取到 "=" 或换行符，去除前导 "-"
    // 同时处理属性行（缩进的 .xxx = ...）
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(id) = trimmed.strip_prefix('-') {
            // 主 message ID: -ability-abl-000042-name
            if let Some(eq_pos) = id.find('=') {
                let message_id = id[..eq_pos].trim();
                let key = message_id.replace('-', ".");
                keys.push(key);
            }
        } else if trimmed.starts_with('.') {
            // 属性行: .desc = Deals {$damage} ...
            // 需与前一个 message ID 拼合
            // 但生成常量时需要独立 key: ability.abl_000042.desc
            // 此逻辑在 generate_keys_file() 中处理
        }
    }
}
```

**实际实现建议**：使用 `Regex` 一次性提取所有 `^-([^=]+)` 模式和属性行，减少逐行解析的脆弱性。

### 5.3 Rust 常量生成

```rust
fn generate_keys_file(keys: &[String]) {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = Path::new(&out_dir)
        .join("src/infra/localization/generated/keys.rs");

    let mut output = String::from(
        "// Auto-generated by build.rs — DO NOT EDIT\n"
    );
    output.push_str("// Source: assets/localization/en-US/*.ftl\n\n");
    output.push_str("#![allow(non_snake_case, non_upper_case_globals)]\n\n");
    output.push_str("pub mod loc {\n");

    // 按 namespace 分组，生成嵌套模块
    // key = "ability.abl_000042.name"
    // → loc::ability::abl_000042::NAME
    let mut namespaces: std::collections::BTreeMap<
        String, Vec<String>
    > = std::collections::BTreeMap::new();

    for key in keys {
        let parts: Vec<&str> = key.splitn(3, '.').collect();
        if parts.len() < 3 {
            // 短 key 如 "core.yes" → loc::core::YES
            // namespace = parts[0], const_name = parts[1].to_uppercase
            let namespace = parts[0].to_string();
            let const_name = parts[1].to_uppercase();
            namespaces
                .entry(namespace)
                .or_default()
                .push(format!(
                    "    pub const {}: &str = \"{}\";",
                    const_name, key
                ));
        } else {
            // 长 key 如 "ability.abl_000042.name"
            // → pub mod ability { pub mod abl_000042 { pub const NAME: &str = "..."; } }
            let namespace = parts[0].to_string();
            let sub = parts[1].to_string();
            let const_name = parts[2].to_uppercase();
            let entry = format!(
                "    pub mod {} {{\n        pub const {}: &str = \"{}\";\n    }}",
                sub, const_name, key
            );
            namespaces
                .entry(namespace)
                .or_default()
                .push(entry);
        }
    }

    for (ns, entries) in &namespaces {
        output.push_str(&format!("    pub mod {} {{\n", ns));
        for entry in entries {
            output.push_str(&format!("{}\n", entry));
        }
        output.push_str("    }\n\n");
    }

    output.push_str("}\n");

    fs::write(&out_path, &output)
        .expect("Failed to write keys.rs");
}
```

### 5.4 生成的 keys.rs 示例

```rust
// Auto-generated by build.rs — DO NOT EDIT
// Source: assets/localization/en-US/*.ftl

#![allow(non_snake_case, non_upper_case_globals)]

pub mod loc {
    pub mod core {
        pub const YES: &str = "core.yes";
        pub const NO: &str = "core.no";
        pub const CONFIRM: &str = "core.confirm";
        pub const CANCEL: &str = "core.cancel";
    }

    pub mod ability {
        pub const abl_000042: &str = "ability.abl_000042";
        pub mod abl_000042 {
            pub const NAME: &str = "ability.abl_000042.name";
            pub const DESC: &str = "ability.abl_000042.desc";
        }
        pub mod abl_000043 {
            pub const NAME: &str = "ability.abl_000043.name";
            pub const DESC: &str = "ability.abl_000043.desc";
        }
    }

    pub mod battle {
        pub const damage_dealt: &str = "battle.damage_dealt";
        pub const heal_received: &str = "battle.heal_received";
    }

    // ... 更多模块
}
```

### 5.5 增量编译控制

```rust
// build.rs —— 控制何时重新生成 keys.rs

fn main() {
    // 1. .ftl 文件内容变化时重新生成
    println!("cargo:rerun-if-changed=assets/localization/en-US/");

    // 2. 新增/删除 .ftl 文件时重新生成
    // cargo:rerun-if-changed 本身会在目录内的文件变化时触发

    // 3. 如果 build.rs 自身变了也要重新生成
    println!("cargo:rerun-if-changed=build.rs");

    // ... 生成逻辑
}
```

**注意**：`cargo:rerun-if-changed=assets/localization/en-US/` 会在该目录下任何 .ftl 文件被修改、新增、删除时触发 `build.rs` 重新运行。这是增量编译控制的推荐做法。

### 5.6 输出文件位置

| 文件 | 路径 | 是否提交到 git |
|------|------|---------------|
| `keys.rs`（自动生成） | `src/infra/localization/generated/keys.rs` | **建议提交** |
| `build.rs`（生成器代码） | `build.rs` | 是 |

**为什么提交 generated/keys.rs？**
- 如果开发者没有 `build.rs` 中使用的依赖（如 `regex`），编译会失败
- CI 流程中可以快速验证 key 完整性而不必运行 build.rs
- 保证 `git clone` 后 `cargo build` 直接可用

---

## 6. 启动校验

### 6.1 validation_system 实现

```rust
use bevy::prelude::*;

/// 启动时校验 —— 必须在所有 .ftl 加载完成后运行
///
/// 注册方式：
/// ```ignore
/// app.add_systems(Startup, validation_system.after(load_all_ftl_system));
/// ```
pub fn validation_system(
    db: Res<LocalizationDatabase>,
) {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 需要一份已生成 key 的列表。
    // 策略: 在 keys.rs 中同时生成一个 ALL_KEYS 常量数组
    // 或者在 plugin.rs 中通过 include! 收集所有 key
    let all_keys: &[&str] = loc::ALL_KEYS; // 由 build.rs 额外生成

    // ── 1. 缺失 Key ──
    // Rust 代码引用了，但 en-US 的 .ftl 中没有
    for key in all_keys {
        if !db.has_key("en-US", key) {
            errors.push(format!(
                "MISSING KEY: '{}' — referenced in code, not found in en-US .ftl files",
                key
            ));
        }
    }

    // ── 2. Orphan Key ──
    // .ftl 中存在，但没有 Rust 代码引用
    for key in db.all_keys("en-US") {
        if !all_keys.contains(&key) {
            warnings.push(format!(
                "ORPHAN KEY: '{}' — defined in .ftl but never referenced in code",
                key
            ));
        }
    }

    // ── 3. 覆盖率检查 ──
    let coverage = db.coverage();
    if coverage < 0.80 {
        warnings.push(format!(
            "LOW COVERAGE: current locale '{}' has {:.1}% translation coverage (threshold: 80%)",
            db.current_locale(),
            coverage * 100.0,
        ));
    }

    // ── 输出 ──
    for warn in &warnings {
        warn!("[Localization] {}", warn);
    }

    if !errors.is_empty() {
        error!(
            "[Localization] Validation failed with {} errors:",
            errors.len()
        );
        for err in &errors {
            error!("  {}", err);
        }
        panic!(
            "Localization validation failed: {} errors (see above). \
             This prevents startup to avoid showing untranslated text.",
            errors.len()
        );
    }

    info!(
        "[Localization] Validation passed. {} keys OK, {} warnings.",
        all_keys.len(),
        warnings.len()
    );
}
```

### 6.2 ALL_KEYS 数组生成

在 `keys.rs` 生成逻辑中追加：

```rust
fn generate_keys_file(keys: &[String]) {
    // ... 生成 loc 模块代码（同上） ...

    // 追加 ALL_KEYS 数组
    output.push_str("\n/// 所有已注册的 Key 列表（用于启动校验）\n");
    output.push_str("pub const ALL_KEYS: &[&str] = &[\n");
    for key in keys {
        output.push_str(&format!("    \"{}\",\n", key));
    }
    output.push_str("];\n");

    fs::write(&out_path, &output).unwrap();
}
```

### 6.3 校验开关

| 环境 | 校验策略 |
|------|---------|
| Debug 构建 | 全量校验：缺失 key → panic，orphan → warn |
| Release 构建 | 缺失 key → panic（不可在有缺失翻译的情况下发布） |
| Test 构建 (`#[cfg(test)]`) | 缺失 key → warn，不 panic（避免测试套件因翻译问题中断） |

```rust
fn validation_system(/* ... */) {
    // Tests: 不 panic，只 warn
    #[cfg(test)]
    if !errors.is_empty() {
        warn!("[Localization] {} validation errors (suppressed in test)", errors.len());
        return;
    }

    // 正常构建: panic on errors
    if !errors.is_empty() {
        panic!("Localization validation failed: {} errors", errors.len());
    }
}
```

---

## 7. Fake Locale (zz-ZZ)

### 7.1 原理

Fake Locale 使用拉丁扩展字符（带重音符号的 ASCII 变体）替换每个字母，使文本保持视觉长度和形状的同时明显区别于正常文本。硬编码文本因未经过 `resolve()` 不会被转换，从而暴露。

```ftl
### assets/localization/zz-ZZ/core.ftl

-core-yes = [Ýéś]
-core-no = [Ñó]
-core-confirm = [Çóñfíŕm]
-core-cancel = [Çáñçéŀ]

### assets/localization/zz-ZZ/ability.ftl

-ability-abl-000042-name = [Fíŕéḃáŀŀ]
-ability-abl-000042-desc = [Đéáŀś {$damage} fíŕé ḿáĝé íñ ŕáďíúś]
```

### 7.2 通过 Cargo feature 启用

```toml
# Cargo.toml
[features]
fake-locale = []   # 启用后 LocalizationPlugin 使用 zz-ZZ 作为默认 locale
```

```rust
// plugin.rs
#[derive(Resource)]
pub struct LocaleConfig {
    pub default_locale: LocaleId,
    pub strict_mode: bool,
}

impl LocalizationPlugin {
    pub fn new() -> Self {
        #[cfg(feature = "fake-locale")]
        let default_locale = "zz-ZZ".into();

        #[cfg(not(feature = "fake-locale"))]
        let default_locale = "en-US".into();

        Self {
            config: LocaleConfig {
                default_locale,
                strict_mode: false,
            },
        }
    }
}
```

### 7.3 检测硬编码文本的原理

运行方式：
```bash
# Fake Locale 模式启动
cargo run --features fake-locale
```

检测流程：
1. `plugin.rs` 检测到 `fake-locale` feature → 设置默认 locale 为 `zz-ZZ`
2. 所有用户可见文本通过 `LocalizedText` Component → `resolve()` → 返回 `[Fíŕéḃáŀŀ]`
3. **未使用 LocalizedText 的文本**（直接 `"Fireball"` 字符串）原样显示
4. 开发者在游戏中看到普通 ASCII 文本即发现未国际化的硬编码文本

**自动化检测**（可选，Tools 层）：
```rust
#[cfg(feature = "fake-locale")]
fn detect_hardcoded_strings(
    text_query: Query<&Text>,
) {
    for text in text_query.iter() {
        for section in &text.sections {
            // 如果在 zz-ZZ 模式下发现任意一段文本不包含 '[' 或 ']'，
            // 则可能是硬编码文本
            if !section.value.contains('[') && !section.value.contains(']') {
                warn!(
                    "POSSIBLE HARDCODED TEXT: '{}' — not wrapped by [zz-ZZ] brackets",
                    section.value
                );
            }
        }
    }
}
```

---

## 8. 热重载方案

### 8.1 架构

使用 `notify` crate 监听文件变化，在运行时重新加载变更的 .ftl 文件并刷新缓存。

**为什么不使用 Bevy Asset API？**
- ADR-053 已拒绝 Bevy Asset API，理由是当前阶段对纯文本资源开销过大
- `notify` crate 是文件系统监控的标准选择，轻量且成熟

### 8.2 Cargo.toml 配置

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
notify = { version = "6", features = ["macos_kqueue"] }  # macOS 使用 kqueue

[target.'cfg(target_arch = "wasm32")'.dependencies]
# WASM 不支持文件系统监控，临时方案: 无热重载
```

### 8.3 热重载系统

```rust
use bevy::prelude::*;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;

/// 资源: 文件系统监控器
#[derive(Resource)]
pub struct LocaleWatcher {
    watcher: RecommendedWatcher,
    receiver: mpsc::Receiver<Result<Event, notify::Error>>,
    locale_dirs: Vec<(LocaleId, std::path::PathBuf)>,
}

/// 系统: 检查文件变化并重载
fn hot_reload_system(
    watcher: NonSend<LocaleWatcher>,
    mut db: ResMut<LocalizationDatabase>,
    mut commands: Commands,
) {
    while let Ok(Ok(event)) = watcher.receiver.try_recv() {
        match event {
            Event::Modify(path) | Event::Create(path) => {
                if let Some(ext) = path.extension() {
                    if ext == "ftl" {
                        let (locale, _) = watcher
                            .locale_dirs
                            .iter()
                            .find(|(_, dir)| path.starts_with(dir))
                            .expect("File must belong to a locale directory");

                        // 重载此文件
                        let content = std::fs::read_to_string(&path)
                            .expect("Failed to read .ftl for hot-reload");
                        let patterns = parse_ftl(&content);
                        db.load_patterns(locale, patterns);

                        info!("[Localization] Hot-reloaded: {:?}", path);

                        // 刷新所有缓存
                        commands.trigger(LocaleChangedEvent(db.current_locale().clone()));
                    }
                }
            }
            _ => {}
        }
    }
}
```

### 8.4 初始化

```rust
fn setup_locale_watcher(mut commands: Commands, asset_server: Res<AssetServer>) {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            let _ = tx.send(res);
        },
        Config::default(),
    )
    .expect("Failed to create file watcher for localization");

    // 监控 assets/localization/ 下的所有 locale 子目录
    let base = std::path::Path::new("assets/localization");
    let mut locale_dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let locale_name = path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                watcher
                    .watch(&path, RecursiveMode::Recursive)
                    .expect("Failed to watch locale directory");
                locale_dirs.push((locale_name, path));
            }
        }
    }

    commands.insert_non_send_resource(LocaleWatcher {
        watcher,
        receiver: rx,
        locale_dirs,
    });
}
```

### 8.5 注意事项

| 注意点 | 说明 |
|--------|------|
| **调试构建 Only** | 热重载仅在 debug 构建中启用，release 构建移除 `setup_locale_watcher` |
| **WASM 不支持** | Web 平台用不到文件系统监控 |
| **notify macOS 特性** | macOS 上使用 `macos_kqueue` feature（或 `fsevent`），避免 Use PollWatcher |
| **线程安全** | `notify` 的回调运行在独立线程，通过 `mpsc` channel 传回主线程 |
| **文件写入锁** | 某些编辑器写入 .ftl 文件时会先清空再写入，可能触发空文件加载。在重载前检查文件内容非空 |

---

## 9. 与现有系统集成

### 9.1 与 Content 层对接

.ftl 文件作为非结构化资产加载，不经过 Bevy Asset API：

```rust
// loader.rs: 在 content 加载管线中调用
fn load_all_locales() -> HashMap<LocaleId, HashMap<String, Pattern>> {
    let base = std::path::Path::new("assets/localization");
    let mut all = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let locale_name = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let mut locale_map = HashMap::new();
            if let Ok(ftl_files) = std::fs::read_dir(&path) {
                for ftl_entry in ftl_files.flatten() {
                    let ftl_path = ftl_entry.path();
                    if ftl_path.extension().is_some_and(|e| e == "ftl") {
                        let content = std::fs::read_to_string(&ftl_path)
                            .unwrap_or_else(|e| {
                                panic!("Failed to read {:?}: {}", ftl_path, e)
                            });
                        let patterns = parse_ftl(&content);
                        locale_map.extend(patterns);
                    }
                }
            }

            all.insert(locale_name, locale_map);
        }
    }

    all
}
```

**Content Plugin 协作流程：**

```
Content Plugin 加载阶段:
  1. RegistryPlugin 初始化 ID 注册表
  2. ContentPlugin 加载 assets/config/ 下的 Def RON 文件
  3. 各领域 Plugin 加载 Def → 注册到 Registry
  ─── 至此 Def 中的 name_key / desc_key 已注册 ───
  4. LocalizationPlugin 加载 assets/localization/ 下所有 .ftl
  ─── 至此 resolve(name_key) 可用 ───
  5. UI Plugin
  ─── 至此渲染时 LocalizedText 可解析 ───
```

### 9.2 与 UI 层对接

UI 系统创建 Entity 时直接 spawn `LocalizedText` + `TextBundle`：

```rust
// ui/battle_log.rs (属于 Presentation 层)
fn spawn_battle_log_entry(
    commands: &mut Commands,
    log_entry: &BattleLogEntry,
) {
    // BattleLogEvent 只存 key + params（符合宪法 §22.1.3）
    let (key, params) = match &log_entry.kind {
        BattleLogKind::DamageDealt { source, target, value } => (
            loc::battle::damage_dealt,
            vec![
                ("source", source.clone()),
                ("target", target.clone()),
                ("damage", value.to_string()),
            ],
        ),
        // ... 更多变体
    };

    commands.spawn((
        LocalizedText::with_params(key, params, battle_log_style()),
        TextBundle { ..default() },
    ));
}
```

UI 侧只需要一个通用的 `render_localized_text` 系统（见 §4.2），无需为每个 UI 元素手写文本解析逻辑。

### 9.3 与 Event/Replay 对接

**核心约束**：宪法 §22.1.3 要求 Replay/Event 只存 Key+参数，不存翻译文本。

```rust
// 合法 ✅
#[derive(Event, Clone, Serialize, Deserialize)]
pub struct BattleLogEvent {
    pub key: &'static str,
    pub params: BTreeMap<&'static str, String>,
}

// 非法 ❌ — 不允许
#[derive(Event)]
pub struct BattleLogEvent {
    pub text: String, // 翻译文本 → 依赖当前 locale，破坏 replay 确定性
}
```

**Replay 录制**：
```rust
// replay 录制时
fn record_battle_log(
    mut events: EventReader<BattleLogEvent>,
    mut replay_writer: ResMut<ReplayWriter>,
) {
    for event in events.read() {
        replay_writer.record(ReplayFrame {
            frame_number: current_frame(),
            kind: ReplayFrameKind::LocalizedEvent {
                key: event.key,
                params: event.params.clone(),
            },
        });
    }
}
```

**Replay 回放**（在任意语言下）：
```rust
// replay 回放时
fn replay_battle_log(
    mut reader: ResMut<ReplayReader>,
    mut commands: Commands,
) {
    while let Some(frame) = reader.next() {
        if let ReplayFrameKind::LocalizedEvent { key, params } = frame.kind {
            // 使用当前 locale 解析文本
            commands.spawn((
                LocalizedText::with_params(key, params, style),
                TextBundle::default(),
            ));
        }
    }
}
```

这意味着：同一份 replay 在 en-US 回放显示英文，在 zh-CN 回放显示中文。

### 9.4 Plugin 注册位置

在 `src/app/plugin.rs` 中，LocalizationPlugin 注册在 Content Plugin 之后、UI Plugin 之前：

```rust
// app/plugin.rs 中的 Plugin 注册顺序
.add_plugins(content::ContentPlugin)              // 加载 Def 配置
.add_plugins(infra::localization::LocalizationPlugin)  // ← 在此处新增
// .add_plugins(ui::UiPlugin)                      // UI Plugin（后续会引入）
```

**为什么在这里？**
- Content Plugin 先将所有 Def 注册到 Registry，Def 中的 `name_key` 等字段此时可用
- LocalizationPlugin 加载 .ftl 文件到 `LocalizationDatabase`
- UI Plugin（后续引入）渲染时依赖 `LocalizationDatabase` 已就绪

**具体实现**：

```rust
// src/infra/localization/plugin.rs

use bevy::prelude::*;

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册类型
            .register_type::<LocalizationDatabase>()
            .register_type::<LocalizedText>()

            // 初始化资源
            .init_resource::<LocalizationDatabase>()

            // 加载系统 — 在 Startup 阶段运行，优先级 1
            .add_systems(Startup, load_all_ftl_system.in_set(StartupSet::PreStartup))

            // 校验系统 — 在 Startup 阶段运行，优先级 2（加载完成后）
            .add_systems(Startup, validation_system.after(load_all_ftl_system))

            // UI 渲染系统 — PreUpdate 阶段（在 UI 更新之前）
            .add_systems(PreUpdate, render_localized_text)
            ;

        // 热重载（仅 debug 构建）
        #[cfg(debug_assertions)]
        app
            .add_systems(Startup, setup_locale_watcher)
            .add_systems(Update, hot_reload_system);
    }
}
```

---

## 10. CI 集成

### 10.1 Key 完整性检查

在 CI 脚本（如 `ci/localization-check.sh`）中：

```bash
#!/bin/bash
# localization-check.sh
# 确保所有 .ftl 中的 key 在 build.rs 生成过程中被正确提取

set -euo pipefail

# 1. 运行 build.rs（模拟 key 生成）
cargo build --features "fake-locale" 2>&1 | tee /tmp/build.log

# 2. 检查是否有 validation error
if grep -q "Localization validation failed" /tmp/build.log; then
    echo "❌ Localization validation failed!"
    exit 1
fi

# 3. 检查 keys.rs 是否最新
if ! git diff --exit-code src/infra/localization/generated/keys.rs; then
    echo "❌ keys.rs is out of date! Run 'cargo build' to regenerate."
    exit 1
fi

echo "✅ Localization check passed"
```

### 10.2 覆盖率阈值（可选）

```yaml
# .github/workflows/localization.yml
name: Localization Check
on: [pull_request]
jobs:
  localization:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Build with localization validation
        run: cargo build 2>&1

      - name: Check key completeness
        run: bash ci/localization-check.sh

      - name: Coverage report
        run: |
          echo "Locale coverage:"
          # 运行一个简单的覆盖率脚本
          # 输出各 locale 相对 en-US 的翻译百分比
          python3 ci/localization-coverage.py
```

### 10.3 CI 流程整合建议

```
PR 提交
  │
  ├── cargo build（含 build.rs key 生成 + startup validation）
  │     └── 验证失败 → ❌ 阻止合并
  │
  ├── cargo nextest run（含 localization 相关测试）
  │     └── 测试失败 → ❌ 阻止合并
  │
  ├── localization-check.sh
  │     └── key 不一致 → ❌ 阻止合并
  │
  └── localization-coverage.py（可选）
        └── 覆盖率低于阈值 → ⚠️ 警告，不阻止合并
```

---

## 11. 性能考虑

### 11.1 LocalizedTextCache

```rust
use std::collections::HashMap;
use bevy::prelude::*;

/// 运行时解析文本缓存
///
/// # 失效策略
/// - `set_locale()` 时全量失效（清空）
/// - 单个 .ftl 热重载时相关 key 失效
///
/// # 线程安全
/// 只在主线程访问，不需要 Arc/RwLock
#[derive(Resource, Default)]
pub struct LocalizedTextCache {
    /// cache[ (locale, key, params_hash) ] = resolved_text
    /// params_hash 用于区分同一 key 不同参数的情况
    cache: HashMap<(LocaleId, String, u64), String>,
}

impl LocalizedTextCache {
    const MAX_ENTRIES: usize = 500; // 防止内存无限增长

    /// 尝试从缓存获取
    pub fn get(
        &self,
        locale: &LocaleId,
        key: &str,
        params: &[(&str, &str)],
    ) -> Option<&str> {
        let params_hash = params_hash(params);
        self.cache.get(&(locale.clone(), key.to_string(), params_hash))
            .map(|s| s.as_str())
    }

    /// 写入缓存
    pub fn set(
        &mut self,
        locale: &LocaleId,
        key: &str,
        params: &[(&str, &str)],
        resolved: String,
    ) {
        if self.cache.len() >= Self::MAX_ENTRIES {
            // 简单策略: 超限时清空一半
            // 更复杂的 LRU 策略在必要时添加
            let remove_count = Self::MAX_ENTRIES / 4;
            let keys: Vec<_> = self.cache.keys().take(remove_count).cloned().collect();
            for k in keys {
                self.cache.remove(&k);
            }
        }

        let params_hash = params_hash(params);
        self.cache.insert(
            (locale.clone(), key.to_string(), params_hash),
            resolved,
        );
    }

    /// 全量失效（locale 切换时调用）
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// 计算参数的哈希（用于区分同一 key 不同参数）
fn params_hash(params: &[(&str, &str)]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    params.len().hash(&mut hasher);
    for (k, v) in params {
        k.hash(&mut hasher);
        v.hash(&mut hasher);
    }
    hasher.finish()
}
```

### 11.2 Cache 感知的 resolve()

```rust
// 在 database.rs 中添加缓存感知版本（可选）
impl LocalizationDatabase {
    /// 带缓存的 resolve — 推荐 UI 系统使用
    pub fn resolve_cached<'a>(
        &self,
        key: &str,
        params: &[(&str, &str)],
        cache: &mut LocalizedTextCache,
    ) -> Result<String, LocError> {
        let locale = &self.current_locale;

        // 尝试缓存
        if let Some(cached) = cache.get(locale, key, params) {
            return Ok(cached.to_string());
        }

        // 缓存未命中 → 解析
        let result = self.resolve(key, params)?;

        // 写入缓存（仅缓存成功解析的结果）
        cache.set(locale, key, params, result.clone());

        Ok(result)
    }
}
```

### 11.3 Changed Filter 避免不必要重渲染

```rust
// 在 UI 渲染系统中
fn render_localized_text(
    db: Res<LocalizationDatabase>,
    mut cache: ResMut<LocalizedTextCache>,
    // 关键: Changed<LocalizedText> — 只有变化的组件才会被处理
    mut query: Query<(&LocalizedText, &mut Text), Changed<LocalizedText>>,
) {
    for (loc_text, mut text) in query.iter_mut() {
        let params: Vec<(&str, &str)> = loc_text
            .params
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();
        match db.resolve_cached(loc_text.key, &params, &mut cache) {
            Ok(resolved) => text.sections[0].value = resolved,
            Err(e) => {
                text.sections[0].value = format!("[LOC_ERR: {}]", e);
            }
        }
    }
}
```

**性能说明**：
- `Changed<LocalizedText>` 查询仅在组件实际的 key/params 发生变化时触发
- 对于静态文本（name、desc），只有在语言切换时 `set_locale()` → `LocaleChangedEvent` → 所有 UI 组件被标记为 changed 时才会重渲染
- 对于动态文本（战斗日志），每次新 Entity 被 spawn 时自然携带新的 `LocalizedText` 组件

### 11.4 Static vs Dynamic 文本不同处理策略

| 文本类型 | 特点 | 处理策略 |
|---------|------|---------|
| **Static**（名称/描述/按钮） | 构建时已知，无参或固定参 | 全量缓存，`Changed<LocalizedText>` 过滤 |
| **Dynamic**（战斗日志/对话） | 运行时生成，参数每行不同 | 逐个解析，cache 按 params_hash 命中 |
| **Bulk**（列表/表格） | 批量渲染同类文本 | 组件复用，避免重复 spawn |

```rust
// Static 文本优化: 构建时预解析
pub struct PreResolvedText {
    pub resolved: String,
    pub key: &'static str,
}

impl PreResolvedText {
    /// 在系统启动时预构建（例如在 Startup System 中）
    pub fn new(key: &'static str, db: &LocalizationDatabase) -> Self {
        let resolved = db.resolve(key, &[]).unwrap_or_else(|_| key.to_string());
        Self { resolved, key }
    }
}
```

### 11.5 Key 构建时预解析方案

对于永远不会变化的静态文本（技能名称、物品名称、UI 标签），可以在加载阶段预解析为 `PreResolvedText`，避免运行时的 `resolve()` 调用：

```rust
fn pre_resolve_static_texts(
    db: Res<LocalizationDatabase>,
    mut commands: Commands,
) {
    // 预解析所有注册的 Def 名称
    for def in ability_registry.iter() {
        let name = db.resolve(def.name_key, &[]).unwrap_or_default();
        let desc = db.resolve(def.desc_key, &[]).unwrap_or_default();
        commands.entity(def.entity).insert(PreResolvedTexts {
            name,
            desc,
        });
    }
}
```

**但注意**：语言切换后预解析值会过时。如果支持运行时切换语言，预解析就不能做。解决方案：
- 只对在 `set_locale()` 后重建的 UI（如菜单）做预解析
- 动态 UI（战斗日志、聊天框）走 resolve_cached()

---

## 12. Risky 区域

### 12.1 Fluent crate 的 Bevy 兼容性

| 风险 | 说明 | 缓解措施 |
|------|------|---------|
| fluent-rs 编译失败 | fluent-rs 依赖 `unic-langid`，可能与 Bevy 的依赖版本冲突 | 初始采用轻量解析方案（零额外依赖） |
| fluent-rs 运行时 panic | `FluentBundle` 的 `add_resource` 可能 panic | 使用 `try_new` + `Result` propagate |
| 跨平台行为差异 | 字符串处理在不同平台可能不同 | 使用 Rust 标准库 `Regex`（经过充分测试） |

**迁移路径**：
```
Light Parser (v0) ──→ fluent-rs Full (v1, need confirmed)
     │                        │
     │ 实现 resolve()         │ 替换 resolve() 内部实现
     │ 支持 {$var} 插值       │ 支持复数/性别/选择
     │ 0 额外依赖            │ 增加 ~5 个依赖
     └──────── 接口不变 ──────┘
```

### 12.2 编译时间控制

| 关注点 | 预计影响 | 缓解 |
|--------|---------|------|
| build.rs 中 Regex 编译 | 首次编译约 200ms | 使用 `lazy_static` 或 `once_cell` |
| .ftl 扫描时间 | 扫描 50 个 .ftl 约 10ms | 仅监控 `en-US` 目录（其他 locale 结构相同） |
| keys.rs 生成 | 生成 5000 key 约 50ms | 仅 .ftl 变化时重新生成（rerun-if-changed） |
| 热重载 notify crate | debug 构建额外依赖（release 排除） | `cfg(debug_assertions)` 条件编译 |

### 12.3 团队适应成本

| 成本项 | 说明 | 缓解 |
|--------|------|------|
| 新约定学习 | 团队需要适应 Key 体系而不是直接写字符串 | 编译期检查 + Fake Locale 双保险，犯错不会上线 |
| .ftl 语法 | 非技术文案人员不熟悉 Fluent | 提供 .ftl 模板文件 + 只用到 `key = text {$var}` 子集 |
| 参数传递繁琐 | `vec![("name", "value")]` 模式较啰嗦 | 提供辅助宏（见下方建议） |

**辅助宏建议**（Feature Developer 可选实现）：

```rust
/// 简化带参数的 LocalizedText 创建
macro_rules! loc_text {
    ($key:expr, $style:expr) => {
        LocalizedText::static_text($key, $style)
    };
    ($key:expr, $style:expr, $($k:ident = $v:expr),+) => {
        LocalizedText::with_params(
            $key,
            vec![$( (stringify!($k), $v.to_string()) ),+],
            $style,
        )
    };
}

// 使用
loc_text!(loc::ability::abl_000042::NAME, normal_style);
loc_text!(loc::battle::damage_dealt, battle_log_style,
    source = source_name,
    target = target_name,
    damage = damage_value,
);
```

---

## 附录 A: LocError 完整定义

```rust
/// Localization 错误类型
#[derive(Debug, Clone)]
pub enum LocError {
    /// Key 在重试所有 fallback locale 后仍未找到
    KeyNotFound {
        key: String,
        locale: LocaleId,
        fallbacks_attempted: Vec<LocaleId>,
    },

    /// 参数不匹配：pattern 需要某参数但未提供
    MissingParameter {
        key: String,
        missing: Vec<String>,
        provided: Vec<String>,
    },

    /// 内部解析错误（.ftl 语法异常）
    ParseError {
        file: Option<String>,
        line: Option<usize>,
        message: String,
    },

    /// 未分类内部错误
    Internal(String),
}

impl std::fmt::Display for LocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocError::KeyNotFound { key, locale, fallbacks_attempted } => {
                write!(
                    f,
                    "Key '{}' not found in locale '{}' (fallbacks attempted: {:?})",
                    key, locale, fallbacks_attempted
                )
            }
            LocError::MissingParameter { key, missing, provided } => {
                write!(
                    f,
                    "Key '{}' missing parameters: {:?} (provided: {:?})",
                    key, missing, provided
                )
            }
            LocError::ParseError { file, line, message } => {
                if let (Some(f), Some(l)) = (file, line) {
                    write!(f, "Parse error at {}:{}: {}", f, l, message)
                } else {
                    write!(f, "Parse error: {}", message)
                }
            }
            LocError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for LocError {}
```

---

## 附录 B: `parse_ftl()` 参考实现

```rust
use regex::Regex;
use std::collections::HashMap;

/// 解析 .ftl 内容为扁平 key → Pattern 映射
pub fn parse_ftl(content: &str) -> HashMap<String, Pattern> {
    let mut result = HashMap::new();
    let id_re = Regex::new(r"^-([a-zA-Z0-9_-]+)\s*=\s*(.*)$").unwrap();
    let attr_re = Regex::new(r"^\s+\.([a-zA-Z0-9_-]+)\s*=\s*(.*)$").unwrap();
    let var_re = Regex::new(r"\{\$([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

    let mut current_id: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // 跳过注释行（以 ### 开头）
        if trimmed.starts_with("###") || trimmed.is_empty() {
            continue;
        }

        // Message ID: -xxx-yyy = value
        if let Some(caps) = id_re.captures(trimmed) {
            let raw_id = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str().trim();
            let key = raw_id.replace('-', ".");

            let vars: Vec<String> = var_re
                .captures_iter(value)
                .map(|c| c[1].to_string())
                .collect();

            result.insert(
                key.clone(),
                Pattern {
                    template: value.to_string(),
                    variables: vars,
                },
            );

            current_id = Some(key);
        }
        // Attribute: .xxx = value
        else if let Some(caps) = attr_re.captures(trimmed) {
            if let Some(ref base_key) = current_id {
                let attr_name = caps.get(1).unwrap().as_str();
                let value = caps.get(2).unwrap().as_str().trim();
                let key = format!("{}.{}", base_key, attr_name);

                let vars: Vec<String> = var_re
                    .captures_iter(value)
                    .map(|c| c[1].to_string())
                    .collect();

                result.insert(
                    key,
                    Pattern {
                        template: value.to_string(),
                        variables: vars,
                    },
                );
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_ftl() {
        let content = r#"
### Sample file
-core-yes = Yes
-core-no = No

-ability-abl-000042-name = Fireball
    .desc = Deals {$damage} fire damage
"#;
        let map = parse_ftl(content);
        assert_eq!(map.get("core.yes").unwrap().template, "Yes");
        assert!(map.get("core.yes").unwrap().variables.is_empty());
        assert_eq!(map.get("ability.abl_000042.name").unwrap().template, "Fireball");
        assert_eq!(map.get("ability.abl_000042.desc").unwrap().template, "Deals {$damage} fire damage");
        assert_eq!(map.get("ability.abl_000042.desc").unwrap().variables, vec!["damage"]);
    }
}
```

---

## 附录 C: 实现顺序建议

以 `src/infra/localization/` 为模块根，建议按以下顺序逐一实现：

| 步骤 | 文件 | 内容 | 验证方式 |
|------|------|------|---------|
| 1 | `error.rs` | `LocError` 枚举定义 | `cargo build` |
| 2 | `database.rs` | `LocalizationDatabase` + `resolve()` + `set_locale()` | 单元测试（mock data） |
| 3 | `loader.rs` | `parse_ftl()` + `load_all_ftl_system` | 解析示例 .ftl 文件 |
| 4 | `plugin.rs`（初版） | 注册 database + loader，基础版本 | 启动后 `info!` 打印已加载 key 数 |
| 5 | `components.rs` | `LocalizedText` 组件定义 | `cargo build` |
| 6 | `plugin.rs`（完整） | 加入 `render_localized_text` | 在 UI 中显示本地化文本 |
| 7 | `build.rs` | Key 代码生成 | `keys.rs` 生成，误拼 key 编译报错 |
| 8 | `cache.rs` | `LocalizedTextCache` | 性能测试（大量重复 resolve） |
| 9 | `validator.rs` | 启动校验 | 删除一个 key → 启动 panic |
| 10 | `audit.rs` | 运行时审计 | 覆盖率报告输出 |
| 11 | 热重载 | `notify` 集成 | 修改 .ftl 无需重启生效 |
| 12 | Fake Locale | zz-ZZ 支持 | `cargo run --features fake-locale` |
| 13 | CI | localization-check.sh | PR 流水线 |

---

*本文档是 Localization 基础设施的技术实现指南。所有代码实现以此为最终依据。修改本文档需经过 @architect + @feature-developer 审核。*
