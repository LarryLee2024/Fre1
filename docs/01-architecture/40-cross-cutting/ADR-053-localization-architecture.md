---
id: 01-architecture.40-cross-cutting.ADR-053
title: "ADR-053: Localization 基础设施架构（Fluent + Key 代码生成 + 三级回退）"
status: Accepted
owner: architect
created: 2026-06-19
tags:
  - architecture
  - localization
  - i18n
  - infrastructure
  - cross-cutting
---

# ADR-053: Localization 基础设施架构（Fluent + Key 代码生成 + 三级回退）

## 状态

Proposed

## 背景

宪法 §22 已新增 Localization 专项规则（v5.1），定义国际化基本原则。数据架构（`docs/04-data/`）中所有 30+ Schema 已使用 `name_key: LocalizationKey` 字段，但：

- 无 `LocalizationPlugin` 注册到 Bevy 应用
- 无 `LocalizationKey` 类型定义与代码生成器
- 无 `assets/localization/` 目录结构与 .ftl 文件
- 无启动时 Key 完整性校验
- 无 Fake Locale 检测机制
- 无运行时缓存与热加载能力

50 万行级 SRPG 项目需要一套完整的、与现有 DDD 三层+横切四层架构兼容的 Localization 基础设施。

## 引用的领域规则

- `docs/00-governance/ai-constitution-complete.md` §1.5(7) — P0 Localization First
- `docs/00-governance/ai-constitution-complete.md` §22 — Localization 专项规则（共 22 条）
- `docs/04-data/README.md` §3.2 — Localization Key 格式规范
- `docs/04-data/infrastructure/localization_schema.md` — Localization 数据 Schema

## 决策

### 1. Locallization 定位与归属

Localization 属于 **Infrastructure 层 (L2)**，不属于 UI 层，不属于 Capabilities 能力层。

**理由：**
- UI、BattleLog、Quest、Dialogue、ErrorMessage、Analytics 均使用 Localization
- 它是全局基础设施，不包含任何游戏规则逻辑
- 符合「边界判断：游戏规则不变的前提下，能不能换一种技术实现方式？」→ 能（从 Fluent 换为其他格式不改变游戏规则）

**依赖关系：**
```
Infrastructure/localization ──→ Core（读取 Def 中的 LocalizationKey）
                              ──→ Shared（使用基础数据类型的强类型 ID）
Content（加载 .ftl 资产）
Tools（覆盖率报告工具）
```

### 2. 组件架构（Sublayer 结构）

代码按六层子目录组织，每层有明确职责，依赖方向从下到上：

```
infra/localization/
├── mod.rs, plugin.rs           # 入口：模块导出 + Bevy Plugin 注册
├──
├── foundation/                 # L0: 零业务语义的纯类型
│   ├── error.rs                #   LocError 枚举（thiserror）
│   ├── locale_id.rs            #   LocaleId 枚举（EnUS/ZhCN/JaJP/ZzZZ）
│   └── pattern.rs              #   Pattern 结构体（template + variables）
├── storage/                    # L1: ECS Resource 存储
│   ├── database.rs             #   LocalizationDatabase（三级回退链）
│   └── cache.rs                #   LocalizedTextCache（运行时缓存）
├── io/                         # L2: 文件 IO 与解析
│   ├── parser.rs               #   parse_ftl() 解析器
│   ├── loader.rs               #   .ftl 文件加载系统
│   └── watcher.rs              #   热重载文件监控（debug only）
├── ui/                         # L3: 表现层
│   ├── components.rs           #   LocalizedText Component
│   └── render.rs               #   render_localized_text 系统
├── facade/                     # L4: 跨层编排（统一入口）
│   └── resolve.rs              #   resolve_cached 封装
├── validation/                 # L5: 校验与审计
│   ├── validator.rs            #   启动时 Key 完整性校验
│   └── audit.rs                #   运行时覆盖率审计
└──
    generated/
        └── keys.rs             #   build.rs 自动生成，扁平常量
```

**层间依赖规则**（从下到上单向依赖）：
- `foundation/` → 零依赖（仅 Rust 标准库 + thiserror）
- `storage/` → `foundation/`
- `io/` → `foundation/` + `storage/`
- `ui/` → `foundation/` + `storage/`
- `facade/` → `storage/` + `ui/`
- `validation/` → `storage/`
- `plugin.rs` 组装所有层

### 3. LocalizationDatabase 接口设计

```rust
/// 核心 Localization 数据库，全局唯一（ECS Resource）
struct LocalizationDatabase {
    /// 当前 locale
    current_locale: LocaleId,
    /// 按 (locale, key) 索引的原始 pattern 映射
    patterns: HashMap<LocaleId, HashMap<String, FluentPattern>>,
    /// fallback 链
    fallback_chain: Vec<LocaleId>,
}

impl LocalizationDatabase {
    /// 设置当前语言
    fn set_locale(&mut self, locale: LocaleId);

    /// 解析文本（带参数插值）
    fn resolve(&self, key: &str, params: &[Param]) -> Result<String, LocError>;

    /// 获取当前 locale 的所有缺失 Key
    fn missing_keys(&self) -> Vec<&str>;
}
```

### 4. LocalizedText Component 设计

```rust
/// UI 组件：本地化文本，携带 Key 和参数
/// UI 系统读取此组件后自动渲染为对应语言文本
#[derive(Component)]
struct LocalizedText {
    key: &'static str,              // 编译期已知的 LocalizationKey
    params: Vec<(&'static str, String)>,  // Fluent 参数
    style: TextStyle,
}
```

使用方式：
```rust
// UI 构建时
commands.spawn((
    LocalizedText {
        key: loc::battle::damage_dealt,
        params: vec![("value", damage.to_string()), ("target", target_name)],
        style: default_text_style(),
    },
    NodeBundle { ... },
));
```

### 5. Key 代码生成策略

build.rs 扫描 `assets/localization/en-US/*.ftl` 生成 Rust 常量模块：

```rust
// 自动生成: generated/keys.rs
pub mod loc {
    pub mod core {
        pub const YES: &str = "core.yes";
        pub const NO: &str = "core.no";
    }
    pub mod ability {
        pub mod abl_000042 {
            pub const NAME: &str = "ability.abl_000042.name";
            pub const DESC: &str = "ability.abl_000042.desc";
        }
    }
    // ...
}
```

优势：
- 编译期检查：误拼 Key（`loc::ability::abl_0000043::NAME`）→ 编译错误
- IDE 自动补全
- 重构安全：Key 重命名时所有引用处编译报错

### 6. Plugin 注册位置

```rust
// Phase 8: Infrastructure
.add_plugins(infra::registry::RegistryPlugin)
.add_plugins(infra::pipeline::PipelinePlugin)
.add_plugins(infra::replay::ReplayPlugin)
.add_plugins(infra::save::SavePlugin)
.add_plugins(infra::input::InputPlugin)
.add_plugins(infra::logging::LoggingPlugin)
.add_plugins(infra::localization::LocalizationPlugin)  // ← 新增
```

LocalizationPlugin 必须在 Content Plugin 之后（等 .ftl 资产加载完毕），在 UI Plugin 之前（确保 UI 渲染时文本就绪）。

### 7. Fluent 集成

采用 `fluent-rs` 生态：

```rust
use fluent::{FluentBundle, FluentResource, FluentMessage};

// 加载 .ftl 内容
let res = FluentResource::try_new(ftl_content)?;
let mut bundle = FluentBundle::new(vec![locale]);
bundle.add_resource(res)?;

// 解析消息
let msg = bundle.get_message("ability-abl-000042-name")
    .expect("Message exists");
let pattern = msg.value()
    .expect("Message has value");
let mut errs = vec![];
let text = bundle.format_pattern(pattern, Some(&args), &mut errs);
```

变量约定（见 `docs/03-technical/localization-design.md`）：
- `{$value}` — 通用数值
- `{$damage}` — 伤害值
- `{$heal}` — 治疗值
- `{$turns}` — 持续回合
- `{$target}` — 目标名称
- `{$source}` — 来源名称

### 8. Fallback 链设计

```
resolve(key, locale=ja-JP):
  1. ja-JP 中有 key? → 返回
  2. en-US 中有 key? → 返回（Fallback 到英文）
  3. key 原始字符串 → 返回（兜底）
```

启动时检测：当前 locale 翻译覆盖率 < 阈值（默认 80%）时输出 WARN 日志。

### 9. 启动校验流程

```rust
fn validation_system(db: Res<LocalizationDatabase>, keys: Res<GeneratedKeyList>) {
    let mut errors = vec![];

    // 1. 缺失 Key：代码引用了但 .ftl 中没有
    for key in keys.all() {
        if !db.has_key("en-US", key) {
            errors.push(format!("Missing key: {}", key));
        }
    }

    // 2. 未引用 Key（orphan）：.ftl 中存在但没有 Rust 代码引用
    for key in db.all_keys("en-US") {
        if !keys.contains(key) {
            warnings.push(format!("Orphan key: {}", key));
        }
    }

    // 3. 参数不匹配（简化版：仅检查变量名一致性）
    // 需要 Fluent AST 解析，详见技术设计文档

    if !errors.is_empty() {
        panic!("Localization validation failed:\n{}", errors.join("\n"));
    }
}
```

### 10. Mod Localization 覆盖

```
加载优先级（从低到高）：
  1. Base: assets/localization/en-US/*.ftl
  2. DLC: assets/dlc/*/localization/en-US/*.ftl
  3. Mod: mods/*/localization/en-US/*.ftl

覆盖规则：
  - 同一个 Key 出现在多个层时，高优先级覆盖低优先级
  - Mod 新增 Key 自动追加到数据库
  - 启动时检测 Mod Key 冲突，输出 WARN
```

### 11. Replay / Event 集成

**现有约束**：宪法 §22.1.3 要求 Replay/Event 只存 Key+参数。

```rust
// 非法：翻译文本存储在事件中
struct BattleLogEvent {
    text: String,                          // ❌ "张三造成 100 点伤害"
}

// 合法：Key + 参数
struct BattleLogEvent {
    key: &'static str,                     // ✅ "battle.damage_dealt"
    params: Vec<(&'static str, String)>,   // ✅ [("actor", "张三"), ("value", "100")]
}
```

这与现有 ADR-041（Replay Determinism）完全一致 — Replay Frame 只存储命令 + 参数，不存在翻译文本。

### 12. Fake Locale (zz-ZZ)

```ftl
### zz-ZZ/core.ftl
-core-yes = [Ýéś]
-core-no = [Ñó]
-ability-abl-000042-name = [Fírébáll]
```

启用方式：
```
cargo run --features fake-locale
```

效果：
- 已国际化的文本：显示 `[Fírébáll]` 等伪翻译
- 硬编码文本：保持原样，直接暴露肉眼可见

### 13. 性能设计

| 场景 | 策略 |
|------|------|
| UI 每帧渲染文本 | LocalizedTextCache 缓存解析后文本，仅语言切换时失效 |
| Static 文本（name/desc） | 构建时预先解析好，无需运行时查找 |
| 动态文本（battle log） | 临时解析，通过 LocalizedTextCache 透传 |
| 批量操作（UI 列表刷新） | 使用 Changed Filter，仅重渲染变化的 LocalizedText |

## 被拒绝的选项

### Option A: 使用 JSON/YAML 作为本地化格式
**拒绝原因**：JSON/YAML 不支持变量插值、复数规则、性别选择。Fluent 是 W3C 标准的 ICU 消息格式替代方案，为本地化场景设计。

### Option B: Localization 作为 Capabilities 层的一部分
**拒绝原因**：Localization 不包含任何游戏规则，属于纯技术实现，放入 Capabilities 会污染双轴架构边界。

### Option C: 运行时动态 Key 检查（无代码生成）
**拒绝原因**：50 万行项目中运行时 Key 拼写错误是最难排查的 Bug 类别之一。编译期常量生成是防止此问题的唯一可靠手段。

### Option D: 使用 Bevy Asset API 管理 .ftl
**拒绝原因**：当前阶段 Bevy Asset API 对 .ftl 这类纯文本资源的资产管理开销过大。使用标准的 `include_str!` + 文件系统监控实现热加载更轻量。后续可考虑迁移到 Bevy Asset 系统。

## 风险

| 风险 | 缓解 |
|------|------|
| fluent-rs 库与 Bevy 0.19 兼容性 | 评估期先使用简化版直接解析 .ftl 关键字段，逐步过渡到 fluent-rs |
| build.rs 增加编译时间 | Key 仅在有 .ftl 变更时重新生成，使用 cargo:rerun-if-changed 控制 |
| 性能：Fluent 模式解析开销 | 仅热路径使用缓存，冷路径（剧情文本）延迟解析 |
| 团队对新 Key 体系适应成本 | 编译期检查 + Fake Locale 双保险 |

## 后续工作

- `docs/04-data/infrastructure/localization_schema.md` — Localization 数据 Schema
- `docs/03-technical/localization-design.md` — 技术设计细节
- `src/infra/localization/` — 代码实现
- `assets/localization/en-US/*.ftl` — 初始本地化文件
- `build.rs` — Key 代码生成器
