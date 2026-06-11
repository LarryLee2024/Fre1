# B0003: `SkillData::can_use` MP/HP 条件检查失效

## 问题描述

`SkillData::can_use` 方法（`src/skill/domain/types.rs:146`）在检查 `SkillCondition::MpCost` 和 `SkillCondition::HpBelow` 条件时，未正确返回错误。当 MP 不足或 HP 未低于阈值时，方法仍返回 `Ok(())`。

## 受影响测试

| 测试文件 | 测试用例 | 结果 |
|---|---|---|
| `tests/feature/skill.rs` | `技能使用条件检查_mp不足不可使用` | FAILED |
| `tests/legacy/skill_system.rs` | `战士_MP不足无法释放火球` | FAILED |
| `tests/legacy/skill_system.rs` | `法师_奥术冲击_MP不足时失败` | FAILED |
| `tests/legacy/skill_system.rs` | `狂暴_HP低于30百分比时可用` | FAILED |

## 问题分析

```rust
// tests/feature/skill.rs:38-53
fn expensive_skill() -> SkillData {
    SkillData {
        cost_mp: 10,
        conditions: vec![SkillCondition::MpCost(10)],  // ✅ 条件正确
        // ...
    }
}

// tests/feature/skill.rs:14-19
fn low_mp_warrior_attrs() -> Attributes {
    let mut attrs = UnitBuilder::warrior().attrs().clone();
    attrs.set_base(AttributeKind::Mp, 3.0);  // MP=3 < 10
    attrs
}

// 测试调用
let result = skill.can_use(&attrs, &tags, None, 0);
// 期望: Err(InsufficientMp { required: 10, current: 3 })
// 实际: Ok(())
```

**关键缺陷**：

`can_use` 方法在检查 `MpCost` 时：

```rust
SkillCondition::MpCost(cost) => {
    let mp = source_attrs.get(AttributeKind::Mp);
    if mp < *cost as f32 {
        return Err(SkillUseError::InsufficientMp { ... });
    }
}
```

可能原因：
1. `Attributes::get(Mp)` 返回值包含修饰符，实际 MP > 10
2. `set_base(Mp, 3.0)` 未正确更新基础值
3. 战士默认属性包含 MP 修饰符（来自装备/trait）

同样，`HpBelow` 检查也失效：

```rust
SkillCondition::HpBelow(pct) => {
    let hp = source_attrs.get(AttributeKind::Hp);
    let max_hp = source_attrs.get(AttributeKind::MaxHp);
    if max_hp > 0.0 && hp / max_hp >= *pct {
        return Err(SkillUseError::HpNotBelow { ... });
    }
}
```

## 建议修复

1. **调试 `Attributes::get()` 返回值**：添加日志确认 MP/HP 实际值
2. **检查 `UnitBuilder::warrior()` 默认属性**：确认是否有意外修饰符
3. **验证 `set_base()` 是否正确更新基础值**

## 验证方法

修复后运行：
```bash
cargo test --test feature -- skill::技能使用条件检查_mp不足不可使用
cargo test --test legacy_turn -- skill_system::战士_MP不足无法释放火球
cargo test --test legacy_turn -- skill_system::法师_奥术冲击_MP不足时失败
cargo test --test legacy_turn -- skill_system::狂暴_HP低于30百分比时可用
```

预期：4 个测试全部通过
