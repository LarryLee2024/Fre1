# docs/09-planning — 规划文档

本目录存放项目执行规划文档。已完成的规划归档至 `done/`。

## 目录结构

```
09-planning/
├── README.md                                     # 本文件
├── bevy-0.19-migration-v3-aggressive.md          # ✅ 活跃 — 迁移总纲 v3.1
│
├── bevy-0.19-migration-compatible.md             # 🔴 已被 v3.1 取代
├── bevy-0.19-migration-domain-checklist.md       # 🔴 已被 v3.1 取代
├── bevy-0.19-migration-features.md               # 🔴 已被 v3.1 取代
├── bevy-0.19-migration-future.md                 # ✅ 参考 — 迁移完成后的架构展望
├── bevy-0.19-migration-plan-overview.md          # 🔴 已被 v3.1 取代
├── new_bevy-0.19-migration-master-plan.md        # 🔴 已被 v3.1 取代
├── new_bevy-0.19-module-checklist.md             # 🔴 已被 v3.1 取代
├── new_bevy-0.19-phase1-aggressive.md            # 🔴 已被 v3.1 取代
├── new_bevy-0.19-phase2-deep-refactor.md         # 🔴 已被 v3.1 取代
│
└── done/                                    # 已完成的规划
```

## 规划文档生命周期

1. **创建** → 文件置于 `09-planning/` 根目录
2. **执行中** → 按规划推进，更新任务状态
3. **完成** → 移动至 `done/`，更新本 README

## 状态说明

### 活跃规划文档

| 文档 | 状态 | 说明 |
|------|------|------|
| `bevy-0.19-migration-v3-aggressive.md` | ✅ v3.1 活跃 | 已完成 ~70%：宪法/架构/规则文档全部对齐，Cutscene Delayed 迁移完成，30+ 类型 Reflect，DiagnosticsOverlay 注册，bevy-inspector-egui 已移除。剩余深层 Reflect 递归 + User Settings 等待 0.19 稳定版 |

### 参考文档

| 文档 | 状态 | 说明 |
|------|------|------|
| `bevy-0.19-migration-future.md` | ✅ 参考 | 迁移完成后的架构展望与宪法级新规则（已并入 v5.2） |

### 已被取代的旧版迁移方案

以下 8 个文件均为 v1.0–v3.0 旧版方案，已被 `bevy-0.19-migration-v3-aggressive.md` v3.1 取代。留作历史参考：

| 文档 | 状态 | 说明 |
|------|------|------|
| `bevy-0.19-migration-compatible.md` | 🔴 取代 | Phase A 核心系统并行重写 |
| `bevy-0.19-migration-domain-checklist.md` | 🔴 取代 | 134 项检查清单 |
| `bevy-0.19-migration-features.md` | 🔴 取代 | Phase B+C 架构现代化 |
| `bevy-0.19-migration-plan-overview.md` | 🔴 取代 | v2.0 迁移总纲 |
| `new_bevy-0.19-migration-master-plan.md` | 🔴 取代 | 激进重构总纲 v3.0 |
| `new_bevy-0.19-module-checklist.md` | 🔴 取代 | 逐模块检查清单 |
| `new_bevy-0.19-phase1-aggressive.md` | 🔴 取代 | Phase 1 全面升级 |
| `new_bevy-0.19-phase2-deep-refactor.md` | 🔴 取代 | Phase 2 深度重构 |

### 已完成归档

- `done/` 中的所有文档均已全部完成
- Phase E（游戏流程集成）已于 2026-06-25 完成并归档
- Localization 基础设施实施已于 2026-06-19 完成并归档
- 日志架构实施已于 2026-06-26 完成并归档
