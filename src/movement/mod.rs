use std::borrow::BorrowMut;
use crate::mapping::terrain::{TerrainType, TileTerrainInfo};
use crate::mapping::tiles::{ObjectStackingClass, TileObjectStacks, TileObjects};
use crate::mapping::{tile_pos_to_centered_map_world_pos, Map, remove_object_from_tile, add_object_to_tile};
use crate::object::{Object, ObjectGridPosition};
use bevy::app::{App, CoreStage};
use bevy::log::info;
use bevy::prelude::{Bundle, Component, Entity, EventReader, EventWriter, IntoSystemDescriptor, ParamSet, Plugin, Query, ResMut, Resource, RunCriteriaDescriptorCoercion, SystemStage, Transform, With, Without, World};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapGridSize, TilemapSize, TilemapType, SquarePos};

/// Movement System

pub struct BggfMovementPlugin;

impl Plugin for BggfMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMovementRules>()
            .init_resource::<MovementInformation>()
            .add_event::<MoveEvent>()
            .add_event::<MoveError>()
            .add_system(handle_move_begin_events)
            .add_system(handle_try_move_events);
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Default)]
pub enum DiagonalMovement{
    Enabled,
    #[default]
    Disabled
}

#[derive(Clone, Eq, Hash, PartialEq, Default, Resource)]
pub struct MovementInformation {
    available_moves: Vec<TilePos>,
    diagonal_movement: DiagonalMovement,
}

pub struct MovementCalculatorTest {
    tilemap_type: TilemapType,
    tilemap_size: TilemapSize,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum MoveError {
    NotValidMove(String),
}

impl Default for MoveError {
    fn default() -> Self {
        MoveError::NotValidMove(String::from("Invalid Move"))
    }
}

///
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum MoveEvent {
    MoveBegin {
        object_moving: Entity,
    },
    MoveCalculated {
        available_moves: Vec<TilePos>,
    },
    TryMoveObject {
        object_moving: Entity,
        new_pos: TilePos,
    },
    MoveComplete {
        object_moved: Entity,
    },
}

// main events
// MoveBegin
// MoveCalculated (Vec<TilePos>)
// MoveObject
// MoveComplete


fn handle_move_begin_events(
    mut move_events: ParamSet<(
        EventReader<MoveEvent>,
        EventWriter<MoveEvent>,
    )>,
    mut object_query: Query<
        (
            &mut ObjectGridPosition,
            &ObjectStackingClass,
            &UnitMovementType,

        ),
        With<Object>,
    >,
    mut tile_query: Query<(&mut TileObjectStacks, &TileTerrainInfo)>,
    mut tilemap_q: Query<
        (
            &mut Map,
            &mut TileStorage,
            &TilemapSize
        ),
        Without<Object>,
    >,
    mut movement_information: ResMut<MovementInformation>,
    mut move_error_writer: EventWriter<MoveError>,
) {
    
    for event in move_events.p0().iter() {
        match event {
            MoveEvent::MoveBegin { object_moving } => {
                calculate_move(
                    object_moving,
                    &mut object_query,
                    &mut tile_query,
                    &mut tilemap_q,
                    &mut movement_information,
                );
            }
            _ =>{}
        }
    }
}


pub fn begin_move(object_moving: Entity) {}

pub fn calculate_move(
    object_moving: &Entity,
    object_query: &mut Query<
        (
            &mut ObjectGridPosition,
            &ObjectStackingClass,
            &UnitMovementType,

        ),
        With<Object>,
    >,
    mut tile_query: &mut Query<(&mut TileObjectStacks, &TileTerrainInfo)>,
    tilemap_q: &mut Query<
        (
            &mut Map,
            &mut TileStorage,
            &TilemapSize
        ),
        Without<Object>,
    >,
    movement_information: &mut ResMut<MovementInformation>,
)  {


    // Get the moving objects stuff
    let (mut object_grid_position, object_stack_class, object_move_type) =
        object_query.get_mut(*object_moving).unwrap();

    // gets the map components
    let (map, mut tile_storage, tilemap_size) = tilemap_q.single_mut();


    let mut tiles_to_evaluate: Vec<TilePos> = get_neighbors_tile_pos(object_grid_position.grid_position, tilemap_size, movement_information);

    while tiles_to_evaluate.len() > 0 {
    
    }

    movement_information.available_moves.append(&mut tiles_to_evaluate);
    /*
    movement_information.available_moves.push(TilePos{ x: object_grid_position.grid_position.x, y: object_grid_position.grid_position.y + 1 });
    movement_information.available_moves.push(TilePos{ x: object_grid_position.grid_position.x + 1, y: object_grid_position.grid_position.y });
    movement_information.available_moves.push(TilePos{ x: object_grid_position.grid_position.x, y: object_grid_position.grid_position.y - 1 });
    movement_information.available_moves.push(TilePos{ x: object_grid_position.grid_position.x - 1, y: object_grid_position.grid_position.y });
    
     */
}



fn handle_try_move_events(
    mut move_events: ParamSet<(
        EventReader<MoveEvent>,
        EventWriter<MoveEvent>,
    )>,
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
    mut movement_information: ResMut<MovementInformation>,
    mut move_error_writer: EventWriter<MoveError>,
) {
    
    let mut result:Result<MoveEvent, MoveError> = Err(MoveError::NotValidMove(String::from("Try move failed")));
    
    for event in move_events.p0().iter() {
        match event {
            MoveEvent::TryMoveObject {
                object_moving,
                new_pos,
            } => {
                if check_move(new_pos, &mut movement_information) {
                    info!("check move worked");
                    result = move_object(
                        object_moving,
                        new_pos,
                        &mut object_query,
                        &mut tile_query,
                        &mut tilemap_q,
                    );
                } else {
                    result = Err(MoveError::default());
                }
            }
            _ =>{}
        }
    }
    match result{
        Ok(move_event) => {
            move_events.p1().send(move_event);
        }
        Err(error) => {
            move_error_writer.send(error);
        }
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
) -> Result<MoveEvent, MoveError> {
    // gets the components needed to move the object
    let (mut transform, mut object_grid_position, object_stack_class) =
        object_query.get_mut(*object_moving).unwrap();

    // gets the map components
    let (map, grid_size, map_type, mut tile_storage, map_transform) = tilemap_q.single_mut();

    // if a tile exists at the selected point
    if let Some(tile_entity) = tile_storage.get(&new_pos) {
        // if the tile has the needed components
        if let Ok((_tile_stack_rules, _tile_objects)) = tile_query.get(tile_entity) {
            remove_object_from_tile(
                *object_moving,
                &object_stack_class,
                &mut tile_storage,
                &mut tile_query,
                object_grid_position.grid_position,
            );
            add_object_to_tile(
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
            
            return Ok(MoveEvent::MoveComplete {
                object_moved: *object_moving,
            });
        } else {
            return Err(MoveError::NotValidMove(String::from("Tile does not have needed components")));
        }
    } else {
        return Err(MoveError::NotValidMove(String::from("Move Position not valid")));
    }
}



pub fn move_complete(object_moving: Entity) {}


pub fn check_move(
    new_pos: &TilePos,
    movement_information: &mut ResMut<MovementInformation>,
) -> bool {
    return if movement_information.available_moves.contains(new_pos) {
        true
    } else {
        false
    };
}

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

pub fn get_neighbors_tile_pos(origin_tile: TilePos, tilemap_size: &TilemapSize, movement_information: &mut ResMut<MovementInformation>,
) -> Vec<TilePos> {
    let mut neighbor_tiles: Vec<TilePos> = vec![];

    if let Some(north) = TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 + 1, &tilemap_size){
        neighbor_tiles.push(north);
    }
    if let Some(east) = TilePos::from_i32_pair(origin_tile.x as i32 + 1, origin_tile.y as i32, &tilemap_size){
        neighbor_tiles.push(east);
    }
    if let Some(south) = TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 - 1, &tilemap_size){
        neighbor_tiles.push(south);
    }
    if let Some(west) = TilePos::from_i32_pair(origin_tile.x as i32 - 1, origin_tile.y as i32, &tilemap_size){
        neighbor_tiles.push(west);
    }
    
    if movement_information.diagonal_movement == DiagonalMovement::Enabled{
        if let Some(north) = TilePos::from_i32_pair(origin_tile.x as i32 - 1, origin_tile.y as i32 + 1, &tilemap_size){
            neighbor_tiles.push(north);
        }
        if let Some(east) = TilePos::from_i32_pair(origin_tile.x as i32 + 1, origin_tile.y as i32 + 1, &tilemap_size){
            neighbor_tiles.push(east);
        }
        if let Some(south) = TilePos::from_i32_pair(origin_tile.x as i32 + 1, origin_tile.y as i32 - 1, &tilemap_size){
            neighbor_tiles.push(south);
        }
        if let Some(west) = TilePos::from_i32_pair(origin_tile.x as i32 - 1, origin_tile.y as i32 - 1, &tilemap_size){
            neighbor_tiles.push(west);
        }
    }
    neighbor_tiles
}



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
    pub unit_movement_type: UnitMovementType,
}

/// Holds a reference to a units [`MovementType`]. A MovementType is used to define what kind of movement
/// costs that the unit uses during movement
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct UnitMovementType {
    pub movement_type: &'static MovementType,
}
