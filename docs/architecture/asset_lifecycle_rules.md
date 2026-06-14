# Asset Lifecycle Rules — 资源生命周期管理规范

Version: 2.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第三节 — 资源生命周期管理、`docs/其他/36.md` — 深度技术审查

本文档定义 Bevy 资源（Asset）的代码级生命周期管理规范，涵盖 Handle 类型选择、加载/卸载策略、引用追踪、错误降级和热重载同步。

交叉引用：
- `docs/architecture/asset-organization.md` — 资源目录组织与美术工作流
- `docs/architecture/infrastructure-design.md` — Infrastructure 层资源加载模块设计
- `docs/architecture.md` — 资源与内容生产总纲
- `docs/AI开发宪法完整版.md` — AI 开发宪法（最高约束力），本文档对应条款：1.1.2（定义与实例分离）、12.1.4（统一 Asset Pipeline）、14.0.2（资源追踪）、14.0.3（生命周期管理）、14.0.5（热重载优先）

> **优化来源**: `docs/其他/36.md` — 类型安全资源注册表、SRPG 动画生命周期、技能 VFX 延迟加载、SafeAssetRef 封装

---

## 1. Handle 类型选择

### 1.1 Strong Handle（强引用）

`Handle<T>` 默认为 Strong Handle。Bevy AssetServer 持有 Strong Handle 时，对应资源**不会被自动卸载**。

**必须使用 Strong Handle 的场景**：

- 当前战斗正在使用的地形图集、角色精灵图
- UI 主题资源（`UiTheme` 引用的颜色、字体）
- 当前关卡的地图瓦片图集
- 已加载的 Registry 配置数据（`SkillRegistry`、`BuffRegistry` 等）

**使用规则**：
- 🟩 Strong Handle 应该存储在 Resource 或 Component 中，随其宿主的生命周期管理
- 🟩 战斗场景切换时，必须显式移除当前场景的所有 Strong Handle
- 🟥 禁止在函数栈上持有 Strong Handle 后丢失引用（导致资源永远无法卸载）

### 1.2 Weak Handle（弱引用）

`Handle<T>::weak()` 创建弱引用。弱引用不阻止资源被自动卸载。

**必须使用 Weak Handle 的场景**：

- 调试面板中临时引用的资源预览
- 可选加载的资源（如 DLC 内容的缩略图）
- 历史记录中引用的已卸载资源

**使用规则**：
- 🟩 使用 Weak Handle 前必须检查资源是否仍然有效
- 🟩 Weak Handle 失效时必须有明确的降级行为（使用占位资源）

### 1.3 选择决策表

| 场景 | Handle 类型 | 理由 |
|------|------------|------|
| 战斗中使用的精灵图 | Strong | 必须保证不被卸载 |
| UI 主题字体 | Strong | 全生命周期使用 |
| 地形图集（当前关卡） | Strong | 关卡内持续使用 |
| 调试面板资源预览 | Weak | 临时引用，允许卸载 |
| 历史战斗日志中的资源引用 | Weak | 资源可能已卸载 |
| MOD 加载的额外资源 | Strong（MOD 生命周期内） | MOD 卸载时统一释放 |

---

## 2. 场景/战斗切换卸载

### 2.1 卸载时机

战斗场景切换涉及两阶段清理：

```
OnExit(AppState::InGame)
  → 阶段1：移除所有战斗相关的 Strong Handle 引用
  → 阶段2：调用 AssetServer::unload() 释放未引用的资源

OnEnter(AppState::InGame)
  → 加载新场景所需资源
```

### 2.2 分阶段卸载策略

🟥 **禁止一次性卸载所有资源**，必须分阶段执行以避免帧尖峰：

```
帧 N  ：卸载地图瓦片图集（最大内存占用）
帧 N+1：卸载角色精灵图（中等内存占用）
帧 N+2：卸载音频资源（小内存占用）
帧 N+3：卸载 UI 装饰资源
```

**规则**：
- 🟩 每帧卸载的资源总大小不超过 4MB
- 🟩 卸载顺序：大资源优先 → 中资源其次 → 小资源最后
- 🟩 卸载过程中允许渲染帧（不阻塞主线程）

### 2.3 必须卸载的资源类型

| 资源类型 | 卸载条件 | 重新加载时机 |
|---------|---------|-------------|
| 地形图集 | 离开当前关卡 | 进入新关卡时 |
| 角色精灵图 | 战斗结束 | 下次战斗需要时 |
| 音频资源 | 离开当前场景 | 进入新场景时 |
| 关卡 RON 配置 | 从 Registry 中移除 | 不自动重新加载 |
| UI 装饰 | 场景切换 | 新场景需要时 |

---

## 2A. SRPG 专项：单位动画与技能特效生命周期

> **优化来源**: `docs/其他/36.md` — 单位动画资源的 SRPG 专项管理规则、技能特效按需加载策略

### 2A.1 单位动画资源的分级管理

角色动画资源（SpriteSheet、AnimationClip）在 SRPG 中是内存大头，必须根据可见性分级管理：

| 单位状态 | Handle 类型 | 理由 |
|---------|------------|------|
| 战斗中可见单位 | Strong Handle | 必须保证动画流畅播放 |
| 战斗中不可见但存活的单位（如地图边缘） | Weak Handle | 允许卸载，重新进入视野时重新加载 |
| 已死亡单位 | 立即卸载 | 战斗结束后统一清理，释放内存 |

**实现要点**：
- 🟩 视野系统每次更新时，检查单位可见性并调整 Handle 强度
- 🟩 Weak Handle 失效时使用"站立待机"的通用帧作为降级
- 🟩 战斗结束时一次性清理所有已死亡单位的动画资源

### 2A.2 技能特效（VFX）按需加载

火球术、治疗术等技能特效（粒子、动画）**不应该在战斗开始时全部加载**。

**分阶段加载策略**：

```
战斗开始时：
  → 预加载"本回合可能使用的技能特效"（根据双方单位技能列表）
  → 不预加载所有技能特效（节省内存）

技能释放时：
  → 检查特效资源是否已加载
  → 未加载 → 触发异步加载，加载完成后播放
  → 已加载 → 直接播放

特效播放完毕后：
  → 延迟 5 秒卸载（避免频繁释放/加载同一特效）
  → 卸载时从 SceneAssets.skill_vfx 中移除
```

🟥 **禁止**：战斗开始时预加载所有技能特效。SRPG 可能有 100+ 技能，全部加载会超出内存预算。

> **优化来源**: `docs/其他/36.md` — 技能特效资源的按需加载、预加载策略、延迟卸载规则

### 2.4 必须保留的资源类型

| 资源类型 | 保留条件 | 理由 |
|---------|---------|------|
| UI 主题 | 全局 | 所有场景共享 |
| 字体 | 全局 | 全局使用 |
| Registry 配置 | 直到替换 | 数据驱动核心 |

---

## 3. 引用追踪

### 3.1 引用有效性验证

🟥 **所有资源引用在使用前必须验证有效性**。

**规则**：
- 🟩 加载资源后立即验证 Handle 是否有效
- 🟩 使用 Weak Handle 前必须调用 `asset_server.is_loaded_with_dependencies(handle)`
- 🟥 禁止使用已知失效的 Handle

### 3.2 SafeAssetRef 封装

> **优化来源**: `docs/其他/36.md` — 重复调用 is_loaded_with_dependencies 的性能开销，推荐 SafeAssetRef 封装

Weak Handle 的有效性检查需要遍历依赖树，在性能敏感路径（如每帧渲染）中调用有开销。引入资源引用封装器：

```rust
/// 类型安全的资源引用包装器，缓存有效性状态避免重复查询 AssetServer
pub struct SafeAssetRef<T: Asset> {
    handle: Handle<T>,
    is_valid: bool,
}

impl<T: Asset> SafeAssetRef<T> {
    pub fn new(handle: Handle<T>, asset_server: &AssetServer) -> Self {
        Self {
            handle,
            is_valid: asset_server.is_loaded_with_dependencies(&handle),
        }
    }

    pub fn get(&self) -> Option<&Handle<T>> {
        if self.is_valid { Some(&self.handle) } else { None }
    }

    /// 每 N 帧重新验证一次有效性（避免过期缓存）
    pub fn revalidate(&mut self, asset_server: &AssetServer) {
        self.is_valid = asset_server.is_loaded_with_dependencies(&self.handle);
    }
}
```

### 3.3 Debug 引用审计

> **优化来源**: `docs/其他/36.md` — 场景卸载前 assert_no_dangling_handles!() 断言

🟩 **Debug 构建中，场景卸载前必须执行悬挂引用断言**：

```rust
fn cleanup_scene_assertions(scene_assets: Res<SceneAssets>, asset_server: Res<AssetServer>) {
    #[cfg(debug_assertions)]
    {
        for handle in &scene_assets.tilemap_atlases {
            assert!(
                asset_server.is_loaded_with_dependencies(handle),
                "Dangling handle detected before scene unload: {:?}",
                handle
            );
        }
        // ... 检查所有类型的资源
    }
}
```

**审计日志格式**：

```text
[ASSET_AUDIT] Scene unload: 12 Strong Handles released
[ASSET_AUDIT]   - tilemap_atlases: 3 handles (12.5 MB)
[ASSET_AUDIT]   - character_sprites: 8 handles (8.2 MB)
[ASSET_AUDIT]   - audio_handles: 1 handles (0.3 MB)
```

### 3.4 无效引用处理

当检测到无效引用时：

```
1. 记录 ERROR 级别日志（包含资源路径和引用来源）
2. 使用 fallback 资源（占位纹理、静音音频）
3. 标记该引用为已降级
4. 不允许 crash 或 panic
```

### 3.5 引用生命周期审计

🟩 建议在 Debug 构建中启用引用审计：

- 每次创建 Strong Handle 时记录引用来源
- 每次销毁 Strong Handle 时验证来源
- 战斗结束时打印所有未释放的 Strong Handle

---

## 4. 加载失败策略

### 4.1 降级策略

🟥 **资源加载失败时绝对禁止 crash 或 panic**。

**降级优先级**：

```
1. 使用 fallback 资源（占位纹理、静音音频）
2. 记录 ERROR 级别日志
3. 标记资源为降级状态
4. 继续游戏运行
```

### 4.2 Fallback 资源规范

| 资源类型 | Fallback 资源 | 说明 |
|---------|--------------|------|
| 纹理（Sprite） | 32x32 品红色棋盘格（Magenta Checkerboard） | 明确的视觉标识，一眼可辨认缺失资源 |
| 音频（SFX） | 静音 | 不影响游戏流程 |
| 音频（BGM） | 静音 | 不影响游戏流程 |
| RON 配置 | 硬编码默认值 | 保证游戏可运行 |
| 字体 | 系统默认字体 | 保证文本可显示 |
| 动画（AnimationClip） | 单帧静态姿势 | 保持单位可见性，避免透明消失 |

### 4.3 重试策略

- 🟩 首次加载失败后重试 1 次（延迟 100ms）
- 🟩 重试仍失败则使用 Fallback
- 🟥 禁止无限重试
- 🟩 重试日志使用 WARN 级别，最终降级使用 ERROR 级别

---

## 5. 热重载同步

### 5.1 热重载边界

| 数据类型 | 可热重载 | 理由 |
|---------|---------|------|
| Definition（RON 配置） | ✅ | 不可变数据，安全替换 |
| 二进制资源（图片/音频） | ✅ | 无业务影响 |
| Instance（运行时状态） | 🟥 | 会导致状态不一致 |
| 战斗中任何数据 | 🟥 | 破坏游戏确定性 |

### 5.2 格式/大小变化处理

当热重载的资源格式或大小发生变化时：

```
1. 验证新资源格式合法性
2. 更新 AssetServer 内部缓存
3. 通知所有依赖该资源的系统重新读取
4. 如果新资源与旧资源不兼容，回退到旧版本并记录 WARN 日志
```

**规则**：
- 🟩 热重载前验证新资源的格式和大小
- 🟩 热重载失败时回退到上次有效状态
- 🟥 禁止热重载未验证的数据

### 5.3 热重载通知机制

```
资源变更 → AssetServer 检测 → 触发 AssetChanged 事件
  → 依赖系统响应 → 重新读取资源 → 更新内部状态
```

---

## 6. 内存预算

### 6.1 每场景内存上限

> **优化来源**: `docs/其他/36.md` — 设备差异化内存预算、数值合理性依据

**默认内存预算**（基于主流 PC 设备 8GB+ RAM、独立显卡）：

| 场景类型 | 内存上限 | 说明 |
|---------|---------|------|
| 主菜单 | 64 MB | 仅 UI 资源 |
| 战斗场景 | 256 MB | 地图 + 角色 + 特效 |
| 过场动画 | 128 MB | 背景 + 角色 + 音频 |

**设备差异化适配**：

| 设备类型 | 战斗场景上限 | 主菜单上限 | 说明 |
|---------|------------|-----------|------|
| PC（8GB+ RAM） | 256 MB | 64 MB | 默认配置 |
| PC（4GB RAM） | 192 MB | 48 MB | 低配 PC |
| 移动端（高端） | 192 MB | 48 MB | 旗舰手机/平板 |
| 移动端（低端） | 128 MB | 32 MB | 入门设备 |

🟥 **禁止**：在低端设备上使用 PC 级别的内存预算。通过 `DeviceProfile` Resource 在启动时检测设备能力并设置对应预算。

**数值依据**：
- 4MB/帧卸载上限：基于 16ms 帧时间（60fps），确保卸载操作不超过一帧的 25%
- 256MB 战斗上限：基于 8GB RAM 设备扣除系统/引擎占用后的可用余量

### 6.2 大型地图流式加载

超过 50MB 的地图资源必须使用流式加载：

- 🟩 地图分为 Chunk，按需加载可视区域的 Chunk
- 🟩 不可视区域的 Chunk 优先卸载
- 🟩 流式加载在后台线程执行，不阻塞主线程

### 6.3 内存监控

🟩 Debug 构建中启用内存监控：

- 每 5 秒采样一次资源内存占用
- 超过阈值时记录 WARN 日志
- 调试面板中显示实时内存占用

---

## 7. 禁止事项

- 🟥 **持有未使用的 Strong Handle 超出场景生命周期**（导致内存泄漏）
- 🟥 **启动时加载所有资源**（应按需加载，使用流式加载）
- 🟥 **缺失资源导致 crash**（必须降级，不允许 panic）
- 🟥 **绕过 AssetServer 直接读文件**（破坏热重载和生命周期管理）
- 🟥 **资源路径硬编码**（必须通过配置或 Registry 引用）
- 🟥 **忽略资源加载错误**（必须记录日志并降级）
- 🟥 **一次性卸载所有资源**（分阶段卸载，避免帧尖峰）
- 🟥 **战斗中热重载**（BattleInProgress 状态下禁止）
- 🟥 **热重载 Instance 数据**（只允许热重载 Definition）

---

## 8. 实现备注

### 8.1 类型安全的资源注册表

> **优化来源**: `docs/其他/36.md` — HandleUntyped 丢失类型信息，推荐类型安全的 SceneAssets 替代

🟥 **禁止使用 `HandleUntyped` 统一管理不同类型的资源**。`HandleUntyped` 丢失了类型信息，在卸载时需要手动转换回 `Handle<T>`，容易出错。

**正确方案**：使用类型安全的资源注册表：

```rust
#[derive(Resource)]
pub struct SceneAssets {
    /// 类型安全，编译期检查
    pub tilemap_atlases: Vec<Handle<TextureAtlas>>,
    pub character_sprites: Vec<Handle<Image>>,
    pub audio_handles: Vec<Handle<AudioSource>>,
    pub skill_vfx: HashMap<SkillId, Vec<Handle<AnimationClip>>>,
}

impl SceneAssets {
    /// 卸载时直接对特定类型资源进行批量操作，避免类型转换错误
    pub fn unload_all(&self, asset_server: &AssetServer) {
        for handle in &self.tilemap_atlases {
            asset_server.unload(handle);
        }
        for handle in &self.character_sprites {
            asset_server.unload(handle);
        }
        // ...
    }
}
```

### 8.2 延迟卸载队列

> **优化来源**: `docs/其他/36.md` — unload_unused() 同步阻塞问题，AssetUnloadQueue 分帧卸载

🟥 **禁止在场景切换时同步调用 `asset_server.unload_unused()`**。如果一次性释放大量资源，依然会造成帧卡顿。

**正确方案**：使用延迟卸载队列，分帧执行：

```rust
#[derive(Resource)]
pub struct AssetUnloadQueue {
    /// 使用 Bevy 0.18 的 UntypedHandle（非已废弃的 HandleUntyped）
    handles_to_unload: Vec<UntypedHandle>,
    max_per_frame: usize, // 每帧最多卸载 4 个资源
}

impl AssetUnloadQueue {
    pub fn enqueue<T: Asset>(&mut self, handle: Handle<T>) {
        self.handles_to_unload.push(handle.untyped());
    }

    pub fn is_empty(&self) -> bool {
        self.handles_to_unload.is_empty()
    }
}

/// 在 PostUpdate 阶段，每帧卸载一部分资源
fn process_unload_queue(
    mut queue: ResMut<AssetUnloadQueue>,
    asset_server: Res<AssetServer>,
) {
    let drain_count = queue.max_per_frame.min(queue.handles_to_unload.len());
    let to_unload: Vec<_> = queue.handles_to_unload.drain(..drain_count).collect();
    for handle in to_unload {
        asset_server.unload(handle);
    }
}
```

### 8.3 场景切换清理流程

```rust
fn cleanup_scene_assets(
    mut commands: Commands,
    mut unload_queue: ResMut<AssetUnloadQueue>,
    scene_assets: Res<SceneAssets>,
) {
    // 1. Debug 模式下执行悬挂引用断言
    #[cfg(debug_assertions)]
    assert_no_dangling_handles(&scene_assets);

    // 2. 将 Strong Handle 加入延迟卸载队列
    for handle in &scene_assets.tilemap_atlases {
        unload_queue.enqueue(handle.clone());
    }
    for handle in &scene_assets.character_sprites {
        unload_queue.enqueue(handle.clone());
    }
    // 3. 由 process_unload_queue 在后续帧逐步卸载
}
```

### 8.4 Fallback 加载器

```rust
fn load_with_fallback<T: Asset>(
    asset_server: &AssetServer,
    path: &Path,
    fallback_path: &Path,
) -> Handle<T> {
    match asset_server.load(path) {
        handle if handle.is_some() => handle,
        _ => {
            error!("Failed to load {}, using fallback", path.display());
            asset_server.load(fallback_path)
        }
    }
}
```

---

## 9. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `asset-organization.md` | 本文档定义代码级管理，该文档定义目录级组织 |
| `infrastructure-design.md` | 本文档的规则在 `infrastructure/assets/` 中实现 |
| `performance_budget.md` | 内存预算规则是性能预算的子集 |
| `hot_reload_rules.md` | 热重载的资源层面规则在本文档定义 |
| `architecture.md` | 本文档是"资源与内容生产"章节的详细补充 |

---

## 附录 A：常见错误代码示例（反面教材）

> **优化来源**: `docs/其他/36.md` — 常见错误代码示例，帮助开发者快速识别反模式

```rust
// ❌ 错误 1：在函数栈上持有 Strong Handle 后丢失
fn bad_example(asset_server: Res<AssetServer>) {
    let handle = asset_server.load("map.png"); // Strong Handle
    // 函数结束，handle 被 drop，但 AssetServer 仍持有引用
    // 资源永远不会被卸载！导致内存泄漏！
}

// ✅ 正确：将 Handle 存储在 Resource 或 Component 中
#[derive(Component)]
struct MapComponent {
    tilemap: Handle<TextureAtlas>,
}

// ❌ 错误 2：使用 HandleUntyped 统一管理
struct BadAssetManager {
    handles: Vec<HandleUntyped>, // 丢失类型信息，卸载时需手动转换
}

// ✅ 正确：使用类型安全的 SceneAssets
struct GoodAssetManager {
    images: Vec<Handle<Image>>,
    audio: Vec<Handle<AudioSource>>,
}

// ❌ 错误 3：在 Hook 中卸载资源（Hook 中禁止阻塞操作）
impl Dead {
    fn on_add_dead(mut world: DeferredWorld, _ctx: HookContext) {
        // 🟥 禁止：Hook 中同步卸载资源
        // world.resource::<AssetServer>().unload(handle);
    }
}
```

---

## 附录 B：调试工具清单

> **优化来源**: `docs/其他/36.md` — 调试工具清单，提升开发期资源管理可见性

| 工具 | 用途 | 实现方式 |
|------|------|---------|
| 资源内存监控面板 | 实时显示各类资源的内存占用 | `egui` 面板，每 5 秒采样 |
| Handle 引用计数查看器 | 显示每个资源的 Strong/Weak 引用数量 | 自定义 `AssetDebugInfo` Resource |
| 卸载队列可视化 | 显示当前待卸载的资源列表和进度 | `AssetUnloadQueue` 的 UI 反映 |
| 热重载测试工具 | 模拟资源文件变更，测试降级和回退逻辑 | 开发模式专用 System |
| 引用审计日志 | 场景切换时输出 Handle 释放明细 | `#[cfg(debug_assertions)]` 条件编译 |
