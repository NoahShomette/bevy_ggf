//!

use bevy::app::App;
use bevy::prelude::{Component, Entity, Plugin, Resource};

pub struct BggfSelectionPlugin;

impl Plugin for BggfSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSelected>();
    }
}

/// Marker component marking that entity as a selectable entity. Built in functionality will fallback
/// to selecting the grid using if no entity with a Selectable component is detected. The order that
/// either the grid or items in the grid are selected follows this:
/// ### {unit > building > tile}
///
#[derive(Component)]
pub struct SelectableEntity;

#[derive(Resource, Default)]
pub struct PlayerSelected {
    selected_entity: Option<Entity>,
}
