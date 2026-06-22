---
id: 09-planning.menu-flow
title: 游戏菜单流程规划 — ScreenType/GameState 路由与导航
status: draft
owner: presentation-architect
created: 2026-06-21
updated: 2026-06-22
review: P0 3/4 完成（ESC→Settings 未实现），P1/P2 场景为桩，继续活跃开发
tags:
  - ui
  - navigation
  - menu
  - game-state
  - flow
---

# 游戏菜单流程规划

## 当前问题

启动后直接 `MainMenu → Combat` 自动过渡，跳过了整个菜单体系：

```
cargo run → MainMenu(0帧) → Combat(测试战斗)
```

用户没有机会：
- 看到主菜单
- 新建游戏（选择关卡/队伍）
- 读档
- 进入设置
- 体验从主菜单到战斗的完整流程

## 两层状态架构

| 层 | 类型 | 职责 | 切换代价 |
|---|------|------|---------|
| GameState | Bevy State | 世界模式切换（卸载/加载场景） | 高：despawn 旧场景、spawn 新场景 |
| ScreenType | UI Overlay | UI 面板叠加（不切换世界） | 低：推入/弹出 UI 面板 |

### 屏幕 vs 状态映射

| 可见内容 | GameState | ScreenType | 说明 |
|---------|-----------|------------|------|
| 主菜单背景 + 按钮 | MainMenu | MainMenu | 默认状态，启动可见 |
| 战斗网格 + HUD | Combat | Battle | 战斗中 |
| 战斗 + 背包覆盖 | Combat | Battle + Inventory(overlay) | 战斗中打开背包 |
| 战术地图 | TacticalMap | — | 大地图探索 |
| 设置面板 | 任意 | Settings | 在任何状态都可打开 |

## 完整导航图

```
                    ┌──────────────────────────────────────┐
                    │         App Launch / cargo run         │
                    └────────────────┬─────────────────────┘
                                     │
                                     ▼
                            ┌─────────────────┐
                    ┌──────│   MainMenu      │──────┐
                    │      │  (GameState)    │      │
                    │      └────────┬────────┘      │
                    │               │               │
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────┐
            │  NewGame  │   │ LoadGame  │   │ Settings  │
            │  (按钮)   │   │  (按钮)   │   │  (按钮)   │
            └─────┬─────┘   └─────┬─────┘   └─────┬─────┘
                  │               │               │
                  ▼               ▼               │
          ┌──────────────┐  ┌──────────┐          │
          │ SaveLoadScreen│  │ SaveLoad │          │
          │ (选择存档)   │  │ (读取)   │          │
          └──────┬───────┘  └────┬─────┘          │
                 │               │                │
                 ▼               ▼                │
          ┌──────────────┐  ┌──────────┐          │
          │  PartySetup  │  │  Combat  │          │
          │  (GameState) │  │ (直接进) │          │
          └──────┬───────┘  └──────────┘          │
                 │                                │
                 ▼                                │
          ┌──────────────┐                        │
          │  TacticalMap │                        │
          │  (GameState) │                        │
          └──────┬───────┘                        │
                 │                                │
          ┌──────┴──────┐                         │
          ▼              ▼                        ▼
   ┌───────────┐  ┌──────────┐            ┌──────────┐
   │  Combat   │  │  Shop    │            │ Settings │
   │(GameState)│  │(Overlay) │            │ (Screen) │
   └─────┬─────┘  └──────────┘            └──────────┘
         │
         ▼
   ┌───────────┐
   │  Result   │
   │(GameState)│
   └─────┬─────┘
         │
         ▼
   ┌───────────┐      ┌───────────┐
   │ GameOver  │ ──── │ MainMenu  │
   │(GameState)│      │ (回到)    │
   └───────────┘      └───────────┘
```

## 当前实现状态

### ✅ 已实现的屏幕

| 屏幕 | GameState | 入口 | 定位 |
|------|-----------|------|------|
| MainMenu | MainMenu | 默认启动 | ✅ 有按钮但从未显示（自动跳过） |
| Battle | Combat | 启动自动进入 | ✅ 9-zone 布局完成 |
| Inventory | Combat(覆盖) | 游戏中打开 | ✅ 有 spawn/despawn |
| Shop | TacticalMap(覆盖) | 进入商店 | ✅ 有 spawn/despawn |
| Settings | 任意(覆盖) | 主菜单或游戏中 | ✅ 有 spawn/despawn |
| SaveLoad | 任意(覆盖) | 主菜单或游戏中 | ✅ 有 spawn/despawn |

### ❌ 未实现的状态

| GameState | 预期内容 | 现状 | 优先级 |
|-----------|---------|------|--------|
| PartySetup | 队伍编成界面 | 只有空 OnEnter | P1 |
| TacticalMap | 大地图探索 | 只有空 OnEnter | P1 |
| Result | 战斗结算界面 | 只有空 OnEnter | P2 |
| CampRest | 营地界面 | 只有空 OnEnter | P2 |
| GameOver | 游戏结束画面 | 只有空 OnEnter | P2 |

### 🔄 已实现的过渡

| 从 | 到 | 触发 | 实现方式 | 状态 |
|---|----|------|---------|------|
| MainMenu | Combat | 启动自动 | Startup system | ❌ 太粗暴，应改为用户点击 NewGame |
| MainMenu | — | NewGame 按钮 | UiCommand::NewGame → GameCommand::NewGame | ✅ 但 GameCommand::NewGame → 无 handler |
| MainMenu | — | LoadGame 按钮 | UiCommand::OpenScreen(SaveLoad) | ✅ |
| MainMenu | — | Settings 按钮 | UiCommand::OpenScreen(Settings) | ✅ |
| Battle | — | EndTurn 按钮 | UiCommand::EndTurn → GameCommand::EndTurn | ✅ |
| Battle | — | Inventory 按钮 | [P1] 待接线 | ❌ |

### ❌ 缺失的过渡

| 从 | 到 | 需要 | 优先级 |
|---|----|------|--------|
| MainMenu/NewGame | PartySetup | 处理 GameCommand::NewGame → 切换到 PartySetup | P0 |
| PartySetup | TacticalMap | 选择队伍后进入地图 | P1 |
| TacticalMap | Combat | 遭遇敌人 → 切换状态 | P1 |
| TacticalMap | Shop(覆盖) | 进入商店 → PushOverlay | P1 |
| Combat | Result | 战斗结束 → 切换 | P2 |
| Result | MainMenu | 结算完毕 → 回到主菜单 | P2 |
| Combat | GameOver | 全灭 → 切换 | P2 |
| GameOver | MainMenu | 点击返回 → 切换 | P2 |
| 任意 | Settings(覆盖) | ESC 键 → PushOverlay | P0 |

## 第一阶段实施（P0）

### 1. 移除启动自动跳 Combat

删除 `app_plugin.rs` 中的 startup system:
```rust
// 删除这行：
app.add_systems(Startup, |mut next: ResMut<NextState<GameState>>| {
    next.set(GameState::Combat);
});
```

改为默认 `GameState::MainMenu`（已是 `#[default]`），让用户看到主菜单。

### 2. 处理 NewGame 命令

`GameCommand::NewGame` 是个元命令（在 `default_command_handler` 中返回 `Dispatched`），但没有任何系统处理后跳到 `PartySetup`。

添加一个 observer 监听 `CommandExecuted` + `NewGame` 并设置 `NextState<GameState>::PartySetup`：

```rust
pub fn on_new_game_command(
    trigger: On<CommandExecuted>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if matches!(&trigger.event().command, GameCommand::NewGame) {
        next_state.set(GameState::PartySetup);
    }
}
```

### 3. PartySetup → Combat 测试入口

PartySetup 没有内容 — 直接加个"Quick Battle"按钮进入 Combat 测试战斗，跳过 PartySetup 的选择流程。

### 4. ESC → Settings

在任意非菜单状态按 ESC 时，打开 Settings overlay（PushOverlay 或 UiCommand::OpenScreen）。

## 第二阶段（P1）

| 任务 | 说明 |
|------|------|
| PartySetup 界面 | 队伍编成 UI（选角色/装备/技能） |
| TacticalMap 基础 | 大地图 + 节点/关卡选择 |
| TacticalMap → Combat | 遭遇敌人时切换 |
| 战斗内打开背包 | BattleScreen 中增加 Inventory 按钮 |
| SaveLoad 全流程 | 存档/读档真正写盘 |

## 第三阶段（P2）

| 任务 | 说明 |
|------|------|
| Result 结算界面 | 经验/掉落展示 |
| CampRest 营地 | 短休/长休/队伍管理 |
| GameOver | 胜利/失败画面 |
| 全流程整合 | MainMenu → PartySetup → TacticalMap → Combat → Result → MainMenu |
