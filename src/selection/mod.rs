//!

use crate::game::{GameId, GameIdProvider};
use crate::mapping::tiles::TileObjects;
use crate::movement::{CurrentMovementInformation, MoveEvent};
use bevy::app::App;
use bevy::log::info;
use bevy::prelude::{
    Component, Entity, EventReader, EventWriter, Local, Plugin, Query, ResMut, Resource,
};
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};
use crate::mapping::MapId;

//TODO: Update this to actually use the Selection Component
pub struct BggfSelectionPlugin;

impl Plugin for BggfSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSelectedObject>()
            .add_event::<TrySelectEvents>()
            .add_event::<SelectionEvents>()
            .add_event::<ClearSelectedObject>()
            .add_system(clear_selected_object)
            .add_system(handle_select_object_event)
            .add_system(handle_selection_events);
    }
}

/// Marker component marking that entity as a selectable entity. Built in functionality will fallback
/// to selecting the grid using if no entity with a Selectable component is detected. The order that
/// either the grid or items in the grid are selected follows this:
/// ### {unit > building > tile}
///
#[derive(Component, Clone)]
pub struct SelectableEntity;

#[derive(Resource, Default)]
pub struct CurrentSelectedObject {
    pub object_entity: Option<GameId>,
}

impl CurrentSelectedObject {
    pub fn select_object() {}
    pub fn deselect_object() {}
}

pub fn select_object(object_to_select: GameId, mut select_object: ResMut<CurrentSelectedObject>) {
    select_object.object_entity = Some(object_to_select);
}

pub struct ClearSelectedObject;

//TODO move the current movement information clearing out of here
fn clear_selected_object(
    mut clear_selected_object_reader: EventReader<ClearSelectedObject>,
    mut selected_object: ResMut<CurrentSelectedObject>,
) {
    for _event in clear_selected_object_reader.iter() {
        selected_object.object_entity = None;
    }
}

/// Tries to select an object based on the enum chosen.
pub enum TrySelectEvents {
    TilePos(TilePos),
}

/// Sent when a selection is valid.
pub enum SelectionEvents {
    ObjectSelected(GameId, MapId),
}

#[derive(Default, Clone)]
pub struct LastSelectedTileInfo {
    tile_pos: TilePos,
    selected_entities: Vec<GameId>,
}

/// Todo update try select events to select on a map and make this get that maps stuff
pub(crate) fn handle_select_object_event(
    mut try_select_events: EventReader<TrySelectEvents>,
    mut selection_events: EventWriter<SelectionEvents>,
    mut selected_object: ResMut<CurrentSelectedObject>,
    mut tile_storage: Query<(&MapId, &mut TileStorage)>,
    mut tile_query: Query<&mut TileObjects>,
    mut tile_selected_info: Local<LastSelectedTileInfo>,
) {
    let Ok((map_id, mut tile_storage)) = tile_storage.get_single_mut()else{
        return;
    };
    for event in try_select_events.iter() {
        match event {
            TrySelectEvents::TilePos(tile_pos) => {
                selected_object.object_entity = None;

                select_object_at_tile_pos(
                    tile_pos,
                    &mut selected_object,
                    &mut tile_storage,
                    *map_id,
                    &mut tile_query,
                    &mut selection_events,
                    &mut tile_selected_info,
                );
            }
        }
    }
}

pub fn select_object_at_tile_pos(
    tile_pos: &TilePos,
    selected_object: &mut ResMut<CurrentSelectedObject>,
    tile_storage: &mut TileStorage,
    on_map: MapId,
    tile_query: &mut Query<&mut TileObjects>,
    selection_events: &mut EventWriter<SelectionEvents>,
    tile_selected_info: &mut Local<LastSelectedTileInfo>,
) {
    let tile_entity = tile_storage.get(tile_pos).unwrap();
    if let Ok(tile_objects) = tile_query.get_mut(tile_entity) {
        // if the tile pos of the selected tile is the same as the one in the tile pos we saved,
        // we want to get the next unselected entity in the tile based on our list
        if *tile_pos != tile_selected_info.tile_pos {
            tile_selected_info.selected_entities.clear();
            tile_selected_info.tile_pos = *tile_pos;
        }
        let mut entity_selected = false;

        for i in 0..tile_objects.entities_in_tile.len() {
            if let Some(object_id) = tile_objects.entities_in_tile.get(i) {
                if tile_selected_info.selected_entities.contains(object_id) {
                    continue;
                }
                info!("Object Selected");
                selected_object.object_entity = Some(*object_id);
                selection_events.send(SelectionEvents::ObjectSelected(*object_id, on_map));
                tile_selected_info.selected_entities.push(*object_id);
                entity_selected = true;
                break;
            }
        }
        if !entity_selected {
            if let Some(entity_in_tile) = tile_objects.entities_in_tile.get(0) {
                info!("Object Selected");
                selected_object.object_entity = Some(*entity_in_tile);
                tile_selected_info.selected_entities.clear();
                tile_selected_info.selected_entities.push(*entity_in_tile);
                selection_events.send(SelectionEvents::ObjectSelected(*entity_in_tile, on_map));
            }
        }
    }
}

fn handle_selection_events(
    mut selection_events: EventReader<SelectionEvents>,
    mut move_events: EventWriter<MoveEvent>,
) {
    for event in selection_events.iter() {
        match event {
            SelectionEvents::ObjectSelected(entity, map_id) => {
                move_events.send(MoveEvent::MoveBegin {
                    object_moving: *entity,
                    on_map: *map_id,
                })
            }
        }
    }
}
