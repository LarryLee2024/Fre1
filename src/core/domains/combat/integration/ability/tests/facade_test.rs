use crate::core::domains::combat::integration::ability::{CombatAbilityFacade, CombatAbilityParam};

#[test]
fn combat_ability_facade_compiles() {
    let _ = std::any::type_name::<CombatAbilityFacade>();
    let _ = std::any::type_name::<CombatAbilityParam<'_, '_>>();
}
