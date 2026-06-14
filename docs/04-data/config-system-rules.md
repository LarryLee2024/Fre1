---
id: 04-data.config-system-rules
title: Config System Rules
status: draft
owner: feature-developer
created: 2026-06-14
updated: 2026-06-14
tags:
  - data
---

# 配置系统领域

Version: 1.0

配置系统领域管理游戏所有配置的加载、校验、热重载和持久化，确保配置数据与代码彻底分离。

核心原则：
- 🟩 **12.1.1 职责划分**：配置定义内容，代码解释配置（宪法条款 12.1.1）
- 🟩 **12.1.4 资源管理**：所有配置资源必须通过统一的 Asset Pipeline 管理（宪法条款 12.1.4）
- 🟩 **12.1.5 热重载优先**：所有配置必须优先支持热重载（宪法条款 12.1.5）
- 🟩 所有配置加载失败使用默认值，零 Crash
- 🟩 四层物理分离，职责清晰
- 🟩 **12.4.1 平衡参数全配置化**：Rust 代码零硬编码数值（宪法条款 12.4.1）

---

# 术语定义

## 四层配置架构

EngineConfig / GameRulesConfig / UserSettings / DebugSwitches 四层配置的物理分离体系。

不是单一配置文件。不是任意散布的 Resource。

关键属性：
- EngineConfig：引擎层参数（窗口、音量基础值），低变更，需重启生效
- GameRulesConfig：游戏规则参数（平衡、战斗），策划调优核心，战斗外可热重载
- UserSettings：用户个人设置（分辨率、音量），存储在平台特定目录
- DebugSwitches：调试开关，Feature Gate 隔离，Release 构建完全排除

---

## 战斗锁（Battle Lock）

AppState::InGame 期间禁止 ALL 配置热重载的保护机制。

不是永久锁。不是玩家禁止操作。

> **优化来源**: `docs/01-architecture/config_system_design.md` §4.1 — 锁范围从 GameRulesConfig 扩展到 ALL 配置，AppState::InGame 替代 BattleState::Running

关键属性：
- 仅在 AppState::InGame 时生效
- 战斗结束后自动解锁
- 禁止 ALL 配置类型热重载（EngineConfig / GameRulesConfig / UserSettings）
- 保护 Replay 系统的确定性
- 热重载请求在战斗期间排队等待

---

## 优雅降级（Graceful Degradation）

所有配置加载失败时使用默认值，禁止 Crash 的容错机制。

不是静默忽略。不是跳过。

> **优化来源**: `docs/01-architecture/config_system_design.md` §5.3 — 按配置类型分级处理加载失败

关键属性：
- EngineConfig 加载失败：使用硬编码默认值 + 记录 ERROR 日志，禁止 panic（宪法 §13.9.4）
- GameRulesConfig 加载失败：ERROR 日志 + 使用硬编码默认值，游戏可降级
- UserSettings 加载失败：WARN 日志 + 重置为默认值，用户设置非关键
- 加载失败时回退到硬编码默认值
- 输出对应级别日志记录失败原因
- 游戏继续运行，不中断（包括 EngineConfig）
- 用户可感知配置加载异常（如 UI 提示）

---

## God Config 拆分

单一 Config 拆分为 BattleConfig / SkillConfig / BuffConfig / CampaignConfig 等细粒度 Resource。

不是单一巨大 Resource。

> **优化来源**: `docs/01-architecture/config_system_design.md` §2.2 — 反"上帝配置"：按领域拆分为细粒度独立 Resource

关键属性：
- 避免 Res<GameRulesConfig> 的全局读锁竞争
- 热重载时仅触发相关领域的 Changed 检测
- 每个子 Config 独立加载、独立校验
- 支持按需加载（如战斗场景只加载 BattleConfig）
- 修改 SkillConfig 不触发 BattleConfig 的 Changed 检测

---

## 防抖写入（Debounce Write）

UserSettings 滑动条变更的延迟写入机制。

不是即时写入。不是禁用自动保存。

> **优化来源**: `docs/01-architecture/config_system_design.md` §2.3 — 防抖窗口从"1-2秒"精确为 200ms

关键属性：
- 滑块拖动时只修改内存中的 UserSettings
- 停止操作 200ms 后异步写入磁盘
- 使用 AsyncComputeTaskPool 避免 IO 阻塞
- 重型设置（分辨率、全屏）需点击"应用"按钮才生效

---

## 热重载事件风暴防御

NeedsRebuild 标记 + 延迟重建，防止同帧重建 N 次的保护机制。

不是同帧重建 N 次。不是忽略重载。

> **优化来源**: `docs/01-architecture/config_system_design.md` §4.1 — 热重载事件风暴缓解

关键属性：
- 收到 ConfigReloaded 事件后标记 NeedsRebuild<T>
- 在下一帧特定 Phase（如 PreparePhase）统一执行重建
- 不打断当前正在执行的关键逻辑（如动画播放）
- 合并同一帧内的多次变更请求
- ConfigReloaded 事件只传递 config_type，不传递 changes 列表

---

## MOD Patch/Override

base/ + overrides/ 目录结构，按 ID 深度合并的配置扩展机制。

不是完全覆盖。不是忽略 MOD。

> **优化来源**: `docs/01-architecture/config_system_design.md` §6 — 明确 Patch（深度合并）vs Override（整体替换）语义差异

关键属性：
- MOD 配置使用 Patch 语义（深度合并），不是 Override（整体替换）
- base/ 目录存储原版配置
- overrides/ 目录存储 MOD 覆盖配置
- 加载时先加载 base，再遍历 overrides 深度合并
- MOD 只需声明差异项（如 `{"skill_id": "fireball", "cooldown": 2}`）
- 数组字段可选 Append（追加）或 Replace（整体替换）策略

---

## 配置版本（Config Version）

配置文件的 SemVer 版本，用于兼容性管理。

不是存档版本。不是代码版本。

关键属性：
- 每个配置文件头部标注版本号
- 旧版本配置自动映射到新版本
- 存档绑定配置版本，加载时校验兼容性
- 废弃参数标记为 deprecated，大版本统一清理

---

# 领域边界

## 本领域负责

- 四层配置的加载、校验、缓存
- 配置热重载的触发和重建
- UserSettings 的防抖写入
- DebugSwitches 的 Feature Gate 隔离
- MOD Patch/Override 的深度合并
- 配置版本的兼容性管理
- 优雅降级的默认值回退
- 战斗锁的实现

## 本领域不负责

- 具体配置参数的业务含义（由各业务领域负责）
- UI 设置界面的渲染（由 UI 领域负责）
- 配置文件的编辑工具（由工具链领域负责）
- 存档的读写（由 Save 领域负责）
- 资产文件的加载（由 Asset 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| ConfigReloaded 事件 | Message | 所有依赖配置的领域 |
| NeedsRebuild 标记 | Resource 访问 | 相关业务领域 |
| 战斗锁状态查询 | 函数调用 | Battle 领域 |
| 配置校验结果 | 函数调用 | Debug/UI 领域 |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Loading | 配置文件加载中 | Ready |
| Ready | 配置就绪，可响应查询 | Ready, Reloading, Locked |
| Reloading | 热重载中，重新加载文件 | Ready |
| Locked | 战斗锁生效，禁止重载 | Ready |

## 状态转换图

Loading → Ready → Reloading → Ready
                ↘ Locked → Ready

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Loading | Ready | 所有配置加载完成 |
| Ready | Reloading | 文件变更检测触发 |
| Reloading | Ready | 重新加载完成 |
| Ready | Locked | AppState::InGame |
| Locked | Ready | AppState 离开 InGame |

---

# 不变量

## 不变量1：所有配置加载失败使用默认值，零 Crash（EngineConfig 除外）

任意时刻：

任何配置文件加载失败（格式错误、文件缺失、解析异常）时，必须回退到硬编码默认值，禁止 panic 或 unwrap 导致崩溃。EngineConfig 加载失败同样使用硬编码默认值 + ERROR 日志，禁止 panic。

> 对应宪法 §13.9.4：核心业务领域绝对禁止 unwrap()/expect()/panic!()，引擎参数缺失也应优雅降级

违反表现：

策划配错 RON 格式后游戏闪退，或未处理的 None 导致 panic。EngineConfig 加载失败时 panic。

---

## 不变量2：AppState::InGame 期间禁止 ALL 配置热重载

任意时刻：

AppState::InGame 期间，ALL 配置类型的热重载请求必须被拒绝或排队，禁止在战斗中途修改任何规则参数。这不仅是 GameRulesConfig，还包括 EngineConfig、UserSettings 等所有配置。

> **优化来源**: `docs/01-architecture/config_system_design.md` §4.1 — 锁范围从 GameRulesConfig 扩展到 ALL 配置

违反表现：

战斗中途伤害公式被修改，导致 Replay 系统确定性破坏。或战斗中修改窗口参数导致崩溃。

---

## 不变量3：Debug 开关编译期隔离

任意时刻：

DebugSwitches 相关代码必须通过 Feature Gate（`#[cfg(feature = "dev")]`）隔离，Release 构建完全排除。

违反表现：

Release 包中包含调试开关代码，玩家可通过内存修改器开启 God Mode。

---

## 不变量4：平衡参数零硬编码

任意时刻：

所有影响游戏平衡的数值（伤害公式、冷却时间、AP 恢复等）必须存储在配置文件中，禁止在 Rust 代码中硬编码。

违反后果：

修改平衡参数必须重新编译代码，策划无法独立调优。

---

## 不变量5：配置加载失败分级处理

不同配置类型的加载失败应有不同的处理策略，统一使用优雅降级，禁止 panic：

| 配置类型 | 失败行为 | 理由 |
|---------|---------|------|
| EngineConfig | ERROR 日志 + 使用默认值 | 引擎参数缺失时优雅降级，禁止 panic |
| GameRulesConfig | ERROR 日志 + 使用默认值 | 游戏规则可降级，不能因配置错误导致闪退 |
| UserSettings | WARN 日志 + 重置为默认值 | 用户设置非关键，重置即可恢复 |

> 对应宪法 §13.9.4：核心业务领域绝对禁止 unwrap()/expect()/panic!()

---

## 不变量6：防抖写入 200ms 窗口

任意时刻：

UserSettings 中的轻量设置（音量、UI 缩放等）修改后，必须在 200ms 防抖窗口结束后才触发磁盘写入。防抖窗口内多次修改只触发一次写入。

> **优化来源**: `docs/01-architecture/config_system_design.md` §2.3 — 防抖窗口精确为 200ms

违反表现：

滑块每次微小变更都触发磁盘写入，导致 IO 卡顿和音频爆音。

---

# 业务规则

## 规则1：四层物理分离

允许：
- EngineConfig 存储在项目目录，RON 格式
- GameRulesConfig 拆分为 BattleConfig / SkillConfig / BuffConfig / CampaignConfig
- UserSettings 存储在平台特定目录（Windows: `%APPDATA%/srpg/settings.ron`，macOS: `~/Library/Application Support/srpg/settings.ron`）
- DebugSwitches 存储在项目目录，RON + Feature Gate

禁止：
- 四层配置混合存储在同一文件
- GameRulesConfig 作为单一巨大 Resource
- UserSettings 存储在项目目录（避免 Git 覆盖）

必须：
- 每层配置独立加载、独立校验
- 层间优先级：DebugSwitches > UserSettings > GameRulesConfig > EngineConfig

---

## 规则2：God Config 禁止

允许：
- 按领域拆分为 BattleConfig / SkillConfig / BuffConfig / CampaignConfig
- 每个子 Config 作为独立 Res 或 Asset
- 热重载时仅触发相关领域的 Changed 检测

禁止：
- 单一 Config 包含所有平衡参数
- 所有 System 读取同一个巨大 Res<GameRulesConfig>
- 热重载时触发所有领域的 Changed 检测

必须：
- 每个子 Config 有独立的校验逻辑
- 子 Config 之间通过函数调用交互，禁止直接引用

---

## 规则3：MOD Patch/Override

允许：
- base/ 目录存储原版配置
- overrides/ 目录存储 MOD 覆盖配置
- 按 ID 深度合并（只声明差异项）
- 数组字段使用 Append（追加）或 Replace（整体替换）策略

禁止：
- MOD 使用 Override（整体替换）语义覆盖原版配置
- MOD 之间互相覆盖同一字段
- 合并时忽略配置版本兼容性

必须：
- 加载顺序：base → overrides（按优先级排序）
- 合并策略为深度合并（Deep Merge）
- 数值字段直接替换，数组字段需显式声明 Append 或 Replace
- 合并后验证参数范围

---

## 规则4：平衡参数管理

允许：
- 所有平衡参数存储在 RON 配置文件中
- 参数命名使用 snake_case + 单位后缀（如 `cooldown_turns: u32`）
- 策划通过编辑 RON 文件调优

禁止：
- Rust 代码中硬编码平衡参数
- 参数命名无单位后缀（如 `cooldown: 3` 不知道是秒还是回合）
- 策划修改后不运行 CI 校验

必须：
- CI 自动运行核心战斗回放测试（Replay Tests）
- 废弃参数标记为 deprecated，不直接删除
- 参数验证在加载时执行（如 `crit_multiplier > 1.0`）

---

## 规则5：SRPG 专属配置

允许：
- GameRulesConfig 包含 AP 恢复规则、Z-OC 触发条件、Permadeath 开关
- DebugSwitches 包含 force_crit、show_damage_formula、reveal_fog
- UserSettings 包含 combat_animation_speed、auto_skip_enemy_turn

禁止：
- SRPG 核心规则硬编码在 Rust 代码中
- 调试开关进入 Release 构建
- 用户设置存储在项目目录

必须：
- AP 恢复值可配置（回合制核心）
- Z-OC 规则可配置（战棋核心）
- Permadeath 开关可配置（SRPG 特色）

---

## 规则6：防抖写入

允许：
- 滑块拖动时只修改内存中的 UserSettings
- 停止操作 200ms 后异步写入磁盘
- 重型设置需点击"应用"按钮才生效

禁止：
- 滑块每次微小变更都触发磁盘写入
- 在主线程执行 IO 操作
- 重型设置（分辨率、全屏）即时生效

必须：
- 使用 AsyncComputeTaskPool 异步写入
- 写入前校验参数范围
- 写入失败时回退到上次保存的值

---

## 规则7：曲线表配置管理

> **优化来源**: `docs/其他/74借鉴.md` §5 — 所有随等级变化的数值通过曲线表配置

允许：
- 所有随等级、阶段、条件变化的数值通过 CurveTable 配置
- 使用 Step（阶梯）、Linear（线性）、Spline（样条）三种插值方法
- 曲线表通过 CurveId 被 Formula 系统引用

禁止：
- 硬编码 match level 分支（如 `match level { 1 => 100, 2 => 250, ... }`）
- 不按 key 升序排列关键点
- 运行时修改曲线表数据

必须：
- 曲线表关键点列表按 key 升序排列
- 插值方法在配置时明确指定
- 曲线表与 GameRulesConfig 同等对待（战斗外可热重载）

---

## 规则8：所有配置必须有 Schema

> 🟩 **宪法条款 12.2.1**：SkillConfig、BuffConfig、CharacterConfig 等所有配置结构体，必须对应明确的 Schema 定义

允许：
- 每个配置结构体有对应的 Schema 定义
- Schema 校验在加载时自动执行

禁止：
- 🟥 没有 Schema 定义的配置结构体
- 🟥 配置字段漂移（字段名或类型不一致）

必须：
- 所有配置结构体必须有对应的 Schema 定义
- Schema 校验在加载时自动执行
- AI 生成配置时有明确的结构依据

---

## 规则9：CI 配置校验

> 🟩 **宪法条款 12.2.2**：所有配置必须支持 CI 自动校验：引用合法性、数值范围、循环依赖

允许：
- CI 自动校验配置文件的引用合法性
- CI 自动校验数值范围（如 crit_multiplier > 1.0）
- CI 自动检测循环依赖

禁止：
- 🟥 校验不通过时允许合并入主分支
- 🟥 跳过 CI 校验直接提交配置

必须：
- CI 必须运行配置校验脚本
- 校验不通过禁止合并入主分支
- 校验结果必须记录日志

---

## 规则10：配置唯一事实源

> 🟩 **宪法条款 12.3.1**：每份配置数据必须有唯一的归属 Feature 与维护入口

允许：
- 每种配置类型（BattleConfig、SkillConfig、BuffConfig）有独立的归属 Feature
- 配置数据通过 Registry 统一管理

禁止：
- 🟥 多个 Feature 同时维护同一份核心配置数据
- 🟥 配置数据分散在多个位置

必须：
- BattleConfig 归属 battle 模块
- SkillConfig 归属 skill 模块
- 配置数据通过 Registry 统一管理

---

# 管线

## 配置加载管线

```
默认值初始化 → RON文件加载 → 用户设置合并 → 校验值范围 → Resource就绪
```

### Step1：默认值初始化

输入：硬编码的默认值常量
处理：创建各层配置的默认实例
输出：带有默认值的配置实例
禁止：默认值为空或 None

### Step2：RON 文件加载

输入：配置文件路径
处理：解析 RON 文件为结构体
输出：解析后的配置实例
禁止：加载失败时 Crash（必须回退到默认值）

### Step3：用户设置合并

输入：基础配置 + 用户设置
处理：按优先级合并（UserSettings 覆盖 EngineConfig）
输出：合并后的最终配置
禁止：合并时忽略层间优先级

### Step4：校验值范围

输入：合并后的配置
处理：执行参数校验（如 crit_multiplier > 1.0）
输出：校验通过的配置
禁止：校验失败时 Crash（必须回退到默认值并输出 warn 日志）

### Step5：Resource 就绪

输入：校验通过的配置
处理：插入 Bevy Resource（如 Res<BattleConfig>）
输出：配置 Resource 可供查询
禁止：跳过校验直接插入 Resource

---

## 热重载管线

```
文件变更检测 → 战斗锁检查 → 重新加载 → NeedsRebuild标记 → 延迟重建
```

### Step1：文件变更检测

输入：文件系统 Watcher 事件
处理：识别变更的配置文件类型
输出：ConfigReloaded 事件（含 config_type）
禁止：在战斗结算阶段处理文件变更

### Step2：战斗锁检查

输入：ConfigReloaded 事件 + AppState
处理：检查当前是否为 AppState::InGame
输出：通过锁检查或排队等待
禁止：绕过战斗锁直接重载

### Step3：重新加载

输入：变更的配置文件
处理：重新解析 RON 文件，校验参数
输出：新的配置实例
禁止：加载失败时使用旧配置（必须回退到默认值）

### Step4：NeedsRebuild 标记

输入：新配置实例
处理：标记 NeedsRebuild<T> Marker
输出：等待下一帧统一重建
禁止：在当前帧立即重建所有数据

### Step5：延迟重建

输入：NeedsRebuild 标记
处理：在 PreparePhase 统一执行重建逻辑
输出：配置 Resource 更新完成
禁止：跳过 NeedsRebuild 检查直接重建

---

# 数据结构

## EngineConfig（引擎配置）

职责：存储引擎层参数，低变更，需重启生效

结构：
- window_width：窗口宽度
- window_height：窗口高度
- master_volume：主音量基础值
- fps_limit：帧率限制

要求：
- RON 格式存储在项目目录
- 修改后需重启生效
- 加载失败时使用硬编码默认值

---

## GameRulesConfig（游戏规则配置）

职责：存储游戏规则参数，策划调优核心，战斗外可热重载

结构：
- battle：BattleConfig（AP 恢复、Z-OC 规则）
- character：CharacterConfig（种族成长、职业属性）
- skill：SkillConfig（冷却时间、伤害公式）
- buff：BuffConfig（持续时间、叠加规则）

要求：
- 拆分为细粒度子 Config
- 战斗中禁止热重载
- 参数校验在加载时执行

---

## UserSettings（用户设置）

职责：存储用户个人设置，平台特定目录

结构：
- resolution：分辨率
- fullscreen：全屏模式
- master_volume：主音量
- music_volume：音乐音量
- sfx_volume：音效音量
- combat_animation_speed：战斗动画倍速
- auto_skip_enemy_turn：自动跳过敌方回合

要求：
- 存储在平台特定目录（非项目目录）
- 滑块变更使用防抖写入
- 重型设置需点击"应用"按钮

---

## DebugSwitches（调试开关）

职责：存储调试开关，Feature Gate 隔离

结构：
- force_crit：强制暴击
- show_damage_formula：显示伤害公式
- reveal_fog：开全图视野
- god_mode：无敌模式
- show_fps：显示帧率

要求：
- Feature Gate 隔离（`#[cfg(feature = "dev")]`）
- Release 构建完全排除
- 通过 bevy_egui 面板控制

---

## ConfigVersion（配置版本）

职责：管理配置文件的版本兼容性

结构：
- major：主版本号
- minor：次版本号
- patch：补丁号

要求：
- SemVer 格式
- 旧版本配置自动映射到新版本
- 存档绑定配置版本

---

## CurveTableConfig（曲线表配置）

> **优化来源**: `docs/其他/74借鉴.md` §5 — UE Curve Table：所有随等级、阶段变化的数值通过曲线表配置

职责：数据驱动的数值映射，所有随等级、阶段、条件变化的数值通过曲线表配置

结构：
- id：曲线表唯一标识
- interpolation：插值方法（Step / Linear / Spline）
- key_value_pairs：关键点列表（必须按 key 升序排列）

关键属性：
- 所有随等级变化的数值必须通过 CurveTable 配置，禁止硬编码 match 分支
- 插值方法决定关键点之间的取值行为：Step（阶梯）、Linear（线性）、Spline（样条）
- 曲线表通过 CurveId 被 Formula 系统引用
- 热重载时与 GameRulesConfig 同等对待（战斗外可重载）

---

# 禁止事项

禁止：配置加载失败导致 Crash

原因：策划配错格式后游戏闪退，严重影响开发效率和玩家体验

违反后果：用户看到游戏崩溃，无法感知是配置问题，反复尝试无果

---

禁止：配置文件没有 Schema 定义

原因：违反宪法条款 12.2.1，AI 生成配置时无明确结构依据，字段漂移与格式错误。

违反后果：配置文件格式不一致，加载时解析失败。

---

禁止：CI 配置校验不通过时合并

原因：违反宪法条款 12.2.2，未校验的配置可能导致运行时崩溃。

违反后果：配置错误在运行时才发现，修复成本高。

---

禁止：多个 Feature 维护同一份配置数据

原因：违反宪法条款 12.3.1，多源修改导致数据不一致。

违反后果：配置数据冲突，加载时产生不可预测行为。

---

禁止：战斗中热重载游戏规则

原因：破坏 Replay 系统的确定性，导致录像回放与实际战斗不一致

违反后果：Replay 系统完全失效，战斗记录无法复现

---

禁止：AppState::InGame 期间热重载任何配置

原因：战斗中修改任何配置都会破坏 Replay 系统的确定性

违反后果：战斗中途参数变化，录像无法复现

---

禁止：硬编码数值映射在 Rust 代码中

原因：修改平衡参数必须重新编译代码，策划无法独立调优

违反后果：每次数值调整都需要程序员介入，开发效率低下

---

禁止：Debug 开关进入 Release 构建

原因：玩家可通过内存修改器开启 God Mode，破坏游戏公平性

违反后果：安全漏洞，玩家可作弊，影响游戏平衡

---

禁止：硬编码平衡参数

原因：修改平衡参数必须重新编译代码，策划无法独立调优

违反后果：每次数值调整都需要程序员介入，开发效率低下

---

禁止：God Config 单一巨大 Resource

原因：所有 System 读取同一个 Res<GameRulesConfig> 产生全局读锁竞争，阻碍并行执行

违反后果：System 并行度下降，帧率降低，热重载时触发所有领域的 Changed 检测

---

禁止：UserSettings 即时写盘

原因：滑块拖动时每次微小变更都触发磁盘 IO，导致卡顿和音频爆音。应使用 200ms 防抖窗口

违反后果：设置界面卡顿，用户体验差，频繁 IO 操作磨损磁盘

---

禁止：热重载事件风暴

原因：同帧内多次变更触发大量重建，打断当前正在执行的关键逻辑

违反后果：动画播放中断，UI 刷新卡顿，游戏体验不稳定

---

禁止：MOD 完全覆盖原版配置

原因：MOD 替换整个配置文件导致原版内容丢失，MOD 之间无法兼容

违反后果：安装 MOD 后原版技能、Buff 等配置被覆盖，无法恢复

---

禁止：硬编码数值映射（match level 分支）

原因：修改数值映射必须重新编译代码，策划无法独立调优

违反后果：每次数值调整都需要程序员介入，策划无法使用曲线表调优

---

禁止：防抖窗口内多次触发磁盘写入

原因：滑块拖动时每次微小变更都触发磁盘 IO，导致卡顿和音频爆音

违反后果：设置界面卡顿，用户体验差，频繁 IO 操作磨损磁盘

---

# AI 修改规则

## 如果新增配置参数

允许：
- 在对应子 Config 结构体中添加字段
- 设置合理的默认值
- 添加参数校验逻辑

禁止：
- 在 Rust 代码中硬编码新参数的默认值（必须在配置文件中）
- 跳过参数校验直接使用
- 修改已有参数的类型或语义

优先检查：
- 默认值是否合理
- 参数校验是否覆盖边界情况
- 是否需要热重载支持

---

## 如果修改热重载逻辑

允许：
- 调整文件变更检测的时机
- 优化 NeedsRebuild 标记的逻辑
- 调整延迟重建的 Phase

禁止：
- 移除战斗锁检查
- 在当前帧立即重建所有数据
- 忽略 ConfigReloaded 事件

优先检查：
- 战斗锁是否正确生效
- NeedsRebuild 标记是否被正确清理
- 延迟重建是否在正确的 Phase 执行

---

## 如果新增 MOD 配置支持

允许：
- 添加 overrides/ 目录的加载逻辑
- 实现深度合并算法
- MOD 配置加前缀避免冲突

禁止：
- MOD 配置覆盖原版配置
- 合并时忽略配置版本兼容性
- 不验证合并后的参数范围

优先检查：
- 合并算法是否为深度合并
- MOD 配置是否加了前缀
- 合并后参数校验是否通过

---

## 如果修改配置校验逻辑

允许：
- 添加新的参数校验规则
- 调整校验失败的处理策略
- 优化校验性能

禁止：
- 校验失败时 Crash（必须回退到默认值）
- 移除已有参数的校验
- 校验逻辑中硬编码业务规则

优先检查：
- 校验失败时是否回退到默认值
- 校验逻辑是否覆盖所有参数
- warn 日志是否记录了失败原因

---

## 如果测试失败

排查顺序：
1. 检查配置文件路径是否正确（assets 目录结构）
2. 检查 RON 格式是否正确（语法错误、字段缺失）
3. 检查参数校验是否通过（值范围、类型匹配）
4. 检查热重载是否触发（文件变更检测、战斗锁状态）
5. 检查默认值回退是否生效（加载失败时的处理）
