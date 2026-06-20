# Core Layer Entity Factory Standardization

> **Status**: Draft
> **Owner**: architect
> **Created**: 2026-06-21
> **Updated**: 2026-06-21
> **Supersedes**: none

## 1. Motivation

Core layer domains frequently need to spawn entities with consistent component bundles — combat participants, inventory items, spell instances, summons, etc. Currently, entity creation logic is scattered across systems, observers, and plugin build methods, leading to:

- Duplicated component insertion patterns
- Missing required components on newly spawned entities
- Inconsistent parameter ordering across similar spawn sites
- Difficulty tracking what components an entity type requires

This document standardizes entity factory conventions for the Core layer, borrowing from the established UI factory pattern (`src/ui/primitives/*/factory.rs`) while respecting the architectural differences of the Core layer.

### 1.1 UI Factory Pattern (Reference)

The UI layer already has a mature factory pattern:

```rust
// src/ui/primitives/button/factory.rs
pub fn spawn_button(
    commands: &mut Commands,
    theme: &Theme,
    label: impl Into<String>,
    variant: ButtonVariant,
) -> Entity {
    // ... bundle all required components, spawn children, return Entity
}
```

Key traits of the UI pattern:
- Free function (not a method on a builder or struct)
- `spawn_` prefix with noun describing the entity
- `commands` as the first parameter
- `Entity` as the return type
- All component decisions internal to the factory

## 2. Factory Naming Convention

### 2.1 Function Naming

```
spawn_{noun}() -> Entity
```

| Entity Type | Factory Function | Domain |
|-------------|-----------------|--------|
| Combat participant | `spawn_combat_participant()` | combat |
| Inventory item | `spawn_item_instance()` | inventory |
| Spell instance | `spawn_active_spell()` | spell |
| Summoned unit | `spawn_summoned_unit()` | summon |
| Quest objective marker | `spawn_quest_objective()` | quest |
| Reaction handler | `spawn_reaction_handler()` | reaction |
| Faction bond record | `spawn_faction_bond()` | faction |
| Party member slot | `spawn_party_slot()` | party |
| Camp rest event | `spawn_camp_event()` | camp_rest |
| Crafting recipe instance | `spawn_crafting_job()` | crafting |
| Economy transaction | `spawn_shop_transaction()` | economy |
| Narrative choice node | `spawn_narrative_choice()` | narrative |
| Progression milestone | `spawn_progression_marker()` | progression |
| Terrain effect | `spawn_terrain_effect()` | terrain |
| Tactical grid entity | `spawn_tactical_entity()` | tactical |

### 2.2 Return Type

Always return `Entity`. The caller receives the entity ID and can optionally attach additional components via `commands.entity(entity)` after the factory call.

```rust
// Good — returns Entity
pub fn spawn_combat_participant(
    commands: &mut Commands,
    params: CombatParticipantParams,
) -> Entity { ... }

// Usage
let unit = spawn_combat_participant(&mut commands, params);
commands.entity(unit).insert(MyCustomMarker);
```

### 2.3 Parameter Order

1. `commands: &mut Commands` — always first
2. `params: Params` — parameter struct (when > 3 args) or flat args (when <= 3)
3. Never use `&mut World` directly in factories; use `Commands` for deferred spawning

### 2.4 Factory Visibility

- `pub(crate)` — factory functions are internal to the domain
- External domains access entity creation exclusively through `WriteFacade` methods

## 3. Parameter Struct Convention

When a factory function requires more than 3 parameters (including `commands`), use a dedicated parameter struct.

### 3.1 Naming

```
{Domain}{Noun}Params  or  {Noun}SpawnParams
```

### 3.2 Struct Requirements

```rust
/// Parameters for spawning a combat participant.
#[derive(Bundle)]
pub struct CombatParticipantParams {
    pub def_id: String,
    pub owner: Entity,
    pub faction_id: FactionId,
    pub initial_hp: i32,
    pub max_hp: i32,
    pub action_points: u32,
    pub position: GridPos,
}
```

### 3.3 Threshold Rule

| Parameter count | Approach |
|----------------|----------|
| 2-3 (incl. commands) | Flat parameters |
| 4+ (incl. commands) | Parameter struct |
| Optional params present | Parameter struct with `..default()` |

```rust
// 2 params: flat is fine
pub fn spawn_item_instance(
    commands: &mut Commands,
    template_id: impl Into<String>,
) -> Entity { ... }

// 5 params: use a struct
pub fn spawn_combat_participant(
    commands: &mut Commands,
    params: CombatParticipantParams,
) -> Entity { ... }

// Optional params: use struct with Default
#[derive(Bund‌le)]
pub struct SpellInstanceParams {
    pub def_id: String,
    pub caster: Entity,
    pub target: Entity,
    pub spell_level: u8,
    pub meta_magic: Option<MetaMagic>,  // optional
}

impl Default for SpellInstanceParams { ... }
```

## 4. Factory Location

### 4.1 Primary Location: `factory.rs`

```
src/core/domains/{domain}/
  ├── factory.rs        # Entity factory functions
  ├── mod.rs            # `pub(crate) mod factory;`
  ├── plugin.rs
  ├── error.rs
  ├── failure.rs
  ├── components.rs
  ├── events.rs
  ├── rules/
  ├── systems/
  └── integration/
```

The `factory.rs` module is `pub(crate)` — visible within the domain but not exposed externally.

### 4.2 Alternative: WriteFacade Method

When the factory needs to perform cross-capability writes (e.g., spawning a combat participant also requires initializing an `ActiveAbilityContainer` and `ActiveEffectContainer`), the factory should be exposed as a method on the domain's `WriteFacade`:

```rust
// src/core/domains/combat/integration/facade.rs
pub struct CombatWriteFacade;

impl CombatWriteFacade {
    /// Spawn a fully initialized combat participant.
    /// This is the ONLY approved way to create a combat participant entity.
    pub fn spawn_combat_participant(
        world: &mut World,
        params: CombatParticipantParams,
    ) -> Entity { ... }
}
```

### 4.3 Decision Matrix

| Scenario | Location | Rationale |
|----------|----------|-----------|
| Single-domain entity, no cross-capability deps | `factory.rs` | Simplest, keeps facade clean |
| Entity requires multiple capability components | `WriteFacade` method | Centralizes capability init |
| Entity creation requires event emission | `WriteFacade` method | Facade has world access for triggers |
| Entity is a simple data container (no behavior) | `factory.rs` | No capability wiring needed |

## 5. Example: Spawning a Combat Participant

This example demonstrates the full factory pattern for a combat participant entity.

### 5.1 Parameter Struct

```rust
/// Parameters for spawning a new combat participant.
///
/// # Required
/// - `owner`: The owning player/team entity
/// - `faction_id`: Faction affiliation for targeting
/// - `position`: Starting grid position
/// - `stats`: Base attribute values (strength, dexterity, etc.)
///
/// # Optional (defaults provided)
/// - `action_points`: Starting AP (defaults to 3)
/// - `abilities`: Pre-configured ability container (defaults to empty)
#[derive(Bundle)]
pub struct CombatParticipantParams {
    pub owner: Entity,
    pub faction_id: FactionId,
    pub position: GridPos,
    pub stats: AttributeSet,
    pub action_points: u32,
    pub abilities: ActiveAbilityContainer,
    pub effects: ActiveEffectContainer,
}

impl CombatParticipantParams {
    pub fn new(
        owner: Entity,
        faction_id: FactionId,
        position: GridPos,
        stats: AttributeSet,
    ) -> Self {
        Self {
            owner,
            faction_id,
            position,
            stats,
            action_points: 3,
            abilities: ActiveAbilityContainer::empty(),
            effects: ActiveEffectContainer::empty(),
        }
    }
}
```

### 5.2 Factory Function (WriteFacade variant)

```rust
// src/core/domains/combat/integration/facade.rs

impl CombatWriteFacade {
    /// Spawn a fully initialized combat participant.
    ///
    /// Creates the entity with:
    /// - CombatParticipant marker
    /// - ActionPoints component
    /// - AttributeSet for stat tracking
    /// - ActiveAbilityContainer for skill execution
    /// - ActiveEffectContainer for buff/debuff tracking
    /// - GridPos for tactical positioning
    /// - FactionId for target filtering
    pub fn spawn_combat_participant(
        world: &mut World,
        params: CombatParticipantParams,
    ) -> Entity {
        let entity = world.spawn((
            CombatParticipant,
            TurnQueuePosition::default(),
            ActionPoints(params.action_points),
            params.stats,
            params.abilities,
            params.effects,
            params.position,
            params.faction_id,
            Name::new(format!("CombatParticipant({:?})", params.owner)),
        )).id();

        // Emit spawn event for observers (reaction triggers, quest tracking, etc.)
        world.send_event(CombatParticipantSpawned {
            entity,
            owner: params.owner,
        });

        entity
    }
}
```

### 5.3 Usage in Systems

```rust
// Inside a system
fn on_battle_start(
    trigger: Trigger<BattleStarted>,
    mut world: Mut<World>,
) {
    let params = CombatParticipantParams::new(
        player_entity,
        FactionId::player(),
        GridPos::new(0, 0),
        AttributeSet::default(),
    );
    let entity = CombatWriteFacade::spawn_combat_participant(
        &mut world,
        params,
    );
}
```

## 6. Relationship with BSN Macro

> Note: The BSN (Bundle-Spawn-Named) macro is a planned code generation tool for entity creation. This factory convention is complementary, not a replacement.

### 6.1 Division of Responsibility

```
┌────────────────────────────────────────────┐
│            Entity Creation Flow            │
│                                            │
│  Factory function (this doc)               │
│    • Business logic & validation           │
│    • Parameter preparation                 │
│    • Conditional component insertion       │
│    • Event emission                        │
│         │                                  │
│         ▼                                  │
│  BSN Macro (planned)                       │
│    • Boilerplate reduction                 │
│    • Bundle type inference                 │
│    • Default component generation          │
│    • Name auto-assignment                  │
└────────────────────────────────────────────┘
```

### 6.2 What the Factory Provides

- **Business semantics**: The factory encodes what it means to "spawn a combat participant" — what components are required, what defaults apply, what events fire.
- **Validation**: Parameter validation (e.g., ensuring `max_hp > 0`, `faction_id` is valid).
- **Integration coordination**: When an entity requires components from multiple capabilities, the factory coordinates their initialization.

### 6.3 What the BSN Macro Provides (Future)

- **Boilerplate generation**: The macro would generate the `commands.spawn((...)).id()` call from a component list.
- **Default detection**: Automatically apply `Default::default()` for components not explicitly provided.
- **Consistency enforcement**: Ensure all entities of a given "type" always receive the same required components.
- **Name auto-generation**: Create descriptive `Name` components from type information.

### 6.4 Coexistence

```rust
// With BSN macro (future):
pub fn spawn_combat_participant(
    commands: &mut Commands,
    params: CombatParticipantParams,
) -> Entity {
    // Business logic stays in the factory
    validate_params(&params);

    // BSN macro handles the spawn boilerplate
    bsn_spawn!(commands, CombatParticipant, {
        TurnQueuePosition::default(),
        ActionPoints(params.action_points),
        params.stats,
        params.abilities,
        params.effects,
        params.position,
        params.faction_id,
    })
}
```

The factory remains the **public API** for entity creation. The BSN macro is an **internal implementation detail** of certain factories, not a replacement for the factory pattern itself.

### 6.5 Current Status

| Component | Status | Timeline |
|-----------|--------|----------|
| Factory convention (this doc) | Draft | Phase 9 |
| BSN macro implementation | Not started | Post-Phase 9 |

## 7. Migration Path

### Phase 9 (Current): Standard Definition

1. Agree on naming and parameter conventions (this document)
2. Identify all `commands.spawn(...)` calls across Core domains
3. Categorize each call as "needs factory" or "ad-hoc spawn"
4. Document priority domains for factory introduction

### Phase 10: Combat Domain

1. Create `combat/factory.rs` with `spawn_combat_participant()`
2. Migrate existing spawn sites in combat systems
3. Add `CombatWriteFacade::spawn_combat_participant()` for cross-domain usage

### Phase 11: Remaining Domains

Priority order:
1. `spell` — `spawn_active_spell()`
2. `inventory` — `spawn_item_instance()`
3. `summon` — `spawn_summoned_unit()`
4. `terrain` — `spawn_terrain_effect()`
5. All remaining domains

## 8. Forbidden Patterns

- **Direct `commands.spawn(bundle)` in system bodies**: Entity creation must go through a factory or WriteFacade method.
- **Mixing factory and ad-hoc patterns for the same entity type**: If a factory exists for `CombatParticipant`, all combat participant creation must use it.
- **Returning `EntityCommands` instead of `Entity`**: Callers that need to chain additional inserts should use `commands.entity(entity)`.
- **Factory methods on Plugin structs**: Plugin's `build()` method registers systems and resources; entity creation belongs in factories.
- **Cross-domain factory calls**: A domain's factory should not be called directly by another domain. Cross-domain entity creation goes through `WriteFacade`.

## 9. Verification Checklist

Each domain factory should satisfy:

- [ ] Naming follows `spawn_{noun}()` convention
- [ ] Returns `Entity`, not `EntityCommands`
- [ ] Takes `commands: &mut Commands` as first parameter
- [ ] Uses parameter struct when parameter count exceeds 3
- [ ] Factory is `pub(crate)` — internal to the domain
- [ ] Cross-domain creation is exposed via `WriteFacade` method
- [ ] All existing `commands.spawn(...)` call sites for that entity type use the factory
- [ ] Factory emits appropriate events when entity creation matters to other domains
