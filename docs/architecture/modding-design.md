# MOD Support Architecture — MOD 支持架构

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的 MOD 支持架构，目标是让社区和内部团队可以通过数据扩展游戏内容，而无需修改 Rust 代码。

---

## 设计目标

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

---

## MOD API 设计

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

🟥 **MOD 绝对禁止**：

| 操作 | 原因 |
|------|------|
| 绕过 Effect Pipeline 直接扣血 | 破坏战斗规则一致性 |
| 绕过 Modifier Stack 直接修改属性 | 破坏属性计算一致性 |
| 直接修改其他 MOD 的数据 | 破坏 MOD 隔离性 |
| 访问 ECS World 直接操作 Entity | 破坏 ECS 安全性 |
| 运行任意 Rust 代码 | 破坏沙箱安全性 |
| 修改核心游戏规则代码 | 破坏 Rule/Content 分离 |

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

- 🟥 MOD 不能直接访问 `World`
- 🟥 MOD 不能直接访问 `Commands`
- 🟥 MOD 不能直接操作 `Entity`
- 🟩 MOD 只能通过 `ModApi` 接口操作
- 🟩 MOD 的写操作必须通过校验管线

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

---

## 未来目标（Phase 2+）

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