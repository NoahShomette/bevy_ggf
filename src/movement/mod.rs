use crate::mapping::terrain::{TerrainClass, TerrainType, TileTerrainInfo};
use crate::mapping::tiles::{ObjectStackingClass, Tile, TileObjectStacks, TileObjects};
use crate::mapping::{
    add_object_to_tile, remove_object_from_tile, tile_pos_to_centered_map_world_pos, Map,
};
use crate::object::{Object, ObjectGridPosition};
use bevy::app::{App, CoreStage};
use bevy::log::info;
use bevy::prelude::{
    Bundle, Component, Entity, EventReader, EventWriter, IntoSystemDescriptor, ParamSet, Plugin,
    Query, ResMut, Resource, RunCriteriaDescriptorCoercion, SystemStage, Transform, With, Without,
    World,
};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{
    SquarePos, TilePos, TileStorage, TilemapGridSize, TilemapSize, TilemapType,
};
use std::borrow::BorrowMut;

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
pub enum DiagonalMovement {
    Enabled,
    #[default]
    Disabled,
}

#[derive(Clone, Eq, PartialEq, Default, Resource)]
pub struct MovementInformation {
    pub available_moves: Vec<TilePos>,
    pub move_nodes: HashMap<TilePos, MoveNode>,
    pub diagonal_movement: DiagonalMovement,
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
    mut move_events: ParamSet<(EventReader<MoveEvent>, EventWriter<MoveEvent>)>,
    mut object_query: Query<
        (&ObjectGridPosition, &ObjectStackingClass, &ObjectMovement),
        With<Object>,
    >,
    mut tile_query: Query<(&TileObjectStacks, &TileTerrainInfo, &TileMovementCosts)>,
    mut tilemap_q: Query<(&mut Map, &mut TileStorage, &TilemapSize), Without<Object>>,
    mut movement_information: ResMut<MovementInformation>,
    mut move_error_writer: EventWriter<MoveError>,
) {
    for event in move_events.p0().iter() {
        match event {
            MoveEvent::MoveBegin { object_moving } => {
                calculate_move(
                    object_moving,
                    &object_query,
                    &tile_query,
                    &mut tilemap_q,
                    &mut movement_information,
                );
            }
            _ => {}
        }
    }
}

pub struct MovementCalculator {
    pub move_nodes: HashMap<TilePos, MoveNode>,
}

impl MovementCalculator {
    pub fn get_node(&mut self, tile_pos: &TilePos, prior_node: MoveNode) -> &mut MoveNode {
        // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
        if self.move_nodes.contains_key(&tile_pos) {
            return self.move_nodes.get_mut(&tile_pos).unwrap();
        } else {
            let node = MoveNode {
                node_pos: *tile_pos,
                prior_node: prior_node.node_pos,
                move_cost: None,
            };
            self.move_nodes.insert(*tile_pos, node);
            self.move_nodes.get_mut(&tile_pos).unwrap()
        }
    }

    pub fn get_node_neighbors(
        &self,
        node_to_get_neighbors: TilePos,
        movement_information: &mut ResMut<MovementInformation>,
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

        if movement_information.diagonal_movement == DiagonalMovement::Enabled {
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
pub fn calculate_move(
    object_moving: &Entity,
    object_query: &Query<
        (&ObjectGridPosition, &ObjectStackingClass, &ObjectMovement),
        With<Object>,
    >,
    mut tile_query: &Query<(&TileObjectStacks, &TileTerrainInfo, &TileMovementCosts)>,
    tilemap_q: &mut Query<(&mut Map, &mut TileStorage, &TilemapSize), Without<Object>>,
    movement_information: &mut ResMut<MovementInformation>,
) {
    // Get the moving objects stuff
    let (object_grid_position, object_stack_class, object_movement) =
        object_query.get(*object_moving).unwrap();

    // gets the map components
    let (map, tile_storage, tilemap_size) = tilemap_q.single_mut();

    let mut movement_calculator = MovementCalculator {
        move_nodes: HashMap::new(),
    };
    movement_calculator.move_nodes.insert(
        object_grid_position.grid_position,
        MoveNode {
            node_pos: object_grid_position.grid_position,
            prior_node: object_grid_position.grid_position,
            move_cost: Some(0),
        },
    );
    // unvisited nodes
    let mut unvisited_nodes: Vec<MoveNode> = vec![MoveNode {
        node_pos: object_grid_position.grid_position,
        prior_node: object_grid_position.grid_position,
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
    movement_calculator: &mut MovementCalculator,
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
    mut movement_information: ResMut<MovementInformation>,
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
    movement_information: &mut ResMut<MovementInformation>,
) -> bool {
    return if movement_information.available_moves.contains(new_pos) {
        true
    } else {
        false
    };
}

pub fn get_neighbors_tile_pos(
    origin_tile: TilePos,
    tilemap_size: &TilemapSize,
    movement_information: &mut ResMut<MovementInformation>,
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
    pub fn new(
        rules: Vec<(&'static MovementType, u32)>,
    ) -> TileMovementCosts {
        let mut hashmap: HashMap<&'static MovementType, u32> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0, rule.1);
        }
        TileMovementCosts{
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
