---
id: 04-data.asset-lifecycle-rules
title: Asset Lifecycle Rules
status: draft
owner: feature-developer
created: 2026-06-14
updated: 2026-06-14
tags:
  - data
---

# 资源生命周期管理领域

Version: 2.0 [NEW since v2.0]
Status: Proposed

资源生命周期管理领域管理游戏资源的加载、使用、引用追踪、卸载和降级的完整周期。

核心原则：
- 🟩 **14.0.3 生命周期管理**：所有资源的生命周期必须显式管理（宪法条款 14.0.3）
- 🟩 **14.0.2 资源追踪**：所有资源加载必须可追踪（宪法条款 14.0.2）
- 🟩 **12.1.4 资源管理**：所有配置资源必须通过统一的 Asset Pipeline 管理（宪法条款 12.1.4）
- 🟩 **14.0.5 热重载优先**：高频修改的资源必须优先支持热重载（宪法条款 14.0.5）
- Strong Handle 生命周期与宿主一致，Weak Handle 使用前必须校验
- 场景切换必须分阶段卸载，禁止一次性清空
- 资源加载失败必须降级，禁止崩溃

---

# 术语定义

## 资源生命周期（Asset Lifecycle）

游戏资源从加载到卸载的完整周期，包括加载、引用持有、使用、释放四个阶段。

不是资源本身。不是资源格式。

关键属性：
- 资源通过 AssetServer 加载，产生 Handle 引用
- Strong Handle 持有时资源不被卸载
- 所有 Handle 释放后资源自动卸载
- 场景切换时必须显式清理当前场景的 Strong Handle

---

## 强引用句柄（Strong Handle）

阻止资源被自动卸载的引用句柄。Bevy AssetServer 持有 Strong Handle 时，对应资源不会被卸载。

不是资源数据。不是文件路径。

关键属性：
- 存储在 Resource 或 Component 中，随宿主的生命周期管理
- 必须使用场景：战斗中的精灵图、UI 主题资源、当前关卡地图瓦片图集、已加载的 Registry 配置数据
- 战斗场景切换时必须显式移除所有 Strong Handle
- 禁止在函数栈上持有 Strong Handle 后丢失引用

---

## 弱引用句柄（Weak Handle）

不阻止资源被自动卸载的引用句柄，使用前必须校验资源是否仍然有效。

不是强引用。不是空指针。

关键属性：
- 通过 Handle::weak() 创建
- 必须使用场景：调试面板临时资源预览、可选加载的 DLC 内容、历史记录中的资源引用
- 使用前必须调用 is_loaded_with_dependencies 验证有效性
- 失效时必须有降级行为（使用占位资源）

---

## 场景切换卸载（Scene Transition Unload）

离开战斗或场景时分阶段释放资源的策略，避免帧尖峰。

不是一次性清空。不是加载。

关键属性：
- 触发时机：OnExit(AppState::InGame)
- 卸载顺序：大资源（地图瓦片）→ 中资源（角色精灵图）→ 小资源（音频、UI 装饰）
- 每帧卸载总量不超过 4MB
- 卸载过程中允许渲染帧，不阻塞主线程

---

## 热重载同步（Hot Reload Sync）

Definition 资源热重载时的状态同步，只更新定义态，不修改运行时 Instance。

不是加载。不是修改 Instance。

关键属性：
- 可热重载：Definition（RON 配置）、二进制资源（图片/音频）
- 禁止热重载：Instance（运行时状态）、战斗中任何数据
- 热重载失败时回退到上次有效状态并记录 WARN 日志
- 触发 AssetChanged 事件通知依赖系统重新读取

---

## 资源降级（Asset Degradation）

资源加载失败时使用替代资源继续运行，确保游戏不崩溃。

不是崩溃。不是静默跳过。

关键属性：
- 降级优先级：重试一次（延迟 100ms）→ 使用 Fallback 资源 → ERROR 日志 → 标记降级 → 继续运行
- Fallback 资源：纹理用 32x32 品红色棋盘格（Magenta Checkerboard）、音频用静音、RON 用硬编码默认值、字体用系统默认

> **优化来源**: docs/01-architecture/asset_lifecycle_rules.md — 品红色棋盘格规则：明确的视觉标识，一眼可辨认缺失资源
- 禁止无限重试
- 重试日志用 WARN 级别，最终降级用 ERROR 级别

---

## 流式加载（Streaming Load）

大地图资源（>50MB）的块状后台加载策略，按需加载可视区域。

不是一次性全加载。

关键属性：
- 地图分为 Chunk，按需加载可视区域
- 不可视区域的 Chunk 优先卸载
- 流式加载在后台线程执行，不阻塞主线程
- 仅适用于超过 50MB 的地图资源

---

## 固定内存预算

每种场景类型有固定且不可突破的内存上限：MainMenu 64MB / Battle 256MB / Cutscene 128MB。

不是建议值。不是无限制。

关键属性：
- MainMenu 场景上限 64MB
- Battle 场景上限 256MB
- Cutscene 场景上限 128MB
- 移动端 Battle 场景上限可下调至 192MB（参见规则8）

---

## AssetUnloadQueue

每帧按固定额度 drain 的卸载队列，用于分帧释放资源，避免帧尖峰。

不是一次性清空。不是即时卸载。

关键属性：
- 维护待卸载 Handle 列表，按资源大小排序
- 每帧 drain 固定数量或固定大小的 Handle
- 每帧卸载总量不超过 4MB，每帧最多卸载 4 个资源
- 大资源优先 drain（大资源→中资源→小资源）
- 在 PostUpdate 阶段逐帧处理

> **优化来源**: docs/01-architecture/asset_lifecycle_rules.md — AssetUnloadQueue 分帧卸载实现细节

---

## SafeAssetRef\<T\>

Weak Handle 有效性缓存包装器，将 is_loaded_with_dependencies 检查结果缓存为布尔标志。

不是裸 Weak Handle。不是 Strong Handle。

关键属性：
- 内部持有 Handle\<T\> 和 is_valid 布尔缓存
- 构造时一次性校验有效性，后续访问零开销
- get() 返回 Option\<&Handle\<T\>\>，失效时返回 None
- 每 N 帧重新验证一次有效性（避免过期缓存导致使用已卸载资源）
- 渲染等高频路径使用此包装避免重复校验

> **优化来源**: docs/01-architecture/asset_lifecycle_rules.md — SafeAssetRef 重新验证机制，避免缓存过期

---

# 领域边界

## 本领域负责

- Handle 类型选择（Strong / Weak）和使用规则
- 场景切换时的分阶段资源卸载
- 引用有效性验证和无效引用处理
- 资源加载失败的降级策略（重试 → Fallback → ERROR 日志）
- 热重载同步（Definition 更新，Instance 不可变）
- 每场景内存预算管理（Menu 64MB, Battle 256MB, Cutscene 128MB）
- 大地图流式加载策略

## 本领域不负责

- 资源目录组织（由 Asset Organization 规范负责）
- 资源加载的具体实现（由 Infrastructure 层负责）
- RON 配置文件的格式定义（由 Content Pipeline 领域负责）
- 战斗中的属性修改（由 Attribute Modifier 领域负责）
- 回放系统中的资源引用（由 Replay 领域负责）
- MOD 资源的生命周期（由 Modding System 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 资源卸载完成 | Message | UI / Debug 领域 |
| 热重载触发 | Message | Content Pipeline 领域 |
| 内存超阈值警告 | Message（WARN 日志） | Debug 领域 |
| 资源降级通知 | Message | UI 领域（显示降级提示） |
| 场景切换清理 | 函数调用 | Battle / Turn 领域 |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Unloaded | 资源未加载 | Loading |
| Loading | 资源正在加载 | Loaded, Failed |
| Loaded | 资源已加载，可使用 | InUse, Unloading |
| InUse | 资源被 Strong Handle 引用 | Loaded, Unloading |
| Failed | 加载失败，已降级 | Unloaded |
| Unloading | 正在卸载 | Unloaded |

## 状态转换图

```
Unloaded → Loading → Loaded → InUse
                ↓         ↓        ↓
             Failed    Unloading  Unloading
                          ↓         ↓
                      Unloaded   Unloaded
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Unloaded | Loading | AssetServer.load() 调用 |
| Loading | Loaded | 资源加载成功 |
| Loading | Failed | 加载失败且 Fallback 也失败 |
| Loaded | InUse | Strong Handle 持有 |
| Loaded | Unloading | 所有 Handle 释放或场景切换 |
| InUse | Loaded | Strong Handle 释放（非场景切换） |
| InUse | Unloading | 场景切换，显式移除所有 Handle |
| Failed | Unloaded | Fallback 资源被替换或卸载 |

---

# 不变量

## 不变量1：每帧卸载资源总量 ≤ 4MB

任意时刻：

场景切换卸载过程中，每帧卸载的资源总大小不超过 4MB。

违反表现：

单帧卸载超过 4MB 导致帧尖峰，游戏卡顿。

---

## 不变量2：Strong Handle 生命周期与宿主一致

任意时刻：

存储在 Resource 或 Component 中的 Strong Handle，必须随宿主（Resource/Component）的生命周期管理。宿主销毁时对应的 Strong Handle 必须被移除。

违反表现：

Strong Handle 在宿主销毁后仍被持有，资源永远无法被卸载，内存泄漏。

---

## 不变量3：热重载只更新 Definition，不修改 Instance

热重载发生时：

Definition（RON 配置）可以被热重载更新。Instance（运行时状态）禁止被热重载修改。

违反表现：

战斗中的运行时状态被热重载覆盖，游戏状态不一致，确定性破坏。

---

## 不变量4：场景类型内存预算不可突破

任意时刻：

每种场景类型的内存占用不得超过其预算上限（Menu 64MB, Battle 256MB, Cutscene 128MB）。

违反表现：

内存超预算导致系统级 OOM 或严重卡顿。

---

# 业务规则

## 规则1：Handle 类型选择

禁止：
- 在函数栈上持有 Strong Handle 后丢失引用（导致资源永远无法卸载）
- 使用 Weak Handle 前不校验有效性
- 使用已知失效的 Handle

必须：
- 战斗中使用的精灵图、UI 主题、地形图集使用 Strong Handle
- 调试面板、可选加载内容使用 Weak Handle
- Weak Handle 失效时使用 Fallback 资源

允许：
- Debug 构建中启用引用审计日志

---

## 规则2：场景切换卸载

必须：
- 战斗场景切换时显式移除所有当前 Strong Handle
- 卸载顺序：大资源 → 中资源 → 小资源
- 分阶段卸载，每帧不超过 4MB

禁止：
- 一次性卸载所有资源
- 卸载过程中阻塞主线程
- 跳过 Strong Handle 的显式移除

允许：
- 卸载过程中允许渲染帧
- 使用流式加载策略处理大地图

---

## 规则3：加载失败降级

必须：
- 首次加载失败后重试 1 次（延迟 100ms）
- 重试仍失败则使用 Fallback 资源
- 记录 ERROR 级别日志
- 标记资源为降级状态
- 继续游戏运行

禁止：
- 资源加载失败直接崩溃或 panic
- 无限重试
- 忽略资源加载错误

允许：
- 使用 Fallback 资源（占位纹理、静音音频、硬编码默认值）

---

## 规则4：热重载管理

必须：
- 热重载前验证新资源的格式和大小
- 热重载失败时回退到上次有效状态
- 仅在战斗外进行热重载

禁止：
- 战斗中热重载（BattleInProgress 状态下）
- 热重载 Instance 数据
- 热重载未验证的数据

允许：
- Definition（RON 配置）热重载
- 二进制资源（图片/音频）热重载

---

## 规则5：内存预算控制

必须：
- 每场景类型内存占用不超过预算上限
- 大地图（>50MB）使用流式块加载
- 不可视区域 Chunk 优先卸载

禁止：
- 启动时加载所有资源
- 不可视区域常驻内存
- 绕过 AssetServer 直接读文件

允许：
- Debug 构建中启用内存监控（每 5 秒采样）
- 调试面板显示实时内存占用

---

## 规则5b：资源加载可追踪

> 🟩 **宪法条款 14.0.2**：所有资源加载必须可追踪

允许：
- 资源加载时记录日志（路径、类型、耗时）
- 资源引用计数可查询
- 资源状态可通过 Debug 面板查看

禁止：
- 🟥 资源加载无日志记录
- 🟥 资源引用不可追踪

必须：
- 资源加载时输出 DEBUG 级别日志（记录路径和类型）
- 资源卸载时输出 DEBUG 级别日志
- 资源引用计数可通过 Resource 查询

---

## 规则5c：热重载优先

> 🟩 **宪法条款 14.0.5**：高频修改的资源必须优先支持热重载

允许：
- Definition（RON 配置）热重载（战斗外）
- 二进制资源（图片/音频）热重载
- 热重载失败时回退到上次有效状态

禁止：
- 🟥 战斗中热重载任何资源
- 🟥 热重载 Instance 数据
- 🟥 热重载未验证的数据

必须：
- 高频修改的资源（RON 配置）必须支持热重载
- 热重载时输出 INFO 级别日志（记录变更的资源）
- 热重载失败时输出 ERROR 级别日志

---

## 规则5d：统一资源管线

> 🟩 **宪法条款 12.1.4**：所有配置资源必须通过统一的 Asset Pipeline 管理

允许：
- 所有资源通过 AssetServer 加载
- 资源加载使用统一的 Handle 类型（Strong/Weak）
- 资源生命周期通过 Handle 引用计数管理

禁止：
- 🟥 绕过 AssetServer 直接读文件
- 🟥 资源加载不通过 Handle 管理

必须：
- 所有资源通过 AssetServer::load() 加载
- 资源引用使用 Handle 类型
- 资源生命周期通过 Handle 引用计数管理

---

## 规则6：SRPG 资源生命周期专项 [NEW since v2.0]

> **优化来源**: docs/01-architecture/asset_lifecycle_rules.md — SRPG 动画分级管理、技能 VFX 按需加载策略

必须：
- 战斗中可见单位的动画资源使用 Strong Handle
- 离屏但存活单位的动画资源使用 Weak Handle（重新进入视野时重新加载）
- 已死亡单位的动画资源立即卸载（战斗结束后统一清理）
- 视野系统每次更新时，检查单位可见性并调整 Handle 强度
- Weak Handle 失效时使用"站立待机"的通用帧作为降级
- 技能 VFX 按战场技能列表预加载，未在列表中的技能不预加载
- 技能 VFX 异步加载完成后播放，播放完毕延迟 5 秒卸载

禁止：
- 战斗开始时预加载所有技能 VFX
- 已死亡单位的动画资源常驻内存

允许：
- 技能 VFX 使用 SafeAssetRef\<T\> 包装减少重复校验
- 流式加载配合 SRPG 大地图使用

---

## 规则7：MOD 资源命名空间隔离 [NEW since v2.0]

必须：
- 所有 MOD 资源路径带命名空间前缀（如 mod_xxx/fireball.png）
- MOD 资源加载走 AssetResolver 的 Resolution Chain
- MOD 资源失效时回退到 base 命名空间

禁止：
- MOD 资源路径与 base 资源路径冲突（不带命名空间前缀）
- MOD 资源绕过 AssetServer 直接读文件

允许：
- MOD 资源使用 Strong Handle（随 MOD 生命周期管理）
- MOD 资源卸载随 MOD 禁用/卸载触发

---

## 规则8：设备差异化内存预算适配 [NEW since v2.0]

> **优化来源**: docs/01-architecture/asset_lifecycle_rules.md — 设备差异化内存预算、DeviceProfile 检测机制

必须：
- 通过 DeviceProfile Resource 在启动时检测设备能力并设置对应预算
- 移动端 Battle 场景内存上限下调至 192MB
- 移动端根据设备内存等级调整预算（低配 128MB / 中配 192MB / 高配 256MB）
- 移动端离屏单位动画资源优先使用 Weak Handle

设备差异化预算表：

| 设备类型 | 战斗场景上限 | 主菜单上限 | 说明 |
|---------|------------|-----------|------|
| PC（8GB+ RAM） | 256 MB | 64 MB | 默认配置 |
| PC（4GB RAM） | 192 MB | 48 MB | 低配 PC |
| 移动端（高端） | 192 MB | 48 MB | 旗舰手机/平板 |
| 移动端（低端） | 128 MB | 32 MB | 入门设备 |

数值依据：
- 4MB/帧卸载上限：基于 16ms 帧时间（60fps），确保卸载操作不超过一帧的 25%
- 256MB 战斗上限：基于 8GB RAM 设备扣除系统/引擎占用后的可用余量

禁止：
- 移动端使用 PC 端相同的内存预算上限
- 移动端不区分设备等级
- 低端设备上使用 PC 级别的内存预算

允许：
- 移动端通过 DeviceTier 配置动态调整预算
- 低端设备启用更积极的卸载策略

---

# 流程管线

## 场景切换卸载管线（Scene Transition Unload Pipeline）

```
OnExit(AppState::InGame) → 移除 Strong Handle → 分阶段卸载 → 清空完成
```

### Step1：移除 Strong Handle 引用

输入：当前场景的所有 Strong Handle 列表
处理：遍历并移除所有战斗相关的 Strong Handle 引用
输出：Strong Handle 列表清空
禁止：在移除过程中修改其他场景的 Handle

### Step2：分阶段卸载

输入：待卸载的资源列表（按大小排序）
处理：每帧卸载一批资源，总量不超过 4MB
输出：资源逐帧释放
禁止：单帧卸载超过 4MB、跳过排序直接卸载

### Step3：清空完成

输入：所有资源已释放
处理：触发 Unloaded 事件，通知依赖系统
输出：内存回收完成
禁止：在卸载完成前加载新资源

---

## 资源加载降级管线（Asset Load Degradation Pipeline）

```
加载请求 → 首次加载 → 失败? → 重试(100ms) → 失败? → Fallback → ERROR 日志 → 标记降级 → 继续
```

### Step1：首次加载

输入：资源路径
处理：调用 AssetServer.load()
输出：Handle 或 加载失败
禁止：跳过 AssetServer 直接读文件

### Step2：重试

输入：首次加载失败
处理：延迟 100ms 后重试 1 次
输出：Handle 或 重试失败
禁止：无限重试、不记录 WARN 日志

### Step3：Fallback 降级

输入：重试失败
处理：使用对应类型的 Fallback 资源，记录 ERROR 日志，标记降级状态
输出：Fallback Handle
禁止：崩溃或 panic、静默跳过

---

# 数据结构

## Strong Handle（强引用句柄）

职责：持有资源的强引用，阻止资源被自动卸载

结构：
- 内部持有 AssetServer 的引用计数
- 关联资源类型 T
- 宿主为 Resource 或 Component

要求：
- 宿主销毁时必须显式移除
- 战斗场景切换时必须移除所有当前场景的 Handle
- 禁止在函数栈上持有后丢失引用

---

## Weak Handle（弱引用句柄）

职责：持有资源的弱引用，不阻止资源卸载

结构：
- 内部持有资源 ID 的引用
- 使用前必须校验有效性
- 失效时返回 Fallback Handle

要求：
- 使用前必须调用 is_loaded_with_dependencies
- 失效时必须有降级行为
- 禁止使用已知失效的 Handle

---

## MemoryBudget（内存预算配置）

职责：定义每种场景类型的内存上限

结构：
- Menu（主菜单）：64 MB
- Battle（战斗场景）：256 MB
- Cutscene（过场动画）：128 MB

要求：
- 内存占用不得超过对应场景类型的预算
- Debug 构建中启用监控
- 超阈值时记录 WARN 日志

---

## FallbackResourceMap（降级资源映射）

职责：记录每种资源类型的 Fallback 资源路径

结构：
- 纹理（Sprite）：32x32 品红色棋盘格（Magenta Checkerboard）
- 音频（SFX/BGM）：静音
- RON 配置：硬编码默认值
- 字体：系统默认字体

要求：
- Fallback 资源必须已加载到 AssetServer
- 降级时使用对应类型的 Fallback
- 降级状态必须记录到日志

---

# 禁止事项

禁止：资源加载失败直接崩溃或 panic

原因：资源加载失败是可恢复错误，崩溃会导致玩家体验中断。

违反后果：游戏崩溃，玩家丢失进度，负面评价。

---

禁止：启动时加载所有资源

原因：启动时全量加载会导致启动时间过长，且占用大量内存。

违反后果：启动时间从秒级增加到分钟级，内存占用远超预算。

---

禁止：一次性卸载所有资源

原因：单帧卸载大量资源会导致帧尖峰，游戏卡顿。

违反后果：卸载帧耗时激增，玩家感知到明显卡顿。

---

禁止：持有未使用的 Strong Handle 超出场景生命周期

原因：Strong Handle 未释放导致资源永远无法被卸载，造成内存泄漏。

违反后果：内存持续增长，最终导致系统级 OOM。

---

禁止：绕过 AssetServer 直接读文件

原因：直接读文件绕过资源管理框架，破坏热重载和生命周期管理。

违反后果：热重载失效，资源无法被正确卸载，内存泄漏。

---

禁止：热重载战斗中的 Instance 数据

原因：战斗中的 Instance 数据是确定性执行的一部分，热重载会破坏游戏状态一致性。

违反后果：回放失败，战斗结果不可复现，Bug 无法稳定复现。

---

禁止：忽略资源加载错误

原因：静默跳过加载失败会导致后续使用空 Handle，引发未定义行为。

违反后果：渲染空纹理、播放空音频、读取空配置，游戏表现异常。

---

禁止：资源路径硬编码

原因：硬编码路径导致资源组织变更时需要修改代码，违反 Rule/Content 分离。

违反后果：每次资源路径变更都需要修改代码，无法支持 MOD 和热重载。

---

禁止：资源加载无日志记录

原因：违反宪法条款 14.0.2，所有资源加载必须可追踪。

违反后果：资源加载问题无法排查，调试困难。

---

禁止：绕过统一资源管线直接读文件

原因：违反宪法条款 12.1.4，所有配置资源必须通过统一的 Asset Pipeline 管理。

违反后果：热重载失效，资源无法被正确卸载，内存泄漏。

---

禁止：高频修改资源不支持热重载

原因：违反宪法条款 14.0.5，高频修改的资源必须优先支持热重载。

违反后果：策划修改配置后需要重启游戏，开发效率低下。

---

# AI 修改规则

## 如果新增资源类型

允许：
- 在 FallbackResourceMap 中添加新的 Fallback 资源
- 选择合适的 Handle 类型（Strong 或 Weak）
- 在 MemoryBudget 中评估是否需要调整预算

禁止：
- 启动时加载新资源类型的所有实例
- 新资源类型不提供 Fallback 资源
- 硬编码资源路径

优先检查：
- Handle 类型选择是否符合规则表
- Fallback 资源是否已加载到 AssetServer
- 场景切换卸载是否包含新资源类型
- 内存预算是否需要调整

---

## 如果修改场景切换卸载逻辑

允许：
- 调整卸载顺序（但大资源必须优先）
- 新增需要卸载的资源类型
- 调整每帧卸载上限

禁止：
- 一次性卸载所有资源
- 卸载过程中阻塞主线程
- 移除 Strong Handle 的显式移除步骤

优先检查：
- 卸载顺序是否正确（大 → 中 → 小）
- 每帧卸载总量是否 ≤ 4MB
- 所有当前场景的 Strong Handle 是否被移除
- 卸载完成后内存是否低于预算

---

## 如果修改热重载策略

允许：
- 扩展可热重载的资源类型（仅 Definition 和二进制资源）
- 调整热重载失败的回退策略

禁止：
- 热重载 Instance 数据
- 战斗中允许热重载
- 跳过格式验证直接热重载

优先检查：
- 新增的可热重载类型是否为 Definition
- 热重载失败时是否回退到上次有效状态
- 是否在战斗外执行

---

## 如果测试失败

排查顺序：
1. 检查 Strong Handle 是否在宿主销毁时被正确移除
2. 检查场景切换卸载是否包含所有相关资源
3. 检查每帧卸载总量是否超过 4MB
4. 检查 Fallback 资源是否已加载到 AssetServer
5. 检查热重载是否意外修改了 Instance 数据
6. 检查资源路径是否硬编码
