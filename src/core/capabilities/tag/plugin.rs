use bevy::prelude::*;

use super::content::register_tags_from_content;
use super::mechanism::TagHierarchy;
use super::mechanism::systems::tag_system::{on_tag_added, on_tag_removed};
use crate::content::LoadedTagDefs;

pub struct TagPlugin;

impl Plugin for TagPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TagHierarchy>();
        app.add_observer(on_tag_added);
        app.add_observer(on_tag_removed);
        app.add_systems(
            Update,
            register_tags_from_content
                .run_if(resource_changed::<LoadedTagDefs>),
        );
    }
}
