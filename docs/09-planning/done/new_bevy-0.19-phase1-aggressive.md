# Phase 1：全面升级 + 全特性采用

> **状态**: 🔴 已被 `bevy-0.19-migration-v3-aggressive.md` v3.1 吸收
>
> 策略：激进重构，不计代价，一步到位
> 不是"先兼容再升级"，而是直接升级到 0.19 并同时采用所有新特性
> 前置文档：`new_bevy-0.19-migration-master-plan.md`

## 准入条件

- [ ] 所有现有测试通过（`cargo nextest run`）
- [ ] 创建激进重构分支 `feat/bevy-0.19-aggressive-refactor`
- [ ] 阅读 `docs/03-technical/bevy-0.19-migration/` 全部 11 篇文档

---

## Batch 1：核心升级 + Observer + Delayed Commands + DevTools

> 最高优先级，解决最大影响面的变更

### Step 1.1：Cargo.toml 激进升级

- [ ] bevy = "0.19"
- [ ] 删除 bevy-inspector-egui（不再等待兼容版）
- [ ] 添加 bevy_settings 依赖
- [ ] 删除所有 bevy-inspector-egui 相关代码和 import
- [ ] `cargo check` 能开始编译（允许有错误）

### Step 1.2：Observer API 全量修复 + run_if 化

**同时做两件事：修编译错误 + 重构 if 守卫**

#### 1.2.1 修复 Observer API 编译错误

- [ ] 扫描所有 `.add_observer()` 调用（35文件/140行），修复 API 变更
- [ ] 扫描所有 `Trigger<T>` 参数（20文件/43行），修复 API 变更
- [ ] 扫描所有 Observer 回调函数签名，修复 API 变更

#### 1.2.2 全量迁移 if 守卫到 run_if

- [ ] 在每个域/能力模块创建 `conditions.rs`
- [ ] 提取共享条件函数：
  - `battle_is_running(state: Res<BattleState>) -> bool`
  - `battle_exists(state: Res<BattleState>) -> bool`
  - `target_is_alive(query: Query<&Health>, target: ...)` → bool（如支持）
  - 各域特定条件...
- [ ] 逐模块迁移所有 Observer if 守卫：
  - [ ] combat/ — P0
  - [ ] ability/ — P0
  - [ ] effect/ — P0
  - [ ] modifier/ — P0
  - [ ] buff（通过 stacking/）/trigger/ — P1
  - [ ] spell/reaction/ — P1
  - [ ] 其余域 — P2
  - [ ] logging/observers/ — P2

#### 1.2.3 验证

- [ ] `cargo check` 零错误
- [ ] `cargo nextest run` 全绿

### Step 1.3：Delayed Commands 全面替代 Timer

- [ ] 扫描所有 Timer 使用（3文件/11行），全量迁移到 Delayed Commands
- [ ] 在 `core/capabilities/runtime/command/` 中创建 Delayed Command 工具：
  - `DelayedBattleCommand` — 内嵌 BattleId 校验
  - `DelayedCommandLoop` — 循环延迟命令（替代循环 Timer）
- [ ] 迁移 localization/Timer → Delayed Commands
- [ ] 迁移 content/hot_reload Timer → Delayed Commands
- [ ] 迁移所有其他延迟逻辑
- [ ] 删除所有 Timer import

#### 验证

- [ ] `cargo check` 零错误
- [ ] `grep -r "Timer" src/` 零匹配（除注释外）
- [ ] `cargo nextest run` 全绿

### Step 1.4：DevTools 替代 inspector-egui

- [ ] 在 `src/tools/dev_tools_plugin.rs` 中添加：
  - DiagnosticsOverlay::fps()
  - DiagnosticsOverlay::frame_time()
  - Text Gizmos（调试标注）
- [ ] Feature Flag 控制：`dev` feature
- [ ] 验证：`cargo run --features dev` 显示 FPS

#### Batch 1 准出

- [ ] `cargo check` 零错误
- [ ] `cargo clippy` 零警告
- [ ] `cargo nextest run` 全绿
- [ ] 所有 Observer if 守卫已迁移到 run_if
- [ ] 所有 Timer 已迁移到 Delayed Commands
- [ ] bevy-inspector-egui 已完全移除
- [ ] DiagnosticsOverlay 正常工作

---

## Batch 2：BSN + Resource Hook + Settings + Reflect + Handle

### Step 2.1：spawn → spawn_scene + bsn!

**全面替代 commands.spawn()**

- [ ] 扫描所有 `commands.spawn()` / `commands.spawn_empty()`（13文件/36行）
- [ ] 为每个 spawn 点编写对应的 Scene 函数：
  ```rust
  // 以前
  commands.spawn((Player { score: 10 }, Health(100), Position::default()));

  // 现在
  fn player() -> impl Scene {
      bsn! { Player { score: 10 }, Health(100) }
  }
  commands.spawn_scene(player());
  ```
- [ ] 迁移 summon_system 中的 spawn
- [ ] 迁移 save/load_system 中的 spawn
- [ ] 迁移 scenes/plugin 中的 spawn
- [ ] 迁移测试中的 spawn

#### 验证

- [ ] `cargo check` 零错误
- [ ] `grep -r "commands.spawn(" src/` 零匹配（除测试外）
- [ ] `cargo nextest run` 全绿

### Step 2.2：Resource Hook 化

- [ ] 为 BattleState 添加 Observer（状态变化监听）
- [ ] 为 TurnState 添加 Observer（回合初始化/清理）
- [ ] 为 InputState 添加 Observer（输入状态变化）
- [ ] 为 MapState 添加 Observer（地图切换）
- [ ] 评估 Resource + Component 混合查询场景

#### 验证

- [ ] 关键 Resource 有 Observer
- [ ] `cargo nextest run` 全绿

### Step 2.3：User Settings 立即引入

- [ ] 创建 `src/infra/settings/` 模块
- [ ] 定义 AudioSettings（Resource + SettingsGroup + Reflect）
- [ ] 定义 VideoSettings
- [ ] 定义 GameplaySettings（battle_speed, show_damage_numbers, show_grid, auto_save）
- [ ] 定义 ControlSettings
- [ ] 注册 PreferencesPlugin::new("com.fre.srpg")
- [ ] 添加 auto_save_preferences 系统
- [ ] 确保 Config vs Settings 分离

#### 验证

- [ ] 设置修改后自动保存
- [ ] 退出时设置持久化
- [ ] `cargo nextest run` 全绿

### Step 2.4：Reflect 全量补齐

- [ ] 扫描所有 `#[derive(Asset)]` 类型，补齐 `Reflect + #[reflect(Asset)]`
- [ ] 扫描所有 `#[derive(Component)]` 类型，补齐 `Reflect + #[reflect(Component)]`
- [ ] 扫描所有 Config 类型，补齐 Reflect
- [ ] 注册所有 reflect 类型

#### 验证

- [ ] `grep -r "derive(Asset)" src/` 的结果都包含 Reflect
- [ ] `cargo nextest run` 全绿

### Step 2.5：Handle 序列化

- [ ] 评估 Config 系统中 Handle<Image> 的序列化需求
- [ ] 如需要，引入 HandleSerializeProcessor / HandleDeserializeProcessor

#### Batch 2 准出

- [ ] `cargo check` 零错误
- [ ] `cargo clippy` 零警告
- [ ] `cargo nextest run` 全绿
- [ ] 所有 spawn 已迁移到 spawn_scene + bsn!
- [ ] 关键 Resource 有 Observer
- [ ] User Settings 正常工作
- [ ] Reflect 全量补齐

---

## Batch 3：Contiguous Query + FontSize + EditableText

### Step 3.1：Contiguous Query 立即使用

- [ ] 识别所有批量运算场景：
  - Attribute 批量运算（Health += Regen）
  - Buff Tick 批量处理
  - Effect Tick 批量处理
- [ ] 为每个场景编写 contiguous_iter 版本：
  ```rust
  fn apply_regen(mut query: Query<(&mut Health, &RegenRate)>) {
      for (mut health, regen) in query.contiguous_iter_mut().unwrap() {
          for (h, r) in health.iter_mut().zip(regen) {
              h.0 = (h.0 + r.0).min(h.max);
          }
      }
  }
  ```
- [ ] 在 `shared/` 中封装迭代器工具函数
- [ ] 高频数据 bypass_change_detection

#### 验证

- [ ] `cargo nextest run` 全绿
- [ ] 批量运算性能有提升（criterion 基准）

### Step 3.2：FontSize 全面采用

- [ ] 扫描所有 font_size 使用（目前仅 cue/ 中 3 行）
- [ ] 迁移 `font_size: f32` → `font_size: FontSize::Px(f32)`
- [ ] 评估哪些场景适合 FontSize::Rem（响应式）
- [ ] 配置 FontCx 默认字体族

#### 验证

- [ ] `grep -r "font_size:" src/` 无 f32 直接赋值
- [ ] `cargo nextest run` 全绿

### Step 3.3：EditableText 引入

- [ ] 在 `src/infra/input/` 或 `src/app/` 中添加 EditableText 支持
- [ ] 角色命名场景
- [ ] 搜索框场景
- [ ] IME 支持（CJK 输入）

#### Batch 3 准出

- [ ] `cargo check` 零错误
- [ ] `cargo clippy` 零警告
- [ ] `cargo nextest run` 全绿
- [ ] 批量运算使用 contiguous_iter
- [ ] font_size 全量迁移到 FontSize 枚举

---

## Phase 1 总准出条件

- [ ] `cargo check` 零错误
- [ ] `cargo clippy` 零警告
- [ ] `cargo nextest run` 全绿
- [ ] 冒烟测试通过
- [ ] 无 bevy-inspector-egui 残留
- [ ] 无 Timer 残留（除注释外）
- [ ] 所有 Observer if 守卫已迁移到 run_if
- [ ] 所有 spawn 已迁移到 spawn_scene + bsn!
- [ ] Reflect 全量补齐
- [ ] User Settings 正常工作
- [ ] DiagnosticsOverlay 正常工作
- [ ] Replay 确定性未受影响
- [ ] Save/Load 兼容性未受影响

---

## 回滚方案

激进重构如果整体失败：

1. `git checkout main` 回到 0.18.1
2. 从 Batch 1 重新开始，但改为渐进策略
3. 保留已完成且通过测试的 Batch

---

## 注意事项

1. **每个 Batch 完成后立即提交**，不要攒大 commit
2. **每个 Batch 完成后运行 nextest**，确保不破坏已有功能
3. **Effect/Modifier 管线不可绕过** — 即使激进重构也要遵守
4. **Replay 确定性不可破坏** — 重构后回放必须仍然一致
5. **允许行为变更** — 如果新 API 行为更优，不需要保持旧行为
