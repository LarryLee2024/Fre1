---
id: 01-architecture.tools-architecture
title: Tools Architecture
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - layer
---

# Tools Architecture — 开发工具链架构

Version: 1.0
Status: Proposed
Source: `docs/其他/33遗漏2.md` B10-B12

本文档定义 SRPG 项目的开发工具链架构。Tools 是第七层架构，仅在开发期间使用，永不发布到玩家手中。

交叉引用：
- `docs/01-architecture/00-overview/project-structure.md` — tools/ 目录结构参考
- `docs/01-architecture/00-overview/layer-contracts.md` — 第七层 Tools 定义
- `docs/01-architecture/save_migration_rules.md` — 存档迁移（区别于内容迁移）
- `docs/01-architecture/content_migration_design.md` — 内容格式迁移

---

## 1. Tools 哲学

### 1.1 核心定位

🟥 **Tools 是开发者工具链，永不进入发布构建，永不包含在 Release 版本中。**

Tools 的存在理由：当内容量增长到数百甚至数千条技能、Buff、任务时，人工审查配置错误变得不可能。Tools 是长期内容密集型开发的杠杆。

### 1.2 设计原则

| 原则 | 说明 | 宪法依据 |
|------|------|---------|
| 开发者专属 | Tools 面向开发者，不面向玩家 | §1.1.4 逻辑与表现分离 |
| 永不发布 | Release 构建完全排除 Tools 代码 | §1.1.4 |
| 只读优先 | Tools 分析数据，不修改游戏数据 | §11.7.1 读路径无副作用 |
| CI 可用 | 所有 Tools 必须支持 headless 模式用于 CI | §20.7.1 CI 门禁标准 |
| 独立二进制 | 每个 Tool 是独立的 Cargo binary，按需编译 | §1.5.1 复杂度优先 |
| 按需实现 | 🟥 禁止提前实现所有 Tools，仅在内容量增长到人工审查不可行时启动 | §1.1.7 只解决当前复杂度 |

---

## 2. Tool 分类

### 2.1 data_validator（数据验证器）

**目的**：自动检查内容数据的正确性，防止配置错误在内容量增长后变得不可追踪。

**检查项**：

| 检查类型 | 说明 | 严重级别 |
|---------|------|---------|
| SkillId 重复 | 同一 SkillId 定义了多个技能 | Error |
| Buff 不存在 | 技能引用了未定义的 BuffId | Error |
| Quest 引用链断裂 | 任务前置/后续引用了不存在的任务 | Error |
| 文本缺失 | 缺少必填的描述/名称字段 | Error |
| 循环依赖 | Buff 之间或技能之间存在循环引用 | Error |
| 图标缺失 | 引用了不存在的图标路径 | Warning |

**集成方式**：
- 单元测试集成：`content/tests/` 中的测试用例
- 交互式运行：`cargo run --bin data_validator`
- CI 集成：`cargo run --bin data_validator -- --check`

**使用场景**：当内容量超过 1000 条技能后，人工检查配置错误变得不可能。data_validator 是内容质量的第一道防线。

### 2.2 content_linter（内容 Lint）

**目的**：类似 Rust Clippy 对代码的检查，content_linter 对游戏内容进行规范性检查。

**检查项**：

| 检查类型 | 说明 | 严重级别 |
|---------|------|---------|
| 缺少技能描述 | 技能没有描述文本 | Warning |
| 缺少图标 | 技能/物品没有对应图标 | Warning |
| 缺少翻译 | 缺少多语言翻译条目 | Warning |
| 未使用 Buff | Buff 已定义但未被任何技能/物品引用 | Info |
| 孤立物品 | 物品已定义但未被任何商店/掉落表引用 | Info |

**严重级别**：

| 级别 | 语义 | CI 行为 |
|------|------|--------|
| Error | 阻塞构建 | CI 必须失败 |
| Warning | 建议修正 | CI 可选失败 |
| Info | 仅供参考 | CI 不失败 |

**CI 集成**：

```bash
# 检查模式（CI 使用）
cargo run --bin content_linter -- --check

# 详细输出模式（开发者使用）
cargo run --bin content_linter -- --verbose
```

### 2.3 balance_checker（平衡性分析）

**目的**：批量模拟战斗，分析数值平衡性。

**功能**：

| 功能 | 说明 |
|------|------|
| 伤害分布统计 | 分析伤害值的分布情况（均值、中位数、方差） |
| 胜率计算 | 模拟多场战斗计算双方胜率 |
| 回合时长分析 | 统计战斗回合数分布 |
| 多版本配置对比 | 对比不同配置版本的平衡性变化 |

**集成方式**：
- 独立工具：`tools/balance_checker/`
- 接口：读取 content/ 目录的 RON 配置，输出统计报告

### 2.4 replay_inspector（回放查看器）

**目的**：检查回放文件，逐帧回放战斗过程，用于调试和验证。

**功能**：

| 功能 | 说明 |
|------|------|
| 逐帧导航 | 前进/后退一帧，跳转到指定帧 |
| 状态哈希验证 | 验证每帧状态哈希与记录是否一致 |
| 差异高亮 | 标记回放中的不一致之处 |

### 2.5 save_inspector（存档查看器）

**目的**：检查存档文件，验证数据完整性，辅助调试存档相关问题。

**功能**：

| 功能 | 说明 |
|------|------|
| 存档格式版本检查 | 检查存档版本是否在支持范围内 |
| 数据结构浏览 | 以树形结构展示存档内容 |
| 迁移测试 | 对指定存档执行迁移链，验证迁移结果 |

### 2.6 schedule_dumper（调度图可视化）

> **优化来源**：`docs/其他/70.md` — 调度图可视化导出（Mermaid/DOT 格式 + 执行时间热力图）

**目的**：导出 Bevy 调度依赖图为可视化格式，帮助主程一眼看出哪些 SystemSet 被意外阻塞。

**功能**：

| 功能 | 说明 |
|------|------|
| DOT 格式导出 | 生成 Graphviz 兼容的依赖图 |
| Mermaid 格式导出 | 可直接嵌入文档或 Wiki |
| 执行时间热力图 | 结合 Tracy Profiler 数据标注每个 Set 的耗时 |
| 串行检测 | 自动检测并行度为 1 的 SystemSet 并告警 |

**使用场景**：

```bash
# 导出调度依赖图
cargo run --bin schedule_dumper -- --format mermaid > schedule_graph.md

# 导出带热力图的 DOT 文件
cargo run --bin schedule_dumper -- --format dot --profiler-data tracy.json > schedule_heatmap.dot

# CI 中自动检测串行瓶颈
cargo run --bin schedule_dumper -- --check-parallelism
```

### 2.7 config_diff_analyzer（配置影响面分析）

> **优化来源**：`docs/其他/70.md` — 基于 Git Diff 的配置影响面分析

**目的**：当策划修改了一个底层配置（如"燃烧"Buff），自动分析这个修改会影响到哪些技能，建议运行哪些回归测试。

**功能**：

| 功能 | 说明 |
|------|------|
| Git Diff 解析 | 提取本次变更的 RON 文件列表 |
| 引用关系图 | 解析所有 RON 文件的引用关系，构建有向依赖图 |
| 影响面报告 | 输出"修改了 buff_burn.ron，影响技能 34 个" |
| 测试建议 | 自动建议运行受影响的 balance_workbench 用例 |

**工作流程**：

```
git diff HEAD~1 -- content/
    ↓
提取变更的 RON 文件列表
    ↓
查询引用关系图（哪些技能引用了变更的 Buff）
    ↓
输出影响面报告
    ↓
自动触发受影响的回归测试
```

**使用场景**：

```bash
# 分析当前变更的影响面
cargo run --bin config_diff_analyzer -- --base HEAD~1

# 输出示例：
# 📋 配置变更影响面分析
# ─────────────────────────────────────
# 变更文件：content/buffs/burn.ron
# 影响技能：34 个（火球术、火焰风暴、燃烧之触...）
# 影响 Buff：8 个（燃烧、烈焰护盾...）
# 建议测试：balance_workbench --skill fireball --skill flame_storm
# ─────────────────────────────────────
```

### 2.8 content_editor / map_editor / dialogue_editor（未来）

**目的**：可视化编辑器，降低内容创作门槛。

| 编辑器 | 用途 |
|--------|------|
| content_editor | 可视化编辑技能、Buff、物品等配置 |
| map_editor | 可视化编辑关卡地图 |
| dialogue_editor | 可视化编辑对话树 |

---

## 3. 目录结构

```
tools/
├── data_validator/           # 数据验证器
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── content_linter/           # 内容 Lint
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── balance_checker/          # 平衡性分析
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── replay_inspector/         # 回放查看器
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── save_inspector/           # 存档查看器
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── schedule_dumper/          # 调度图可视化导出
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── config_diff_analyzer/     # 配置影响面分析
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── content_editor/           # 内容编辑器（future）
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── map_editor/               # 地图编辑器（future）
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
├── dialogue_editor/          # 对话编辑器（future）
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
└── migration_tool/           # 数据迁移工具
    ├── Cargo.toml
    └── src/
        └── main.rs
```

---

## 4. 构建配置

> **优化来源**：`docs/其他/70.md` — CLI 工具 Workspace 隔离（独立 Cargo Workspace，避免每次运行编译整个 Bevy）

### 4.1 Feature Gate

所有 Tools 通过 feature gate 控制编译：

```toml
# 项目根 Cargo.toml
[features]
default = []
tools = ["data_validator", "content_linter", "balance_checker"]

[dependencies]
data_validator = { path = "tools/data_validator", optional = true }
content_linter = { path = "tools/content_linter", optional = true }
balance_checker = { path = "tools/balance_checker", optional = true }
```

### 4.2 独立 Cargo Workspace（关键设计）

🟥 **CLI 工具绝对不能依赖 bevy 主 crate，否则每次运行都要重新编译整个 Bevy（几分钟）。必须放在独立的 Cargo Workspace member 中。**

```toml
# 工作空间根 Cargo.toml
[workspace]
members = [
    ".",           # 主游戏 crate
    "tools/data_validator",
    "tools/content_linter",
    "tools/balance_checker",
    "tools/replay_inspector",
    "tools/save_inspector",
    "tools/schedule_dumper",
]

[workspace.dependencies]
# CLI 工具只依赖轻量级库，不依赖 bevy
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"
clap = { version = "4.0", features = ["derive"] }
# 领域核心库（shared crate，无 Bevy 依赖）
shared = { path = "src/shared" }
```

```toml
# tools/data_validator/Cargo.toml
[package]
name = "data_validator"

[dependencies]
# ✅ 只依赖轻量级库 + shared crate
serde = { workspace = true }
ron = { workspace = true }
clap = { workspace = true }
shared = { workspace = true }  # 领域核心库（无 Bevy）

# 🟥 禁止依赖 bevy 主 crate
# bevy = "0.18"  // ❌ 这会导致每次运行编译整个引擎
```

编译时间对比：

| 方案 | 首次编译 | 增量编译 | 运行时间 |
|------|---------|---------|---------|
| 依赖 bevy 主 crate | 3-5 分钟 | 30-60 秒 | 每次 30-60 秒 |
| 独立 Workspace | 10-20 秒 | < 5 秒 | 每次 < 5 秒 |

### 4.3 编译时隔离

Tools 代码通过条件编译隔离：

```rust
// 仅在开发模式或 tools feature 启用时编译
#[cfg(any(debug_assertions, feature = "tools"))]
mod tools_integration {
    // 工具与游戏的集成代码
}
```

### 4.3 Release 构建排除

🟥 **Release 构建必须完全排除 Tools 代码**：

```bash
# Release 构建 — 不包含 Tools
cargo build --release

# 开发构建 — 包含 Tools
cargo build --features tools
```

### 4.4 独立编译

每个 Tool 可以独立编译运行：

```bash
# 独立运行某个工具
cargo run --bin data_validator
cargo run --bin content_linter -- --check
cargo run --bin balance_checker
```

---

## 5. CI 集成

### 5.1 必须 Headless

🟥 **CI 环境禁止使用交互式工具**。所有 Tool 必须支持 headless 模式。

### 5.2 CI 流水线

```yaml
# .github/workflows/ci.yml
- name: Content Validation
  run: cargo run --bin data_validator -- --check

- name: Content Lint
  run: cargo run --bin content_linter -- --check

- name: Balance Check
  run: cargo run --bin balance_checker -- --ci
```

### 5.3 测试覆盖

Tools 的核心逻辑必须有单元测试：

```rust
// tools/data_validator/src/validators/skill_id.rs
#[cfg(test)]
mod tests {
    #[test]
    fn detects_duplicate_skill_ids() {
        // ...
    }
    
    #[test]
    fn detects_missing_buff_reference() {
        // ...
    }
}
```

---

## 6. Tool 与游戏的边界

### 6.1 权限模型

| 操作 | 允许 | 禁止 |
|------|------|------|
| 读取 Registry | ✅ | |
| 读取 World 状态 | ✅ | |
| 读取内容文件 | ✅ | |
| 修改游戏数据 | | 🟥 |
| 修改 Registry | | 🟥 |
| 修改 Entity | | 🟥 |

### 6.2 数据流向

```
content/ (RON 文件) → Tools (读取分析) → 报告/输出
                                      ↛ 不写回 content/
                                      ↛ 不修改 Registry
```

---

## 7. 运行时调试工具

> **优化来源**：`docs/其他/70.md` — 帧率毛刺自动捕获 + bevy_inspector_egui 集成限制

### 7.1 帧率毛刺自动捕获（Frame Spike Catcher）

**目的**：游戏平时跑 60 帧，偶尔掉到 15 帧，很难复现和定位。此工具自动捕获帧率毛刺。

**实现原理**：

```rust
// debug/frame_spike_catcher.rs
use bevy::prelude::*;

#[derive(Resource)]
pub struct FrameSpikeCatcher {
    threshold_ms: f32,
    tracy_enabled: bool,
}

impl Default for FrameSpikeCatcher {
    fn default() -> Self {
        Self {
            threshold_ms: 16.67, // 60fps 的一帧时间
            tracy_enabled: cfg!(feature = "tracy"),
        }
    }
}

fn detect_frame_spike(
    time: Res<Time>,
    mut spike: ResMut<FrameSpikeCatcher>,
    mut frame_count: Local<u64>,
) {
    *frame_count += 1;
    let delta_ms = time.delta_secs_f32() * 1000.0;

    if delta_ms > spike.threshold_ms {
        // 帧率毛刺！自动记录快照
        eprintln!(
            "⚠️ Frame spike detected: {:.1}ms (threshold: {:.1}ms) at frame {}",
            delta_ms, spike.threshold_ms, *frame_count
        );

        // 如果启用了 Tracy，自动标记 span
        #[cfg(feature = "tracy")]
        if spike.tracy_enabled {
            tracy_client::frame_mark(); // 标记当前帧
        }

        // 自动保存 ECS World 状态快照到磁盘
        save_world_snapshot(*frame_count, delta_ms);
    }
}

fn save_world_snapshot(frame: u64, delta_ms: f32) {
    let snapshot_dir = std::path::Path::new("debug/spikes");
    std::fs::create_dir_all(snapshot_dir).ok();

    let filename = format!("frame_{}_{}ms.ron", frame, delta_ms as u32);
    // 序列化关键 ECS 状态到文件
    // 开发者可以用 replay_inspector 加载分析
}
```

**CI 集成**：

```yaml
# .github/workflows/performance-monitoring.yml
- name: Frame Spike Detection
  run: |
    # 运行 1000 帧的压力测试
    cargo run --features dev -- --headless --frames 1000 --check-spikes
    
    # 检查是否有毛刺记录
    if ls debug/spikes/*.ron 1> /dev/null 2>&1; then
      echo "⚠️ Frame spikes detected! See debug/spikes/"
      exit 1
    fi
```

### 7.2 bevy_inspector_egui 集成限制

> **优化来源**: `docs/其他/74借鉴.md` §10 — Godot Inspector → bevy-inspector-egui 对应关系

**跨引擎类比**：Godot Inspector 是开发期实时查看/修改节点属性的必备工具。Bevy 的对应方案是 `bevy-inspector-egui`，开发期必备，但必须限制 Reflect 范围。

**目的**：运行时调试 UI，支持 Entity 列表、Component 实时修改、System 执行耗时图表。

🟥 **`#[derive(Reflect)]` on all types causes huge compilation time increase. 必须严格限制 Reflect 的使用范围。**

为了让 egui 能够反射和修改组件，必须给所有 Component 加上 `#[derive(Reflect)]`。在大型项目中，这会导致：
- 编译时间呈指数级增长（每个 Reflect 类型增加 100-500ms 编译时间）
- 二进制体积膨胀（Reflect 元数据占用大量空间）
- 增量编译失效（任何 Reflect 类型变更触发全量重编译）

**解决方案**：

```rust
// ✅ 正确：只在 Debug 构建中启用 Reflect
#[cfg(debug_assertions)]
#[derive(Reflect, Component)]
pub struct Attributes {
    pub max_hp: i32,
    pub current_hp: i32,
    pub attack: i32,
    pub defense: i32,
}

// ✅ 更好：使用条件编译隔离 Reflect
#[derive(Component)]
pub struct Attributes {
    pub max_hp: i32,
    pub current_hp: i32,
    pub attack: i32,
    pub defense: i32,
}

// 只在 Debug 插件中注册需要反射的类型
impl Plugin for DebugInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Attributes>();
        app.register_type::<Unit>();
        // 只注册需要调试的核心类型，不注册全部
    }
}
```

**性能影响对比**：

| 方案 | 编译时间 | 二进制体积 | 运行时性能 |
|------|---------|-----------|-----------|
| 全量 Reflect | +30-60 秒 | +20-50MB | 无影响 |
| 仅 Debug Reflect | +5-10 秒 | +5-10MB | 无影响 |
| 无 Reflect | 基准 | 基准 | 无影响 |

**帧率侵蚀防护**：

```rust
// egui 的 immediate mode 渲染会吃掉主线程时间
// 必须将 Debug UI 放在独立 Schedule 中，支持一键禁用
app.add_systems(PostUpdate, (
    inspector_ui_system,
).run_if(in_state(AppState::InGame).and_then(|config: Res<DebugConfig>| config.enabled)));
```

---

## 7. 禁止事项

🟥 **以下禁止项基于宪法相关条款**：

- 🟥 **禁止 Tools 出现在 Release 构建中** — Tools 代码必须通过 feature gate 完全排除（宪法 §1.1.4 逻辑与表现分离）
- 🟥 **禁止 Tools 直接修改游戏数据** — Tools 只读分析，不修改任何内容（宪法 §11.7.1 读路径无副作用）
- 🟥 **禁止 CI 依赖交互式工具** — 所有 Tool 必须支持 headless 模式（宪法 §20.7.1 CI 门禁标准）
- 🟥 **禁止 Tools 包含业务逻辑** — Tools 只做检查和分析，不包含游戏规则（宪法 §1.1.4 逻辑与表现分离）
- 🟥 **禁止 Tools 依赖 Core 层** — Tools 通过读取 RON 文件和 Registry 获取数据，不依赖 Core 规则（宪法 §1.3.2 依赖方向铁则）
- 🟥 **禁止未通过 feature gate 的 Tools 代码** — 所有 Tools 代码必须在 `#[cfg(feature = "tools")]` 或 `#[cfg(debug_assertions)]` 保护下

---

## 8. 实现路线

| 阶段 | 工具 | 触发条件 |
|------|------|---------|
| Phase 1 | data_validator | 内容量超过 100 条技能 |
| Phase 2 | content_linter | 内容量超过 500 条技能 |
| Phase 3 | balance_checker | 开始数值调优 |
| Phase 4 | replay_inspector | 回放系统完成后 |
| Phase 5 | save_inspector | 存档系统完成后 |
| Phase 6 | schedule_dumper | SystemSet 超过 20 个 |
| Phase 7 | config_diff_analyzer | 内容量超过 300 条技能 |
| Future | content_editor / map_editor / dialogue_editor | 内容团队扩大 |

---

## 9. Excel 导入管线（Content Pipeline）

> **优化来源**：`docs/其他/74借鉴.md` §24 — 很多独立开发者后期最痛苦的是没有 Content Pipeline
>
> ⚠️ **宪法 §1.1.7 提醒**：Excel 导入管线属于未实现的未来能力。以下设计为预留参考，**禁止在内容量未增长到人工审查不可行时提前实现完整管线**。Phase 1 仅实现 data_validator，导入管线在明确需要时再启动。

### 9.1 为什么需要 Excel 导入管线

独立开发者后期最痛苦的问题：策划（或自己）在 Excel 里维护了大量技能/Buff/角色数据，但没有自动化管线将 Excel 数据导入游戏。每次修改都要手动转为 RON 文件，容易出错且无法校验引用完整性。

暴雪等大厂的成熟管线：`Excel → Validator → JSON/RON → Game`。独立开发者也应尽早建立这条管线。

### 9.2 管线架构

```
Excel/CSV（策划编辑）
    ↓  [Excel Parser]
结构化数据（JSON 中间格式）
    ↓  [Validator — 引用完整性/数值合法性/格式校验]
校验通过的数据
    ↓  [Converter]
RON 配置文件
    ↓  [AssetServer 加载]
游戏 Registry
```

### 9.3 各阶段职责

| 阶段 | 工具 | 输入 | 输出 | 说明 |
|------|------|------|------|------|
| **Excel Parser** | `tools/excel_importer/` | `.xlsx`/`.csv` | JSON | 解析 Excel 表格为结构化数据 |
| **Validator** | `tools/data_validator/`（复用） | JSON | 校验报告 | 引用完整性、数值范围、格式合规 |
| **Converter** | `tools/excel_importer/` | 校验通过的 JSON | `.ron` | 转换为游戏配置格式 |
| **Importer** | `tools/excel_importer/` | `.ron` + 变更日志 | 增量更新 | 只更新变更部分，减少热重载范围 |

### 9.4 Excel 表格规范

```csv
# skills.csv（技能表）
id,name,selector,effect_type,effect_value,buff_ref,mp_cost,cooldown,tags
fireball,火球术,EnemySingle,Damage,120,,30,3,"magic,fire,aoe"
heal,治疗术,AllySingle,Heal,100,,20,2,"magic,holy"
poison_arrow,中毒箭,EnemySingle,Damage,80,poison,15,2,"physical,ranged"
```

**表格规范**：
- 第一行为列名（英文 ID）
- 第二行为中文注释（可选，导入时忽略）
- 所有引用字段（如 `buff_ref`）必须指向已定义的 ID
- Tags 列使用逗号分隔

### 9.5 管线 CLI

```bash
# 从 Excel 导入所有内容
cargo run --bin excel_importer -- --input content/raw/ --output content/

# 只校验不导入（CI 使用）
cargo run --bin excel_importer -- --input content/raw/ --validate-only

# 增量导入（只处理变更的 Excel 文件）
cargo run --bin excel_importer -- --input content/raw/ --incremental

# 生成变更报告
cargo run --bin excel_importer -- --input content/raw/ --diff-report
```

### 9.6 管线与 Tools 的关系

```
content/raw/*.xlsx    ← 策划编辑的 Excel 表格
        ↓
excel_importer        ← 解析 + 校验 + 转换
        ↓
content/*.ron         ← 游戏配置文件
        ↓
data_validator       ← CI 二次校验（双保险）
        ↓
content_linter       ← 规范性检查
        ↓
balance_checker      ← 数值平衡分析
```

> **关键原则**：Excel 导入管线是"写入方"，data_validator 是"校验方"，两者独立但互补。导入管线保证数据格式正确，validator 保证引用完整性和规则一致性。

> 交叉引用：`docs/01-architecture/03-data-config-asset/content-pipeline.md`（数据驱动架构）、`docs/01-architecture/validation_rules.md` §10（启动时校验）
