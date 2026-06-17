# B0004: 伤害/治疗流水线 HP 边界处理缺陷

## 问题描述

伤害和治疗流水线未正确处理 HP 边界：
1. **伤害未将 HP 钳制到 0**：致命伤害后 HP 仍为正数
2. **治疗未将 HP 钳制到 MaxHp**：治疗预览/实际治疗可超过最大 HP
3. **DOT/HOT 未尊重 HP 边界**：每回合伤害/治疗可导致 HP 越界

## 受影响测试

| 测试文件 | 测试用例 | 结果 |
|---|---|---|
| `tests/feature/death.rs` | `致命伤害触发死亡_dead标记和character_died消息` | FAILED |
| `tests/legacy/combat_pipeline.rs` | `治疗预览不超过最大hp` | FAILED |
| `tests/legacy/buff_lifecycle.rs` | `dot_buff_每轮扣血` | FAILED |
| `tests/legacy/buff_lifecycle.rs` | `hot_buff_每轮回血_不超过最大hp` | FAILED |
| `tests/legacy/edge_cases.rs` | `伤害_精确致死` | FAILED |
| `tests/legacy/edge_cases.rs` | `伤害_超过hp` | FAILED |
| `tests/golden/golden_battle.rs` | `致命伤害_角色死亡` | FAILED |

## 问题分析

### 缺陷 1：致命伤害后 HP 未归零

```rust
// tests/feature/death.rs:188-220
let goblin = spawn_unit(&mut app, UnitBuilder::goblin().with_hp(5.0), "哥布林");
enqueue_damage(&mut app, warrior, goblin, 10);  // 造成 10 伤害

let hp = app.world().get::<Attributes>(goblin).unwrap().get(AttributeKind::Hp);
// 期望: hp == 0.0
// 实际: hp == 20.0  (初始 HP 未正确设置)
```

**关键问题**：`UnitBuilder::goblin().with_hp(5.0)` 未正确设置 HP 到 5.0

### 缺陷 2：治疗预览超过 MaxHp

```rust
// tests/legacy/combat_pipeline.rs:314
// 期望: heal_amount 不超过 max_hp
// 实际: heal_amount 超过 max_hp
```

### 缺陷 3：DOT/HOT 未钳制 HP

```rust
// tests/legacy/buff_lifecycle.rs:182
// DOT: 期望 HP 减少但不低于 0
// 实际: HP 可能为负数

// tests/legacy/buff_lifecycle.rs:232
// HOT: 期望 HP 增加但不超过 max_hp
// 实际: HP 可能超过 max_hp
```

## 建议修复

### 1. 修复 `Attributes::set_base()` 或 `UnitBuilder::with_hp()`

确认 HP 基础值正确设置：

```rust
// 在 damage handler 中添加 HP 钳制
fn apply_damage(attrs: &mut Attributes, damage: f32) {
    let hp = attrs.get(AttributeKind::Hp);
    let new_hp = (hp - damage).max(0.0);
    attrs.set_base(AttributeKind::Hp, new_hp);
}
```

### 2. 修复治疗预览钳制

```rust
fn preview_heal(attrs: &Attributes, heal: f32) -> f32 {
    let hp = attrs.get(AttributeKind::Hp);
    let max_hp = attrs.get(AttributeKind::MaxHp);
    (hp + heal).min(max_hp) - hp
}
```

### 3. 修复 DOT/HOT 钳制

在 `resolve_status_effects` 系统中添加边界检查：

```rust
// DOT
let new_hp = (hp - dot_damage).max(0.0);

// HOT
let new_hp = (hp + hot_heal).min(max_hp);
```

## 验证方法

修复后运行：
```bash
cargo test --test feature -- death::致命伤害触发死亡
cargo test --test legacy_combat -- combat_pipeline::治疗预览不超过最大hp
cargo test --test legacy_buff -- buff_lifecycle::dot_buff_每轮扣血
cargo test --test legacy_buff -- buff_lifecycle::hot_buff_每轮回血_不超过最大hp
cargo test --test legacy_edge -- edge_cases::伤害_精确致死
cargo test --test legacy_edge -- edge_cases::伤害_超过hp
cargo test --test golden -- golden_battle::致命伤害_角色死亡
```

预期：7 个测试全部通过
