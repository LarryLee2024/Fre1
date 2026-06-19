# docs/09-planning — 规划文档

本目录存放项目执行规划文档。已完成的规划归档至 `done/`。

## 目录结构

```
09-planning/
├── README.md                                     # 本文件
├── bevy-0.19-migration-v3-aggressive.md          # ✅ v3.2 活跃 — 迁移总纲（~85%）
│
└── done/                                         # 已完成的规划（含全部旧版迁移方案）
    ├── bevy-0.19-migration-compatible.md         # 旧 v1.0 Phase A
    ├── bevy-0.19-migration-domain-checklist.md   # 旧 v1.0 检查清单
    ├── bevy-0.19-migration-features.md           # 旧 v1.0 Phase B+C
    ├── bevy-0.19-migration-future.md             # 架构展望（内容已并入 v5.2）
    ├── bevy-0.19-migration-plan-overview.md      # 旧 v2.0 迁移总纲
    ├── new_bevy-0.19-migration-master-plan.md    # 旧 v3.0 总纲
    ├── new_bevy-0.19-module-checklist.md         # 旧 v3.0 检查清单
    ├── new_bevy-0.19-phase1-aggressive.md        # 旧 v3.0 Phase 1
    ├── new_bevy-0.19-phase2-deep-refactor.md     # 旧 v3.0 Phase 2
    └── ... (其他已完成规划)
```

## 规划文档生命周期

1. **创建** → 文件置于 `09-planning/` 根目录
2. **执行中** → 按规划推进，更新任务状态
3. **完成** → 移动至 `done/`，更新本 README

## 状态说明

### 活跃规划文档

| 文档 | 状态 | 说明 |
|------|------|------|
| `bevy-0.19-migration-v3-aggressive.md` | ✅ v3.2 活跃 | 已完成 ~85%：宪法/架构/规则文档全部对齐，Cutscene Delayed 迁移完成，30+ 类型 Reflect，DiagnosticsOverlay 注册，bevy-inspector-egui 已移除，全量文档版本替换完成，16 处测试编译修复，1530 测试全绿。剩余：深层 Reflect 递归 + User Settings 等待 0.19 稳定版 |

### 已完成归档（迁移相关）

以下 9 个旧版迁移方案已全部移至 `done/`：

| 文档 | 原状态 | 说明 |
|------|--------|------|
| `bevy-0.19-migration-plan-overview.md` | v2.0 总纲 | 被 v3.1 取代 |
| `bevy-0.19-migration-compatible.md` | v1.0 Phase A | 被 v3.1 取代 |
| `bevy-0.19-migration-domain-checklist.md` | v1.0 检查清单 | 被 v3.1 取代 |
| `bevy-0.19-migration-features.md` | v1.0 Phase B+C | 被 v3.1 取代 |
| `bevy-0.19-migration-future.md` | 架构展望 | 内容已并入宪法 v5.2 |
| `new_bevy-0.19-migration-master-plan.md` | v3.0 总纲 | 被 v3.1 取代 |
| `new_bevy-0.19-module-checklist.md` | v3.0 检查清单 | 被 v3.1 取代 |
| `new_bevy-0.19-phase1-aggressive.md` | v3.0 Phase 1 | 被 v3.1 吸收 |
| `new_bevy-0.19-phase2-deep-refactor.md` | v3.0 Phase 2 | 被 v3.1 吸收 |

以下 11 篇 Bevy 0.19 迁移技术知识库文档已归档至 `done/`（`docs/03-technical/bevy-0.19-migration/` → `docs/09-planning/done/`）：

| 文档 | 说明 |
|------|------|
| `00-migration-overview.md` | 迁移总览 |
| `01-bsn-scene-system.md` | BSN 场景系统 |
| `02-observer-enhancements.md` | Observer 增强 |
| `03-delayed-commands.md` | Delayed Commands |
| `04-resources-as-components.md` | Resource → Entity |
| `05-contiguous-query.md` | Contiguous Query |
| `06-user-settings.md` | User Settings |
| `07-text-and-ui.md` | 文本与 UI |
| `08-rendering-and-devtools.md` | 渲染与 DevTools |
| `09-asset-system.md` | 资产系统 |
| `10-srpg-architecture-impact.md` | SRPG 架构影响 |

### 其他已完成归档

- `done/` 中其余规划文档均已全部完成
- Phase E（游戏流程集成）已于 2026-06-25 完成并归档
- Localization 基础设施实施已于 2026-06-19 完成并归档
- 日志架构实施已于 2026-06-26 完成并归档
