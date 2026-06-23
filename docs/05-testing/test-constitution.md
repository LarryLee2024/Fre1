---
id: TEST-CONSTITUTION
title: 测试与确定性宪法
status: accepted
stability: stable
layer: testing
related:
  - ai-constitution-complete.md
tags:
  - testing
  - determinism
  - invariant
  - replay
---

> **原文来源**：`ai-constitution-complete.md` 第十二编（L1210–L1289）
> **锚定总宪法**：第十二编

## 第十二编 测试与确定性宪法
### 12.1 测试核心原则
- 🟩 所有功能必须优先编写测试，其次才是手工验证
- 🟩 发现 Bug 后必须先编写重现测试，再修复 Bug

### 12.2 测试架构体系

#### 核心原则

> **测试跟领域走（Feature First），但不写在源码文件内部。**

- 🟥 禁止 `#[cfg(test)] mod tests` 内联测试（对 AI 上下文污染严重，AI 会误改测试、引用测试代码、浪费 token）
- 🟥 禁止将所有测试平铺到根 `tests/unit/`（200+ 文件变成垃圾场）
- 🟩 测试与被测领域同目录放置，形成 Feature Folder 结构
- 🟩 根 `tests/` 仅保留跨领域测试（战斗流程、存档、回归、E2E）

#### 领域内聚测试结构（四层）

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

#### 四层测试定义

| 层 | 名称 | 职责 | 示例 |
|------|------|------|------|
| **unit** | 单元测试 | 验证单个函数/纯规则的正确性 | HP 计算、Tag 包含检查、Modifier 优先级 |
| **integration** | 集成测试 | 验证领域内多组件协作 | 装备穿戴→Modifier→Attribute 联动 |
| **invariant** | 不变量测试 | 验证领域不变量（**最高价值**） | Tag bit 唯一、Buff 不重复叠加、HP>=0 |
| **fixtures** | 测试数据 | Builder 模式构造的测试数据 | RON 格式角色模板、技能配置 |

#### 不变量测试（最重要）

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

#### 跨领域测试（根 tests/）

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

### 12.3 测试基础设施
- 🟩 核心测试必须使用 Builder 模式构造测试数据，禁止每个测试手动构造大量实体
- 🟩 稳定输出必须使用金文件对比测试，版本升级后输出变化必须显式确认

### 12.4 确定性要求
- 🟩 战斗完全可重现：相同初始状态 + 相同输入序列 + 相同 RNG 种子，必须得到完全一致的战斗结果
- 🟩 禁止业务逻辑依赖系统时间，必须使用统一的 GameTime 服务
- 🟩 所有战斗相关 Bug 必须通过 Battle Replay 重现并转化为永久测试用例
- 🟩 测试必须覆盖所有核心规则，不追求表面的覆盖率数字
