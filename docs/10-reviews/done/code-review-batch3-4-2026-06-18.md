---
id: 10-reviews.code-review-batch3-4
title: 代码审查 — 第 3+4 批（法术 / 反应 / 任务 / 经济 / 制作 / 召唤）
status: completed
owner: code-reviewer
created: 2026-06-18
tags:
  - code-review
  - batch3
  - batch4
---

# 代码审查报告 — 第 3+4 批

## 审查范围

合并执行计划第 3+4 批实现的 6 个骨架领域：
- **第 3 批**：法术、反应
- **第 4 批**：任务、经济、制作、召唤

总计：约 6,500 行，约 80 个文件。

## ✅ 通过的检查项

### 架构合规性
- [x] Feature First：所有领域都遵循标准结构（components/error/events/resources/rules/systems/plugin/mod）
- [x] 无跨领域直接导入 — 所有内部引用均使用 `super::super::*` 模式
- [x] 领域边界受尊重：没有 Capabilities 类型逻辑泄漏到系统中
- [x] 无类型错误抑制（业务代码中没有 `as any`、`#[allow()]`）
- [x] 插件注册：全部 6 个领域插件已添加到 `core_plugin.rs`
- [x] 尚无 `integration/` 层 — 在骨架阶段可接受（已记录为 `tests/integration/` 缺口）
- [x] Core 层不依赖 Infra 或 App 层

### ECS 模式合规性
- [x] 组件为纯数据 — impl 块中无业务逻辑
- [x] 系统为无状态函数
- [x] Observer 模式正确使用 `On<T>`、`commands.trigger()`
- [x] 未发现 OOP 模式（如 `entity.attack()`）
- [x] 未使用布尔字段替代 Tag 组件（法术 `SpellComponents` 是合法的 D&D 数据，除外）

### Rust 代码质量
- [x] 业务代码中无 `unwrap()` / `expect()`
- [x] 规则函数中正确处理 `Result`
- [x] pub 可见性适当限定（默认私有）
- [x] 无不必要的 trait 抽象
- [x] 无全局可变状态
- [x] 使用迭代器模式（`.iter_mut().find()`、`.retain()`）

### Bevy 0.18 最佳实践
- [x] 跨领域通信通过 Event（无直接数据引用）
- [x] 组件使用 `register_type` 注册
- [x] 资源使用 `init_resource` 初始化
- [x] 领域事件采用 Observer 驱动流程

## ❌ 发现的问题

### 中等问题

#### M1. 经济领域观察者中存在未使用的参数
- **位置**：`src/core/domains/economy/systems/economy_system.rs:19-20, 70-71`
- **规则**：Rust 代码质量 — 声明但未使用的参数
- **描述**：`shop_query: Query<&mut ShopInstance>` 和 `economy_config: Res<EconomyConfig>` 在 `on_purchase_request` 和 `on_sell_request` 中都声明了，但两者均未被使用（商店库存管理被简化掉了）。这会产生编译器警告。
- **严重程度**：中等（警告，无行为影响）
- **建议**：删除未使用的参数；如果保留供未来使用，则添加 `#[allow(dead_code)]`；更好的做法是保留并添加 TODO 注释说明何时重新启用。

#### M2. `use` 语句位于 economy_system.rs 底部
- **位置**：`src/core/domains/economy/systems/economy_system.rs:104`
- **规则**：Rust 代码规范 — `use` 语句应位于模块顶部
- **描述**：`use super::super::components::CurrencyType;` 出现在第 104 行，在所有函数定义之后，而非与其他导入语句一起放在顶部。
- **严重程度**：中等（规范违反）
- **建议**：移动到文件顶部其他 `use` 语句处。

#### M3. 任务系统中存在空函数 `accept_quest`
- **位置**：`src/core/domains/quest/systems/quest_system.rs:15-18`
- **规则**：死代码 — 函数已定义、公开且在 plugin.rs 中注册，但函数体为空
- **描述**：`pub fn accept_quest(...)` 函数体为空且从未被调用（实际逻辑在 `on_accept_quest_request` 中）。它在 `systems/mod.rs` 中声明，但未在 plugin.rs 中注册。
- **严重程度**：中等（死代码干扰）
- **建议**：删除该函数，或将其转换为说明委派模式的文档注释。

#### M4. summon_system.rs 中被注释掉的死代码
- **位置**：`src/core/domains/summon/systems/summon_system.rs:53-60`
- **规则**：死代码 — 被注释掉的代码块应有明确的保留期限
- **描述**：`on_caster_died` 被完全注释掉并带有 TODO。这在骨架阶段可以接受，但应被跟踪。
- **严重程度**：低（骨架阶段可接受，但应跟踪）
- **建议**：将 TODO 转换为 `docs/09-planning/` 中带优先级的跟踪任务项。

### 低等问题

#### L1. 法术单元测试中冗余的 `#[cfg(test)]`
- **位置**：`src/core/domains/spell/tests/unit/mod.rs`（所有测试模块）
- **规则**：代码一致性 — `tests/` 目录中的 `#[cfg(test)]` 模块是冗余的
- **描述**：法术测试在每个测试组外围使用 `#[cfg(test)] mod ... { ... }` 包装。其他 5 个领域使用扁平的 `#[test]` 函数而无模块包装。这造成不一致。
- **严重程度**：低（外观问题，无行为影响）
- **建议**：从模块声明中移除 `#[cfg(test)]` 以匹配项目规范（或者如果偏好显式属性则保留）。

#### L2. mod.rs 注释详略不一致
- **位置**：`src/core/domains/{economy,crafting,summon}/mod.rs`
- **规则**：文档质量 — 其他领域的 mod.rs 有更详细的注释
- **描述**：任务和反应领域的 mod.rs 有详细的模块文档注释。经济、制作、召唤的 mod.rs 只有简短的一行描述。
- **严重程度**：低（文档一致性）
- **建议**：添加少量多行文档注释以匹配项目标准。

#### L3. 经济系统中硬编码的价格修正系数
- **位置**：`src/core/domains/economy/systems/economy_system.rs:31-33`
- **规则**：内容债务 — 业务值应从配置中获取
- **描述**：`reputation_modifier: 1.0, supply_modifier: 1.0, stolen_modifier: 1.0` 在购买处理器中硬编码。这些应使用 `EconomyConfig` 中的值。
- **严重程度**：低（骨架阶段，配置系统尚未完全集成）
- **建议**：在配置加载激活时将其标记为内容债务。

## 总结

| 严重程度 | 数量 |
|----------|------|
| 严重 | 0 |
| 高 | 0 |
| 中等 | 4 |
| 低 | 3 |

## 结论

**结果：通过**

未发现严重或高等问题。4 个中等问题主要是编译器警告和代码规范问题。3 个低等问题属于外观性的。

实现代码在结构上是健全的：
- 架构和 ECS 模式正确遵循
- 领域边界得到尊重
- 跨领域通信采用事件驱动
- 全部 6 个领域编译通过且测试编译通过

### 建议操作
1. 修复 M1、M2、M3（未使用的参数、位置错误的 use 语句、空函数） — 快速清理，约 5 分钟
2. 在后续阶段添加集成测试脚手架（需要 Bevy App 构建器模式）
3. 将 summon_system.rs 中的死代码作为规划项跟踪
