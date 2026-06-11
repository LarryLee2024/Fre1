# B0005: 消耗品 `use_item_system` 未处理效果

## 问题描述

`use_item_system`（`src/inventory/use_item.rs:43`）在处理消耗品使用时，未正确执行 `UseEffect::RestoreVital` 和 `UseEffect::GrantBuff` 效果。导致治疗药水不恢复 HP，力量药水不赋予 Buff。

## 受影响测试

| 测试文件 | 测试用例 | 结果 |
|---|---|---|
| `tests/feature/consumable.rs` | `治疗药水恢复hp_受伤角色使用后hp修饰符增加` | FAILED |
| `tests/feature/consumable.rs` | `药水赋予buff_使用力量药水后获得buff` | FAILED |

## 问题分析

### 缺陷 1：治疗药水不恢复 HP

```rust
// tests/feature/consumable.rs:107-172
fn 治疗药水恢复hp_受伤角色使用后hp修饰符增加() {
    let mut app = consumable_app();
    register_consumables(&mut app);
    let entity = UnitBuilder::warrior().spawn(&mut app);
    
    // 手动降低 HP
    attrs.set_base(AttributeKind::Hp, max_hp - 80.0);
    
    // 使用治疗药水
    app.world_mut().write_message(UseItem { ... });
    app.update();
    
    // 验证 HP 修饰符
    let hp_mods = attrs.modifiers.iter()
        .filter(|m| m.kind == AttributeKind::Hp)
        .count();
    // 期望: hp_mods == 1
    // 实际: hp_mods == 0
}
```

### 缺陷 2：力量药水不赋予 Buff

```rust
// tests/feature/consumable.rs:180-207
fn 药水赋予buff_使用力量药水后获得buff() {
    let mut app = consumable_app();
    register_consumables(&mut app);
    let entity = UnitBuilder::warrior().spawn(&mut app);
    
    // 使用力量药水
    app.world_mut().write_message(UseItem { ... });
    app.update();
    
    // 验证 Buff
    let buffs = app.world().get::<ActiveBuffs>(entity).unwrap();
    // 期望: buffs 包含 "strength_up"
    // 实际: buffs 为空
}
```

## 问题分析

`use_item_system` 可能的问题：

1. **未遍历 `use_effects`**：消耗品定义中的效果列表未被处理
2. **效果类型匹配不完整**：`UseEffect::RestoreVital` 和 `UseEffect::GrantBuff` 未在 match 中处理
3. **ECS 系统执行顺序**：`use_item_system` 可能在其他系统之后执行，导致状态不一致

## 建议修复

检查 `use_item_system` 实现：

```rust
pub fn use_item_system(
    mut messages: MessageReader<UseItem>,
    // ...其他参数...
) {
    for msg in messages.read() {
        // 1. 查找消耗品定义
        let item_def = registry.get(&msg.def_id);
        
        // 2. 遍历 use_effects
        for effect in &item_def.use_effects {
            match effect {
                UseEffect::RestoreVital { kind, amount } => {
                    // 添加 HP 修饰符
                    attrs.add_modifier(*kind, ModifierInstance {
                        op: ModifierOp::Add,
                        value: *amount,
                        source: ModifierSource::Item,
                    });
                }
                UseEffect::GrantBuff { buff_id, duration } => {
                    // 添加 Buff
                    buffs.add(BuffInstance {
                        buff_id: buff_id.clone(),
                        remaining_turns: *duration,
                        // ...
                    });
                }
                _ => {}
            }
        }
        
        // 3. 减少物品数量
        container.reduce_stack(msg.instance_id, 1);
    }
}
```

## 验证方法

修复后运行：
```bash
cargo test --test feature -- consumable::治疗药水恢复hp_受伤角色使用后hp修饰符增加
cargo test --test feature -- consumable::药水赋予buff_使用力量药水后获得buff
```

预期：2 个测试全部通过

## 附加验证

运行消耗品相关测试：
```bash
cargo test --test feature -- consumable
```

预期：所有消耗品测试通过（当前 3 passed, 2 failed）
