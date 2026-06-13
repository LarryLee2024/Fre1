# character_rules.md → character_rules_v1.md 更新差异

Version: 1.0 → 1.1

---

## 变更总览

| 变更类型 | 数量 |
|----------|------|
| 新增 | 8 |
| 修改 | 16 |
| 删除 | 0 |

---

## 逐项变更

### 1. 核心原则（修改）

**位置**：核心原则

**变更类型**：修改

**原内容**：
```
- Definition / Instance 分离
- 组合优于继承（Trait + Modifier 组合能力）
- Entity 只是 ID
```

**新内容**：
```
- 🟥 Definition / Instance 分离（宪法 1.1.2）
- 🟥 组合优于继承（宪法 1.1.6，Trait + Modifier 组合能力）
- 🟥 Entity 只是 ID（宪法 2.1.1）
- 🟥 Rule / Content 分离（宪法 1.1.3）
- 🟩 数据驱动（宪法 1.1.5）
```

**原因**：宪法要求所有规则标注强制等级；补充宪法条款引用；新增 Rule/Content 分离和数据驱动原则。

---

### 2. 术语定义 Dead 添加宪法引用（修改）

**位置**：术语定义 → Dead

**变更类型**：修改

**原内容**：
```
不是 is_dead: bool。Tag Component 优于 bool 字段。
```

**新内容**：
```
不是 is_dead: bool。Tag Component 优于 bool 字段（宪法 2.1.4）。
```

**原因**：补充宪法条款引用。

---

### 3. 不变量1-5 添加强制等级（修改）

**位置**：不变量1-5

**变更类型**：修改

**原内容**：无强制等级标记

**新内容**：
```
不变量1：Unit 完整性 🟥
不变量2：UnitTemplate 不可变 🟥
不变量3：Dead 标记一致性 🟥
不变量4：TraitCollection 来源一致 🟥
不变量5：GridPosition 与 OccupancyGrid 同步 🟥
```

**原因**：宪法要求所有规则标注强制等级。所有不变量均为🟥绝对禁止级别。

---

### 4. 不变量2 新增架构违规检测（修改）

**位置**：不变量2

**变更类型**：修改

**新增内容**：
```
架构违规检测：
发现运行时修改 UnitTemplate 时，必须停止。必须输出：
ARCHITECTURE VIOLATION: 运行时修改 UnitTemplate，违反 Definition/Instance 分离原则。
```

**原因**：Definition 不可变是核心架构约束，应有明确违规检测。

---

### 5. 不变量6：禁止用 bool 代替 Tag Component（新增）

**位置**：不变量

**变更类型**：新增

**新内容**：
```
## 不变量6：禁止用 bool 代替 Tag Component 🟥

宪法依据：2.1.4

任意时刻：
状态标记必须使用 Tag Component（如 Dead），禁止使用 bool 字段（如 is_dead: bool）。

违反表现：
使用 Added/Changed/Removed 无法检测状态变更。
```

**原因**：宪法 2.1.4 明确禁止用 bool 代替 Tag Component，这是 Character 领域的核心不变量（Dead Tag vs is_dead）。

---

### 6. 规则1-6 添加强制等级和宪法引用（修改）

**位置**：业务规则1-6

**变更类型**：修改

**原内容**：无强制等级标记，无宪法引用

**新内容**：
```
规则1：单位生成 🟥
宪法依据：1.1.2（Definition/Instance 分离）、1.1.5（数据驱动）

规则2：阵营归属 🟩
规则3：死亡处理 🟥
宪法依据：2.1.4（禁止 bool 代替 Tag）、5.0（通信三原则）

规则4：TraitCollection 管理 🟩
宪法依据：6.0.2（Trait 用于扩展点）

规则5：PersistentTags 分层 🟩
规则6：移动动画 🟩
```

**原因**：宪法要求所有规则标注强制等级和宪法条款引用。

---

### 7. 规则1 禁止事项添加强制等级（修改）

**位置**：规则1

**变更类型**：修改

**原内容**：
```
禁止：
- 跳过 Required Components
- 运行时创建新的 UnitTemplate
- 直接修改 UnitTemplate
```

**新内容**：
```
禁止：
- 🟥 跳过 Required Components
- 🟥 运行时创建新的 UnitTemplate
- 🟥 直接修改 UnitTemplate
```

**原因**：宪法要求所有禁止事项标注强制等级。

---

### 8. 规则3 新增禁止 is_dead: bool（修改）

**位置**：规则3

**变更类型**：修改

**新增内容**：
```
禁止：
- 🟥 用 is_dead: bool 代替 Dead Tag
```

**原因**：宪法 2.1.4 明确禁止用 bool 代替 Tag Component。

---

### 9. 规则4 禁止事项添加强制等级（修改）

**位置**：规则4

**变更类型**：修改

**原内容**：
```
禁止：
- 为每种能力来源写独立逻辑
- 硬编码 Trait 效果
- 装备穿脱时不更新 TraitCollection
```

**新内容**：
```
禁止：
- 🟥 为每种能力来源写独立逻辑
- 🟥 硬编码 Trait 效果
- 🟥 装备穿脱时不更新 TraitCollection
```

**原因**：宪法要求所有禁止事项标注强制等级。

---

### 10. 流程管线步骤添加强制等级（修改）

**位置**：流程管线 → Step1-7

**变更类型**：修改

**原内容**：无强制等级标记

**新内容**：
```
Step2: 🟥 禁止：直接设置 Vital Resources 基础值（宪法 2.2.1）
Step3: 🟥 禁止：跳过 Intrinsic 标记
Step4: 🟥 禁止：处理非 Passive 触发类型
Step6: 🟥 禁止：跳过此步骤
Step7: 🟥 禁止：遗漏任何 Required Component
```

**原因**：宪法要求所有禁止事项标注强制等级；Vital Resources 推导涉及宪法 2.2.1。

---

### 11. 数据结构添加强制等级（修改）

**位置**：数据结构 → Unit / UnitTemplate / TraitCollection / TraitSource / GridPosition / MovingUnit / PersistentTags

**变更类型**：修改

**新增内容**：
```
Unit: 🟥 生成时自动插入 9 个 Required Components
UnitTemplate: 🟥 不可变，加载后不可修改（宪法 1.1.2）、🟥 RON 配置路径（宪法 1.1.5）
TraitCollection: 🟥 remove_by_source 时精确清理
TraitSource: 🟥 装备穿脱时使用 Equipment 变体、🟥 内在 Trait 使用 Intrinsic 变体
GridPosition: 🟥 移动动画完成后更新
MovingUnit: 🟥 is_finished() 判断、🟥 动画完成后必须移除
PersistentTags: 🟥 最终 GameplayTags = from_traits | from_equipment、🟥 装备穿脱只修改 from_equipment
```

**原因**：宪法要求所有规则标注强制等级和宪法条款引用。

---

### 12. 禁止事项添加强制等级和新增2条（修改+新增）

**位置**：禁止事项

**变更类型**：修改 + 新增

**修改内容**：原有6条禁止事项全部添加🟥标记和宪法引用

**新增内容**：
```
🟥 禁止：用 is_dead: bool 代替 Dead Tag
原因：宪法 2.1.4 明确禁止用 bool 代替 Tag Component

🟥 禁止：硬编码单位逻辑
原因：Rule / Content 分离（宪法 1.1.3），新单位只修改 RON 配置
```

**原因**：宪法 2.1.4 和 1.1.3 明确要求。

---

### 13. AI 修改规则添加强制等级和测试要求（修改）

**位置**：AI 修改规则

**变更类型**：修改

**新增内容**：
```
禁止：
- 🟥 修改 Unit 组件结构
- 🟥 修改 spawn_unit_from_template 流程
- 🟥 硬编码新单位逻辑
- 🟥 修改 TraitData 的方法
- 🟥 修改 apply_passive_traits 流程
- 🟥 修改 Dead Hook 的固有行为
- 🟥 在 HP 变化时内联死亡逻辑

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证生成管线各步骤
- 🟩 集成测试：验证完整单位生命周期
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）
```

**原因**：宪法要求所有禁止事项标注强制等级；宪法 13.0.1-13.0.3 要求测试优先。

---

### 14. 宪法条款映射（新增）

**位置**：文档末尾

**变更类型**：新增

**新内容**：
```
| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.2 Definition/Instance 分离 | UnitTemplate(Definition) vs Unit(Instance) |
| 1.1.3 Rule/Content 分离 | 生成管线是规则，RON 配置是内容 |
| 1.1.5 数据驱动 | UnitTemplate 从 RON 加载 |
| 1.1.6 组合优于继承 | Trait + Modifier 组合能力 |
| 2.1.1 Entity 只是 ID | Entity 不承载行为 |
| 2.1.4 禁止 bool 代替 Tag | Dead Tag 代替 is_dead |
| 2.1.6 禁止手写状态标记 | 使用 Added/Changed/Removed 检测 |
| 2.2.1 禁止直接修改最终属性 | Vital Resources 由 fill_vital_resources() 推导 |
| 5.0 通信三原则 | Dead Hook(固有行为) + Observer(局部响应) + Message(广播) |
| 6.0.2 Trait 用于扩展点 | TraitEffectHandler 分发 |
```

**原因**：便于 AI 快速定位宪法条款与领域规则的对应关系。

---

### 15. 架构违规检测（新增）

**位置**：文档末尾

**变更类型**：新增

**新内容**：
```
| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 运行时修改 UnitTemplate | 代码审查 | ARCHITECTURE VIOLATION: 运行时修改 UnitTemplate... |
| 用 bool 代替 Tag Component | 代码审查 | ARCHITECTURE VIOLATION: 使用 is_dead: bool 代替 Dead Tag... |
| Entity 承载行为 | 代码审查 | ARCHITECTURE VIOLATION: Entity 承载行为... |
```

**原因**：架构文档定义了标准违规检测模式，领域文档应纳入。

---

## 变更统计

| 强制等级 | 原文档数量 | 新文档数量 |
|----------|-----------|-----------|
| 🟥 绝对禁止 | 0 | 18 |
| 🟩 必须遵守 | 0 | 8 |
| 宪法条款引用 | 0 | 10 |
| 架构违规检测 | 0 | 3 |
