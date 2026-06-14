---
id: 01-architecture.asset-namespace-design
title: Asset Namespace Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# Asset Namespace Design — 资源命名空间设计

Version: 1.0
Status: Proposed

来源：`docs/其他/33遗漏2.md` C14

> **优化来源**: `docs/其他/37.md`

交叉引用：
- `docs/01-architecture/modding-design.md`、`docs/01-architecture/project-structure.md`、`docs/01-architecture/layer-contracts.md`
- `docs/AI开发宪法完整版.md` — AI 开发宪法（最高约束力），本文档对应条款：1.1.2（定义与实例分离）、12.1.4（统一 Asset Pipeline）、12.3.1（唯一事实源）、3.0.6（路径命名规范）

---

## 1. 问题背景

### 1.1 为什么需要命名空间

当多个 MOD 同时注册同名内容时，会产生命名冲突：

```
MOD A 定义: "fireball" → 造成 120 点火焰伤害
MOD B 定义: "fireball" → 造成 80 点冰霜伤害
```

没有命名空间时，后加载的 MOD 会覆盖先加载的 MOD，导致不可预期的行为。更严重的是，MOD 可能无意间覆盖基础游戏内容（如 `base:heal`）。

### 1.2 问题场景

| 场景 | 无命名空间后果 |
|------|--------------|
| MOD A 与 MOD B 都定义 `fireball` | 后加载者覆盖前者，行为不可预期 |
| MOD 定义 `heal` 技能 | 覆盖基础游戏的治疗技能 |
| MOD 定义 `steel_sword` | 覆盖基础游戏的钢剑 |
| 基础游戏更新新增 `ice_blast` | 与 MOD 中同名内容冲突 |

### 1.3 设计目标

1. **隔离**：每个 MOD 的内容有独立的命名空间
2. **安全**：MOD 不能覆盖基础游戏内容
3. **兼容**：向后兼容无前缀的引用（默认 `base:`）
4. **可检测**：冲突在加载时即可发现，而非运行时

> **优化来源**: `docs/其他/74借鉴.md` §12 — Unity Addressable Asset 资源ID化思想

### 1.4 Addressable Asset 模式

借鉴 Unity 的 Addressable Asset 系统，所有资源引用通过 `AssetId` 机制解耦：

```
AssetId（逻辑ID） → Registry（ID→资源映射） → 实际文件路径
```

**核心原则**：

🟥 **禁止在代码或配置中硬编码文件路径**（如 `"assets/ui/icon.png"`），必须通过 `AssetId` 引用资源。

**AssetId overlay 优先级链**：

```
AssetId → Base Game → DLC → MOD（后加载覆盖先加载）
```

查找顺序：精确匹配 → 当前命名空间 → `base:` 回退。AssetId 解析器负责 ID→实际文件路径的映射，支持运行时热切换（如 DLC 加载后更新映射表）。

---

## 2. 命名空间前缀方案

### 2.1 命名空间格式

```
<namespace>:<category>/<name>
```

- `namespace`：小写字母 + 下划线，最长 32 字符
- `category`：功能分类（skills、buffs、items 等）
- `name`：具体资源名
- 分隔符：冒号 `:` 分隔 namespace 与 category，斜杠 `/` 分隔 category 与 name

> **优化来源**: `docs/其他/37.md` — 将 `content_id` 细化为 `{namespace, category, name}` 三元组结构，消灭字符串拼接带来的拼写错误和路径遍历漏洞。

### 2.1.1 结构化 AssetId

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct AssetId {
    /// 命名空间（如 base、mod_a）
    pub namespace: Namespace,
    /// 功能分类（如 skills、buffs、items）
    pub category: AssetCategory,
    /// 具体资源名（如 fireball、heal）
    pub name: InternedStr,
}

/// 使用 Interning（字符串驻留）+ u64 Hash 的高性能版本
/// 用于热路径（每帧数百次资源查询）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetKey(u64);

impl AssetKey {
    /// 启动时或加载时计算一次，后续全部用 u64 比较
    pub fn from_asset_id(id: &AssetId) -> Self {
        let mut hasher = FxHasher::default(); // 非加密快速哈希
        hasher.write(id.namespace.as_bytes());
        hasher.write(id.category.as_bytes());
        hasher.write(id.name.as_bytes());
        Self(hasher.finish())
    }
}

// 保留全局 HashMap<AssetKey, String> 用于调试和序列化
// 运行时逻辑只用 AssetKey(u64)，零拷贝比较
```

> **优化来源**: `docs/其他/37.md` — 采用 Interning + u64 Hash 模式，解决 String 在热路径上的分配和比较开销。SRPG 战斗中每帧可能有数百次资源查询。

### 2.2 命名空间分配

| 命名空间 | 用途 | 说明 |
|----------|------|------|
| `base` | 基础游戏内容 | 基础游戏的所有内容使用此前缀 |
| `official_dlc` | 官方 DLC 内容 | 官方扩展包使用此前缀 |
| `<mod_name>` | MOD 自定义内容 | 每个 MOD 使用其声明的命名空间 |

### 2.3 示例

```text
base:fireball        → 基础游戏的火球术
base:heal            → 基础游戏的治疗术
base:steel_sword     → 基础游戏的钢剑

mod_a:fireball       → MOD A 的火球术（与 base:fireball 共存）
mod_a:custom_buff    → MOD A 的自定义 Buff

mod_b:fireball       → MOD B 的火球术（与 base:fireball、mod_a:fireball 共存）

official_dlc:ice_lance → 官方 DLC 的冰矛
```

### 2.4 命名空间命名规则

🟥 **禁止**：
- 命名空间包含冒号 `:`（分隔符冲突）
- 命名空间包含大写字母（大小写不敏感，避免歧义）
- 命名空间以数字开头
- 命名空间长度超过 32 字符
- 命名空间与保留名冲突（`base`、`official_dlc`）

🟩 **允许**：
- 小写字母 + 下划线：`fire_expansion`、`dark_magic_mod`
- 数字（非首位）：`mod_v2`、`expansion1`

---

## 3. 命名空间注册

### 3.1 基础游戏命名空间

基础游戏内容自动使用 `base:` 前缀：

```rust
// content/skills/fireball.ron
(
    id: "base:fireball",
    name: "火球术",
    mp_cost: 15,
    ...
)
```

基础游戏不需要在 `mod.toml` 中声明命名空间，`base` 是内建保留名。

### 3.2 MOD 命名空间声明

每个 MOD 在 `manifest.ron` 中声明其命名空间：

```ron
// mods/community/fire_expansion/manifest.ron
(
    id: "fire_expansion",
    namespace: "fire_expansion",  // ← MOD 命名空间
    name: "火系扩展包",
    version: "1.0.0",
    ...
)
```

### 3.3 官方 DLC 命名空间

官方 DLC 使用 `official_dlc:` 前缀：

```ron
// mods/official/dlc_1/manifest.ron
(
    id: "dlc_1",
    namespace: "official_dlc",  // ← 官方 DLC 命名空间
    name: "冰霜扩展包",
    version: "1.0.0",
    ...
)
```

### 3.4 命名空间冲突检测

在 MOD 加载阶段（`src/modding/loaders/`），MOD 加载器必须：

1. 收集所有 MOD 声明的命名空间
2. 检测命名空间是否重复
3. 检测命名空间是否与 `base` 或 `official_dlc` 冲突
4. 发现冲突时，拒绝加载并报告错误

```rust
// src/modding/loaders/mod_loader.rs
fn validate_namespace_conflicts(mods: &[ModManifest]) -> Result<(), ModLoadError> {
    let mut namespaces = HashSet::new();
    namespaces.insert("base".to_string());
    namespaces.insert("official_dlc".to_string());

    for mod_manifest in mods {
        if !namespaces.insert(mod_manifest.namespace.clone()) {
            return Err(ModLoadError::NamespaceConflict {
                namespace: mod_manifest.namespace.clone(),
                mod_id: mod_manifest.id.clone(),
            });
        }
    }
    Ok(())
}
```

---

## 4. 注册表集成

### 4.1 Registry 键值设计

所有 Registry（`SkillRegistry`、`BuffRegistry`、`ItemRegistry` 等）的键使用完整的命名空间 ID：

```rust
pub struct SkillRegistry {
    skills: HashMap<String, SkillData>,  // key = "base:fireball" 或 "mod_a:fireball"
}
```

### 4.2 Registry 查找策略

查找顺序：

1. **精确匹配**：先搜索 `namespace:content_id` 的完整键
2. **上下文回退**：如果未找到，在当前上下文的命名空间中搜索
3. **基础回退**：最后搜索 `base:content_id`

```rust
impl SkillRegistry {
    /// 查找技能，按优先级搜索
    pub fn lookup(&self, full_id: &str, context_namespace: &str) -> Option<&SkillData> {
        // 1. 精确匹配
        if let Some(skill) = self.skills.get(full_id) {
            return Some(skill);
        }

        // 2. 如果没有冒号分隔符，假设 base: 前缀
        if !full_id.contains(':') {
            let base_id = format!("base:{}", full_id);
            return self.skills.get(&base_id);
        }

        // 3. 如果只有命名空间没有内容 ID，无效格式
        None
    }
}
```

### 4.3 向后兼容：无前缀引用

当 RON 配置中没有命名空间前缀时，默认假设 `base:` 前缀：

```ron
// 以下两种写法等价
SkillDef(
    skill_id: "base:heal",     // 显式前缀
    ...
)

SkillDef(
    skill_id: "heal",           // 隐式前缀，自动解析为 base:heal
    ...
)
```

**规则**：
- 没有冒号 `:` 的 ID 自动添加 `base:` 前缀
- 有冒号 `:` 的 ID 按完整路径解析
- MOD 内容引用必须使用完整命名空间 ID

---

## 5. RON 配置中的命名空间

### 5.1 基础游戏内容 RON

```ron
// content/skills/fireball.ron
SkillDef(
    id: "base:fireball",
    name: "火球术",
    description: "发射一枚火球",
    mp_cost: 15,
    cooldown: 2,
    effects: [
        EffectDef(
            type: "base:direct_damage",
            value: 120,
        ),
    ],
)
```

### 5.2 MOD 内容 RON

```ron
// mods/community/fire_expansion/content/skills/fire_storm.ron
SkillDef(
    id: "fire_expansion:fire_storm",
    name: "火焰风暴",
    description: "释放毁灭性的火焰风暴",
    mp_cost: 40,
    cooldown: 5,
    effects: [
        EffectDef(
            type: "base:direct_damage",
            value: 300,
        ),
    ],
)
```

### 5.3 MOD 引用基础内容

```ron
// mods/community/fire_expansion/content/skills/fire_boost.ron
SkillDef(
    id: "fire_expansion:fire_boost",
    name: "火焰增幅",
    description: "增强火球术伤害",
    effects: [
        EffectDef(
            type: "base:buff",
            buff_id: "fire_expansion:flame_aura",  // 引用 MOD 自己的 Buff
            ...
        ),
    ],
    // 引用基础游戏技能进行增幅
    target_skill: "base:fireball",
)
```

### 5.4 内容引用规则

| 引用场景 | 规则 | 示例 |
|----------|------|------|
| 引用基础内容 | 使用 `base:` 前缀或省略前缀 | `skill_id: "heal"` |
| 引用 MOD 自己的内容 | 使用自己的命名空间 | `buff_id: "mod_a:custom_buff"` |
| 引用其他 MOD 内容 | 使用目标 MOD 的命名空间 | `skill_id: "mod_b:ice_shield"` |
| MOD 不可引用 `base:` 的覆盖版本 | 引用的是原始基础内容 | — |

---

## 6. Modding API 中的命名空间

### 6.1 自动命名空间注入

MOD 通过 `mod_api` 注册内容时，命名空间由 MOD 加载器自动注入：

```rust
// src/modding/api/mod_api.rs
pub fn register_skill(ctx: &mut ModContext, skill: SkillDef) -> ModResult<()> {
    // 自动为 skill.id 添加 MOD 命名空间前缀
    let namespaced_id = format!("{}:{}", ctx.namespace(), skill.id);

    // 禁止 MOD 使用 base: 命名空间
    if ctx.namespace() == "base" {
        return Err(ModError::ForbiddenNamespace {
            namespace: "base".to_string(),
        });
    }

    // 注册到 Registry
    ctx.skill_registry_mut().insert(namespaced_id, skill)?;
    Ok(())
}
```

### 6.2 命名空间约束

```rust
pub fn register_skill(ctx: &mut ModContext, skill: SkillDef) -> ModResult<()> {
    // 1. MOD 不能使用 base: 命名空间
    // 2. MOD 不能使用 official_dlc: 命名空间
    // 3. MOD 只能使用自己的命名空间
    // 4. 内容 ID 不能包含冒号

    validate_content_id(&skill.id)?;  // 检查 ID 格式

    let namespaced_id = format!("{}:{}", ctx.namespace(), skill.id);
    ctx.skill_registry_mut().insert(namespaced_id, skill)?;
    Ok(())
}
```

### 6.3 命名空间隔离

MOD 之间不能直接引用对方的内部数据：

```rust
// 🟥 禁止：MOD A 直接访问 MOD B 的内部数据
pub fn forbidden_example(mod_b_data: &InternalData) {
    // 编译时/运行时拒绝
}

// 🟩 允许：通过 Registry API 查询
pub fn allowed_example(ctx: &ModContext, skill_id: &str) -> Option<&SkillData> {
    ctx.query_skill(skill_id)  // 只读查询
}
```

---

## 7. 命名空间在内容引用中的解析

### 7.1 引用解析规则

当 RON 配置中出现引用（如技能引用 Buff、装备引用效果）时，解析规则如下：

```rust
fn resolve_reference(raw_id: &str, current_namespace: &str) -> String {
    if raw_id.contains(':') {
        // 已有命名空间前缀，直接使用
        raw_id.to_string()
    } else {
        // 无前缀：在当前上下文的命名空间中解析
        format!("{}:{}", current_namespace, raw_id)
    }
}
```

### 7.2 解析示例

| 当前上下文 | 引用 ID | 解析结果 |
|-----------|---------|---------|
| `base` | `"heal"` | `"base:heal"` |
| `mod_a` | `"custom_buff"` | `"mod_a:custom_buff"` |
| `mod_a` | `"base:fireball"` | `"base:fireball"` |
| `mod_a` | `"mod_b:ice_shield"` | `"mod_b:ice_shield"` |

### 7.3 上下文感知解析

MOD 加载器在加载 MOD 内容时，将当前 MOD 的命名空间作为上下文：

```rust
fn load_mod_content(mod_manifest: &ModManifest, content_dir: &Path) -> Result<Vec<SkillDef>> {
    let context_namespace = &mod_manifest.namespace;

    for entry in read_dir(content_dir)? {
        let skill: SkillDef = deserialize_ron(&entry.path())?;

        // 解析所有引用
        resolve_all_references(&mut skill, context_namespace);

        loaded_skills.push(skill);
    }
    Ok(loaded_skills)
}
```

---

## 8. 禁止事项

🟥 **命名空间使用禁止**：

| 禁止操作 | 原因 |
|----------|------|
| MOD 使用 `base:` 命名空间注册内容 | 篡改基础游戏内容 |
| 两个 MOD 使用相同 namespace | 命名空间冲突，内容不可区分 |
| MOD 之间通过 namespace 直接引用内部数据 | 破坏 MOD 隔离性 |
| 内容 ID 包含冒号 `:` | 与命名空间分隔符冲突 |
| 命名空间使用大写字母 | 大小写歧义 |
| MOD 覆盖 `base:` 命名空间的内容 | MOD 不应修改基础游戏内容 |

🟥 **Registry 操作禁止**：

| 禁止操作 | 原因 |
|----------|------|
| 绕过 Registry 直接写入内容 | 绕过命名空间校验 |
| 删除其他 MOD 注册的内容 | 破坏 MOD 隔离性 |
| 修改其他 MOD 注册的内容 | 破坏 MOD 数据完整性 |

🟩 **允许操作**：

| 允许操作 | 说明 |
|----------|------|
| MOD 在自己的命名空间内注册新内容 | 标准 MOD 扩展 |
| MOD 覆盖自己命名空间内的内容 | 自我更新 |
| 通过 Registry API 查询其他 MOD 的内容 | 只读查询 |
| 基础游戏新增 `base:` 命名空间内容 | 游戏更新 |

---

## 附录：命名空间完整流程

```
游戏启动
    ↓
1. 加载基础内容（content/）
   → 所有内容标记为 base: 命名空间
    ↓
2. 扫描 mods/ 目录
    ↓
3. 解析所有 manifest.ron
   → 收集命名空间声明
    ↓
4. 命名空间冲突检测
   → 拒绝重复命名空间
    ↓
5. 按 priority 顺序加载 MOD 内容
   → 自动注入 MOD 命名空间前缀
   → 解析内容引用（无前缀 → 当前命名空间）
    ↓
6. 运行校验管线
   → 引用完整性检查（命名空间是否存在）
    ↓
7. 合并到 Registry
   → 键为完整命名空间 ID
    ↓
   8. 游戏就绪
```

---

## 9. Manifest 驱动的元数据管理

> **宪法 §1.1.2（定义与实例分离）**：Manifest 和 Registry 中的命名空间注册信息均为 Definition 数据（不可变配置），运行时通过 AssetResolver 查询，不修改 Registry 本身。

> **优化来源**: `docs/其他/37.md`

### 9.1 核心原则

不依赖文件系统扫描，而是通过 `mod_manifest.ron` 声明资源和依赖。优势：
- 加载速度提升一个数量级（避免 IO 阻塞）
- 为创意工坊验证、版本兼容性检查、DLC 加密预留接口
- MOD 作者只需提供"差异化"资源，无需复制整个基础包

### 9.2 Manifest 结构

```ron
// mods/community/fire_expansion/mod_manifest.ron
(
    id: "fire_expansion",
    namespace: "fire_expansion",
    name: "火系扩展包",
    version: "1.0.0",
    description: "新增火系技能和装备",

    // 声明本 MOD 提供的资源（无需文件系统扫描）
    resources: [
        "skills/fire_storm",
        "skills/fire_boost",
        "buffs/flame_aura",
    ],

    // 依赖声明
    dependencies: [
        (namespace: "base", version_require: ">=0.8.0"),
    ],

    // 覆盖权限声明
    allowed_override_patterns: [
        "base:skills/*",       // 允许覆盖基础技能图标/特效
        "base:ui/icons/*",     // 允许覆盖 UI 图标
    ],
)
```

### 9.3 依赖校验

加载阶段校验依赖完整性：
- 依赖的 MOD 是否已加载
- 版本约束是否满足
- 依赖链是否有循环

```rust
fn validate_dependencies(mods: &[ModManifest]) -> Result<(), ModLoadError> {
    let loaded: HashMap<&str, &str> = mods.iter()
        .map(|m| (m.namespace.as_str(), m.version.as_str()))
        .collect();

    for mod_manifest in mods {
        for dep in &mod_manifest.dependencies {
            if !loaded.contains_key(dep.namespace.as_str()) {
                return Err(ModLoadError::MissingDependency {
                    mod_id: mod_manifest.id.clone(),
                    required: dep.namespace.clone(),
                });
            }
        }
    }
    Ok(())
}
```

---

## 10. 性能优化

> **优化来源**: `docs/其他/37.md`

### 10.1 Registry 分层存储

Registry 用完整字符串作为 HashMap 键时，大量 MOD 加载时高频查找有性能损耗。优化方案：

```rust
pub struct SkillRegistry {
    // 分层存储：先按 namespace 索引子 HashMap，再查 content_id
    namespaces: HashMap<String, HashMap<String, SkillData>>,
}

impl SkillRegistry {
    pub fn lookup(&self, asset_id: &AssetId) -> Option<&SkillData> {
        self.namespaces
            .get(&asset_id.namespace)?
            .get(&asset_id.name)
    }
}
```

### 10.2 Resolved Path Cache（解析路径缓存）

每次请求资源时按优先级链逐个查找有 CPU 开销。引入缓存：

```rust
#[derive(Resource)]
pub struct AssetResolver {
    /// 缓存已解析的最终路径，避免重复走 Resolution Chain
    cache: HashMap<AssetKey, ResolvedPath>,
    /// 当前激活的优先级链
    chain: Vec<Namespace>,
}

impl AssetResolver {
    pub fn resolve(&mut self, asset_id: &AssetId) -> Option<&ResolvedPath> {
        let key = AssetKey::from_asset_id(asset_id);

        // 命中缓存：O(1) 直接返回
        if let Some(cached) = self.cache.get(&key) {
            return Some(cached);
        }

        // 未命中：走 Resolution Chain，解析后写入缓存
        let resolved = self.resolve_through_chain(asset_id)?;
        self.cache.insert(key, resolved.clone());
        self.cache.get(&key)
    }

    /// MOD 启用/禁用时：清空缓存，重建 chain
    pub fn invalidate(&mut self) {
        self.cache.clear();
    }
}
```

### 10.3 Dev Mode Watcher（热重载支持）

MOD 开发时的实时反馈：
- 监听 `mods/` 目录下的文件变更
- 检测到 `mod_manifest.ron` 变更 → 自动重新加载 Manifest + 清空 Resolver Cache
- 检测到资源文件变更 → 触发 Bevy 的 `AssetEvent::Modified`

```rust
/// MOD 热重载系统（仅 dev 模式启用）
fn mod_hot_reload_watcher(
    mut asset_resolver: ResMut<AssetResolver>,
    mut mod_events: EventReader<ModFileEvent>,
) {
    for event in mod_events.read() {
        match event {
            ModFileEvent::ManifestChanged { mod_id } => {
                info!("MOD manifest changed: {}, reloading...", mod_id);
                asset_resolver.invalidate();
                // 重新加载该 MOD 的 manifest
            }
            ModFileEvent::AssetChanged { path } => {
                info!("MOD asset changed: {:?}", path);
                // 触发 Bevy AssetEvent::Modified
            }
        }
    }
}
```

---

## 11. SRPG 特殊覆盖规则

> **优化来源**: `docs/其他/37.md`

核心原则：**视觉/表现层开放覆盖，逻辑/规则层限制覆盖或仅允许扩展**。

| 场景 | 命名空间策略 | 理由 |
|------|-------------|------|
| 技能图标/特效 | ✅ 允许 MOD 覆盖 `base:skills/*` | 视觉替换是 MOD 最常见需求 |
| 战斗公式/数值 | 🟥 禁止 MOD 覆盖 `base:formulas/*` | 数值平衡是核心体验，应由本体控制；MOD 只能新增公式 |
| UI 布局/样式 | ✅ 允许覆盖，但需 Schema 验证 | UI 结构变更易导致崩溃，需严格校验 |
| 对话/剧情文本 | ✅ 允许追加，🟥 禁止覆盖本体 Key | 防止 MOD 破坏主线剧情完整性 |
| 音频/语音 | ✅ 允许覆盖，但限制文件大小 | 防止恶意 MOD 塞入巨大音频导致内存爆炸 |

这些规则需要在 Manifest Schema 中通过 `allowed_override_patterns` 字段强制执行：

```ron
// 基础游戏的覆盖权限声明
(
    override_policy: {
        // 视觉层：允许覆盖
        "base:skills/*": { allow_override: true },
        "base:ui/icons/*": { allow_override: true },
        "base:audio/*": { allow_override: true, max_file_size_mb: 10 },

        // 逻辑层：禁止覆盖
        "base:formulas/*": { allow_override: false },
        "base:dialogue/main_*": { allow_override: false },

        // 扩展层：允许追加
        "base:dialogue/*": { allow_append: true, allow_override: false },
    }
)
```

---

## 12. Resolution Chain（解析优先级链）

> **优化来源**: `docs/其他/37.md`

查找顺序明确定义为：

```
User Override > Active Mods (Priority) > Base Game
```

### 12.1 优先级链规则

| 优先级 | 来源 | 说明 |
|--------|------|------|
| 最高 | User Override（用户覆盖） | 玩家个人自定义资源 |
| 高 | Active Mods（按 priority 排序） | MOD 按 manifest 中的 priority 字段排序 |
| 最低 | Base Game（基础游戏） | 本体资源作为最终 fallback |

### 12.2 Fallback 机制

当 MOD 资源缺失时，自动回退到 base 命名空间：
- MOD 作者只需提供"差异化"资源，无需复制整个基础包
- 降低 MOD 制作门槛和体积

### 12.3 冲突诊断工具

提供 Debug 命令或 UI 面板，输入 AssetId 后显示完整的 Resolution Chain 命中过程：

```rust
pub fn diagnose_asset_resolution(asset_id: &AssetId, resolver: &AssetResolver) {
    info!("=== Asset Resolution Diagnosis: {} ===", asset_id);
    for (i, namespace) in resolver.chain.iter().enumerate() {
        match resolver.lookup_in_namespace(asset_id, namespace) {
            Some(path) => info!("  [{}] {} → FOUND: {:?}", i, namespace, path),
            None => info!("  [{}] {} → miss", i, namespace),
        }
    }
    info!("Final resolved: {:?}", resolver.resolve(asset_id));
}
```
