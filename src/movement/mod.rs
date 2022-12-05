use crate::mapping::terrain::{TerrainClass, TerrainType};
use crate::mapping::tiles::{ObjectStackingClass, TileObjectStacks, TileObjects};
use crate::mapping::{tile_pos_to_centered_map_world_pos, Map};
use crate::object::{Object, ObjectGridPosition, ObjectInfo};
use bevy::app::App;
use bevy::prelude::{Bundle, Component, Entity, EventReader, EventWriter, Plugin, Query, Resource, Transform, With, Without};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapGridSize, TilemapSize, TilemapType};

/// Movement System

pub struct BggfMovementPlugin;

impl Plugin for BggfMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMovementRules>()
            .add_event::<MoveObjectEvent>()
            .add_event::<MoveCompleteEvent>()
            .add_system(handle_move_object_events);
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MoveObjectEvent {
    pub object_moving: Entity,
    pub new_pos: TilePos,
}

pub(crate) fn handle_move_object_events(
    mut move_event_reader: EventReader<MoveObjectEvent>,
    mut object_query: Query<
        (
            &mut Transform,
            &mut ObjectGridPosition,
            &ObjectStackingClass,
        ),
        With<Object>,
    >,
    mut tile_query: Query<(&mut TileObjectStacks, &mut TileObjects)>,
    mut tilemap_q: Query<
        (
            &mut Map,
            &TilemapGridSize,
            &TilemapType,
            &mut TileStorage,
            &Transform,
        ),
        Without<Object>,
    >,
    mut move_event_writer: EventWriter<MoveCompleteEvent>

) {
    for event in move_event_reader.iter() {
        move_object(
            &event.object_moving,
            &event.new_pos,
            &mut object_query,
            &mut tile_query,
            &mut tilemap_q,
            &mut move_event_writer,
        );
    }
}

/// this is an unchecked move of an object. It adds the object to the tile at new_pos,
/// it removes the object from the old tile, and then it moves the object transform and updates the
/// object tile_pos.
/// it does not provide any sort of valid movement
///
/// currently only works for one map entity. If there are more than one it will panic

pub fn move_object(
    object_moving: &Entity,
    new_pos: &TilePos,
    object_query: &mut Query<
        (
            &mut Transform,
            &mut ObjectGridPosition,
            &ObjectStackingClass,
        ),
        With<Object>,
    >,
    mut tile_query: &mut Query<(&mut TileObjectStacks, &mut TileObjects)>,
    tilemap_q: &mut Query<
        (
            &mut Map,
            &TilemapGridSize,
            &TilemapType,
            &mut TileStorage,
            &Transform,
        ),
        Without<Object>,
    >,
    mut move_event_writer: &mut EventWriter<MoveCompleteEvent>
) {
    
    // gets the components needed to move the object
    let (mut transform, mut object_grid_position, object_stack_class) =
        object_query.get_mut(*object_moving).unwrap();

    // gets the map components
    let (map, grid_size, map_type, mut tile_storage, map_transform) = tilemap_q.single_mut();

    // if a tile exists at the selected point
    if let Some(tile_entity) = tile_storage.get(&new_pos) {
        // if the tile has the needed components
        if let Ok((_tile_stack_rules, _tile_objects)) = tile_query.get(tile_entity) {
            map.remove_object_from_tile(
                *object_moving,
                &object_stack_class,
                &mut tile_storage,
                &mut tile_query,
                object_grid_position.grid_position,
            );
            map.add_object_to_tile(
                *object_moving,
                &mut object_grid_position,
                &object_stack_class,
                &mut tile_storage,
                &mut tile_query,
                *new_pos,
            );

            // have to transform the tiles position to the transformed position to place the object at the right point
            let tile_world_pos =
                tile_pos_to_centered_map_world_pos(&new_pos, map_transform, grid_size, map_type);

            transform.translation = tile_world_pos.extend(5.0);
            
            move_event_writer.send_default();
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub struct MoveCompleteEvent;

/*
// just quick example of a movement system might work for a unit
struct UnitMovementRules {
    terrain_base_rules: HashMap<&'static TerrainClass, bool>,
    terrain_extension_rules: HashMap<&'static TerrainType, bool>,
}


fn test() {
    let mut movement_rules = UnitMovementRules {
        terrain_base_rules: HashMap::new(),
        terrain_extension_rules: HashMap::new(),
    };

    movement_rules
        .terrain_base_rules
        .insert(&TERRAIN_BASE_TYPES[0], true);
    movement_rules
        .terrain_extension_rules
        .insert(&TERRAIN_EXTENSION_TYPES[2], false);
}

 */

/// Struct used to define a new [`MovementType`]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MovementType {
    pub name: &'static str,
}

/// Component that must be added to a tile in order to define that tiles movement cost.
///
/// Contains a hashmap that holds a reference to a [`MovementType`] as a key and a u32 as the value. The u32 is used
/// in pathfinding as the cost to move into that tile.
#[derive(Clone, Eq, PartialEq, Component)]
pub struct TileMovementCosts {
    pub movement_type_cost: HashMap<&'static MovementType, u32>,
}

/// Defines a resource that will hold all [`TileMovementCosts`] - references to a specific TileMovementCosts
/// are stored in each tile as their current cost.
#[derive(Resource, Default)]
pub struct TileMovementRules {
    pub movement_cost_rules: HashMap<TerrainType, TileMovementCosts>,
}

//UNIT MOVEMENT STUFF

/// Basic Bundle that supplies all needed movement components for a unit
#[derive(Bundle)]
pub struct UnitMovementBundle {
    unit_movement_type: UnitMovementType,
}

/// Holds a reference to a units [`MovementType`]. A MovementType is used to define what kind of movement
/// costs that the unit uses during movement
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct UnitMovementType {
    movement_type: &'static MovementType,
}
