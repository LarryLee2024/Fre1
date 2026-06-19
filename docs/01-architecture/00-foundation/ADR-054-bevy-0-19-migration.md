---
id: ADR-054
title: Bevy 0.19 迁移决策（Observer 优先 / Delayed Commands / BSN 范围 / Relationship）
status: accepted
date: 2026-06-19
tags:
  - bevy-0.19
  - migration
  - observer
  - delayed-commands
  - bsn
  - relationship
---

# ADR-054: Bevy 0.19 迁移决策

## 背景

项目已升级至 `bevy = "0.19.0-rc.3"`，且大量代码已在实际开发中采用 0.19 API（Observer/Trigger、`ButtonInput<T>`、`Single<T>`、`AppExit` 等）。需要正式记录迁移决策，为后续开发和文档提供依据。

## 采用矩阵

### 已采用（代码库已全面使用）

| 特性 | 说明 |
|------|------|
| Observer + trigger | 全面替代 EventWriter/EventReader |
| `ButtonInput<T>` | 替代旧 `Input<T>` |
| `AppExit` | `fn main() -> AppExit` |
| `Single<T>` | 替代 `Res<T>` 注入 |

### 全量迁移（Phase 1）

| 特性 | 说明 |
|------|------|
| Run Conditions + Observer run_if | 替代系统/Observer 内的 if 守卫 |
| Delayed Commands | 替代一次性 Timer 轮询（3 处） |
| `FontSize` 枚举 | `font_size` 字段类型变更 |
| Reflect 全覆盖 | 所有 Component/Event/Resource 补 Reflect |
| DiagnosticsOverlay | Dev 模式诊断工具 |
| User Settings | 统一设置管理 |

### 有条件采用（Phase 2）

| 特性 | 范围 | 条件 |
|------|------|------|
| BSN (bsn!) | UI 层（app/scenes） | 核心玩法层用工厂函数 |
| Relationship | 核心关系（CasterOf/TargetOf） | 临时引用/值语义继续用 Entity 字段 |
| Resource → Singleton Entity | 评估后决定 | 观察 0.19 稳定版 API |
| Contiguous Query | 热点场景 | 性能不足时启用 |

### 忽略

| 特性 | 理由 |
|------|------|
| Solari / Skinned Mesh / Bindless | 3D 特性，与 2D SRPG 无关 |
| SceneComponent | 等待 BSN asset loader 稳定 |
| Render Graph as Systems | 与本项目渲染架构无关 |

## 决策详情

### DR-001: Observer 是默认跨领域通信机制

- **决策**：所有跨 Feature 事件使用 `trigger(T)` + `On<T>` Observer
- **禁止**：`EventWriter<T>` / `EventReader<T>`
- **理由**：Observer 支持 run_if 条件守卫、自动注册、Entity 级作用域

### DR-002: Delayed Commands 替代一次性 Timer

- **决策**：一次性延迟效果使用 `Delayed<T>` 或 `FreDelayed<T>` 包装
- **保留 Timer**：基础设施周期性任务（热重载/审计）、可暂停的长周期 Buff
- **理由**：消除样板代码，声明式生命周期

### DR-003: BSN 作用域限制

- **决策**：BSN 仅允许用于声明式静态场景，禁止用于可复用 Widget 和 Screen
- **允许范围**：
  - `src/app/scenes/` ✅ — Composition Root，一次性装配
  - Editor Prototype ✅ — 快速原型
  - Debug UI ✅ — 工具不涉及业务
- **禁止范围**：
  - `src/ui/screens/` 🟥 — 禁止 BSN，Screen 有复杂生命周期
  - `src/ui/widgets/` 🟥 — 禁止 BSN，Widget 需要 Factory 契约
- **替代方案**：所有 Screen/Widget 通过 Factory 构建（`spawn_xxx(commands, props)` 或 `XxxFactory`），Factory 是 UI 的唯一构建入口
- **BSN 使用约束**：BSN 内容必须保持无状态、无逻辑、无业务语义
- **理由**：BSN API 可能变动；BSN 树结构隐藏边界，AI 易在其中塞入状态和逻辑；Factory 模式天然形成 Widget Contract 边界，更利于测试、复用、AI 独立生成

### DR-004: Relationship 有限采用

- **决策**：CasterOf / TargetOf / SummonedBy 等核心关系使用 Relationship
- **不适用**：临时引用（当前选中单位）、值语义（队伍 ID）
- **理由**：关系查询、架构清晰，但仅限明确场景

### DR-005: Reflect 全覆盖

- **决策**：所有 Component/Event/Resource 必须 derive Reflect
- **理由**：编辑器支持、序列化、Scene 兼容

## 准出条件

- `cargo check` 零错误
- `cargo nextest run` 全绿
- `cargo clippy -- -D warnings` 零警告
- 宪法/架构/领域/数据/测试文档全部对齐 0.19
