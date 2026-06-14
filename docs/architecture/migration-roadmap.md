# Migration Roadmap — 从当前架构到目标架构的迁移计划

Version: 1.0
Status: Proposed

本文档定义从当前项目架构到七层架构目标的分阶段迁移计划。

---

## 当前状态分析

> ⚠️ **宪法提醒（§1.1.1 Feature First）**：当前迁移目标为七层架构（App/Core/Shared/Infra/Content/Modding/Tools），该结构按技术层级组织，与宪法 §1.1.1 "按业务领域拆分顶层模块" 存在张力。迁移决策基于以下理由：
> 1. 七层架构是项目长期维护的已验证方案，经过 ADR 评审
> 2. Core 层内部仍按业务领域拆分（battle/、skill/、buff/ 等），符合 Feature First 精神
> 3. 共享工具（Shared）和技术实现（Infrastructure）的分离降低了跨模块耦合
> 4. **如未来架构复盘发现七层结构导致业务模块内聚性下降，应优先回归 Feature First 顶层结构**

### 当前目录结构

```
src/
├── ai/               # AI 行为系统
├── assets.rs         # 资源加载（应在 infrastructure/）
├── battle/           # 战斗效果管线
├── buff/             # Buff/Debuff 系统
├── campaign/          # 战役系统
├── character/         # 角色与 Trait 扩展
├── core/              # 属性系统、效果管线、修饰规则、标签系统
│   ├── attribute/
│   ├── effect/
│   ├── error/         # 错误类型（应在 shared/error/ + 各领域/domain/）
│   ├── id/            # 强类型 ID（应在 shared/ids/）
│   ├── attribute_def.rs
│   ├── mod.rs
│   ├── modifier_rule.rs
│   ├── registry_loader.rs
│   ├── snapshot.rs
│   ├── tag.rs
│   └── tag_def.rs
├── debug/             # 调试面板
├── equipment/         # 装备系统
├── infrastructure/    # 基础设施（已有！）
│   ├── audit/         # 审计轨迹
│   └── logging/       # 日志
├── input.rs           # 输入处理（应在 ui/）
├── inventory/         # 背包系统
├── lib.rs             # 库入口
├── main.rs            # 主入口
├── map/               # 地图与寻路
├── skill/             # 技能系统
├── turn/              # 回合状态机
└── ui/                # 用户界面
```

### 问题分析

| 问题 | 当前状态 | 目标状态 |
|------|---------|---------|
| 业务模块在顶层 | `src/battle/`, `src/skill/` 等 | `src/core/battle/`, `src/core/skill/` 等 |
| ID 在 core 中 | `src/core/id/` | `src/shared/ids/` |
| 错误在 core 中 | `src/core/error/` | 各领域 `domain/` + `src/shared/error/` |
| 缺少 App 层 | 无 `src/app/` | `src/app/` 包含游戏装配 |
| 缺少 Content 层 | 无 `src/content/` | `src/content/` 连接数据与规则 |
| 缺少 Modding 层 | 无 `src/modding/` | `src/modding/` MOD 支持 |
| 资源加载在根目录 | `src/assets.rs` | `src/infrastructure/assets/` |
| 输入在根目录 | `src/input.rs` | `src/ui/input.rs` |
| 配置数据在 assets 中 | `assets/units/*.ron` | `content/characters/*.ron` |
| 无 content/ 目录 | 无 | `content/` 数据树 |

---

## 迁移原则

1. 🟥 **每个阶段必须保持项目可编译、可运行**（宪法 §20.7.1 CI 门禁标准）
2. 🟥 **每个阶段必须通过所有现有测试**（宪法 §18.1.1 测试优先）
3. 🟩 **优先迁移基础层（Shared），再迁移业务层（Core）**
4. 🟩 **每阶段的迁移范围最小化**（宪法 §1.5.1 复杂度优先）
5. 🟩 **每阶段完成后做一次完整的架构 Review**（宪法 §17.0.8 架构复盘每 3 个月）

---

## 回滚机制（Git Tag）

每个 Phase 必须建立明确的 Git Tag 回滚点，确保出现未预见问题时可快速回退：

**操作流程**：
1. **Phase 开始前**：创建起始 Tag — `git tag -a phase-N-start -m "Phase N 开始"`
2. **Phase 完成后**：创建完成 Tag — `git tag -a phase-N-complete -m "Phase N 完成"`
3. **验证通过后**：合入主分支

**分支策略**：
- 基于 `develop` 创建 `feature/phase-N` 分支进行迁移
- 验证通过后合入 `develop`
- 出现问题时 `git revert phase-N-complete` 或 `git reset --hard phase-N-start`

**高风险阶段额外策略**：
- Phase 4（Content 层迁移配置数据）：采用"灰度迁移"策略 — 先迁移 10% 配置验证，再全量迁移

> **优化来源**: `docs/其他/57.md`（回滚机制补充建议）

---

## Phase 0：准备工作

**目标**：创建目标目录结构，但不移动代码。

**步骤**：
1. 创建 `src/app/` 目录，先把 `main.rs` 的 Plugin 注册逻辑迁移到 `src/app/app_plugin.rs`
2. 创建 `src/shared/` 空目录结构
3. 创建 `src/content/` 空目录结构
4. 创建 `src/modding/` 空目录结构
5. 创建 `content/` 项目根目录
6. 更新 `.gitignore` 和 Git LFS 配置

**验证**：
- `cargo build` 通过
- `cargo test` 通过
- 目录结构存在但代码未迁移

**自动化工具支持**：
1. 预配置 `cargo fix`：在 `.cargo/config.toml` 中配置路径替换规则，便于批量更新 `use` 语句
2. 路径替换脚本：编写 `scripts/migrate_paths.py`，自动扫描并替换旧模块路径为新路径
3. 验证脚本：编写 `scripts/verify_migration.sh`，自动化执行 `cargo build` + `cargo test` + `cargo clippy` + 依赖图检查
4. 架构约束检查脚本：编写 `scripts/check_arch.sh`，验证层间依赖合规（如 `core` 不依赖 `infrastructure`）

> **优化来源**: `docs/其他/57.md`（自动化工具支持建议）

---

## Phase 1：Shared 层迁移

**目标**：建立 Shared 层，迁移通用工具。

### 1.1 迁移强类型 ID

```
迁移：src/core/id/ → src/shared/ids/
```

**步骤**：
1. 创建 `src/shared/ids/` 目录和文件
2. 移动 `unit_id.rs`, `skill_id.rs`, `buff_id.rs`, `item_id.rs` 到 `src/shared/ids/`
3. 更新所有 `use` 引用
4. 验证编译通过

**影响范围**：所有引用 `UnitId`, `SkillId`, `BuffId`, `ItemId` 的代码

### 1.2 迁移共享错误工具

```
迁移：src/core/error/game_result.rs → src/shared/error/result.rs
新建：src/shared/error/context.rs
新建：src/shared/error/extensions.rs
```

**步骤**：
1. 创建 `src/shared/error/` 目录
2. 迁移 `GameResult<T>` 到 `src/shared/error/result.rs`
3. 创建 `ErrorContext` trait 到 `src/shared/error/context.rs`
4. 创建 `LogIfError` trait 到 `src/shared/error/extensions.rs`
5. 更新所有 `use` 引用
6. 验证编译通过

### 1.3 创建 Shared 层其他模块（空壳）

```
新建：
src/shared/events/mod.rs
src/shared/audit/（迁移自 src/infrastructure/audit/）
src/shared/random/mod.rs
src/shared/math/mod.rs
src/shared/time/mod.rs
src/shared/collections/mod.rs
src/shared/validation/mod.rs
src/shared/constants/mod.rs
src/shared/traits/mod.rs
src/shared/testing/mod.rs
src/shared/versioning/mod.rs
```

**步骤**：
1. 创建上述目录和 `mod.rs` 空壳
2. 迁移 `infrastructure/audit/` 的白名单部分到 `shared/audit/`
3. 创建基本的 `SharedPlugin`
4. 验证编译通过

---

## Phase 2：Core 层重组

**目标**：将业务模块迁移到 `src/core/` 下。

### 2.1 创建 Core 层目录

```
创建：
src/core/battle/（迁移自 src/battle/）
src/core/skill/（迁移自 src/skill/）
src/core/buff/（迁移自 src/buff/）
src/core/character/（迁移自 src/character/）
src/core/equipment/（迁移自 src/equipment/）
src/core/inventory/（迁移自 src/inventory/）
src/core/ai/（迁移自 src/ai/）
src/core/map/（迁移自 src/map/）
src/core/turn/（迁移自 src/turn/）
src/core/campaign/（迁移自 src/campaign/）
```

**步骤**（每个模块）：
1. 在 `src/core/` 下创建对应目录
2. 将原顶层目录的文件移动到 `src/core/xxx/`
3. 更新所有 `use` 引用和 `mod` 声明
4. 验证编译通过

**注意**：这是一个大变更，建议逐模块迁移，每迁一个模块就验证一次。

### 2.2 重组错误架构

```
迁移：
src/core/error/battle_error.rs → src/core/battle/domain/battle_error.rs
src/core/error/skill_error.rs  → src/core/skill/domain/skill_error.rs
src/core/error/buff_error.rs   → src/core/buff/domain/buff_error.rs
src/core/error/inventory_error.rs → src/core/inventory/domain/inventory_error.rs
```

**步骤**：
1. 在各业务模块下创建 `domain/` 子目录
2. 迁移对应的错误枚举
3. 更新所有 `use` 引用
4. 删除 `src/core/error/` 临时目录
5. 验证编译通过

### 2.3 更新 Core 层 mod.rs

```rust
// src/core/mod.rs (目标形态)
pub mod attribute;
pub mod attribute_def;
pub mod battle;
pub mod buff;
pub mod character;
pub mod effect;
pub mod equipment;
pub mod inventory;
pub mod ai;
pub mod map;
pub mod skill;
pub mod turn;
pub mod campaign;
pub mod modifier_rule;
pub mod registry_loader;
pub mod snapshot;
pub mod tag;
pub mod tag_def;
// 后续模块...
```

---

## Phase 3：Infrastructure 层扩展

**目标**：扩展基础设施层。

### 3.1 迁移资源加载

```
迁移：src/assets.rs → src/infrastructure/assets/
```

### 3.2 创建 Infrastructure 层新模块

```
新建：
src/infrastructure/persistence/
src/infrastructure/localization/
src/infrastructure/replay/
src/infrastructure/config/
src/infrastructure/hot_reload/
```

**步骤**：
1. 创建目录和 `mod.rs`
2. 创建基本 Plugin
3. 验证编译通过

---

## Phase 4：Content 层

**目标**：建立内容桥接层，分离数据与规则。

### 4.1 创建 Content 层骨架

```
新建：
src/content/content_plugin.rs
src/content/skills/
src/content/buffs/
src/content/classes/
src/content/characters/
src/content/equipments/
src/content/items/
src/content/stages/
src/content/terrains/
src/content/ai_behaviors/
```

### 4.2 创建 content/ 项目根目录

```
新建：
content/
content/skills/
content/buffs/
content/classes/
content/characters/
content/equipments/
content/items/
content/stages/
content/terrains/
content/ai_behaviors/
```

### 4.3 迁移配置数据

```
迁移：
assets/units/*.ron      → content/characters/
assets/skills/*.ron     → content/skills/
assets/buffs/*.ron      → content/buffs/
assets/terrains/*.ron   → content/terrains/
assets/ai/*.ron         → content/ai_behaviors/
assets/maps/*.ron       → content/stages/
assets/traits/*.ron     → content/classes/（部分）
```

**步骤**：
1. 复制配置文件到新位置
2. 更新 `AssetServer` 加载路径
3. 更新 `RegistryLoader` 加载路径
4. 验证游戏正常运行
5. 删除旧位置的配置文件

**风险**：这是影响最大的迁移步骤，必须逐步进行，每迁移一类配置就验证一次。

**功能验证清单**（Phase 4 完成后必须逐项验证）：
- [ ] 角色属性加载正常（`content/characters/*.ron` 正确读取，属性值无丢失）
- [ ] 技能释放效果符合预期（技能定义加载正确，Effect Pipeline 正常执行）
- [ ] 地图寻路无异常（地形配置加载正确，移动范围/攻击范围计算正确）
- [ ] Buff 结算正确（Buff 定义加载正确，Modifier Stack 求值无误）
- [ ] 关卡配置加载正常（胜利/失败条件正确触发）
- [ ] 存档/读档功能正常（序列化/反序列化路径正确）

> **优化来源**: `docs/其他/57.md`（Phase 4 功能验证清单建议）

---

## Phase 5：App 层

**目标**：建立游戏装配层。

### 5.1 创建 App 层

```
新建：
src/app/app_plugin.rs
src/app/game_state.rs
src/app/schedules.rs
src/app/sets.rs
src/app/startup.rs
src/app/shutdown.rs
src/app/plugins.rs
```

**步骤**：
1. 将 `main.rs` 中的 Plugin 注册逻辑迁移到 `app_plugin.rs`
2. 将 `AppState` 定义迁移到 `game_state.rs`
3. 将 `SystemSet` 定义迁移到 `sets.rs`
4. 将启动逻辑迁移到 `startup.rs`
5. 更新 `main.rs` 只保留入口点

---

## Phase 6：Modding 层（未来）

> ⚠️ **宪法 §1.1.7 提醒**：以下为预留设计，禁止提前实现完整 Mod 框架。Phase 6 仅在明确需要 MOD 支持时启动，且只实现轻量扩展点，不构建完整框架。

**目标**：建立 MOD 支持框架。

### 6.1 创建 Modding 层骨架

```
新建：
src/modding/api/
src/modding/registry/
src/modding/loaders/
src/modding/validators/
src/modding/sandbox/
src/modding/compatibility/
```

### 6.2 实现 MOD API 基础

- `ModApi` trait 定义
- `ModContext` 上下文
- `ModPlugin` 注册

---

## Phase 7：Tools 层（未来）

> ⚠️ **宪法 §1.1.7 提醒**：Tools 层按需实现，仅在内容量增长到人工审查不可行时启动。参见 `docs/architecture/tools_architecture.md` 的实现路线。

**目标**：建立开发工具链。

### 7.1 创建 Tools 二进制

```
新建：
tools/
tools/content_editor/
tools/data_validator/
tools/balance_checker/
tools/replay_inspector/
```

---

## 迁移时间线估算

| 阶段 | 预估工作量 | 可行性 | 优先级 |
|------|-----------|--------|--------|
| Phase 0: 准备 | 1 天 | 高 | 🔴 立即 |
| Phase 1: Shared 层 | 2-3 天 | 高 | 🔴 立即 |
| Phase 2: Core 重组 | 5-7 天 | 中 | 🟡 Phase 1 后 |
| Phase 3: Infra 扩展 | 3-5 天 | 中 | 🟡 Phase 2 后 |
| Phase 4: Content 层 | 5-7 天 | 中 | 🟢 Phase 3 后 |
| Phase 5: App 层 | 2-3 天 | 低 | 🟢 Phase 4 后 |
| Phase 6: Modding | 10-15 天 | 低 | 🔵 长期 |
| Phase 7: Tools | 10-15 天 | 低 | 🔵 长期 |

### 关键路径

```
Phase 0 → Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5
                                                  ↓
                                             Phase 6 (独立)
                                                  ↓
                                             Phase 7 (独立)
```

---

## 迁移期间的双重兼容

在 Phase 1-3 期间，项目将处于"两种结构并存"的状态：

```
src/
├── battle/                    # 旧位置（逐步迁出）
├── core/
│   ├── id/                    # 旧位置（→ shared/ids/）
│   ├── error/                 # 旧位置（→ 各领域/domain/）
│   ├── battle/                # 新位置（从顶层迁入）
│   ├── skill/                 # 新位置
│   └── ...
├── shared/                    # 新目录
│   ├── ids/                   # 从 core/id/ 迁入
│   └── error/                 # 从 core/error/ 部分迁入
└── ...
```

**处理方式**：
- 使用 `pub use` 在旧位置创建重新导出，直到所有引用更新完成
- 每 Phase 完成后做一次全局搜索替换，清理所有旧引用
- 使用编译器警告发现遗漏的引用

### 技术债务清理窗口期

每个 Phase 完成后预留 **1 天** 作为技术债务清理窗口期：

1. **`#[deprecated]` 标记清理**：搜索所有 `#[deprecated]` 标记的旧路径重导出，替换为直接引用新路径
2. **死代码删除**：清理不再使用的 `pub use` 重导出、临时兼容代码
3. **全局搜索验证**：`grep -r "旧路径"` 确认无残留引用
4. **Clippy 检查**：`cargo clippy -- -D warnings` 确认无警告

**示例**：
```rust
// Phase 1 完成后，在 Phase 2 开始前清理：
// #[deprecated(note = "请使用 crate::shared::ids::UnitId")]  ← 删除此行
// pub use crate::shared::ids::UnitId;                          ← 删除此行
// 所有引用已更新为 crate::shared::ids::UnitId
```

> **优化来源**: `docs/其他/57.md`（技术债务清理窗口期建议）

---

## 迁移风险与缓解

| 风险 | 缓解措施 |
|------|---------|
| 编译断路 | 每个 Phase 都必须通过 `cargo build` |
| 测试断路 | 每个 Phase 都必须通过 `cargo test` |
| 引用遗漏 | 全局搜索替换 + 编译器警告 |
| 模块循环依赖 | Phase 2 完成后检查 `cargo check` 的依赖图 |
| 配置路径变更 | Phase 4 使用兼容层，同时支持新旧路径 |
| 团队混乱 | 每个 Phase 完成后通知全团队，更新开发文档 |

### 团队协作规范

**Code Review 参与人与标准**：
- **参与人**：技术负责人（必须参与）+ 核心开发者（1-2 人）
- **Review 标准**：
  1. 模块依赖关系是否合规（不违反层级约束）
  2. 旧路径的 `pub use` 重新导出是否已创建
  3. 所有 `use` 引用是否已更新到新路径
  4. 是否引入了循环依赖
  5. 测试覆盖率是否满足要求

**文档同步要求**：
- 每 Phase 完成后必须更新 `docs/architecture.md` 中的目录结构说明
- 更新 `AGENTS.md` 中的模块引用规则
- 更新开发规范文档中的模块路径约定

**任务分配矩阵**：

| Phase | 核心开发者 | 测试 | 数据/配置 | 技术负责人 |
|-------|-----------|------|----------|-----------|
| Phase 0-1 | 1 人 | — | — | Review |
| Phase 2 | 1 人 | 1 人 | — | Review |
| Phase 3 | 1 人 | — | — | Review |
| Phase 4 | 1 人 | 1 人 | 1 人 | Review |
| Phase 5 | 1 人 | — | — | Review |

> **优化来源**: `docs/其他/57.md`（团队协作细节补充建议）

---

## 迁移完成后的验证清单

🟥 **以下验证项全部通过才能确认迁移完成**：

- [ ] `src/` 下无顶层业务模块（battle/、skill/ 等都在 core/ 下）
- [ ] `src/core/id/` 不存在（已迁移到 `src/shared/ids/`）
- [ ] `src/core/error/` 不存在（错误已迁移到各领域 `domain/` + `src/shared/error/`）
- [ ] `src/app/` 存在且包含游戏装配逻辑
- [ ] `src/content/` 存在且包含数据加载逻辑
- [ ] `content/` 项目根目录存在且包含 RON 配置
- [ ] `assets/` 不包含 RON 配置数据（只有二进制资源）
- [ ] `src/shared/` 不包含任何业务逻辑
- [ ] 🟥 `src/core/` 不依赖 `src/infrastructure/`（宪法 §1.3.2 依赖方向铁则）
- [ ] 🟥 `src/shared/` 不依赖任何其他层（宪法 §1.3.2）
- [ ] 所有测试通过（宪法 §20.7.1）
- [ ] 编译无警告（宪法 §20.1.1 Warning Budget = 0）
- [ ] 架构 Review 完成（宪法 §17.0.8）