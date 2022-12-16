use crate::mapping::terrain::{TerrainClass, TerrainType, TileTerrainInfo};
use crate::mapping::tiles::{ObjectStackingClass, Tile, TileObjectStacks, TileObjects};
use crate::mapping::{
    add_object_to_tile, remove_object_from_tile, tile_pos_to_centered_map_world_pos, Map,
};
use crate::object::{Object, ObjectGridPosition};
use bevy::app::{App, CoreStage};
use bevy::ecs::system::SystemState;
use bevy::log::info;
use bevy::prelude::{Bundle, Component, Entity, EventReader, EventWriter, IntoSystemDescriptor, ParamSet, Plugin, Query, QueryState, Res, ResMut, Resource, RunCriteriaDescriptorCoercion, SystemStage, Transform, With, Without, World};
use bevy::utils::tracing::event;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapGridSize, TilemapSize, TilemapType};

/// Movement System

pub struct BggfMovementPlugin;

impl Plugin for BggfMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMovementRules>()
            .init_resource::<CurrentMovementInformation>()
            .add_event::<MoveEvent>()
            .add_event::<MoveError>()
            .add_system(handle_move_begin_events)
            .add_system(handle_try_move_events);
    }
}

// movement system arch

// movement calculator trait
//-- any movement system resource implements this and then internal movement stuff just needs to call trait functions
// -- stuff like, calculate move, etc

// MovementSystem - a resource which holds the relevant information for that movement system. maybe this is built by the dev? and then it implements the movement calculator

// make a resource, that holds dyn structs that implement the TileMoveCheck trait. This trait will get called to ensure a move is valid
// so when we register the resource we should throw in the move systems

pub trait AppMovementSystem {
    fn new(
        map_type: TilemapType,
        diagonal_movement: DiagonalMovement,
        tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
    ) -> SquareMovementSystem;

    fn register_movement_system(
        app: &mut App,
        map_type: TilemapType,
        diagonal_movement: DiagonalMovement,
        tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
    );
}

pub trait MovementCalculator: 'static + Send + Sync {
    fn calculate_move(
        &self,
        movement_system: &ResMut<MovementSystem>,
        object_moving: &Entity,
        world: &mut World,
    ) -> Vec<TilePos>;
}

#[derive(Resource)]
pub struct MovementSystem {
    movement_calculator: Box<dyn MovementCalculator>,
    pub map_type: TilemapType,
    pub tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
}

impl MovementSystem {
    fn new(
        map_type: TilemapType,
        movement_calculator: Box<dyn MovementCalculator>,
        tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
    ) -> MovementSystem {
        MovementSystem {
            movement_calculator,
            map_type,
            tile_move_checks,
        }
    }

    fn register_movement_system(
        app: &mut App,
        map_type: TilemapType,
        movement_calculator: Box<dyn MovementCalculator>,
        tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
    ) {
        let movement_system = MovementSystem::new(map_type, movement_calculator, tile_move_checks);
        app.world.insert_resource(movement_system);
    }
}

pub struct SquareMovementSystem {
    pub diagonal_movement: DiagonalMovement,
}

impl MovementCalculator for SquareMovementSystem {
    fn calculate_move(
        &self,
        movement_system: &ResMut<MovementSystem>,
        object_moving: &Entity,
        mut world: &mut World,

    ) -> Vec<TilePos> {

        // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
        // as if you were writing an ordinary system.
        let mut system_state: SystemState<(
            Query<(&ObjectGridPosition, &ObjectStackingClass, &ObjectMovement), With<Object>>,
            Query<(&mut Map, &mut TileStorage, &TilemapSize), Without<Object>>,
            ResMut<CurrentMovementInformation>,
        )> = SystemState::new(&mut world);

        // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
        // system_state.get(&world) provides read-only versions of your system parameters instead.
        let (
            object_query,
            mut tilemap_q,
            mut movement_information,
        ) = system_state.get_mut(&mut world);
        
        // Get the moving objects stuff
        let (object_grid_position, object_stack_class, object_movement) =
            object_query.get(*object_moving).unwrap();

        // gets the map components
        let (map, tile_storage, tilemap_size) = tilemap_q.single_mut();

        let mut move_info = MovementNodes {
            move_nodes: HashMap::new(),
        };

        let mut available_moves: Vec<TilePos> = vec![];

        // insert the starting node at the moving objects grid position
        move_info.move_nodes.insert(
            object_grid_position.tile_position,
            MoveNode {
                node_pos: object_grid_position.tile_position,
                prior_node: object_grid_position.tile_position,
                move_cost: Some(0),
            },
        );

        // unvisited nodes
        let mut unvisited_nodes: Vec<MoveNode> = vec![MoveNode {
            node_pos: object_grid_position.tile_position,
            prior_node: object_grid_position.tile_position,
            move_cost: Some(0),
        }];
        let mut visited_nodes: Vec<TilePos> = vec![];

        while unvisited_nodes.len() > 0 {
            unvisited_nodes.sort_by(|x, y| {
                x.move_cost
                    .unwrap()
                    .partial_cmp(&y.move_cost.unwrap())
                    .unwrap()
            });

            let Some(current_node) = unvisited_nodes.get(0) else {
                continue;
            };

            let neighbors = move_info.get_node_neighbors(
                current_node.node_pos,
                self.diagonal_movement.is_diagonal(),
                tilemap_size,
            );

            let current_node = *current_node;

            for neighbor in neighbors.iter() {
                if visited_nodes.contains(neighbor) {
                    continue;
                }
                let Some(tile_entity) = tile_storage.get(&neighbor) else {
                    continue;

                };
                move_info.add_node(neighbor, current_node);

                // checks the tile against each of the move rules added if its false kill this loop
                for i in 0..movement_system.tile_move_checks.len() {
                    let check = movement_system.tile_move_checks[i].as_ref();
                    if check.is_valid_move(
                        *object_moving,
                        tile_entity,
                        neighbor,
                        &current_node.node_pos,
                        &mut move_info,
                        world,
                    ) {
                    } else {
                        continue;
                    }
                }

                // if none of them return false and cancel the loop then we can infer that we are able to move into that neighbor
                // we add the neighbor to the list of unvisited nodes and then push the neighbor to the available moves list
                unvisited_nodes.push(*move_info.get_node_mut(neighbor).unwrap()); //unwrap is safe because we know we add the node in at the beginning of this loop
                available_moves.push(*neighbor);
            }

            unvisited_nodes.remove(0);
            visited_nodes.push(current_node.node_pos);
        }
        movement_information.move_nodes = move_info.move_nodes;
        available_moves
    }
}

pub trait TileMoveCheck {
    fn is_valid_move(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        tile_pos: &TilePos,
        move_from_tile_pos: &TilePos,
        movement_nodes: &mut MovementNodes,
        world: &World,
    ) -> bool;
}

pub struct MovementCostCheck;

impl TileMoveCheck for MovementCostCheck {
    fn is_valid_move(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        tile_pos: &TilePos,
        move_from_tile_pos: &TilePos,
        movement_nodes: &mut MovementNodes,
        world: &World,
    ) -> bool {
        let Some(object_movement) = world.get::<ObjectMovement>(entity_moving) else {
            return false;
        };
        let Some(tile_movement_costs) = world.get::<TileMovementCosts>(tile_entity) else {
            return false;
        };
        
        let Some((tile_node, move_from_tile_node)) = movement_nodes.get_two_node_mut(tile_pos, move_from_tile_pos) else{
            return false;
        };
        if tile_node.move_cost.is_some() {
            if (move_from_tile_node.move_cost.unwrap()
                + *tile_movement_costs
                    .movement_type_cost
                    .get(object_movement.movement_type)
                    .unwrap_or_else(|| &1) as i32)
                < (tile_node.move_cost.unwrap())
            {
                tile_node.move_cost = Some(
                    move_from_tile_node.move_cost.unwrap()
                        + *tile_movement_costs
                            .movement_type_cost
                            .get(object_movement.movement_type)
                            .unwrap_or_else(|| &1) as i32,
                );
                tile_node.prior_node = move_from_tile_node.node_pos;
                return true;
            }
        } else {
            if (move_from_tile_node.move_cost.unwrap()
                + *tile_movement_costs
                    .movement_type_cost
                    .get(object_movement.movement_type)
                    .unwrap_or_else(|| &1) as i32)
                <= object_movement.move_points
            {
                tile_node.move_cost = Some(
                    move_from_tile_node.move_cost.unwrap()
                        + *tile_movement_costs
                            .movement_type_cost
                            .get(object_movement.movement_type)
                            .unwrap_or_else(|| &1) as i32,
                );
                tile_node.prior_node = move_from_tile_node.node_pos;
                return true;
            }
        };
        return false;
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Default)]
pub enum DiagonalMovement {
    Enabled,
    #[default]
    Disabled,
}

impl DiagonalMovement {
    pub fn is_diagonal(&self) -> bool {
        return match self {
            DiagonalMovement::Enabled => true,
            DiagonalMovement::Disabled => false,
        };
    }
}

#[derive(Clone, Eq, PartialEq, Default, Resource)]
pub struct CurrentMovementInformation {
    pub available_moves: Vec<TilePos>,
    pub move_nodes: HashMap<TilePos, MoveNode>,
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

fn handle_move_begin_events(mut world: &mut World) {
    // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
    // as if you were writing an ordinary system.
    let mut system_state: SystemState<(
        ParamSet<(EventReader<MoveEvent>, EventWriter<MoveEvent>)>,
        ResMut<MovementSystem>,
    )> = SystemState::new(&mut world);

    // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
    // system_state.get(&world) provides read-only versions of your system parameters instead.
    let (
        mut move_events,
        movement_system,
    ) = system_state.get_mut(&mut world);
    
    let mut move_events = move_events.p0();
    let events: Vec<&MoveEvent> = move_events.iter().collect();
    for event in events {
        match event {
            MoveEvent::MoveBegin { object_moving } => {
                movement_system.movement_calculator.calculate_move(
                    &movement_system,
                    &object_moving,
                    &mut world,
                );
            }
            _ => {}
        }
    }
    move_events.clear();
}

pub struct MovementNodes {
    pub move_nodes: HashMap<TilePos, MoveNode>,
}

impl MovementNodes {
    pub fn add_node(&mut self, tile_pos: &TilePos, prior_node: MoveNode) {
        // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
        if self.move_nodes.contains_key(&tile_pos) {
        } else {
            let node = MoveNode {
                node_pos: *tile_pos,
                prior_node: prior_node.node_pos,
                move_cost: None,
            };
            self.move_nodes.insert(*tile_pos, node);
        }
    }

    pub fn get_node_mut(&mut self, tile_pos: &TilePos) -> Option<&mut MoveNode> {
        // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
        self.move_nodes.get_mut(&tile_pos)
    }

    pub fn get_two_node_mut(
        &mut self,
        node_one: &TilePos,
        node_two: &TilePos,
    ) -> Option<(&mut MoveNode, &mut MoveNode)> {
        // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
        return if let Some(nodes) = self.move_nodes.get_many_mut([node_one, node_two]) {
            match nodes {
                [node1, node2] => {
                    if node1.node_pos == *node_one {
                        Some((node1, node2))
                    } else {
                        Some((node2, node1))
                    }
                }
            }
        } else {
            None
        };
    }

    pub fn get_node_neighbors(
        &self,
        node_to_get_neighbors: TilePos,
        diagonal_movement: bool,
        tilemap_size: &TilemapSize,
    ) -> Vec<TilePos> {
        let mut neighbor_tiles: Vec<TilePos> = vec![];
        let origin_tile = node_to_get_neighbors;
        if let Some(north) = TilePos::from_i32_pair(
            origin_tile.x as i32,
            origin_tile.y as i32 + 1,
            &tilemap_size,
        ) {
            neighbor_tiles.push(north);
        }
        if let Some(east) = TilePos::from_i32_pair(
            origin_tile.x as i32 + 1,
            origin_tile.y as i32,
            &tilemap_size,
        ) {
            neighbor_tiles.push(east);
        }
        if let Some(south) = TilePos::from_i32_pair(
            origin_tile.x as i32,
            origin_tile.y as i32 - 1,
            &tilemap_size,
        ) {
            neighbor_tiles.push(south);
        }
        if let Some(west) = TilePos::from_i32_pair(
            origin_tile.x as i32 - 1,
            origin_tile.y as i32,
            &tilemap_size,
        ) {
            neighbor_tiles.push(west);
        }

        if diagonal_movement {
            if let Some(northwest) = TilePos::from_i32_pair(
                origin_tile.x as i32 - 1,
                origin_tile.y as i32 + 1,
                &tilemap_size,
            ) {
                neighbor_tiles.push(northwest);
            }
            if let Some(northeast) = TilePos::from_i32_pair(
                origin_tile.x as i32 + 1,
                origin_tile.y as i32 + 1,
                &tilemap_size,
            ) {
                neighbor_tiles.push(northeast);
            }
            if let Some(southeast) = TilePos::from_i32_pair(
                origin_tile.x as i32 + 1,
                origin_tile.y as i32 - 1,
                &tilemap_size,
            ) {
                neighbor_tiles.push(southeast);
            }
            if let Some(southwest) = TilePos::from_i32_pair(
                origin_tile.x as i32 - 1,
                origin_tile.y as i32 - 1,
                &tilemap_size,
            ) {
                neighbor_tiles.push(southwest);
            }
        }
        neighbor_tiles
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq)]
pub struct MoveNode {
    pub node_pos: TilePos,
    pub prior_node: TilePos,
    pub move_cost: Option<i32>,
}

// djikstras
// We calculate each neighboring node, whether we can move to them or not and what their movement cost is
// we keep all the nodes we've built but havent visited in a list sorted by the movement cost
// we guessed for them. Every node in this list will have a movement speed guess as we only insert after we have guessed it
// every time we evauluate all of the nodes of a neighbor we mark the node as visited, remove it from the unvisited list
// then pick the next shortest unvisited node to evaluate its neighbors. We should be able to cross reference
// the visited list with the nodes neighbors to ensure we dont ever get a node we've already visited before
/*
while nodes to evaluate > 0{

get node with shortest guess

get that nodes neighbors
 if the visited list contains a neighbor we ignore that neighbor
guess for all the nodes neighbors
guess for each neighbor
--- run the can move here functions. if we can move then we guess its movement cost, move it to the unvisited list and then keep going
after evaluating each neighbor we end that loop and restart again if we have more nodes to visit




}
*/
/*
pub fn calculate_move(
    object_moving: &Entity,
    object_query: &Query<
        (&ObjectGridPosition, &ObjectStackingClass, &ObjectMovement),
        With<Object>,
    >,
    mut tile_query: &Query<(&TileObjectStacks, &TileTerrainInfo, &TileMovementCosts)>,
    tilemap_q: &mut Query<(&mut Map, &mut TileStorage, &TilemapSize), Without<Object>>,
    movement_information: &mut ResMut<CurrentMovementInformation>,
) {
    // Get the moving objects stuff
    let (object_grid_position, object_stack_class, object_movement) =
        object_query.get(*object_moving).unwrap();

    // gets the map components
    let (map, tile_storage, tilemap_size) = tilemap_q.single_mut();

    let mut movement_calculator = MovementNodes {
        move_nodes: HashMap::new(),
    };
    movement_calculator.move_nodes.insert(
        object_grid_position.tile_position,
        MoveNode {
            node_pos: object_grid_position.tile_position,
            prior_node: object_grid_position.tile_position,
            move_cost: Some(0),
        },
    );

    // unvisited nodes
    let mut unvisited_nodes: Vec<MoveNode> = vec![MoveNode {
        node_pos: object_grid_position.tile_position,
        prior_node: object_grid_position.tile_position,
        move_cost: Some(0),
    }];
    let mut visited_nodes: Vec<TilePos> = vec![];

    while unvisited_nodes.len() > 0 {
        unvisited_nodes.sort_by(|x, y| {
            x.move_cost
                .unwrap()
                .partial_cmp(&y.move_cost.unwrap())
                .unwrap()
        });

        let Some(current_node) = unvisited_nodes.get(0) else {
            continue;
            };

        let neighbors = movement_calculator.get_node_neighbors(
            current_node.node_pos,
            movement_information,
            tilemap_size,
        );

        let current_node = *current_node;

        for neighbor in neighbors.iter() {
            if visited_nodes.contains(neighbor) {
                continue;
            }
            let Some(tile_entity) = tile_storage.get(&neighbor) else {
                continue;

                };
            // if the tile has the needed components
            if let Ok((tile_objects, tile_terrain_info, tile_movement_costs)) =
                tile_query.get(tile_entity)
            {
                //info!("{:?}", tile_terrain_info.terrain_type);
                //info!("{:?}", neighbor);

                if tile_objects.has_space(object_stack_class) != true {
                    //info!("No Space");
                    continue;
                }

                if object_movement
                    .object_terrain_movement_rules
                    .can_move_on_tile(tile_terrain_info)
                    != true
                {
                    //info!("Not allowed on terrain");
                    visited_nodes.push(*neighbor);
                    continue;
                }

                //
                if calculate_move_node(
                    current_node,
                    neighbor,
                    &mut movement_calculator,
                    tile_movement_costs,
                    object_movement,
                ) {
                    //info!("had enough movement");
                    unvisited_nodes.push(*movement_calculator.get_node(neighbor, current_node));
                    movement_information.available_moves.push(*neighbor);
                } else {
                    //info!("Not enough movement");
                }

                // we have the tile that got the neighbor, the tile we are checking, and that tile has
                // its cost as well as the current lowest cost tile that reached it

                // We want to send the tile that got the current tile, the current tile, and whatever it
                // needs to a function and return a bool telling us if we can move there. If we can then
                // we want to add this new tile to the list of available moves and then add it to the list of
                // nodes to evaluate. And then we remove the current node after we finished all the neighbors

                // the tile has space. so now we need to decide can we even move to this tile
                // basically we take the move cost from the prior_move_node, add the move cost from
                // the new tile, and see if we have enough movement to make it into it
            }
        }

        unvisited_nodes.remove(0);
        visited_nodes.push(current_node.node_pos);
    }

    movement_information.move_nodes = movement_calculator.move_nodes;
}

pub fn calculate_move_node(
    tile_moving_from: MoveNode,
    tile_moving_to: &TilePos,
    movement_calculator: &mut MovementNodes,
    tile_movement_costs: &TileMovementCosts,
    object_movement: &ObjectMovement,
) -> bool {
    let mut node = movement_calculator.get_node(tile_moving_to, tile_moving_from);
    if node.move_cost.is_some() {
        if (tile_moving_from.move_cost.unwrap()
            + *tile_movement_costs
                .movement_type_cost
                .get(object_movement.movement_type)
                .unwrap_or_else(|| &1) as i32)
            < (node.move_cost.unwrap())
        {
            node.move_cost = Some(
                tile_moving_from.move_cost.unwrap()
                    + *tile_movement_costs
                        .movement_type_cost
                        .get(object_movement.movement_type)
                        .unwrap_or_else(|| &1) as i32,
            );
            node.prior_node = tile_moving_from.node_pos;
            return true;
        }
    } else {
        if (tile_moving_from.move_cost.unwrap()
            + *tile_movement_costs
                .movement_type_cost
                .get(object_movement.movement_type)
                .unwrap_or_else(|| &1) as i32)
            <= object_movement.move_points
        {
            node.move_cost = Some(
                tile_moving_from.move_cost.unwrap()
                    + *tile_movement_costs
                        .movement_type_cost
                        .get(object_movement.movement_type)
                        .unwrap_or_else(|| &1) as i32,
            );
            node.prior_node = tile_moving_from.node_pos;
            return true;
        }
    }

    return false;
}

 */

// we have the tile that got the neighbor, the tile we are checking, and that tile has
// its cost as well as the current lowest cost tile that reached it

// We want to send the tile that got the current tile, the current tile, and whatever it
// needs to a function and return a bool telling us if we can move there. If we can then
// we want to add this new tile to the list of available moves and then add it to the list of
// nodes to evaluate. And then we remove the current node after we finished all the neighbors

// the tile has space. so now we need to decide can we even move to this tile
// basically we take the move cost from the prior_move_node, add the move cost from
// the new tile, and see if we have enough movement to make it into it

fn handle_try_move_events(
    mut move_events: ParamSet<(EventReader<MoveEvent>, EventWriter<MoveEvent>)>,
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
    mut movement_information: ResMut<CurrentMovementInformation>,
    mut move_error_writer: EventWriter<MoveError>,
) {
    let mut result: Result<MoveEvent, MoveError> =
        Err(MoveError::NotValidMove(String::from("Try move failed")));

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
            _ => {}
        }
    }
    match result {
        Ok(move_event) => {
            movement_information.available_moves.clear();
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
                object_grid_position.tile_position,
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
            return Err(MoveError::NotValidMove(String::from(
                "Tile does not have needed components",
            )));
        }
    } else {
        return Err(MoveError::NotValidMove(String::from(
            "Move Position not valid",
        )));
    }
}

pub fn move_complete(object_moving: Entity) {}

pub fn check_move(
    new_pos: &TilePos,
    movement_information: &mut ResMut<CurrentMovementInformation>,
) -> bool {
    return if movement_information.available_moves.contains(new_pos) {
        true
    } else {
        false
    };
}
/*
pub fn get_neighbors_tile_pos(
    origin_tile: TilePos,
    tilemap_size: &TilemapSize,
    movement_information: &mut ResMut<CurrentMovementInformation>,
) -> Vec<MoveNode> {
    let mut neighbor_tiles: Vec<MoveNode> = vec![];

    if let Some(north) = TilePos::from_i32_pair(
        origin_tile.x as i32,
        origin_tile.y as i32 + 1,
        &tilemap_size,
    ) {
        neighbor_tiles.push(MoveNode {
            node_pos: north,
            prior_node: origin_tile,
            move_cost: None,
        });
    }
    if let Some(east) = TilePos::from_i32_pair(
        origin_tile.x as i32 + 1,
        origin_tile.y as i32,
        &tilemap_size,
    ) {
        neighbor_tiles.push(MoveNode {
            node_pos: east,
            prior_node: origin_tile,
            move_cost: None,
        });
    }
    if let Some(south) = TilePos::from_i32_pair(
        origin_tile.x as i32,
        origin_tile.y as i32 - 1,
        &tilemap_size,
    ) {
        neighbor_tiles.push(MoveNode {
            node_pos: south,
            prior_node: origin_tile,
            move_cost: None,
        });
    }
    if let Some(west) = TilePos::from_i32_pair(
        origin_tile.x as i32 - 1,
        origin_tile.y as i32,
        &tilemap_size,
    ) {
        neighbor_tiles.push(MoveNode {
            node_pos: west,
            prior_node: origin_tile,
            move_cost: None,
        });
    }

    if movement_information.diagonal_movement == DiagonalMovement::Enabled {
        if let Some(northwest) = TilePos::from_i32_pair(
            origin_tile.x as i32 - 1,
            origin_tile.y as i32 + 1,
            &tilemap_size,
        ) {
            neighbor_tiles.push(MoveNode {
                node_pos: northwest,
                prior_node: origin_tile,
                move_cost: None,
            });
        }
        if let Some(northeast) = TilePos::from_i32_pair(
            origin_tile.x as i32 + 1,
            origin_tile.y as i32 + 1,
            &tilemap_size,
        ) {
            neighbor_tiles.push(MoveNode {
                node_pos: northeast,
                prior_node: origin_tile,
                move_cost: None,
            });
        }
        if let Some(southeast) = TilePos::from_i32_pair(
            origin_tile.x as i32 + 1,
            origin_tile.y as i32 - 1,
            &tilemap_size,
        ) {
            neighbor_tiles.push(MoveNode {
                node_pos: southeast,
                prior_node: origin_tile,
                move_cost: None,
            });
        }
        if let Some(southwest) = TilePos::from_i32_pair(
            origin_tile.x as i32 - 1,
            origin_tile.y as i32 - 1,
            &tilemap_size,
        ) {
            neighbor_tiles.push(MoveNode {
                node_pos: southwest,
                prior_node: origin_tile,
                move_cost: None,
            });
        }
    }
    neighbor_tiles
}

 */

/// Struct used to define a new [`MovementType`]
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct MovementType {
    pub name: &'static str,
}

/// Component that must be added to a tile in order to define that tiles movement cost.
///
/// Contains a hashmap that holds a reference to a [`MovementType`] as a key and a u32 as the value. The u32 is used
/// in pathfinding as the cost to move into that tile.
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub struct TileMovementCosts {
    pub movement_type_cost: HashMap<&'static MovementType, u32>,
}

impl TileMovementCosts {
    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn new(rules: Vec<(&'static MovementType, u32)>) -> TileMovementCosts {
        let mut hashmap: HashMap<&'static MovementType, u32> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0, rule.1);
        }
        TileMovementCosts {
            movement_type_cost: hashmap,
        }
    }
    pub fn calculate_unit_move_cost(&self) {}
}

/// Defines a resource that will hold all [`TileMovementCosts`] - references to a specific TileMovementCosts
/// are stored in each tile as their current cost.
#[derive(Resource, Default, Debug)]
pub struct TileMovementRules {
    pub movement_cost_rules: HashMap<TerrainType, TileMovementCosts>,
}

//UNIT MOVEMENT STUFF

/// Basic Bundle that supplies all needed movement components for a unit
#[derive(Bundle)]
pub struct UnitMovementBundle {
    pub object_movement: ObjectMovement,
}

#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub struct ObjectMovement {
    pub move_points: i32,
    pub movement_type: &'static MovementType,
    pub object_terrain_movement_rules: ObjectTerrainMovementRules,
}

/// Defines what type of terrain an object can move onto.
///
/// The rules are evaluated in a two step process. terrain_type_rules first, and then terrain_class_rules second
///
/// - terrain_type_rules should be considered an exception to terrain_class_rules and only used if you want to
/// allow or deny specific [`TerrainType`]s. Whatever bool you set that specific [`TerrainType`] controls
/// whether that tile is a valid move tile or not. Rules in this will be followed over any TerrainClass
/// rules.
/// - terrain_class_rules should be the first option used when assigning what terrain an object can
/// move on and only using terrain_type_rules if you need to make an exception. Every [`TerrainClass`]
/// added to terrain_class_rules denotes that the object can move onto any TerrainTypes that has a reference
/// to that TerrainClass.
///
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ObjectTerrainMovementRules {
    pub terrain_class_rules: Vec<&'static TerrainClass>,
    pub terrain_type_rules: HashMap<&'static TerrainType, bool>,
}

impl ObjectTerrainMovementRules {
    /// Returns true if the object can move onto that tiles terrain. Returns false if it cannot
    ///
    /// # Logic
    /// It checks self.terrain_type_rules for a rule for the tiles [`TerrainType`]. If it finds a rule
    /// it returns that directly. If it doesn't find a rule it checks if self.terrain_class_rules
    /// contains a reference to the tiles [`TerrainClass`]. If it does then it returns true. Else
    /// it returns false.
    pub fn can_move_on_tile(&self, tile_terrain_info: &TileTerrainInfo) -> bool {
        return if let Some(terrain_type_rule) =
            self.terrain_type_rules.get(&tile_terrain_info.terrain_type)
        {
            *terrain_type_rule
        } else {
            if self
                .terrain_class_rules
                .contains(&tile_terrain_info.terrain_type.terrain_class)
            {
                true
            } else {
                false
            }
        };
    }

    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn new_terrain_type(
        rules: Vec<(&'static TerrainType, bool)>,
    ) -> HashMap<&'static TerrainType, bool> {
        let mut hashmap: HashMap<&'static TerrainType, bool> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0, rule.1);
        }

        hashmap
    }
}

/// Marker component signifying that the unit has moved and cannot move anymore
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct ObjectMoved;
