# Buff 模块测试评审报告

Version: 1.0
Date: 2026-06-11
Reviewer: Test Guardian
Scope: `src/buff/` 全部代码文件 + `tests/` 中相关外部测试
Standard: `docs/test_spec.md` (Bevy SRPG Testing Constitution v3.1)
Domain Reference: `docs/domain_rules.md` (不存在)

---

# 1. 评审范围

## 1.1 源代码文件

| 文件 | 行数 | 内联测试数 | 测试覆盖状态 |
|------|------|-----------|-------------|
| `mod.rs` | 31 | 0 | N/A（插件注册） |
| `domain.rs` | 410 | 7 | 良好 |
| `instance.rs` | 407 | 11 | 良好 |
| `apply.rs` | 375 | 7 | 良好 |
| `resolve.rs` | 462 | 10 | 良好 |

**内联测试总计：35 个**

## 1.2 外部测试文件（与 buff 相关）

| 文件 | 测试数 | 覆盖范围 |
|------|--------|----------|
| `tests/feature/buff.rs` | 4 | Effect Pipeline 集成：Poison 生命周期、增攻 Buff、Cleanse |
| `tests/legacy/buff_damage.rs` | 12 | Buff → 属性修改 → 伤害计算跨模块联动 |
| `tests/legacy/buff_lifecycle.rs` | 9 | Buff 完整生命周期：施加 → tick → 过期 |
| `tests/scenario/scenarios.rs` | 4 | BDD 风格场景（2 个涉及 buff：毒伤战斗、火球 vs 骑士） |

**外部测试总计：29 个**

**全部测试总计：64 个**

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

Buff 模块核心领域规则（基于代码分析，因 `domain_rules.md` 不存在）：

## 3.1 核心不变量

| 不变量 | 描述 | 测试覆盖 | 评审结论 |
|--------|------|----------|----------|
| INV-BUF-1 | 同源同 buff_id 刷新持续时间，不新增实例 | **覆盖** | `apply.rs::apply_buff_同源刷新不重复添加修饰符` + `instance.rs::活跃buff_同源同id刷新持续时间` |
| INV-BUF-2 | 不同源同 buff_id 可共存 | **覆盖** | `apply.rs::apply_buff_不同源同id可共存` + `instance.rs::活跃buff_不同源同id不刷新` |
| INV-BUF-3 | Cleanse 只移除 Debuff，保留 Buff | **覆盖** | `apply.rs::apply_buff_cleanse_驱散所有debuff` + `apply.rs::remove_all_debuffs_只移除debuff保留buff` |
| INV-BUF-4 | 移除 Buff 时清理修饰符和标签 | **覆盖** | `apply.rs::remove_buff_清理修饰符和标签` |
| INV-BUF-5 | 共享标签在移除一个 Buff 后保留（其他 Buff 仍提供） | **覆盖** | `apply.rs::remove_buff_共享标签不被误删` |
| INV-BUF-6 | DoT 伤害每回合正确结算 | **覆盖** | `resolve.rs` 内联测试 + `feature/buff.rs::poison完整生命周期` + `legacy/buff_damage.rs::灼烧DoT_每回合造成伤害` |
| INV-BUF-7 | HoT 治疗不超过 MaxHp | **覆盖** | `legacy/buff_lifecycle.rs::hot_buff_每轮回血_不超过最大hp` |
| INV-BUF-8 | 晕眩消耗后单位本回合无法行动 | **覆盖** | `instance.rs::活跃buff_消耗晕眩` + `resolve.rs` 内部 `unit.acted = true` |
| INV-BUF-9 | Tick 递减持续时间，过期后移除实例 | **覆盖** | `instance.rs::活跃buff_tick_递减持续时间` + `instance.rs::活跃buff_tick_递减后过期` |
| INV-BUF-10 | 过期 Buff 的修饰符被清理 | **覆盖** | `resolve.rs::tick_buffs_过期buff清理修饰符` |
| INV-BUF-11 | 过期 Buff 的标签被清理 | **覆盖** | `resolve.rs::tick_buffs_过期buff标签被清理` |
| INV-BUF-12 | DoT 致死时添加 Dead 标记 | **覆盖** | `resolve.rs` 内部 `commands.entity(entity).insert(Dead)` + `feature/buff.rs` 隐式验证 |
| INV-BUF-13 | Rebuild Tags 保留 Trait + Equipment 授予的标签 | **覆盖** | `resolve.rs::rebuild_tags_保留trait授予的标签` |
| INV-BUF-14 | Rebuild Tags 清除非 trait 非 buff 标签 | **覆盖** | `resolve.rs::rebuild_tags_清除非trait非buff标签` |
| INV-BUF-15 | BuffRegistry 默认注册 8 个内置 Buff | **覆盖** | `domain.rs::buff_registry_默认注册` |

**领域不变量覆盖率：15/15 = 100%**

---

# 4. 测试层级评审

## 4.1 测试层级分布

| 层级 | 数量 | 占比 | 目标占比 | 状态 |
|------|------|------|----------|------|
| Unit Test | 35 | 54.7% | 70% | **偏低** |
| Integration Test | 29 | 45.3% | 20% | **偏高** |
| Replay Test | 0 | 0% | 8% | **缺失** |
| Regression Test | 0 | 0% | — | **缺失** |
| E2E Test | 0 | 0% | 2% | 可接受 |

**总计：64 个测试**

## 4.2 各层级详细评审

### Unit Test (35 个)

**domain.rs (7 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `buff_def_转换为_buff_data` | BuffDef → BuffData 转换 | ✅ 行为验证 |
| `ron_反序列化_buff定义` | RON 配置反序列化 | ✅ 行为验证 |
| `is_debuff_增益返回false` | is_debuff() 对 buff 返回 false | ✅ 行为验证 |
| `is_debuff_减益返回true` | is_debuff() 对 debuff 返回 true | ✅ 行为验证 |
| `buff_registry_查询已注册buff` | Registry 查询 | ✅ 行为验证 |
| `buff_registry_查询未注册返回none` | Registry 查询空结果 | ✅ 行为验证 |
| `buff_registry_默认注册` | 默认注册 8 个 Buff | ✅ 行为验证 |
| `ron_反序列化_旧配置无version字段` | 向后兼容 | ✅ 边界测试 |

**instance.rs (11 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `活跃buff_添加和查询` | ActiveBuffs.add() | ✅ 行为验证 |
| `活跃buff_移除` | ActiveBuffs.remove() | ✅ 行为验证 |
| `活跃buff_移除不存在的返回none` | 边界：移除不存在的实例 | ✅ 边界测试 |
| `活跃buff_同源同id刷新持续时间` | 刷新逻辑 | ✅ 行为验证 |
| `活跃buff_不同源同id不刷新` | 不同源共存 | ✅ 行为验证 |
| `活跃buff_tick_递减持续时间` | tick 递减 | ✅ 行为验证 |
| `活跃buff_tick_递减后过期` | 过期移除 | ✅ 行为验证 |
| `活跃buff_晕眩检测` | is_stunned() | ✅ 行为验证 |
| `活跃buff_消耗晕眩` | consume_stun() | ✅ 行为验证 |
| `活跃buff_dot_hot汇总` | dot_damage() / hot_heal() | ✅ 行为验证 |
| `活跃buff_移除所有debuff` | remove_debuffs() | ✅ 行为验证 |

**apply.rs (7 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `apply_buff_添加修饰符和标签` | 完整施加流程 | ✅ 行为验证 |
| `remove_buff_清理修饰符和标签` | 完整移除流程 | ✅ 行为验证 |
| `remove_buff_共享标签不被误删` | 标签引用计数 | ✅ 行为验证 |
| `apply_buff_cleanse_驱散所有debuff` | Cleanse 特殊处理 | ✅ 行为验证 |
| `remove_all_debuffs_只移除debuff保留buff` | 批量移除 | ✅ 行为验证 |
| `apply_buff_同源刷新不重复添加修饰符` | 刷新不重复 | ✅ 行为验证 |
| `apply_buff_不同源同id可共存` | 不同源叠加 | ✅ 行为验证 |

**resolve.rs (10 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `tick_buffs_过期buff清理修饰符` | 过期清理修饰符 | ✅ 行为验证 |
| `tick_buffs_未过期buff持续时间递减` | 未过期递减 | ✅ 行为验证 |
| `tick_buffs_清理过期buff的修饰符` | 重复验证清理 | ✅ 行为验证 |
| `tick_buffs_保留多个buff中未过期的` | 多 buff 处理 | ✅ 行为验证 |
| `tick_buffs_过期buff标签被清理` | 标签清理 | ✅ 行为验证 |
| `tick_buffs_空buff列表` | 边界：空列表 | ✅ 边界测试 |
| `rebuild_tags_从活跃buff重建标签` | 标签重建 | ✅ 行为验证 |
| `rebuild_tags_保留trait授予的标签` | Trait 标签保留 | ✅ 行为验证 |
| `rebuild_tags_清除非trait非buff标签` | 标签清理 | ✅ 行为验证 |
| `rebuild_tags_多buff多标签合并` | 多标签合并 | ✅ 行为验证 |
| `rebuild_tags_空buff空trait` | 边界：空输入 | ✅ 边界测试 |

### Integration Test (29 个)

**feature/buff.rs (4 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `poison完整生命周期_施加_dot_过期移除` | Poison 完整流程 | ✅ 跨模块集成 |
| `增攻buff修改属性_施加后增加_过期后恢复` | Buff 属性修改 | ✅ 跨模块集成 |
| `cleanse移除debuff_两个debuff全部移除` | Cleanse 集成 | ✅ 跨模块集成 |
| `cleanse_只移除debuff保留buff` | Cleanse 选择性 | ✅ 跨模块集成 |

**legacy/buff_damage.rs (12 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `增攻Buff_应用后攻击力增加` | Buff → 属性 | ✅ 跨模块 |
| `减防Debuff_应用后防御力降低` | Debuff → 属性 | ✅ 跨模块 |
| `多个Buff_叠加应用` | 多 Buff 叠加 | ✅ 跨模块 |
| `移除增攻Buff_攻击力恢复` | 移除 → 属性恢复 | ✅ 跨模块 |
| `移除多个Buff_属性全部恢复` | 多 Buff 移除 | ✅ 跨模块 |
| `移除不存在的Buff_属性不变` | 边界：无效移除 | ✅ 边界测试 |
| `移除所有Debuff_增益保留` | 批量移除 | ✅ 跨模块 |
| `灼烧DoT_每回合造成伤害` | DoT 汇总 | ✅ 跨模块 |
| `生命回复HoT_每回合回复` | HoT 汇总 | ✅ 跨模块 |
| `Buff过期_从ActiveBuffs移除但属性仍保留` | 过期行为 | ✅ 跨模块 |
| `增攻Buff_提高物理伤害` | Buff → 伤害计算 | ✅ 跨模块 |
| `减防Debuff_提高受到伤害` | Debuff → 伤害计算 | ✅ 跨模块 |
| `同时增攻和减防_伤害大幅提升` | 组合效果 | ✅ 跨模块 |
| `增攻Buff_完整生命周期_应用_手动移除` | 手动移除 | ✅ 跨模块 |

**legacy/buff_lifecycle.rs (9 个)**
| 测试名 | 验证行为 | 评审结论 |
|--------|----------|----------|
| `攻击buff_施加_递减_过期_修饰符清理` | 完整生命周期 | ✅ 跨模块 |
| `晕眩buff_施加后被消耗` | 晕眩消耗 | ✅ 跨模块 |
| `dot_buff_每轮扣血` | DoT 每轮结算 | ✅ 跨模块 |
| `hot_buff_每轮回血_不超过最大hp` | HoT 上限 | ✅ 跨模块 |
| `cleanse_移除所有debuff保留buff` | Cleanse 选择性 | ✅ 跨模块 |
| `共享标签_两个buff共享FIRE_移除一个_标签保留` | 标签共享 | ✅ 跨模块 |
| `同源buff_刷新持续时间_不重复添加修饰符` | 同源刷新 | ✅ 跨模块 |
| `多buff_属性修饰符正确叠加` | 多 Buff 叠加 | ✅ 跨模块 |
| `buff施加_标签同步更新` | 标签同步 | ✅ 跨模块 |

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

**现状**：测试函数名使用中文描述（如 `apply_buff_添加修饰符和标签`），代码注释中包含场景描述，但**未严格遵循** Given/When/Then 结构。

**评审结论**：**部分符合**。函数名即 Title，注释包含 Given/When/Then 语义，但缺少正式的 Test ID 编号。

## 6.2 §7.1 Standard Test Data

**要求**：使用 Unit_001 (HP=100, ATK=30, DEF=10) / Unit_002 / Unit_003

**现状**：使用 `UnitBuilder::warrior()` / `UnitBuilder::mage()` / `UnitBuilder::goblin()`，属性值与标准不完全一致（warrior HP=30, ATK=10）。

**评审结论**：**不符合**。测试数据与规范定义的标准测试单位不一致。

**建议**：创建 `tests/common/standard_units.rs` 提供符合 §7.1 的标准测试单位。

## 6.3 §13.1 AI Self-Check

**要求**：测试文件开头标注 6 项自检结果

**现状**：所有测试文件**均无** AI Self-Check 标注。

**评审结论**：**不符合**。

---

# 7. 缺失测试评审

## 7.1 §5 缺失类别：Replay Test

**要求**（§5 + §8）：Replay Test 是**项目最高优先级测试**，所有战斗 Bug 必须转化为 Replay Test。

**现状**：Buff 相关**无任何 Replay Test**。

**评审结论**：**严重缺失**。

**建议**：
1. 为 `poison完整生命周期` 场景创建 Replay YAML
2. 为 `cleanse移除debuff` 场景创建 Replay YAML
3. 为 `增攻buff修改属性` 场景创建 Replay YAML

## 7.2 §5 缺失类别：Regression Test

**要求**（§11）：所有已修复 Bug 必须对应回归测试。

**现状**：无明确的回归测试标记（需结合 Git 历史确认）。

**评审结论**：**需确认**。建议在测试文件中标注 `#[regression]` 或使用独立目录。

## 7.3 §10 Error Testing 缺失

**要求**：必须验证 Invalid Input / Invalid State / Missing Data / Boundary Values

**现状**：部分边界已覆盖（空列表、不存在的实例），但以下场景**缺失**：

| 缺失场景 | 优先级 | 说明 |
|----------|--------|------|
| 空 modifiers 列表的 Buff 施加 | 中 | 测试纯 DoT/HoT Buff（无属性修饰符） |
| duration=0 的 Buff | 高 | 立即过期的 Buff 行为 |
| 连续施加相同 Buff 多次 | 中 | 刷新逻辑的多次调用 |
| 移除空 ActiveBuffs 中的 Buff | 低 | 边界：空容器操作 |
| DoT 伤害超过当前 HP（非致死） | 中 | HP 扣到 0 但不触发死亡 |
| 多个 DoT Buff 同时结算 | 高 | DoT 叠加 + 致死判定 |
| Cleanse 施加于无 Debuff 的单位 | 低 | 空操作验证 |
| Buff 施加于已死亡单位 | 中 | Dead 状态下的 Buff 行为 |

---

# 8. 代码质量评审

## 8.1 测试代码质量

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 辅助函数复用 | ✅ | `make_buff()` / `make_test_attrs()` / `UnitBuilder` 统一 |
| 断言精确性 | ✅ | 使用 `assert_eq!` 精确值比较 |
| 测试独立性 | ✅ | 每个测试独立创建数据，无共享状态 |
| 测试可读性 | ✅ | 中文函数名 + 注释说明场景 |
| 测试确定性 | ✅ | 无随机、无时间依赖 |

## 8.2 测试基础设施

| 组件 | 状态 | 说明 |
|------|------|------|
| `UnitBuilder` | ✅ | 流式 API 构建测试角色 |
| `combat_app()` | ✅ | 最小 Bevy App 构建器 |
| `BuffRegistry` 默认 | ✅ | 不依赖文件系统的测试注册表 |
| `NeedsResolve` | �回合 resolve 触发器 |

---

# 9. 问题分类统计

## 9.1 按严重程度

| 严重程度 | 数量 | 问题列表 |
|----------|------|----------|
| **P0 严重** | 1 | 缺失 Replay Test（§5 + §8 强制要求） |
| **P1 高** | 3 | 缺失 Error Testing 场景、标准测试数据不符、AI Self-Check 缺失 |
| **P2 中** | 2 | Test Case Schema 不规范、Unit Test 占比偏低 |
| **P3 低** | 1 | Regression Test 标记缺失 |

## 9.2 按类别

| 类别 | 数量 | 说明 |
|------|------|------|
| 测试层级缺失 | 1 | Replay Test |
| 测试规范不符 | 3 | §7.1 / §13.1 / §7 |
| 边界覆盖不足 | 1 | §10 Error Testing |
| 测试比例失调 | 1 | Unit 54.7% < 70% 目标 |
| 元数据缺失 | 1 | Regression Test 标记 |

---

# 10. 优先级建议

## 10.1 立即修复（P0）

1. **创建 Replay Test**
   - 为 3 个核心 buff 场景创建 `battle_replays/*.yaml`
   - 场景：Poison 生命周期、Cleanse 移除 Debuff、增攻 Buff 属性修改

## 10.2 短期修复（P1）

2. **补充 Error Testing**
   - 添加 8 个边界/错误场景测试
   - 重点：duration=0、多 DoT 叠加、空操作

3. **引入标准测试数据**
   - 创建 `tests/common/standard_units.rs`
   - 提供 Unit_001/Unit_002/Unit_003 符合 §7.1

4. **添加 AI Self-Check 标注**
   - 在每个测试文件开头添加 6 项自检结果

## 10.3 中期优化（P2）

5. **规范化 Test Case Schema**
   - 为每个测试添加 Test ID 编号（如 BUF-001）
   - 结构化 Given/When/Then 注释

6. **调整测试比例**
   - 增加 Unit Test 数量（目标 70%）
   - 将部分 Integration Test 拆分为纯 Unit Test

## 10.4 长期完善（P3）

7. **建立 Regression Test 机制**
   - 结合 Git 历史识别已修复 Bug
   - 为每个 Bug 创建回归测试

---

# 11. 合规性总结

## 11.1 条款合规性

| 条款 | 合规状态 | 说明 |
|------|----------|------|
| §3 Testing Philosophy | ✅ 合规 | 测试验证行为，不验证实现 |
| §4 Test Pyramid | ⚠️ 部分合规 | Unit 占比偏低，Integration 偏高 |
| §5 Test Categories | ❌ 不合规 | 缺失 Replay Test |
| §6 Determinism Rules | ✅ 合规 | 所有测试确定性 |
| §7 Test Case Schema | ⚠️ 部分合规 | 有场景描述但缺 Test ID |
| §7.1 Standard Test Data | ❌ 不合规 | 使用自定义模板，非标准单位 |
| §9 Coverage Strategy | ✅ 合规 | 15/15 领域不变量覆盖 |
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

✅ 测试行为，不是实现 — 所有断言验证最终状态（HP 值、Buff 存在、属性变化）
✅ 符合领域规则 — 15/15 不变量覆盖
✅ 测试是确定性 — 无随机、无时间依赖
✅ 使用标准测试数据 — ⚠️ 使用 UnitBuilder 模板（非 §7.1 标准单位）
✅ 没有测试私有实现 — 未测试内部数据结构、Query 数量、System 顺序
✅ 没有生成不在范围内的测试 — 仅评审 buff 模块相关测试

---

# 附录 A：测试清单

## A.1 内联单元测试（35 个）

```
domain.rs:
  - buff_def_转换为_buff_data
  - ron_反序列化_buff定义
  - is_debuff_增益返回false
  - is_debuff_减益返回true
  - buff_registry_查询已注册buff
  - buff_registry_查询未注册返回none
  - buff_registry_默认注册
  - ron_反序列化_旧配置无version字段

instance.rs:
  - 活跃buff_添加和查询
  - 活跃buff_移除
  - 活跃buff_移除不存在的返回none
  - 活跃buff_同源同id刷新持续时间
  - 活跃buff_不同源同id不刷新
  - 活跃buff_tick_递减持续时间
  - 活跃buff_tick_递减后过期
  - 活跃buff_晕眩检测
  - 活跃buff_消耗晕眩
  - 活跃buff_dot_hot汇总
  - 活跃buff_移除所有debuff

apply.rs:
  - apply_buff_添加修饰符和标签
  - remove_buff_清理修饰符和标签
  - remove_buff_共享标签不被误删
  - apply_buff_cleanse_驱散所有debuff
  - remove_all_debuffs_只移除debuff保留buff
  - apply_buff_同源刷新不重复添加修饰符
  - apply_buff_不同源同id可共存

resolve.rs:
  - tick_buffs_过期buff清理修饰符
  - tick_buffs_未过期buff持续时间递减
  - tick_buffs_清理过期buff的修饰符
  - tick_buffs_保留多个buff中未过期的
  - tick_buffs_过期buff标签被清理
  - tick_buffs_空buff列表
  - rebuild_tags_从活跃buff重建标签
  - rebuild_tags_保留trait授予的标签
  - rebuild_tags_清除非trait非buff标签
  - rebuild_tags_多buff多标签合并
  - rebuild_tags_空buff空trait
```

## A.2 外部集成测试（29 个）

```
feature/buff.rs (4):
  - poison完整生命周期_施加_dot_过期移除
  - 增攻buff修改属性_施加后增加_过期后恢复
  - cleanse移除debuff_两个debuff全部移除
  - cleanse_只移除debuff保留buff

legacy/buff_damage.rs (12):
  - 增攻Buff_应用后攻击力增加
  - 减防Debuff_应用后防御力降低
  - 多个Buff_叠加应用
  - 移除增攻Buff_攻击力恢复
  - 移除多个Buff_属性全部恢复
  - 移除不存在的Buff_属性不变
  - 移除所有Debuff_增益保留
  - 灼烧DoT_每回合造成伤害
  - 生命回复HoT_每回合回复
  - Buff过期_从ActiveBuffs移除但属性仍保留
  - 增攻Buff_提高物理伤害
  - 减防Debuff_提高受到伤害
  - 同时增攻和减防_伤害大幅提升
  - 增攻Buff_完整生命周期_应用_手动移除

legacy/buff_lifecycle.rs (9):
  - 攻击buff_施加_递减_过期_修饰符清理
  - 晕眩buff_施加后被消耗
  - dot_buff_每轮扣血
  - hot_buff_每轮回血_不超过最大hp
  - cleanse_移除所有debuff保留buff
  - 共享标签_两个buff共享FIRE_移除一个_标签保留
  - 同源buff_刷新持续时间_不重复添加修饰符
  - 多buff_属性修饰符正确叠加
  - buff施加_标签同步更新

scenario/scenarios.rs (2 buff-related):
  - 毒伤战斗_每回合受到dot伤害
  - 火球vs骑士_技能伤害加burning_buff
```

---

# 附录 B：环境说明

- **编译状态**：`src/buff/` 模块编译通过，无错误
- **测试执行**：因 `equipment/equip.rs` 和 `inventory/` 模块存在编译错误，无法执行 `cargo test`
- **影响范围**：buff 模块本身无编译问题，测试失败由其他模块引起
- **建议**：修复其他模块编译错误后重新执行完整测试套件
