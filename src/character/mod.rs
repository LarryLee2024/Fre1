// 角色模块：单位组件、生成、模板、特性
// 合并了原 unit.rs、data/unit_template.rs、core/trait_def.rs

mod components;
mod marker;
mod movement;
mod plugin;
mod spawn;
mod template;
mod traits;

// 公共 re-exports
pub use components::*;
pub use marker::*;
pub use movement::*;
pub use plugin::CharacterPlugin;
pub use traits::{TraitCollection, TraitData, TraitEffect, TraitEffectHandlerRegistry, TraitRegistry, TraitSource, TraitTrigger};
