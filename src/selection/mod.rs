//!

use crate::mapping::tiles::{TileObjectStacks, TileObjects};
use bevy::app::App;
use bevy::prelude::{Component, Entity, EventReader, Plugin, Query, ResMut, Resource};
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};

pub struct BggfSelectionPlugin;

impl Plugin for BggfSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedObject>()
            .add_event::<SelectObjectEvent>()
            .add_event::<ClearSelectedObject>()
            .add_system(clear_selected_object)
            .add_system(handle_select_object_event);
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
pub struct SelectedObject {
    pub selected_entity: Option<Entity>,
}

pub fn select_object(object_to_select: Entity, mut select_object: ResMut<SelectedObject>) {
    select_object.selected_entity = Some(object_to_select);
}

pub struct ClearSelectedObject;

fn clear_selected_object(
    mut clear_selected_object_reader: EventReader<ClearSelectedObject>,
    mut selected_object: ResMut<SelectedObject>,
) {
    for _event in clear_selected_object_reader.iter() {
        selected_object.selected_entity = None;
    }
}

pub struct SelectObjectEvent {
    pub tile_pos: TilePos,
}

pub(crate) fn handle_select_object_event(
    mut select_object_event: EventReader<SelectObjectEvent>,
    mut selected_object: ResMut<SelectedObject>,
    mut tile_storage: Query<&mut TileStorage>,
    mut tile_query:  Query<&mut TileObjects>,
) {
    let mut tile_storage = tile_storage.single_mut();

    for event in select_object_event.iter(){
        select_object_at_tile_pos(event.tile_pos, &mut selected_object, &mut tile_storage, &mut tile_query);
    }
}

pub fn select_object_at_tile_pos(
    tile_pos: TilePos,
    mut selected_object: &mut ResMut<SelectedObject>,
    tile_storage: &mut TileStorage,
    tile_query: &mut Query<&mut TileObjects>,
) {
    let tile_entity = tile_storage.get(&tile_pos).unwrap();
    if let Ok(tile_objects) = tile_query.get_mut(tile_entity) {
        if let Some(entity_in_tile) = tile_objects.entities_in_tile.get(0) {
            selected_object.selected_entity = Some(*entity_in_tile);
        }
    }
}
