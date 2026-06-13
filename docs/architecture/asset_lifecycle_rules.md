# Asset Lifecycle Rules — 资源生命周期管理规范

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第三节 — 资源生命周期管理

本文档定义 Bevy 资源（Asset）的代码级生命周期管理规范，涵盖 Handle 类型选择、加载/卸载策略、引用追踪、错误降级和热重载同步。

交叉引用：
- `docs/architecture/asset-organization.md` — 资源目录组织与美术工作流
- `docs/architecture/infrastructure-design.md` — Infrastructure 层资源加载模块设计
- `docs/architecture.md` — 资源与内容生产总纲

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

### 3.2 无效引用处理

当检测到无效引用时：

```
1. 记录 ERROR 级别日志（包含资源路径和引用来源）
2. 使用 fallback 资源（占位纹理、静音音频）
3. 标记该引用为已降级
4. 不允许 crash 或 panic
```

### 3.3 引用生命周期审计

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
| 纹理（Sprite） | 1x1 品红色像素 | 明确的视觉标识 |
| 音频（SFX） | 静音 | 不影响游戏流程 |
| 音频（BGM） | 静音 | 不影响游戏流程 |
| RON 配置 | 硬编码默认值 | 保证游戏可运行 |
| 字体 | 系统默认字体 | 保证文本可显示 |

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

| 场景类型 | 内存上限 | 说明 |
|---------|---------|------|
| 主菜单 | 64 MB | 仅 UI 资源 |
| 战斗场景 | 256 MB | 地图 + 角色 + 特效 |
| 过场动画 | 128 MB | 背景 + 角色 + 音频 |

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

### 8.1 资源管理服务

建议在 `infrastructure/assets/` 中实现统一的资源管理服务：

```rust
pub struct AssetLifecycleManager {
    /// 当前场景的 Strong Handle 注册表
    scene_handles: HashMap<SceneId, Vec<HandleUntyped>>,
    /// 降级资源映射
    fallbacks: HashMap<AssetTypeId, HandleUntyped>,
    /// 内存预算配置
    memory_budget: MemoryBudget,
}
```

### 8.2 场景切换清理流程

```rust
fn cleanup_scene_assets(
    commands: Commands,
    asset_server: Res<AssetServer>,
    scene_handles: Res<SceneAssetHandles>,
) {
    // 1. 移除 Strong Handle 引用
    for handle in scene_handles.current() {
        commands.remove_asset(handle);
    }
    // 2. 触发延迟卸载（由 AssetServer 在后续帧执行）
    asset_server.unload_unused();
}
```

### 8.3 Fallback 加载器

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
