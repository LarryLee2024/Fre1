# Buff 领域

Version: 2.0

## Purpose

Buff 领域管理所有临时状态效果（增益、减益、DoT、HoT、晕眩），遵循 Definition / Instance 分离，通过统一修饰符管线影响属性。Buff = 临时 Trait，修饰符通过 ModifierSource 追踪归属。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| BuffData | Buff 的静态定义，描述 Buff"是什么" | ≠ BuffInstance：Data 不可变，Instance 是运行时实例 |
| BuffInstance | Buff 的运行时实例，挂载在单位上 | ≠ BuffData：Instance 有剩余回合和来源 |
| ActiveBuffs | 单位上所有活跃 Buff 实例的容器 | ≠ BuffRegistry：ActiveBuffs 是实例集合，Registry 是定义注册表 |
| DoT | 持续伤害效果，每回合结算 | ≠ 即时伤害：DoT 在回合开始结算，即时伤害在管线中结算 |
| HoT | 持续治疗效果，每回合结算 | ≠ 即时治疗：HoT 在回合开始结算 |
| Cleanse | 净化效果，立即驱散所有 Debuff | ≠ Buff 实例：Cleanse 不创建实例，立即执行驱散 |
| ModifierSource | 修饰符来源标识，用于追踪修饰符归属 | ≠ 修饰符本身：Source 是来源标识 |

---

## Responsibilities

### Owns

- BuffData 定义和注册表
- BuffInstance 的施加、移除、同源刷新
- ActiveBuffs 容器管理
- DoT / HoT / 晕眩的回合结算
- Buff 修饰符的添加和清理
- 标签重建

### Does Not Own

- 属性计算和修饰符管线 → stat_system
- 效果管线中 ApplyBuff 的生成 → battle_rules
- 死亡判定 → battle_rules（DoT 致死时调用 Dead Tag）
- UI 展示 → ui_rules
- Trait 效果分发 → trait_rules

---

## Invariants

### INV-BUF-01：Buff 必须有来源 🟥

每个 BuffInstance 的来源实体必须有值（或明确为 None 表示系统施加）。

违反：无法追踪 Buff 来源，同源刷新逻辑失效。

### INV-BUF-02：Buff 必须有过期条件 🟥

每个活跃 BuffInstance 的剩余回合必须 > 0。

违反：永不过期的 Buff 累积，属性无限叠加。

### INV-BUF-03：过期 Buff 修饰符必须清理 🟥

宪法：2.2.1

过期 Buff 的修饰符必须从 Attributes 中移除。

违反：过期 Buff 的属性修饰仍然生效。

### INV-BUF-04：标签安全移除 🟥

移除 Buff 时，只有没有其他活跃 Buff 提供的标签才从 GameplayTags 中移除。

违反：移除一个 Buff 的标签时误删其他 Buff 提供的相同标签。

### INV-BUF-05：Cleanse 不创建实例 🟩

Cleanse 立即驱散所有 Debuff，不创建 BuffInstance。

违反：Cleanse 作为 Buff 实例存在，占用槽位且无法正常过期。

### INV-BUF-06：修饰符必须通过 ModifierSource 添加 🟥

宪法：2.2.1

Buff 的属性修饰必须通过 ModifierSource 添加到 Attributes，禁止直接修改属性值。

违反：Buff 直接修改 HP、ATK 等最终属性值。

### INV-BUF-07：标签必须完全重建 🟥

每次 tick 后必须完全重建 GameplayTags，禁止增量更新。

违反：过期 Buff 的标签残留。

### INV-BUF-08：tick 先移除再递减 🟥

tick 时先移除剩余回合为 0 的实例，再递减所有剩余实例。

违反：剩余回合为 0 的 Buff 被递减为负数。

---

## State Machine

### Buff 实例状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Defined | BuffData 在注册表中 | — |
| Active | BuffInstance 在 ActiveBuffs 中 | Ticking, Expired |
| Ticking | 回合结算中 | Active, Expired |
| Expired | 剩余回合 = 0 | Removed |
| Removed | 实例和修饰符已清理 | — |

```
Defined → Active → Ticking → Active（递减后）
                  ↘ Expired → Removed
```

---

## Business Rules

### BR-BUF-01：施加 Buff

- 同源同 buff_id 刷新持续时间（不新增实例）
- 不同来源的同 ID Buff 可共存
- 修饰符通过 ModifierSource 添加到 Attributes
- 标签添加到 GameplayTags
- 净化立即驱散所有 Debuff

### BR-BUF-02：移除 Buff

- 从 ActiveBuffs 移除实例
- 从 Attributes 移除修饰符
- 检查共享标签后再从 GameplayTags 移除

### BR-BUF-03：回合结算

- 通过标记保证每回合一次
- 结算顺序：晕眩 → DoT → HoT → tick
- DoT 致死时插入 Dead Tag + 发送 CharacterDied
- HoT 回血上限为 MaxHp
- 晕眩消耗后移除

### BR-BUF-04：标签重建

- 每次 tick 后完全重建 GameplayTags
- 三层叠加：Trait 授予 → 装备授予 → Buff 授予
- 过期 Buff 的标签不参与重建

---

## Pipelines

### 施加管线

查找定义 → Cleanse 检查 → 生成实例 → 添加修饰符 → 添加标签

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 查找定义 | buff_id + 注册表 | BuffData | buff_id 不存在时静默跳过 |
| Cleanse 检查 | BuffData 类型 | 驱散结果 | Cleanse 不创建实例（INV-BUF-05） |
| 生成实例 | BuffData + 来源 + 持续时间 | BuffInstance | 同源同 ID 刷新不新增（BR-BUF-01） |
| 添加修饰符 | 修饰符列表 + ModifierSource | 属性变化 | 禁止绕过 Modifier 管线（INV-BUF-06） |
| 添加标签 | 标签列表 | 标签变化 | 禁止跳过标签添加 |

### 回合结算管线

晕眩结算 → DoT 结算 → HoT 结算 → tick 递减 → 标签重建

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 晕眩结算 | 晕眩 Buff | 标记已行动 + 移除晕眩 | 禁止跳过晕眩检查 |
| DoT 结算 | DoT Buff | HP 变化 + 可能死亡 | 禁止跳过致死判定 |
| HoT 结算 | HoT Buff | HP 变化 | 回血不超过 MaxHp |
| tick 递减 | ActiveBuffs | 过期 Buff 列表 | 先移除再递减（INV-BUF-08） |
| 标签重建 | 标签 + ActiveBuffs | 更新后的标签 | 完全重建（INV-BUF-07） |

---

## Data Model

### BuffData（Definition）

Buff 的静态定义，不可变。

- 标识：id / name
- 默认持续回合
- 属性修饰符列表
- 标签列表
- 持续效果数值（DoT 伤害 / HoT 治疗）
- 类型标记（晕眩 / 净化 / 增益）
- 配置来源：RON（assets/buffs/）

### BuffInstance（Instance）

Buff 的运行时实例。

- 唯一实例 ID
- 对应 BuffData ID
- 显示名称
- 剩余回合（必须 > 0）
- 来源实体
- 标签副本
- 增益/减益标记
- 持续效果数值

### ActiveBuffs（Instance Component）

单位上所有活跃 Buff 容器。

- BuffInstance 列表
- 实例 ID 生成器
- 施加时同源刷新
- tick 时先移除再递减

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 晕眩结算 | StunApplied Message | ui / battle_record |
| DoT 伤害 | DotApplied Message | ui / battle_record |
| HoT 治疗 | HotApplied Message | ui / battle_record |
| DoT 致死 | Dead Tag + CharacterDied Message | battle / ui |
| 修饰符变化 | 通过 ModifierSource 添加到 Attributes | stat_system |

---

## Change Rules

### 新增 Buff

- 允许：新增 RON 配置 + 新增 BuffData 字段（需配套反序列化）
- 禁止：修改施加/移除/tick 流程
- 检查：注册表注册、修饰符 Source 区间是否冲突、标签是否在枚举中

### 新增持续效果类型

- 允许：在 BuffData 中新增字段 + 在结算中新增步骤
- 禁止：修改结算顺序（晕眩 → DoT → HoT → tick）、修改 tick 递减逻辑
- 检查：结算顺序是否影响其他效果、Message 是否需要新增

### 新增 Buff 交互规则

- 允许：新增同源刷新变体
- 禁止：修改"不同源可共存"规则、修改标签安全移除逻辑
- 检查：同源刷新条件、标签共享检查、修饰符清理

---

## Architecture Violations

发现架构违规时统一输出：

```
ARCHITECTURE VIOLATION:
Rule: <RuleID>
Reason: <Why>
Fix: <How>
```

| RuleID | 违规行为 | Reason | Fix |
|--------|----------|--------|-----|
| INV-BUF-03 | 过期 Buff 修饰符未清理 | 修饰符必须随 Buff 过期移除 | 在 tick 中收集过期修饰符并清理 |
| INV-BUF-06 | Buff 直接修改属性值 | 属性修改必须通过 Modifier 管线 | 通过 ModifierSource 添加修饰符 |
| INV-BUF-07 | 增量更新标签 | 标签必须完全重建 | 改为三层叠加重建 |
| INV-BUF-08 | tick 先递减再移除 | 剩余为 0 的 Buff 不应被递减 | 调整为先移除再递减 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证施加/移除/tick 逻辑
- 集成测试：验证完整 Buff 生命周期
- Bug 修复必须先编写重现测试

排查顺序：
1. Buff 是否正确施加（修饰符 + 标签）
2. tick 递减顺序（先移除再递减）
3. 过期 Buff 修饰符是否清理
4. 标签重建是否正确（三层叠加）
5. 同源刷新是否生效
