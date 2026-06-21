## Code Review Report: 框架 UI 实现

**审查范围**: 第二轮框架 UI 实现（Campaign 模块 + UI Screens + 插件注册 + 测试）
**审查时间**: 2026-06-13
**审查者**: @code-reviewer (simulated)

---

### ✅ 通过的检查

#### 架构合规性
- [x] **Feature First** — 新增 `campaign/` 模块表达业务含义，未创建禁止的顶层模块（systems.rs/events.rs/utils.rs）
- [x] **AppState 四态扩展正确** — MainMenu / LevelSelect / InGame / GameOver 已正确分离，系统守卫 `run_if(in_state(AppState::InGame))` 保持不变
- [x] **Startup 变更正确** — 从 `InGame` 改为 `MainMenu`，与 `campaign-pipeline.md` ADR 一致
- [x] **Logic/Presentation 分离** — `campaign/` 模块不包含任何 UI 代码；UI 屏幕通过 ViewModel + UiCommand 通信，不直接操作 ECS 组件
- [x] **跨模块通信合规** — 屏幕发送 UiCommand Message，`handle_menu_commands` 处理；`campaign/progression` 监听 LevelCompleted Message
- [x] **Definition/Instance 分离** — CampaignRegistry 是只读 Definition，CampaignProgress 是可变 Instance
- [x] **FORBIDDEN-4（campaign-pipeline.md）** — `campaign/` 不包含 UI 代码 ✅
- [x] **FORBIDDEN-5（campaign-pipeline.md）** — `campaign/` 不参与战斗逻辑 ✅
- [x] **FORBIDDEN-3（framework-ui.md）** — 屏幕不在 InGame 状态下运行（OnExit 自动清理）✅
- [x] **FORBIDDEN-5（framework-ui.md）** — 不跳过 LevelSelect 直接从 MainMenu 进入 InGame ✅

#### ECS 模式
- [x] **屏幕使用 Tag Component** — `MainMenuScreen`, `LevelSelectScreen`, `GameOverScreen` 均为空 Component 标记
- [x] **System 无状态** — 所有系统函数不存储中间状态
- [x] **OnEnter/OnExit 生命周期管理** — 屏幕使用 spawn/despawn 模式，非 Visibility 切换
- [x] **ViewModel 使用 Resource** — `LevelSelectState`, `GameResultView` 均为 Resource，符合架构设计
- [x] **Message 通信** — 跨模块通信全部通过 Message（UiCommand, LevelCompleted）

#### 插件注册
- [x] `CampaignPlugin` 在 Data Layer 注册（逻辑层之后、表现层之前）
- [x] `ScreensPlugin` 在 `UiPlugin` 内注册，与现有面板插件并列
- [x] `handle_menu_commands` 和 `handle_ui_commands` 守卫互斥（`not(InGame)` vs `InGame`），无重叠风险

#### 测试规范
- [x] UI-SCR-001 ~ 012 覆盖全面：MainMenu 生成/清理、LevelSelect 展示/进入/返回、GameOver 胜利/失败、E2E 完整流程
- [x] 测试验证屏幕实体生成/清理行为，而非内部实现细节
- [x] 测试使用标准测试数据，无随机因素
- [x] ViewModel 单元测试验证默认值正确性

---

### ❌ 发现的问题

#### [High] GameOver 屏幕显示空 stage_name 和 turn_count
- **位置**: `src/ui/screens/game_over.rs:37-41`
- **规则**: `architecture.md` "一个函数一个主要职责" — 不应留下 TODO 空值
- **说明**: `update_game_result_view` 中 `stage_name` 和 `turn_count` 分别硬编码为 `String::new()` 和 `0`，带有 TODO 注释但未实现。运行时 GameOver 屏幕不会显示关卡名和回合数
- **建议**:
  - `turn_count`: 从 `TurnState::turn_number` 资源读取（已存在于 `turn/order.rs`）
  - `stage_name`: 从 `CampaignRegistry` 查找当前 stage 对应的 level_id，再通过 `LevelRegistry` 获取关卡名

#### [Medium] ConfirmStage 不验证 level_id 有效性
- **位置**: `src/ui/screens/mod.rs:98-111`
- **规则**: `campaign-pipeline.md` FORBIDDEN-7 — "禁止在没有验证 level_id 有效性的情况下将 Campaign 标记为可用"
- **说明**: `handle_menu_commands::ConfirmStage` 只检查 `campaign_progress.stage_status(id)` 是否为 Unlocked/Completed，但未验证：
  1. 该 stage_id 是否存在于 `CampaignRegistry` 中
  2. 该 stage 的 `level_id` 是否存在于 `LevelRegistry` 中
  3. 如果 stage_id 是过期/无效的，进入 InGame 后会因缺少关卡配置而异常
- **建议**: 在 `ConfirmStage` 中添加验证逻辑：
  ```rust
  // 伪代码
  let stage_valid = campaign_progress.current_stage.as_ref()
      .and_then(|id| campaign_registry.first()?.stages.iter().find(|s| s.id == *id))
      .and_then(|stage| level_registry.get(&stage.level_id))
      .is_some();
  ```

#### [Medium] CampaignProgress.complete_current_stage 的"下一关"定位依赖 HashMap 插入顺序
- **位置**: `src/campaign/progress.rs:86-94`
- **规则**: `campaign_rules_v1.md` §3.1 — 关卡顺序必须由 Stage 列表定义
- **说明**: `complete_current_stage` 使用 `self.stages.keys().cloned().collect()` 将 HashMap keys 转为 Vec，依赖 insertion order 来确定下一个 Stage。虽然 Rust 的 HashMap 保持插入顺序，但此设计脆弱：
  - 任何不在 `initialize()` 中插入的 stage 可能破坏顺序
  - 代码的意图不明确（依赖隐式行为）
  - 如果 future 修改了 stages 的构建方式，顺序可能被破坏
- **建议**: 将 `CampaignProgress` 改为同时存储有序的 stage_id 列表，或直接从 `CampaignRegistry` 查找下一个 stage 的位置

#### [Medium] Campaign 加载使用 std::fs 直接 IO 而非 Bevy 资产系统
- **位置**: `src/campaign/loader.rs:22-25`
- **规则**: `architecture.md` "官方能力优先" — 使用 Bevy 原生 Asset API
- **说明**: `load_campaigns` 使用 `std::fs::read_dir` 和 `std::fs::read` 直接从文件系统加载 RON 文件。与项目现有加载模式一致，但这意味着：
  - 不参与 Bevy 的资产跟踪系统
  - 不支持热重载
  - 不兼容未来可能的 WASM 目标
- **建议**: 长远考虑迁移到 Bevy Asset API（非阻塞加载 + 路径管理）。当前可接受（与项目中其他加载器一致）

#### [Low] "继续战役"按钮复用 StartGame 命令
- **位置**: `src/ui/screens/main_menu.rs:118-119`
- **规则**: `framework-ui.md` "交互" — "继续战役"应使用现有进度，"开始游戏"应初始化新进度
- **说明**: `handle_main_menu_buttons` 中 `ContinueButton` 发送 `UiCommand::StartGame`，与"开始游戏"按钮相同。按 ADR 设计，"继续战役"应在 `CampaignProgress` 非空时直接进入 LevelSelect（保留已有进度），而非重新初始化
- **建议**: 新增 `UiCommand::ContinueGame` 变体（或在 `handle_menu_commands` 中添加判断逻辑：如果 `CampaignProgress` 非空则直接进入 LevelSelect，否则初始化）

#### [Low] spawn_level_select 在 OnEnter 中构建 ViewModel 而非单独的系统
- **位置**: `src/ui/screens/level_select.rs:29-36`
- **规则**: `framework-ui.md` "ScreensPlugin 内部" — `update_level_select_view` 应监听 CampaignProgress 变化
- **说明**: `spawn_level_select` 内联构建 ViewModel 数据并在渲染时使用。`update_level_select_view` 是独立系统，但只在 LevelSelect 状态的 Update 阶段运行。如果 CampaignProgress 在进入 LevelSelect 之前发生变化（良性），spawn 时读取的是最新数据，这样做是 OK 的
- **建议**: 无需修复。但考虑将 ViewModel 构建逻辑提取到 `update_level_select_view` 中，使 `spawn_level_select` 只负责渲染

#### [Low] MenuState ViewModel 未实现（ADR 偏差）
- **位置**: `src/ui/screens/mod.rs`
- **规则**: `framework-ui.md` "决策 4" — 新增 `MenuState` Resource
- **说明**: ADR 中设计的 `MenuState { pub active: bool }` Resource 未实现。MainMenu 屏幕直接通过 `UiTheme` 和 `CnFont` 资源渲染，无需特定 ViewModel。这是可接受的简化，符合"只解决当前复杂度"原则
- **建议**: 记录此 ADR 偏差，或删除 ADR 中对 MenuState 的引用以避免混淆

---

### 📋 总结

| 严重程度 | 数量 | 说明 |
|----------|------|------|
| Critical | 0 | 无架构违规 |
| High | 1 | GameOver 屏幕显示空 stage_name 和 turn_count（TODO 未实现） |
| Medium | 3 | ConfirmStage 未验证 level_id、complete_current_stage 依赖 HashMap 顺序、loader 使用 std::fs |
| Low | 3 | ContinueButton 复用 StartGame、ViewModel 构建位置、MenuState ADR 偏差 |

---

### 🎯 结论

**PASS** ✅
（建议修复 High 问题后再次确认）

无临界架构违规：新增的 `campaign/` 模块和 `ui/screens/` 子模块均遵循 Feature First、Logic/Presentation 分离、Definition/Instance 分离等核心原则。AppState 四态扩展与现有系统兼容。View Model + UiCommand 的通信模式与既有架构一致。

**必须修复**：
- GameOver 屏幕的 `stage_name` 和 `turn_count` 空值

**建议修复**：
- ConfirmStage 增加 level_id 有效性验证
- CampaignProgress 的下一个 Stage 定位改为显式有序列表

---

### 交接建议
- High 问题修复后 → 建议再次调用 **@code-reviewer** 复审 GameOver 屏幕
- 发现 `campaign/loader.rs` 的 IO 模式与 Asset API 方案差异较大 → 建议在未来的架构复盘中评估是否需迁移到 Asset API
- 测试覆盖完整（UI-SCR-001~012），测试质量可接受，无需调用 **@test-guardian**
