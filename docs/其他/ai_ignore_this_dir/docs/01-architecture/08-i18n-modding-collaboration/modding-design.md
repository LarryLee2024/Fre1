---
id: 01-architecture.modding-design
title: Modding Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# MOD Support Architecture — MOD 支持架构

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的 MOD 支持架构，目标是让社区和内部团队可以通过数据扩展游戏内容，而无需修改 Rust 代码。

---

## 设计目标

> **宪法 §17.2.1 Mod 支持预留**：核心系统设计时预留轻量扩展点，不提前实现完整 Mod 框架。公共 API、配置格式、事件体系保持稳定，为未来 Mod 能力铺路。

1. **策划可扩展**：新增技能、Buff、装备只需添加 RON 文件
2. **社区可扩展**：MOD 作者可以创建新内容包
3. **安全可控**：MOD 不能破坏核心游戏规则
4. **兼容可管理**：MOD 之间冲突可检测、可解决
5. **热可加载**：MOD 可以在运行时加载/卸载（未来目标）

---

## MOD 架构总览

```
┌─────────────────────────────────────────────┐
│  MOD API (modding/api/)                     │
│  - 稳定公开接口                               │
│  - MOD 作者唯一需要了解的部分                    │
├─────────────────────────────────────────────┤
│  MOD Registry (modding/registry/)            │
│  - MOD 注册表                                │
│  - 依赖解析                                   │
│  - 加载顺序                                   │
├─────────────────────────────────────────────┤
│  MOD Loaders (modding/loaders/)              │
│  - 内容加载器                                 │
│  - 基础内容 + MOD 内容合并                     │
├─────────────────────────────────────────────┤
│  MOD Validators (modding/validators/)         │
│  - Schema 校验                               │
│  - 引用完整性                                 │
│  - 冲突检测                                   │
├─────────────────────────────────────────────┤
│  MOD Sandbox (modding/sandbox/)              │
│  - 沙箱环境                                  │
│  - 权限控制                                   │
├─────────────────────────────────────────────┤
│  MOD Compatibility (modding/compatibility/)  │
│  - 版本检查                                  │
│  - 兼容性矩阵                                │
└─────────────────────────────────────────────┘
```

---

## MOD 生命周期

### 1. 发现

```
mods/
├── official/                 # 官方 MOD
│   └── example_mod/
│       ├── manifest.ron      # MOD 清单
│       ├── content/          # MOD 内容
│       └── compatibility/    # 兼容性声明
│
├── community/                # 社区 MOD（gitignore）
│
└── dev/                      # 开发中 MOD
    └── test_mod/
        ├── manifest.ron
        └── content/
```

### 2. MOD 清单格式

```ron
// mods/community/fire_expansion/manifest.ron
(
    id: "fire_expansion",
    name: "火系扩展包",
    version: "1.0.0",
    author: "community_author",
    
    // 游戏版本兼容性
    game_version: ">=0.1.0",
    
    // 依赖的其他 MOD
    dependencies: [],
    
    // 冲突的 MOD
    conflicts: ["ice_overhaul"],
    
    // MOD 内容优先级（数字越大越后加载）
    priority: 100,
    
    // 提供的内容
    provides: [
        "skills/fire_storm",
        "buffs/inferno",
        "characters/fire_elemental",
    ],
    
    // 覆盖的内容（谨慎使用）
    overrides: [],
)
```

### 3. 加载流程

```
游戏启动
    ↓
1. 扫描 mods/ 目录
    ↓
2. 解析所有 manifest.ron
    ↓
3. 依赖解析 & 冲突检测
    ↓
4. 确定加载顺序
    ↓
5. 加载基础内容 (content/)
    ↓
6. 按 priority 顺序加载 MOD 内容
    ↓
7. 运行校验管线
    ↓
8. MOD 内容合并到 Registry
    ↓
9. 游戏就绪
```

### MOD 性能预算

MOD 加载必须满足以下性能约束，避免影响游戏启动体验：

| 指标 | 预算 | 说明 |
|------|------|------|
| 单 MOD 加载时间 | ≤ 50ms | 包含解析、校验、注册全流程 |
| 总 MOD 加载时间 | ≤ 200ms | 所有已启用 MOD 的加载总时间 |
| MOD 内存占用 | ≤ 128MB | 所有 MOD 内容的内存总预算 |
| manifest 解析缓存 | 🟩 | 缓存已解析的 manifest，避免重复解析 |
| 增量校验 | 🟩 | 仅校验变更 MOD，未变更的跳过校验 |
| 异步加载非核心内容 | 🟩 | 美术/音频资源异步加载，不阻塞主线程 |

```rust
// 性能监控示例
fn log_mod_loading_performance(mods_loaded: u32, total_time_ms: f64) {
    if total_time_ms > 200.0 {
        warn!(
            mods_count = mods_loaded,
            total_ms = total_time_ms,
            "MOD loading exceeded 200ms budget"
        );
    }
    // 集成 tracing span 跟踪每个 MOD 的加载性能
}
```

> **优化来源**: `docs/其他/58.md`（性能预算与监控建议）

---

## MOD API 设计

> 🟥 **宪法 §3.0.1 接口最小化原则**：MOD API 只暴露必要的公共接口，所有内部实现必须设为私有。MOD 作者只能通过 `modding/api/` 访问稳定接口，禁止直接访问 `modding/registry/`、`modding/loaders/` 等内部模块。

### 稳定公开接口

```rust
// src/modding/api/mod_api.rs

/// MOD API 是 MOD 作者唯一需要了解的稳定接口。
/// 所有核心规则的扩展点都通过此 API 暴露。
pub mod mod_api {
    // ===== 技能系统扩展 =====
    
    /// 注册新技能
    pub fn register_skill(ctx: &mut ModContext, skill: SkillDef) -> ModResult<()> {
        // 校验 → 注册到 Registry
    }
    
    /// 覆盖现有技能
    pub fn override_skill(ctx: &mut ModContext, skill_id: &str, skill: SkillDef) -> ModResult<()> {
        // 校验 → 覆盖到 Registry
    }
    
    // ===== Buff 系统扩展 =====
    
    /// 注册新 Buff
    pub fn register_buff(ctx: &mut ModContext, buff: BuffDef) -> ModResult<()> {
        // 校验 → 注册到 Registry
    }
    
    /// 覆盖现有 Buff
    pub fn override_buff(ctx: &mut ModContext, buff_id: &str, buff: BuffDef) -> ModResult<()> {
        // 校验 → 覆盖到 Registry
    }
    
    // ===== 装备系统扩展 =====
    
    /// 注册新装备
    pub fn register_equipment(ctx: &mut ModContext, equipment: EquipmentDef) -> ModResult<()> {
        // 校验 → 注册到 Registry
    }
    
    // ===== 角色扩展 =====
    
    /// 注册新角色模板
    pub fn register_character(ctx: &mut ModContext, character: UnitTemplate) -> ModResult<()> {
        // 校验 → 注册到 Registry
    }
    
    // ===== 地图扩展 =====
    
    /// 注册新关卡
    pub fn register_stage(ctx: &mut ModContext, stage: StageDef) -> ModResult<()> {
        // 校验 → 注册到 Registry
    }
    
    // ===== 查询接口 =====
    
    /// 查询已注册技能
    pub fn query_skill(ctx: &ModContext, skill_id: &str) -> Option<&SkillData> {
        // 只读查询
    }
    
    /// 查询已注册 Buff
    pub fn query_buff(ctx: &ModContext, buff_id: &str) -> Option<&BuffData> {
        // 只读查询
    }
}
```

### MOD 安全边界

🟥 **MOD 绝对禁止（宪法相关条款）**：

| 操作 | 原因 | 宪法依据 |
|------|------|---------|
| 绕过 Effect Pipeline 直接扣血 | 破坏战斗规则一致性 | §7.0.4 Modifier 管线统一 |
| 绕过 Modifier Stack 直接修改属性 | 破坏属性计算一致性 | §8.0.3 修改规范 |
| 直接修改其他 MOD 的数据 | 破坏 MOD 隔离性 | §3.0.4 跨模块交互规范 |
| 访问 ECS World 直接操作 Entity | 破坏 ECS 安全性 | §2.1.1 Entity 只是 ID |
| 运行任意 Rust 代码 | 破坏沙箱安全性 | §1.1.4 逻辑与表现分离 |
| 修改核心游戏规则代码 | 破坏 Rule/Content 分离 | §1.1.3 规则与内容分离 |

🟩 **MOD 允许**：

| 操作 | 说明 |
|------|------|
| 添加新技能 | 通过 `register_skill` |
| 添加新 Buff | 通过 `register_buff` |
| 添加新装备 | 通过 `register_equipment` |
| 添加新角色 | 通过 `register_character` |
| 添加新关卡 | 通过 `register_stage` |
| 覆盖数值属性 | 通过 `override_skill` 等 |
| 添加新对话 | 通过 `register_dialogue` |
| 添加新任务 | 通过 `register_quest` |

### MOD 分级权限策略

不同来源的 MOD 拥有不同的权限级别，形成分级管控体系：

| 级别 | 来源 | 权限范围 | 审核要求 |
|------|------|---------|---------|
| Level 0（官方） | 开发团队发布 | 扩展接口 + 轻度自定义战斗逻辑 | 内部 Code Review |
| Level 1（社区） | 社区作者提交 | 严格沙箱，仅开放基础内容扩展接口 | Steam Workshop 审核 + 自动校验 |
| Level 2（测试） | 开发环境专属 | 全权限（含调试接口），便于 MOD 开发调试 | 无（本地环境） |

**权限差异示例**：
- 官方 MOD（Level 0）：可通过 `register_custom_rule` 注册自定义战斗规则（如特殊元素交互）
- 社区 MOD（Level 1）：只能通过 `register_skill` / `register_buff` 等基础接口扩展内容
- 测试 MOD（Level 2）：可访问 `ModDebugApi`，读取内部状态、模拟加载/卸载

```rust
/// MOD 权限级别
pub enum ModPrivilegeLevel {
    /// 官方 MOD：可扩展战斗逻辑
    Official,
    /// 社区 MOD：仅扩展内容数据
    Community,
    /// 测试 MOD：开发环境全权限
    Test,
}
```

> **优化来源**: `docs/其他/58.md`（权限分级策略建议）

---

## MOD 注册表

### 依赖解析

```rust
// src/modding/registry/dependency_resolver.rs

pub struct ModDependencyResolver;

impl ModDependencyResolver {
    /// 解析 MOD 依赖关系，返回加载顺序
    pub fn resolve(mods: &[ModManifest]) -> Result<Vec<ModId>, ModError> {
        // 拓扑排序
        // 检测循环依赖
        // 检测版本冲突
        // 检测缺失依赖
    }
    
    /// 检测 MOD 冲突
    pub fn detect_conflicts(mods: &[ModManifest]) -> Vec<ModConflict> {
        // 同一内容被多个 MOD 覆盖
        // MOD 之间的声明冲突
    }
}
```

### 内容合并策略

```
基础内容 (content/)
    ↓ 加载
MOD 1 内容 (mods/community/mod1/content/)
    ↓ 合并（新增 / 覆盖）
MOD 2 内容 (mods/community/mod2/content/)
    ↓ 合并（新增 / 覆盖）
    ↓
最终 Registry
```

合并规则：
1. **新增**：MOD 新增的内容直接注册
2. **覆盖**：MOD 覆盖的内容替换基础内容（需 manifest 中声明）
3. **冲突**：两个 MOD 覆盖同一内容 → 按优先级决定，或报错

### 覆盖优先级管控

**核心内容不可覆盖（白名单机制）**：

以下核心内容定义为"受保护内容"，任何 MOD 不得覆盖：

| 受保护内容 | 原因 |
|-----------|------|
| 基础职业定义（如 Warrior、Mage） | 影响核心战斗平衡 |
| 元素交互规则 | 影响核心战斗逻辑 |
| 回合状态机定义 | 影响核心流程控制 |
| 胜负条件检查逻辑 | 影响游戏基本规则 |
| 属性计算 Modifier 规则 | 影响核心数值体系 |

**多 MOD 覆盖冲突降级策略**：
1. 两个 MOD 覆盖同一内容 → 按 `priority` 数值决定，高优先级覆盖生效
2. 优先级相同 → 后加载的 MOD 覆盖先加载的（加载顺序由依赖解析决定）
3. 产生覆盖冲突 → 记录审计日志（含 MOD ID、覆盖内容 ID、优先级），便于排查
4. 核心内容被覆盖 → 校验器拦截，返回 `ModError::ProtectedContent` 错误

```rust
// 覆盖审计日志示例
pub fn log_override_event(mod_id: &str, content_id: &str, priority: u32) {
    info!(
        mod_id = mod_id,
        content_id = content_id,
        priority = priority,
        "MOD content override applied"
    );
}
```

> **优化来源**: `docs/其他/58.md`（覆盖优先级管控建议）

---

## MOD 校验器

### Schema 校验

```rust
// src/modding/validators/schema_validator.rs

pub fn validate_skill_schema(skill: &SkillDef) -> Result<(), ModValidationError> {
    // 必填字段检查
    // 类型检查
    // 数值范围检查
    // 标签格式检查
}
```

### 引用完整性校验

```rust
// src/modding/validators/conflict_detector.rs

pub fn validate_references(skill: &SkillDef, registry: &SkillRegistry) -> Result<(), ModValidationError> {
    // effect_ids 必须存在
    // buff_ids 必须存在
    // 标签必须存在
}
```

### 冲突检测

> **命名空间防止冲突**：参见 `docs/01-architecture/asset_namespace_design.md`（`base:`/`mod_x:` 前缀方案从根本上防止 MOD 间 ID 冲突）。

```rust
pub fn detect_content_conflicts(
    base_content: &ContentIndex,
    mods: &[ModManifest],
) -> Vec<ModConflict> {
    // 同一 ID 被多个 MOD 覆盖
    // 依赖缺失
    // 版本不兼容
}
```

---

## MOD 沙箱

### 沙箱原则

```rust
// src/modding/sandbox/sandbox_runner.rs

/// MOD 沙箱环境
/// 确保 MOD 只能通过 API 操作，不能直接访问 ECS World
pub struct ModSandbox {
    /// MOD 可访问的注册表（只读 + API 受控写入）
    context: ModContext,
    /// MOD 可访问的查询接口（只读）
    query: ModQuery,
}
```

### 沙箱约束

- 🟥 MOD 不能直接访问 `World`（宪法 §2.1.1 Entity 只是 ID）
- 🟥 MOD 不能直接访问 `Commands`
- 🟥 MOD 不能直接操作 `Entity`（宪法 §2.1.1）
- 🟩 MOD 只能通过 `ModApi` 接口操作（宪法 §3.0.1 接口最小化）
- 🟩 MOD 的写操作必须通过校验管线（宪法 §3.0.4 跨模块交互规范）

### MOD 调试机制

为 MOD 作者提供完善的调试支持，降低开发门槛：

**MOD 加载日志接口**：
```rust
/// MOD 加载完成事件
#[derive(Event)]
pub struct LogModsLoaded {
    pub loaded_mods: Vec<ModLoadInfo>,
    pub total_time_ms: f64,
    pub errors: Vec<ModLoadError>,
}

pub struct ModLoadInfo {
    pub mod_id: String,
    pub version: String,
    pub load_time_ms: f64,
    pub content_count: ContentCount,
}
```

**状态查询 API**：
```rust
/// 查询当前已加载的所有 MOD
pub fn query_loaded_mods(ctx: &ModContext) -> Vec<ModStatus>;

/// 查询指定 MOD 的依赖关系
pub fn query_mod_dependencies(ctx: &ModContext, mod_id: &str) -> Vec<String>;

/// 查询指定 MOD 的冲突项
pub fn query_mod_conflicts(ctx: &ModContext, mod_id: &str) -> Vec<ModConflict>;

/// 查询指定 MOD 注册的所有内容
pub fn query_mod_content(ctx: &ModContext, mod_id: &str) -> ContentIndex;
```

**冲突检测预览工具**：
- 在 MOD 加载前提供"干运行"模式（dry-run），预览加载结果和冲突项
- 输出格式化的冲突报告（含冲突内容 ID、涉及的 MOD、建议解决方案）

> **优化来源**: `docs/其他/58.md`（调试与排查机制建议）

---

## MOD 兼容性

### 版本检查

```rust
// src/modding/compatibility/version_checker.rs

pub struct ModVersionChecker;

impl ModVersionChecker {
    /// 检查 MOD 是否与当前游戏版本兼容
    pub fn check_compatibility(
        mod_manifest: &ModManifest,
        game_version: &GameVersion,
    ) -> Result<(), ModCompatibilityError> {
        // 检查 game_version 语义版本
        // 检查 API 版本兼容性
        // 检查依赖版本
    }
}
```

### 兼容性矩阵

```ron
// mods/community/fire_expansion/compatibility/main.ron
(
    game_version: ">=0.1.0, <0.2.0",
    api_version: "1",
    compatible_mods: ["*"],           // 兼容所有 MOD
    incompatible_mods: ["ice_overhaul"],  // 不兼容此 MOD
)
```

---

## MOD 内容目录结构

一个完整的 MOD 示例：

```
mods/community/fire_expansion/
├── manifest.ron                    # MOD 清单
├── compatibility/                  # 兼容性声明
│   └── main.ron
├── content/                        # MOD 内容
│   ├── skills/                     # 新增/覆盖技能
│   │   ├── fire_storm.ron
│   │   └── inferno_blast.ron
│   ├── buffs/                      # 新增/覆盖 Buff
│   │   ├── inferno.ron
│   │   └── burning_soul.ron
│   ├── characters/                 # 新增角色
│   │   └── fire_elemental.ron
│   ├── equipments/                 # 新增装备
│   │   ├── flame_sword.ron
│   │   └── fire_staff.ron
│   └── stages/                     # 新增关卡
│       └── fire_dungeon.ron
├── assets/                         # MOD 美术资源（可选）
│   ├── art/
│   │   └── effects/
│   │       └── fire_storm/
│   └── audio/
│       └── sfx/
│           └── fire_storm/
└── README.md                       # MOD 说明文档
```

### 资源命名空间规则

MOD 的美术/音频资源必须使用命名空间前缀，防止资源冲突：

**命名空间格式**：
- 基础资源：`base:art/effects/fire_storm`（游戏自带资源）
- MOD 资源：`mod_{mod_id}:art/effects/fire_storm`（MOD 专属资源）

**资源加载/卸载生命周期**：
1. **加载阶段**：MOD 加载时，将 MOD 的 `assets/` 目录注册到 AssetServer，资源 ID 自动加前缀
2. **引用阶段**：RON 配置中引用资源时使用完整命名空间路径（如 `mod_fire:art/effects/fire_storm`）
3. **卸载阶段**：MOD 卸载时，遍历 Registry 中该 MOD 注册的所有资源 ID，从 AssetServer 移除
4. **内存回收**：卸载后触发 AssetServer 的资源回收，释放未使用的资源内存

```rust
/// MOD 资源管理器
pub struct ModAssetManager {
    /// 已注册的 MOD 资源 ID 列表（用于卸载时清理）
    mod_assets: HashMap<ModId, Vec<AssetId>>,
}

impl ModAssetManager {
    /// 注册 MOD 资源目录
    pub fn register_mod_assets(&mut self, mod_id: &ModId, assets_path: &Path) {
        // 扫描 assets/ 目录，注册所有资源并记录 ID
    }
    
    /// 卸载 MOD 资源
    pub fn unregister_mod_assets(&mut self, mod_id: &ModId, asset_server: &AssetServer) {
        if let Some(assets) = self.mod_assets.remove(mod_id) {
            for asset_id in assets {
                asset_server.unload(asset_id);
            }
        }
    }
}
```

> **优化来源**: `docs/其他/58.md`（资源命名空间与生命周期建议）

---

## 未来目标（Phase 2+）

> ⚠️ **宪法 §1.1.7 提醒**：以下未来目标为预留设计，**禁止在 Phase 1 提前实现完整 WASM 沙箱、运行时 MOD 加载/卸载等复杂框架**。Phase 1 仅实现轻量扩展点（注册/查询接口），完整框架在明确需要时再启动。

### WASM 沙箱预留接口（Phase 1 设计）

为避免未来脚本支持引入时的架构颠覆，Phase 1 应预留 `ScriptEngine` trait 接口：

> **优化来源**: `docs/其他/58.md`（WASM 沙箱预留接口建议）

```rust
/// 脚本引擎抽象 trait（Phase 1 定义接口，Phase 2 实现）
/// 支持 wasmi（解释执行）和 wasmtime（JIT 编译）两种后端
pub trait ScriptEngine: Send + Sync {
    /// 执行脚本代码
    fn execute(&self, script: &str, context: &mut ScriptContext) -> ScriptResult;
    
    /// 注册脚本可访问的 API
    fn register_api(&mut self, api: ScriptApi);
    
    /// 引擎名称（用于日志和调试）
    fn name(&self) -> &str;
}

/// 脚本上下文（沙箱环境）
pub struct ScriptContext<'a> {
    /// 只读的 ModApi 引用
    pub mod_api: &'a ModApi,
    /// 脚本可访问的游戏状态（只读）
    pub game_state: &'a GameStateSnapshot,
}
```

**关键约束**：
- 脚本必须通过 `ModApi` 接口操作游戏数据，禁止特殊权限
- 脚本运行在 WASM 沙箱中，无法访问宿主内存
- 脚本禁止绕过 Effect Pipeline
- Phase 1 只定义 trait 接口，不引入 WASM 运行时依赖

### 运行时 MOD 加载/卸载

- 修改 Registry 为可变版本（ModdableRegistry）
- MOD 卸载时回滚所有注册的内容
- MOD 加载时重新初始化所有受影响的系统

### MOD Workshop 集成

- Steam Workshop 上传/下载
- MOD 元数据管理
- MOD 评分和评论

### MOD 脚本支持

- 通过 `infrastructure/scripting_runtime/` 提供脚本能力
- 最初可能支持 Lua 或 WASM
- 脚本只在沙箱环境中运行
- 🟥 脚本禁止绕过 Effect Pipeline

---

## 禁止事项

- 🟥 MOD 不能修改核心游戏规则代码
- 🟥 MOD 不能直接操作 ECS Entity
- 🟥 MOD 不能绕过 Effect Pipeline
- 🟥 MOD 不能绕过 Modifier Stack
- 🟥 MOD 不能访问其他 MOD 的内部数据
- 🟩 MOD 可以添加新内容（技能、Buff、装备、角色、关卡）
- 🟩 MOD 可以覆盖现有内容的数值
- 🟩 MOD 可以添加新美术资源
- 🟩 MOD 可以添加新音频资源