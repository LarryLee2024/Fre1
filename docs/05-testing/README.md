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

## 文档列表

| 文件 | 主题 |
|------|------|
| `test-spec.md` | 测试宪法 v3.1 — 测试分层、回放测试、覆盖率策略 |
| `test-spec-json.md` | 测试规范 JSON 格式定义 |
| `testing-rules.md` | 测试金字塔、回放测试规范 |

## 测试体系

项目采用五层测试金字塔：
- **单元测试 (70%)** — 验证规则（伤害、Buff、属性、寻路）
- **领域集成测试 (20%)** — 验证 Feature（装备、背包、战斗）
- **回放测试 (8%)** — 验证状态流（Battle Replay，Seed=42 确定性）
- **E2E 测试 (2%)** — 完整游戏流程验证
- **Testbeds** — 性能基准测试

测试代码位于 `tests/` 目录，包含 `e2e/`、`golden/`、`integration/`、`rule/`、`system/` 子目录及 `common/` 共享工具。
