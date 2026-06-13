# AGENTS.md — Bevy SRPG Project

## Project Overview
Bevy 0.18.1 tactical RPG (回合制战棋) with strict ECS architecture. All game logic follows ECS patterns with enforced separation of concerns.

## Key Commands
```bash
cargo build                    # Build project
cargo test                     # Run all tests
cargo test rule                # Run rule tests only (proptest)
cargo test feature             # Run feature tests only
cargo test scenario            # Run scenario tests only
cargo test golden              # Run golden/snapshot tests (insta)
cargo test system              # Run ECS system integration tests
cargo test -- --test-threads=1 # Sequential tests (if needed)
cargo run --features dev       # Run with dev tools (file_watcher, debug_stepping)
```

## Architecture Rules (Non-Negotiable)

### Plugin Registration Order
Must follow in `main.rs`:
1. Core layer: `EffectPlugin`, `ModifierRulePlugin`, `AttributeDefPlugin`, `TagDefPlugin`
2. Data layer: `SkillPlugin`, `BuffPlugin`, `AiBehaviorPlugin`, `EquipmentPlugin`, `InventoryPlugin`
3. Logic layer: `AssetsPlugin`, `TurnPlugin`, `MapPlugin`, `CharacterPlugin`, `BattlePlugin`, `AiPlugin`
4. Presentation layer: `UiPlugin`, `InputPlugin`, `DebugPlugin`

### Module Organization
- **Feature First**: Code organized by business domain (`battle/`, `character/`, `buff/`, etc.)
- **Forbidden**: `components/`, `systems/`, `events/`, `utils/` as top-level modules
- **core/**: Must not depend on any business module (attributes, tags, effects, modifiers)
- **Module Header Comment**: Every `mod.rs` must start with a comment block describing module purpose, followed by inline comments on each `mod` declaration:
  ```rust
  // 模块名称：一句话说明模块职责
  // 补充说明（可选）

  mod sub_a; // 子模块 A 的职责
  mod sub_b; // 子模块 B 的职责
  ```
- **Mod Sync Rule**: When adding/removing/renaming files in a module directory, the `mod.rs` must be updated to match. Missing or stale `mod` declarations cause compilation errors.

### ECS Constraints
- **Entity = ID only**: No methods, no `EntityManager`, no OOP patterns
- **Component = data only**: No logic in components
- **System = behavior only**: No state storage in systems
- **Tag Components > bool**: Use `Dead`, `Stunned` instead of `is_dead: bool`

### Communication
- **Hook**: Component add/remove side effects (`#[component(on_add=...)]`)
- **Observer**: Same-feature state changes
- **Message**: Cross-feature broadcast
- **Forbidden**: Events for intra-module calls, Observer for high-frequency logic

### Data Flow
- **Definition (immutable)**: Config loaded from RON files in `assets/`
- **Instance (mutable)**: Runtime state per entity
- **Rule/Content separation**: New content = new RON files, never modify logic code

### Effect Pipeline (Critical)
All combat effects must follow:
```
CombatIntent → Generate → Modify → Execute
```
**Forbidden**: Direct HP modification, direct buff application, skipping pipeline

### Modifier Pipeline (Critical)
All attribute modifications must follow:
```
Modifier → Attribute Resolver → Final Stat
```
**Forbidden**: Direct stat modification, bypassing resolver

## Testing Conventions

### Test Structure
```
tests/
├── common/          # Shared test utilities
│   ├── fixtures.rs  # UnitBuilder with standard units
│   ├── app_builder.rs
│   ├── assertions.rs
│   └── combat_helpers.rs
├── rule/            # Formula/logic tests (proptest)
├── feature/         # Complete feature tests
├── scenario/        # Player flow tests (BDD style)
├── golden/          # Snapshot tests (insta)
└── system/          # ECS system integration tests
```

### Standard Test Units
Use `UnitBuilder` from `tests/common/fixtures.rs`:
- `UnitBuilder::warrior()` —战士 (Might=5, Vitality=5, Agility=6)
- `UnitBuilder::mage()` — 法师 (Intelligence=8, Willpower=6)
- `UnitBuilder::goblin()` — 哥布林 (low stats, Enemy faction)
- `UnitBuilder::unit_001()` — 标准战士 (HP=100, ATK=30, DEF=10, SPD=10)
- `UnitBuilder::unit_002()` — 标准法师 (HP=80, ATK=40, DEF=5, SPD=12)
- `UnitBuilder::unit_003()` — 标准坦克 (HP=150, ATK=20, DEF=20, SPD=5)

### Test Rules
- All tests must be deterministic (Seed=42 if random)
- Test behavior, not implementation
- Bug fix: write failing test first, then fix code
- Use standard test units, not ad-hoc values

## Code Style
- **Naming**: Types=PascalCase, Functions=snake_case, Constants=SCREAMING_SNAKE_CASE
- **Files**: One topic per file, 300-500 lines ideal, >1000 lines must split
- **Functions**: 20-50 lines ideal, >100 lines must refactor, max 3 nesting levels
- **Early Return**: Prefer over deep nesting
- **No unwrap/expect** in business code (Result only)
- **tracing** for logging (no println!/dbg!)

## Critical Warnings
1. **Never modify Definition objects** at runtime
2. **Never bypass Effect/Modifier Pipeline**
3. **Never create `utils.rs` or `components.rs`** as top-level modules
4. **Never use Entity as OOP object** (no `entity.attack()`)
5. **Never store state in systems**
6. **Never use events for intra-module calls**

## Reference Docs
- `docs/architecture.md` — Full architecture spec (highest priority)
- `docs/AI开发宪法.md` — AI development constitution
- `docs/coding_rules.md` — Coding rules
- `.trae/rules/` — Additional rules and conventions
