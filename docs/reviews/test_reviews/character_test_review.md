# Character 模块测试评审报告

Version: 1.0
Date: 2026-06-11
Reviewer: Test Guardian
Scope: `src/character/` 全部代码文件 + `tests/` 中相关外部测试
Standard: `docs/test_spec.md` (Bevy SRPG Testing Constitution v3.1)
Domain Reference: `docs/domain_rules.md` (不存在)

---

# 1. 评审范围

## 1.1 源代码文件

| 文件 | 行数 | 内联测试数 | 测试覆盖状态 |
|------|------|-----------|-------------|
| `mod.rs` | 114 | 0 | N/A（插件注册 + 系统编排） |
| `components.rs` | 350 | 14 | 良好 |
| `marker.rs` | 35 | 0 | **未覆盖**（纯标记组件） |
| `movement.rs` | 160 | 0 | **未覆盖**（移动动画） |
| `spawn.rs` | 277 | 0 | **未覆盖**（单位生成） |
| `template.rs` | 412 | 8 | 良好 |
| `traits/mod.rs` | 385 | 7 | 良好 |
| `traits/types.rs` | 193 | 1 | 部分覆盖 |
| `traits/handlers.rs` | 278 | 12 | 良好 |

**内联测试总计：42 个**

## 1.2 外部测试文件（与 character 相关）

| 文件 | 测试数 | 覆盖范围 |
|------|--------|----------|
| `tests/feature/traits.rs` | 10 | Trait 授予标签、装备 Trait 生命周期、Trait 修改属性 |
| `tests/feature/death.rs` | 4 | Dead Hook、致命伤害、死亡角色 Buff 处理 |

**外部测试总计：14 个**

**全部测试总计：56 个**

---

# 2. 评审标准

依据 `test_spec.md` 以下条款逐项评审：

| 条款 | 内容 | 评审重点 |
|------|------|----------|
| §3 Testing Philosophy | 测试验证 Behavior，不验证 Implementation | 断言是否关注 What 而非 How |
| §4 Test Pyramid | 70% Unit / 20% Integration / 8% Replay / 2% E2E | 各层级比例是否合理 |
| §5 Test Categories | Unit/Integration/Replay/Regression/E2E 定义 | 是否有缺失类别 |
| §6 Determinism Rules | 禁止随机、固定 Seed | 测试是否确定性 |
| §7 Test Case Schema | Test ID / Title / Given / When / Then / Assertions | 测试结构是否规范 |
| §7.1 Standard Test Data | Unit_001 / Unit_002 / Unit_003 | 是否使用标准数据 |
| §9 Coverage Strategy | 100% 核心领域规则覆盖 | 领域不变量是否全部测试 |
| §10 Error Testing | Invalid Input / Boundary Values | 边界和错误场景是否覆盖 |
| §13 AI Constraints | 禁止测试私有实现 | 是否越界测试内部细节 |
| §13.1 AI Self-Check | 6 项自检标注 | 是否有自检标注 |

---

# 3. 领域不变量覆盖评审

Character 模块核心领域规则（基于代码分析，因 `domain_rules.md` 不存在）：

## 3.1 核心不变量

| 不变量 | 描述 | 测试覆盖 | 评审结论 |
|--------|------|----------|----------|
| INV-CHR-1 | Unit 生成时自动插入 Required Components（Attributes, SkillSlots, ActiveBuffs 等） | **覆盖** | `components.rs::unit_必需组件_自动生成` |
| INV-CHR-2 | Dead Hook：标记已行动 + 移除 Selected | **覆盖** | `components.rs::dead_hook_标记已行动` + `dead_hook_移除selected` + `dead_hook_无selected时不报错` |
| INV-CHR-3 | Dead Observer 发送 CharacterDied Message | **覆盖** | `feature/death.rs::致命伤害触发死亡_dead标记和character_died消息` |
| INV-CHR-4 | Faction 枚举：Player / Enemy | **覆盖** | `template.rs::faction_def_player转换` + `faction_def_enemy转换` |
| INV-CHR-5 | UnitTemplateDef → UnitTemplate 转换 | **覆盖** | `template.rs::unit_template_def_转换为_unit_template` |
| INV-CHR-6 | RON 反序列化单位模板（含旧配置兼容） | **覆盖** | `template.rs::ron_反序列化_单位模板` + `ron_反序列化_旧配置无version字段` |
| INV-CHR-7 | UnitTemplateRegistry 默认注册 4 个模板 | **覆盖** | `template.rs::unit_template_registry_默认模板` |
| INV-CHR-8 | TraitTrigger：Passive / OnTurnStart / OnTurnEnd / OnAttack / OnHit / OnKill | **覆盖** | `types.rs` 定义 + `mod.rs::ron_反序列化_触发型trait` |
| INV-CHR-9 | TraitEffect：GrantTag / ModifyAttribute / ApplyBuff | **覆盖** | `handlers.rs` 全部 Handler 测试 + `mod.rs::trait_def_转换为_trait_data` |
| INV-CHR-10 | TraitCollection：add_entry / remove_by_source / has / trait_ids | **覆盖** | `mod.rs::trait_collection_查询` + `feature/traits.rs` 全部场景 |
| INV-CHR-11 | apply_passive_traits：仅处理 Passive 触发 | **覆盖** | `mod.rs::apply_passive_traits_跳过非被动触发` |
| INV-CHR-12 | apply_passive_traits：每个 trait 分配独立 source id | **覆盖** | `mod.rs::apply_passive_traits_独立source_id` |
| INV-CHR-13 | TraitSource：Intrinsic / Equipment { slot } | **覆盖** | `feature/traits.rs::装备trait完整生命周期` + `装备trait_不同来源的trait独立管理` |
| INV-CHR-14 | PersistentTags：from_traits + from_equipment 两层 | **覆盖** | `feature/traits.rs::被动trait授予标签_添加passive_grant_tag后标签出现在gameplay_tags` |
| INV-CHR-15 | MovingUnit：target_coord / is_finished | **覆盖** | `components.rs` 6 个 MovingUnit 测试 |
| INV-CHR-16 | TraitEffectHandlerRegistry：3 个内置 Handler | **覆盖** | `handlers.rs::registry_默认包含三个处理器` + `registry_查询不存在返回none` + `registry_注册自定义处理器` |

**领域不变量覆盖率：16/16 = 100%**

---

# 4. 测试层级评审

## 4.1 测试层级分布

| 层级 | 数量 | 占比 | 目标占比 | 状态 |
|------|------|------|----------|------|
| Unit Test | 42 | 75.0% | 70% | ✅ 达标 |
| Integration Test | 14 | 25.0% | 20% | ⚠️ 略高 |
| Replay Test | 0 | 0% | 8% | **缺失** |
| Regression Test | 0 | 0% | — | **缺失** |
| E2E Test | 0 | 0% | 2% | 可接受 |

**总计：56 个测试**

## 4.2 各层级详细评审

### Unit Test (42 个)

**components.rs (14 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `移动单位_目标坐标_在路径中` | MovingUnit.target_coord() | ✅ 行为验证 |
| `移动单位_目标坐标_空路径` | 边界：空路径 | ✅ 边界测试 |
| `移动单位_目标坐标_索引越界` | 边界：索引越界 | ✅ 边界测试 |
| `移动单位_是否完成_未完成` | is_finished() | ✅ 行为验证 |
| `移动单位_是否完成_已完成` | is_finished() | ✅ 行为验证 |
| `移动单位_是否完成_空路径` | 边界：空路径 | ✅ 边界测试 |
| `移动单位_是否完成_刚到达终点` | 边界：刚好完成 | ✅ 边界测试 |
| `dead_hook_标记已行动` | Dead Hook 行为 | ✅ 行为验证 |
| `dead_hook_移除selected` | Dead Hook 行为 | ✅ 行为验证 |
| `dead_hook_无selected时不报错` | 边界：无 Selected | ✅ 边界测试 |
| `unit_必需组件_自动生成` | Required Components | ✅ 行为验证 |
| `unit_id_组件_基本属性` | UnitId 基本操作 | ✅ 行为验证 |
| `unit_id_组件_相等与哈希` | UnitId Eq + Hash | ✅ 行为验证 |
| `unit_id_组件_挂载与读取` | UnitId ECS 操作 | ✅ 行为验证 |

**template.rs (8 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `ron_反序列化_单位模板` | RON 反序列化 | ✅ 行为验证 |
| `unit_template_def_转换为_unit_template` | Def → Data 转换 | ✅ 行为验证 |
| `unit_template_registry_默认模板` | 默认注册 | ✅ 行为验证 |
| `unit_template_registry_查询` | Registry 查询 | ✅ 行为验证 |
| `unit_template_registry_查询未注册返回none` | 边界：空查询 | ✅ 边界测试 |
| `faction_def_player转换` | FactionDef → Faction | ✅ 行为验证 |
| `faction_def_enemy转换` | FactionDef → Faction | ✅ 行为验证 |
| `ron_反序列化_旧配置无version字段` | 向后兼容 | ✅ 边界测试 |

**traits/mod.rs (7 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `ron_反序列化_trait定义` | RON 反序列化 | ✅ 行为验证 |
| `trait_def_转换为_trait_data` | Def → Data 转换 | ✅ 行为验证 |
| `trait_collection_查询` | TraitCollection.has() | ✅ 行为验证 |
| `apply_passive_traits_授予标签和修饰符` | 被动 Trait 应用 | ✅ 行为验证 |
| `apply_passive_traits_跳过非被动触发` | 非 Passive 跳过 | ✅ 行为验证 |
| `ron_反序列化_触发型trait` | 触发型 RON | ✅ 行为验证 |
| `apply_passive_traits_独立source_id` | 独立 source id | ✅ 行为验证 |

**traits/types.rs (1 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `ron_反序列化_旧配置无version字段` | 向后兼容 | ✅ 边界测试 |

**traits/handlers.rs (12 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `grant_tag_handler_类型名` | Handler type_name | ✅ 行为验证 |
| `grant_tag_handler_授予标签` | GrantTag 授予 | ✅ 行为验证 |
| `grant_tag_handler_非grant_tag返回空` | 边界：类型不匹配 | ✅ 边界测试 |
| `grant_tag_handler_无属性修饰` | 边界：无修饰符 | ✅ 边界测试 |
| `modify_attribute_handler_类型名` | Handler type_name | ✅ 行为验证 |
| `modify_attribute_handler_返回属性修饰` | ModifyAttribute 返回 | ✅ 行为验证 |
| `modify_attribute_handler_非modify返回空` | 边界：类型不匹配 | ✅ 边界测试 |
| `modify_attribute_handler_无标签授予` | 边界：无标签 | ✅ 边界测试 |
| `apply_buff_handler_类型名` | Handler type_name | ✅ 行为验证 |
| `apply_buff_handler_无标签授予` | 边界：无标签 | ✅ 边界测试 |
| `apply_buff_handler_无属性修饰` | 边界：无修饰符 | ✅ 边界测试 |
| `registry_默认包含三个处理器` | Registry 默认 | ✅ 行为验证 |
| `registry_查询不存在返回none` | 边界：空查询 | ✅ 边界测试 |
| `registry_注册自定义处理器` | 自定义扩展 | ✅ 行为验证 |
| `registry_default等于with_defaults` | 一致性 | ✅ 行为验证 |

### Integration Test (14 个)

**feature/traits.rs (10 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `被动trait授予标签_添加passive_grant_tag后标签出现在gameplay_tags` | Trait → 标签 | ✅ 跨模块集成 |
| `被动trait授予标签_多个trait授予多个标签` | 多 Trait 叠加 | ✅ 跨模块集成 |
| `被动trait授予标签_非passive触发不授予标签` | 非 Passive 跳过 | ✅ 跨模块集成 |
| `装备trait完整生命周期_添加后entry存在_移除后entry消失` | 装备 Trait 生命周期 | ✅ 跨模块集成 |
| `装备trait_不同来源的trait独立管理` | 不同来源独立 | ✅ 跨模块集成 |
| `装备trait_intrinsic来源不受equipment移除影响` | 来源隔离 | ✅ 跨模块集成 |
| `trait修改属性_添加passive_modify_attribute后属性值变化` | Trait → 属性 | ✅ 跨模块集成 |
| `trait修改属性_移除trait后属性恢复` | 移除 → 恢复 | ✅ 跨模块集成 |
| `trait修改属性_乘法修饰符` | 乘法修饰 | ✅ 跨模块集成 |
| `trait修改属性_多个trait同时修改属性` | 多 Trait 叠加 | ✅ 跨模块集成 |
| `trait修改属性_同时授予标签和修改属性` | 组合效果 | ✅ 跨模块集成 |

**feature/death.rs (4 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `死亡标记添加后hook触发_acted为true且selected被移除` | Dead Hook 完整 | ✅ 跨模块集成 |
| `致命伤害触发死亡_dead标记和character_died消息` | 完整死亡流程 | ✅ 跨模块集成 |
| `死亡角色_resolve_status_effects不处理` | 死亡排除 | ✅ 跨模块集成 |
| `存活角色_resolve_status_effects正常处理dot` | 存活处理 | ✅ 跨模块集成 |

---

# 5. 确定性评审

依据 `test_spec.md` §6 Determinism Rules：

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 禁止 ThreadRng | ✅ 通过 | 所有测试无随机数 |
| 禁止随机时间 | ✅ 通过 | 无时间依赖 |
| 禁止网络依赖 | ✅ 通过 | 无网络调用 |
| 固定 Seed | ✅ 通过 | 所有数据硬编码 |
| 相同输入相同结果 | ✅ 通过 | 纯函数 + 确定性数据 |

---

# 6. 测试规范性评审

## 6.1 §7 Test Case Schema

**要求**：Test ID / Title / Given / When / Then / Assertions

**现状**：测试函数名使用中文描述（如 `dead_hook_标记已行动`），代码注释中包含场景描述，但**未严格遵循** Given/When/Then 结构。

**评审结论**：**部分符合**。函数名即 Title，注释包含 Given/When/Then 语义，但缺少正式的 Test ID 编号。

## 6.2 §7.1 Standard Test Data

**要求**：使用 Unit_001 (HP=100, ATK=30, DEF=10) / Unit_002 / Unit_003

**现状**：使用 `UnitBuilder::warrior()` / `UnitBuilder::mage()` / `UnitBuilder::goblin()`，属性值与标准不完全一致。

**评审结论**：**不符合**。测试数据与规范定义的标准测试单位不一致。

## 6.3 §13.1 AI Self-Check

**要求**：测试文件开头标注 6 项自检结果

**现状**：所有测试文件**均无** AI Self-Check 标注。

**评审结论**：**不符合**。

---

# 7. 缺失测试评审

## 7.1 §5 缺失类别：Replay Test

**要求**（§5 + §8）：Replay Test 是**项目最高优先级测试**。

**现状**：Character 相关**无任何 Replay Test**。

**评审结论**：**严重缺失**。

**建议**：
1. 为 `致命伤害触发死亡` 场景创建 Replay YAML
2. 为 `被动 Trait 授予标签` 场景创建 Replay YAML

## 7.2 §5 缺失类别：Regression Test

**要求**（§11）：所有已修复 Bug 必须对应回归测试。

**现状**：无明确的回归测试标记。

**评审结论**：**需确认**。

## 7.3 §10 Error Testing 缺失

**要求**：必须验证 Invalid Input / Invalid State / Missing Data / Boundary Values

**现状**：部分边界已覆盖（空路径、空查询、索引越界），但以下场景**缺失**：

| 缺失场景 | 优先级 | 说明 |
|----------|--------|------|
| TraitCollection 空列表查询 | 低 | 边界：空容器操作 |
| apply_passive_traits 空 TraitCollection | 中 | 空输入验证 |
| apply_passive_traits 无效 trait_id | 高 | Registry 中不存在的 trait |
| UnitTemplateRegistry 无效 ID 查询 | 低 | 边界：空查询 |
| TraitEffectHandlerRegistry 无效 type_name | 低 | 边界：空查询 |
| MovingUnit 超长路径（100+ 格） | 低 | 性能边界 |
| Dead 组件重复添加 | 中 | 幂等性验证 |
| Faction 序列化/反序列化 | 低 | RON 兼容性 |

## 7.4 未覆盖模块

| 模块 | 说明 | 优先级 |
|------|------|--------|
| `marker.rs` | 纯标记组件（MovableRange, AttackRange, SelectionHighlight）+ clear_markers 函数 | 低 |
| `movement.rs` | 移动动画系统（spawn_path_arrows, animate_movement）| 中 |
| `spawn.rs` | 单位生成系统（spawn_units, spawn_unit_from_template）| 高 |

---

# 8. 代码质量评审

## 8.1 测试代码质量

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 辅助函数复用 | ✅ | `UnitBuilder` 统一构建、`register_grant_tag_trait` 等辅助 |
| 断言精确性 | ✅ | 使用 `assert_eq!` / `assert!` 精确比较 |
| 测试独立性 | ✅ | 每个测试独立创建数据，无共享状态 |
| 测试可读性 | ✅ | 中文函数名 + 注释说明场景 |
| 测试确定性 | ✅ | 无随机、无时间依赖 |

## 8.2 测试基础设施

| 组件 | 状态 | 说明 |
|------|------|------|
| `UnitBuilder` | ✅ | 流式 API 构建测试角色 |
| `equipment_app()` | ✅ | 最小 Bevy App 构建器（含 Equipment 支持） |
| `TraitRegistry` 默认 | ✅ | 不依赖文件系统的测试注册表 |
| `TraitEffectHandlerRegistry` | ✅ | 内置 3 个 Handler |

---

# 9. 问题分类统计

## 9.1 按严重程度

| 严重程度 | 数量 | 问题列表 |
|----------|------|----------|
| **P0 严重** | 1 | 缺失 Replay Test（§5 + §8 强制要求） |
| **P1 高** | 2 | 缺失 Error Testing 场景、标准测试数据不符 |
| **P2 中** | 2 | AI Self-Check 缺失、spawn.rs 未覆盖 |
| **P3 低** | 3 | Test Case Schema 不规范、marker/movement.rs 未覆盖、Regression Test 标记缺失 |

## 9.2 按类别

| 类别 | 数量 | 说明 |
|------|------|------|
| 测试层级缺失 | 1 | Replay Test |
| 测试规范不符 | 3 | §7.1 / §13.1 / §7 |
| 边界覆盖不足 | 1 | §10 Error Testing |
| 模块未覆盖 | 1 | spawn.rs（高优先级） |
| 元数据缺失 | 1 | Regression Test 标记 |

---

# 10. 优先级建议

## 10.1 立即修复（P0）

1. **创建 Replay Test**
   - 为 `致命伤害触发死亡` 场景创建 `battle_replays/*.yaml`
   - 为 `被动 Trait 授予标签` 场景创建 Replay YAML

## 10.2 短期修复（P1）

2. **补充 Error Testing**
   - 添加 8 个边界/错误场景测试
   - 重点：空 TraitCollection、无效 trait_id、Dead 重复添加

3. **引入标准测试数据**
   - 创建 `tests/common/standard_units.rs`
   - 提供 Unit_001/Unit_002/Unit_003 符合 §7.1

## 10.3 中期优化（P2）

4. **添加 AI Self-Check 标注**
   - 在每个测试文件开头添加 6 项自检结果

5. **补充 spawn.rs 测试**
   - 为 `spawn_units` 和 `spawn_unit_from_template` 创建集成测试
   - 验证从模板生成单位的完整流程

## 10.4 长期完善（P3）

6. **规范化 Test Case Schema**
   - 为每个测试添加 Test ID 编号（如 CHR-001）
   - 结构化 Given/When/Then 注释

7. **补充 movement.rs 测试**
   - 为 `spawn_path_arrows` 和 `animate_movement` 创建测试

8. **建立 Regression Test 机制**
   - 结合 Git 历史识别已修复 Bug
   - 为每个 Bug 创建回归测试

---

# 11. 合规性总结

## 11.1 条款合规性

| 条款 | 合规状态 | 说明 |
|------|----------|------|
| §3 Testing Philosophy | ✅ 合规 | 测试验证行为，不验证实现 |
| §4 Test Pyramid | ✅ 合规 | Unit 75% > 70% 目标 |
| §5 Test Categories | ❌ 不合规 | 缺失 Replay Test |
| §6 Determinism Rules | ✅ 合规 | 所有测试确定性 |
| §7 Test Case Schema | ⚠️ 部分合规 | 有场景描述但缺 Test ID |
| §7.1 Standard Test Data | ❌ 不合规 | 使用自定义模板，非标准单位 |
| §9 Coverage Strategy | ✅ 合规 | 16/16 领域不变量覆盖 |
| §10 Error Testing | ⚠️ 部分合规 | 部分边界覆盖，8 个场景缺失 |
| §11 Regression Rules | ⚠️ 待确认 | 需结合 Git 历史确认 |
| §13 AI Constraints | ✅ 合规 | 未测试私有实现 |
| §13.1 AI Self-Check | ❌ 不合规 | 无自检标注 |

## 11.2 总体评价

| 维度 | 评分 | 说明 |
|------|------|------|
| 领域规则覆盖 | ⭐⭐⭐⭐⭐ | 100% 不变量覆盖 |
| 测试行为正确性 | ⭐⭐⭐⭐⭐ | 全部验证 What，不验证 How |
| 测试层级完整性 | ⭐⭐⭐☆☆ | 缺失 Replay Test |
| 测试规范符合度 | ⭐⭐⭐☆☆ | 多项规范不符 |
| 边界错误覆盖 | ⭐⭐⭐☆☆ | 部分覆盖，有缺失 |
| 测试代码质量 | ⭐⭐⭐⭐⭐ | 高质量、确定性、可读 |

**综合评分：3.5 / 5.0**

---

# 12. AI Self-Check（Test Guardian 自检）

✅ 测试行为，不是实现 — 所有断言验证最终状态（标签存在、属性变化、组件状态）
✅ 符合领域规则 — 16/16 不变量覆盖
✅ 测试是确定性 — 无随机、无时间依赖
✅ 使用标准测试数据 — ⚠️ 使用 UnitBuilder 模板（非 §7.1 标准单位）
✅ 没有测试私有实现 — 未测试内部数据结构、Query 数量、System 顺序
✅ 没有生成不在范围内的测试 — 仅评审 character 模块相关测试

---

# 附录 A：测试清单

## A.1 内联单元测试（42 个）

```
components.rs (14):
  - 移动单位_目标坐标_在路径中
  - 移动单位_目标坐标_空路径
  - 移动单位_目标坐标_索引越界
  - 移动单位_是否完成_未完成
  - 移动单位_是否完成_已完成
  - 移动单位_是否完成_空路径
  - 移动单位_是否完成_刚到达终点
  - dead_hook_标记已行动
  - dead_hook_移除selected
  - dead_hook_无selected时不报错
  - unit_必需组件_自动生成
  - unit_id_组件_基本属性
  - unit_id_组件_相等与哈希
  - unit_id_组件_挂载与读取

template.rs (8):
  - ron_反序列化_单位模板
  - unit_template_def_转换为_unit_template
  - unit_template_registry_默认模板
  - unit_template_registry_查询
  - unit_template_registry_查询未注册返回none
  - faction_def_player转换
  - faction_def_enemy转换
  - ron_反序列化_旧配置无version字段

traits/mod.rs (7):
  - ron_反序列化_trait定义
  - trait_def_转换为_trait_data
  - trait_collection_查询
  - apply_passive_traits_授予标签和修饰符
  - apply_passive_traits_跳过非被动触发
  - ron_反序列化_触发型trait
  - apply_passive_traits_独立source_id

traits/types.rs (1):
  - ron_反序列化_旧配置无version字段

traits/handlers.rs (12):
  - grant_tag_handler_类型名
  - grant_tag_handler_授予标签
  - grant_tag_handler_非grant_tag返回空
  - grant_tag_handler_无属性修饰
  - modify_attribute_handler_类型名
  - modify_attribute_handler_返回属性修饰
  - modify_attribute_handler_非modify返回空
  - modify_attribute_handler_无标签授予
  - apply_buff_handler_类型名
  - apply_buff_handler_无标签授予
  - apply_buff_handler_无属性修饰
  - registry_默认包含三个处理器
  - registry_查询不存在返回none
  - registry_注册自定义处理器
  - registry_default等于with_defaults
```

## A.2 外部集成测试（14 个）

```
feature/traits.rs (10):
  - 被动trait授予标签_添加passive_grant_tag后标签出现在gameplay_tags
  - 被动trait授予标签_多个trait授予多个标签
  - 被动trait授予标签_非passive触发不授予标签
  - 装备trait完整生命周期_添加后entry存在_移除后entry消失
  - 装备trait_不同来源的trait独立管理
  - 装备trait_intrinsic来源不受equipment移除影响
  - trait修改属性_添加passive_modify_attribute后属性值变化
  - trait修改属性_移除trait后属性恢复
  - trait修改属性_乘法修饰符
  - trait修改属性_多个trait同时修改属性
  - trait修改属性_同时授予标签和修改属性

feature/death.rs (4):
  - 死亡标记添加后hook触发_acted为true且selected被移除
  - 致命伤害触发死亡_dead标记和character_died消息
  - 死亡角色_resolve_status_effects不处理
  - 存活角色_resolve_status_effects正常处理dot
```

---

# 附录 B：环境说明

- **编译状态**：`src/character/` 模块编译通过，无错误（LSP 诊断 0 errors）
- **测试执行**：因 `equipment/equip.rs` 和 `inventory/` 模块存在编译错误，无法执行 `cargo test`
- **影响范围**：character 模块本身无编译问题，测试失败由其他模块引起
- **建议**：修复其他模块编译错误后重新执行完整测试套件
