---
name: project-open-menu-esc-bridge
description: ESC→Settings 桥接（GameCommand::OpenMenu → UiCommand::OpenScreen）
metadata:
  type: project
  updated: 2026-06-23
---

# ESC→Settings 快捷键桥接

实现了 `ESC 键 → Settings Screen` 的完整交互链。

## 实现

- **Observer**: `src/app/scenes/open_menu.rs` — 监听 `CommandExecuted(GameCommand::OpenMenu)` → 触发 `UiCommand::OpenScreen(ScreenType::Settings)`
- **注册**: `src/app/scenes/plugin.rs`

## 数据流

```
ESC key → InputAction → GameCommand::OpenMenu → command_processing_system
  → CommandExecuted { command: OpenMenu }
  → on_menu_command_executed observer
  → UiCommand::OpenScreen(Settings)
  → ScreenStack.push(SettingsScreen)
```

**Why:** 实现 ESC→Settings 的 P0 需求。采用 Observer 解耦（不修改 command_processing_system），遵循架构的四级通信优先级（Observer 优先）。

**How to apply:**
- 新增 GameCommand→UiCommand 的桥接时，在 `src/app/scenes/` 下创建新的 observer 模块
- 遵循 `on_<action>_command_executed` 命名惯例
- 注册到 `src/app/scenes/plugin.rs`
