---
id: 10-reviews.feature-developer-infrastructure-alignment
title: Review — Shared/Infra/Cross-cutting 代码 vs 文档对齐分析
status: completed
owner: feature-developer
created: 2026-06-17
updated: 2026-06-17
tags:
  - review
  - infrastructure
  - shared
  - cross-cutting
  - feature-developer
  - code-alignment
---

# Feature Developer 视角：Shared/Infra/Cross-cutting 代码与文档对齐分析

**Reviewer**: @feature-developer  
**Scope**: `src/shared/*/` + `src/infra/*/` + `src/app/` + `src/content/` + `src/tools/` + `src/modding/`  
**Standards**: 架构总纲 §3 模块映射 + §6.1 Plugin 注册顺序

---

## Part 1: Shared (L0) — 原子层

### 1.1 文档定义的 12 个模块

| 模块 | 架构文档 §3.1 定义 | 代码状态 | 对齐 |
|------|-------------------|---------|------|
| IDs | 强类型 ID | ✅ **已实现** | 🟢 |
| Error | 错误上下文工具 | ⬜ 骨架（`// TODO`） | 🟡 |
| Math | 纯数学工具 | ⬜ 骨架（`// TODO`） | 🟡 |
| Random | 确定性随机数 | ⬜ 骨架（`// TODO`） | 🟡 |
| Time | GameTime, TurnCount | ⬜ 骨架（`// TODO`） | 🟡 |
| Collections | 通用集合扩展 | ⬜ 骨架（`// TODO`） | 🟡 |
| Hashing | 非加密高速哈希 | ⬜ 骨架（`// TODO`） | 🟡 |
| Validation | 链式校验器 | ⬜ 骨架（`// TODO`） | 🟡 |
| Testing | 测试构建工具 | ⬜ 骨架（`// TODO`） | 🟡 |
| Traits | 横切能力抽象 | ⬜ 骨架（`// TODO`） | 🟡 |
| Prelude | 统一导出 | ⬜ 骨架（`// TODO`） | 🟡 |
| Path | 路径工具 | ⬜ 骨架（`// TODO`） | 🟡 |

### 1.2 IDs 模块 — 唯一实现的核心

已实现的 ID 类型：

| ID 类型 | 用途 | 示例 |
|---------|------|------|
| `AttributeId` | 属性标识 | `attr:attr_000001` |
| `TagId` | 标签标识 | `tag:tag_000001` |
| `ModifierId` | 修改器标识 | `mod:mod_000001` |
| `EffectId` | 效果标识 | `eff:eff_000001` |
| `AbilityId` | 技能标识 | `ability:abl_000001` |
| `TriggerId` | 触发标识 | `trig:trig_000001` |
| `CueId` | 表现信号标识 | `cue:cue_000001` |
| `CharacterId` | 角色标识 | `char:char_000001` |
| `UnitId` | 单位标识 | `unit:unit_000001` |
| `EquipmentId` | 装备标识 | `equip:equip_000001` |
| `ItemId` | 物品标识 | `item:item_000001` |
| `FactionId` | 阵营标识 | `faction:faction_000001` |

与数据架构 `docs/04-data/README.md` §3.1 ID 策略对比：

| 预期 ID | 代码实现 | 状态 |
|---------|---------|------|
| `attr_` | `AttributeId ("attr")` | 🟢 对齐 |
| `tag_` | `TagId ("tag")` | 🟢 对齐 |
| `mod_` | `ModifierId ("mod")` | 🟢 对齐 |
| `eff_` | `EffectId ("eff")` | 🟢 对齐 |
| `abl_` | `AbilityId ("ability")` | 🟢 对齐 |
| `trg_` | `TriggerId ("trig")` | 🟡 prefix 不同（`trg` vs `trig`） |
| `cue_` | `CueId ("cue")` | 🟢 对齐 |
| `itm_` | `ItemId ("item")` | 🟡 prefix 不同（`itm` vs `item`） |
| `qst_` | ❌ 缺失 | 🟥 未定义 |
| `spl_` | ❌ 缺失 | 🟥 未定义 |
| `buf_` | ❌ 缺失 | 🟥 未定义 |
| `fct_` | `FactionId ("faction")` | 🟡 prefix 不同（`fct` vs `faction`） |
| `ter_` | ❌ 缺失 | 🟥 未定义 |
| `rcp_` | ❌ 缺失 | 🟥 未定义 |
| `oot_` | ❌ 缺失 | 🟥 未定义 |

### 1.3 Shared 层关键问题

| 问题 | 严重度 | 说明 |
|------|--------|------|
| 🟥 **Random 模块缺失** | **高** | `shared/random/` 只有 `// TODO`。架构 ADR-041 要求确定性 PRNG 确保回放兼容。违反 P0 铁律 #3 (Replay First) |
| 🟥 **Error 模块缺失** | **中** | 宪法禁止全局 AppError/anyhow，但 `shared/error/` 无任何实现。各领域错误枚举需通用工具支撑 |
| 🟡 **Math 模块缺失** | **中** | Tactical 领域需要网格坐标/距离计算等数学工具 |
| 🟡 **Time 模块缺失** | **中** | Effect 生命周期需要 GameTime，回放需要帧计数 |
| 🟡 **Testing 模块缺失** | **低** | `shared/testing/` 只为测试构建工具，不影响主线功能 |
| 🟡 **ID prefix 不一致** | **低** | 文档定义 `trg_`, `itm_` 等与代码实现 `trig`, `item` 不一致。不影响功能但应统一 |
| 🟡 **SharedPlugin 为空** | **低** | 无任何 Resource 注册 |

---

## Part 2: Infra (L2) — 技术实现层

### 2.1 文档定义的 5 个模块

| 模块 | 架构文档 §3.4 定义 | 数据 Schema | 代码状态 |
|------|-------------------|-------------|---------|
| registry | ID 注册、冲突检测、热重载 | `infrastructure/registry_schema.md` | ⬜ 骨架 |
| pipeline | 通用执行管线引擎 | `infrastructure/pipeline_schema.md` | ⬜ 骨架 |
| replay | 命令录制、确定性回放 | `infrastructure/replay_schema.md` | ⬜ 骨架 |
| save | 存档序列化、版本迁移 | `foundation/save_architecture.md` | ⬜ 骨架 |
| input | 输入抽象、命令层 | — | ⬜ 骨架 |

### 2.2 当前实现（全部相同模式）

```rust
// mod.rs
mod plugin;
pub use plugin::*;

// plugin.rs
use bevy::prelude::*;
pub struct <Xxx>Plugin;
impl Plugin for <Xxx>Plugin {
    fn build(&self, _app: &mut App) {
        // TODO: register components, resources, systems
    }
}
```

### 2.3 关键依赖关系

| Infra 模块 | 被哪个 Capabilities/Domains 依赖 | 阻塞程度 |
|-----------|--------------------------------|---------|
| `registry` | Content 层、所有 Definition 加载 | 🟥 **阻塞** — 无 Registry 则无法加载配置 |
| `pipeline` | Core runtime、Combat、Effect | 🟥 **阻塞** — 无管线引擎则 Effect Pipeline 无法执行 |
| `replay` | Core runtime、Combat | 🟥 **阻塞** — 违反 Replay First 原则 |
| `save` | 所有 Domain | 🟡 中优先级 — 游戏需要持久化 |
| `input` | App 层 | 🟡 中优先级 — 命令层基础 |

### 2.4 架构合规检查

| 检查项 | 结果 | 说明 |
|--------|------|------|
| Infra 模块路径与架构一致 | ✅ | 全部位于 `src/infra/` |
| Plugin 注册顺序正确 | ✅ | Phase 8 (CorePlugin 之后) |
| 依赖方向正确 | ✅ | Infra 依赖 Core + Shared（代码中未出现反向引用） |
| 架构文档引用 | ✅ | 每个 `mod.rs` 正确引用 `docs/04-data/` 对应 Schema |

---

## Part 3: Cross-cutting 横切四层

### 3.1 App — 启动装配层（横切1）

| 检查项 | 状态 | 评分 |
|--------|------|------|
| Composition Root 角色 | ✅ | 唯一知道所有层的入口 |
| Plugin Phase 0–9 顺序 | ✅ | 完全对齐架构文档 §6.1 |
| Feature-gated 启动 | ✅ | `#[cfg(feature = "dev")]` for DevToolsPlugin |
| 内容 | ✅ | 清晰的阶段注释 |

**评价**：🟢 **完全对齐** — App 层是当前代码中实现质量最高的部分之一。

### 3.2 Content — 内容桥接层（横切2）

| 检查项 | 状态 | 评分 |
|--------|------|------|
| 目录位置 | ✅ | `src/content/` |
| 依赖范围 | ✅ | Core + Infra |
| 实现 | ❌ | `build()` 为空（`// TODO`） |

**评价**：🟡 **目录对齐，实现为空** — `ContentPlugin` 未注册任何 Asset Loader、Config Watcher 或 Validation Pipeline。

### 3.3 Tools — 开发工具层（横切3）

| 检查项 | 状态 | 评分 |
|--------|------|------|
| 目录位置 | ✅ | `src/tools/` |
| Feature-gated | ✅ | `#[cfg(feature = "dev")]` |
| 实现 | ❌ | 仅 DeviceToolsPlugin 骨架 |

**评价**：🟡 **基本对齐** — 结构正确但无实质工具内容。可后续按需开发 Debug 面板、性能分析等。

### 3.4 Modding — Mod 扩展层（横切4）

| 检查项 | 状态 | 评分 |
|--------|------|------|
| 目录位置 | ✅ | `src/modding/` |
| 依赖项 | ✅ | `src/core/mod_api/` |
| 实现 | ❌ | `build()` 为空（`// TODO`） |

**评价**：🟡 **目录对齐，实现为空** — Mod 功能需要 Core 层稳定 API 后才能实现。

### 3.5 mod_api — Mod 稳定 API

| 检查项 | 状态 | 评分 |
|--------|------|------|
| 目录位置 | ✅ | `src/core/mod_api/` |
| 内容 | ❌ | 仅 `mod.rs`，无任何导出 |

**评价**：🟡 — 目前只预留了位置，尚未定义任何 Mod 可用的稳定 API。

---

## 综合优先级矩阵

基于 P0 铁则和依赖关系，建议按以下矩阵推进：

```
高影响 ┼──────────────────────────────────────────
       │                                           
       │  ① shared/random/                         
 对    │     (Replay First 铁则)                    
 后    │  ② infra/pipeline/                        
 续    │     (Effect Pipeline 前置)                
 开    │  ③ infra/replay/                          
 发    │     (Replay First + 命令录制)              
 的    │                                           
 阻    │  ④ shared/time/                           
 塞    │     (GameTime 被多处依赖)                  
 程    │  ⑤ shared/error/                          
 度    │     (领域错误枚举支撑)                     
       │  ⑥ infra/registry/                        
 低    │     (配置加载前置)                         
 影    │                                            
 响    │  ⑦ shared/math/                           
       │     (网格系统前置)                         
       └────────────────────────────────────────────
             低 实施难度 ──────→ 高
```

### 第一阶段（Phase A）：立即实施

| 模块 | 预计工作量 | 阻塞方 |
|------|-----------|--------|
| `shared/random` — `SeededRng` | 1 文件 ~50 行 | 无（纯独立） |
| `shared/time` — `GameTime` | 1 文件 ~30 行 | 无（纯独立） |
| `shared/error` — 错误上下文 | 1 文件 ~80 行 | 无（纯独立） |

### 第二阶段（Phase B）：Capabilities 管线依赖

| 模块 | 预计工作量 | 阻塞方 |
|------|-----------|--------|
| `infra/pipeline` — 通用管线引擎 | 3–5 文件 ~300 行 | Core runtime |
| `infra/replay` — 回放框架 | 3–5 文件 ~250 行 | Core runtime |
| `infra/registry` — ID 注册中心 | 2–3 文件 ~200 行 | Content 层 |

### 第三阶段（Phase C）：Domain 实现前置

| 模块 | 预计工作量 | 阻塞方 |
|------|-----------|--------|
| `shared/math` — 网格工具 | 1–2 文件 ~150 行 | Tactical domain |
| `shared/collections` — 集合扩展 | 1 文件 ~50 行 | 多个 domain |

---

## 违反红线清单汇总

| # | 红线 | 位置 | 说明 |
|---|------|------|------|
| 1 | 红线 #3 (Replay First) | `shared/random/` | 无确定性 RNG 实现，核心战斗逻辑无法保证回放兼容 |
| 2 | 红线 #38 (无上下文 TODO) | 多个 plugin.rs | `// TODO: register components, systems, events` 无 `[P0-P3][领域][日期]` 格式 |
| 3 | 红线 #9 (禁止全局 AppError) | 间接违反 | `shared/error/` 未提供错误上下文工具，Domain 层缺乏统一的错误处理基础设施 |

---

## 结论

**Shared 和 Infra 层是当前项目的最大瓶颈**。虽然 Capabilities 层的基础类型定义已经完成，但 Shared 层的基础设施（RNG、时间、错误处理）和 Infra 层的管线引擎（Pipeline、Replay、Registry）尚未实现，导致 Capabilities 和 Domains 无法在运行时有实质行为。

**App 层是唯一高质量完成的横切层**，其 Composition Root 实现和 Plugin 注册顺序可以视为项目标准。

---

*本报告由 @feature-developer 基于 `src/shared/`、`src/infra/`、横切四层源码与 `docs/` 对齐性分析生成。*
