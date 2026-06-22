---
id: 05-testing.README
title: Testing
status: stable
owner: test-guardian
created: 2026-06-14
updated: 2026-06-16
tags:
  - testing
---

# Testing

测试规范与文档。

## 核心原则

> **测试跟领域走（Feature First），但不写在源码文件内部。**

- 🟥 禁止 `#[cfg(test)] mod tests` 内联测试（对 AI 上下文污染严重）
- 🟥 禁止将所有测试平铺到根 `tests/unit/`（后期变成大杂烩）
- 🟩 测试与被测领域同目录放置，形成 Feature Folder 结构
- 🟩 根 `tests/` 仅保留跨领域测试（战斗流程、存档、回归、E2E）

## 文档列表

| 文件 | 主题 |
|------|------|
| `test-spec.md` | 测试宪法 — 测试分层、回放测试、覆盖率策略 |
| `testing-rules.md` | 测试金字塔、不变量测试、CI 门禁规则 |
| `test-guardian-batch3-4-2026-06-18.md` | Test Guardian Batch 3+4 — 测试守护扫描报告 |

## 领域内聚测试结构（四层）

每个领域/能力模块内部自包含测试：

```
<domain>/
├── components/
├── systems/
├── events/
├── services/
├── tests/
│   ├── unit/          # 单元测试：验证领域纯函数、核心规则
│   ├── integration/   # 集成测试：验证领域内多组件协作
│   ├── invariant/     # 不变量测试：验证领域不变量（**最核心**）
│   └── fixtures/      # 测试数据（Builder 模式 / RON 文件）
```

## 四层测试定义

| 层 | 名称 | 职责 | 示例 |
|------|------|------|------|
| **unit** | 单元测试 | 验证单个函数/纯规则的正确性 | HP 计算、Tag 包含检查、Modifier 优先级 |
| **integration** | 集成测试 | 验证领域内多组件协作 | 装备穿戴→Modifier→Attribute 联动 |
| **invariant** | 不变量测试 | 验证领域不变量（**最高价值**） | Tag bit 唯一、Buff 不重复叠加、HP>=0 |
| **fixtures** | 测试数据 | Builder 模式构造的测试数据 | RON 格式角色模板、技能配置 |

## 不变量测试（最重要）

SRPG 核心架构（Attribute / Tag / Effect / Modifier / Buff / Skill / Turn）有大量领域不变量：

| 不变量 | 说明 |
|--------|------|
| Tag bit 唯一 | 同一 Tag 不能在位掩码中重复设置 |
| Buff 不重复叠加 | 同源同类型 Buff 不会无限堆叠 |
| Effect 不修改不存在属性 | Effect 引用的 AttributeId 必须已注册 |
| HP 永远 >= 0 | HP 计算结果不能为负 |
| Modifier 不改变基础值 | Modifier 只影响聚合后的当前值 |
| 回合先攻排序稳定 | 同先攻值的单位顺序确定 |
| 技能消耗原子性 | 消耗失败时不产生部分效果 |

> 不变量测试的价值远大于普通单元测试，是架构稳定性的最后防线。

## 跨领域测试（根 tests/）

仅保留不属于任何单一领域的跨域测试：如：

```
tests/
├── battle_flow/     # 完整战斗流程
├── save_load/       # 存档/读档完整性
├── regression/      # 回归测试（历史 Bug 复现）
├── replay/          # 回放确定性
├── golden/          # 金文件对比
├── simulation/      # 战斗模拟与数值平衡
├── performance/     # 性能回归
└── e2e/             # 端到端测试
```
