use crate::mapping::terrain::{TerrainClass, TerrainType, TileTerrainInfo};
use crate::mapping::tiles::{ObjectStackingClass, TileObjectStacks, TileObjects};
use crate::mapping::{
    add_object_to_tile, remove_object_from_tile, tile_pos_to_centered_map_world_pos, Map,
    MapHandler,
};
use crate::object::{Object, ObjectGridPosition};
use bevy::app::{App, CoreStage};
use bevy::ecs::system::SystemState;
use bevy::log::info;
use bevy::math::IVec2;
use bevy::prelude::{
    Bundle, Component, Entity, EventReader, EventWriter, IntoSystemDescriptor, Mut, ParamSet,
    Plugin, Query, Res, ResMut, Resource, Transform, With, Without, World,
};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapGridSize, TilemapSize, TilemapType};

/// Core plugin for the bevy_ggf Movement System. Contains basic needed functionality.
/// Does not contain a MovementSystem. You have to insert that yourself
///
pub struct BggfMovementPlugin;

impl Plugin for BggfMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMovementRules>()
            .init_resource::<CurrentMovementInformation>()
            .add_event::<MoveEvent>()
            .add_event::<MoveError>()
            .add_system_to_stage(CoreStage::PostUpdate, handle_move_begin_events.at_end())
            .add_system(handle_try_move_events);
    }
}

/// A trait defining a new MovementCalculator - define the [`calculate_move`] fn in order to control
/// exactly how the movement works. Add this to a [`MovementSystem`] and insert that as a resource
/// to define your movement system
///
/// Bevy_GGF contains a MovementCalculator for Square based maps called [`SquareMovementSystem`]
pub trait MovementCalculator: 'static + Send + Sync {
    /// The main function of a [`MovementCalculator`]. This is called when a [`MoveEvent`] is received
    /// and the result is pushed into the [`CurrentMovementInformation`] Resource automatically. Use
    /// this function to define your own movement algorithm.
    fn calculate_move(
        &self,
        movement_system: &Res<MovementSystem>,
        object_moving: &Entity,
        world: &World,
    ) -> (Vec<TilePos>, MovementNodes);
}

/// Defines a MovementSystem. This resource is used to calculate movement, define the list of checks
/// for the [`MovementCalculator`], and holds the [`TilemapType`]
#[derive(Resource)]
pub struct MovementSystem {
    pub movement_calculator: Box<dyn MovementCalculator>,
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

/// Built in struct with an implementation for a [`MovementCalculator`] for a simple square based map.
/// The pathfinding algorithm is an implementation of Djikstras.
/// Contains a field for a [`DiagonalMovement`] enum. The pathfinding algorithm will include diagonal
/// tiles based on this enum.
#[derive(Clone)]
pub struct SquareMovementSystem {
    pub diagonal_movement: DiagonalMovement,
}

impl MovementCalculator for SquareMovementSystem {
    fn calculate_move(
        &self,
        movement_system: &Res<MovementSystem>,
        object_moving: &Entity,
        world: &World,
    ) -> (Vec<TilePos>, MovementNodes) {
        let Some(object_grid_position) = world.get::<ObjectGridPosition>(*object_moving) else {
            return (vec![], MovementNodes {
                move_nodes: HashMap::new(),
            });
        };

        let Some(map_handler) = world.get_resource::<MapHandler>() else {
            return (vec![], MovementNodes {
                move_nodes: HashMap::new(),
            });        };

        let Some(tile_storage) = world.get::<TileStorage>(map_handler.get_map_entity(IVec2{x: 0, y: 0}).unwrap()) else {
            return (vec![], MovementNodes {
                move_nodes: HashMap::new(),
            });        };
        let Some(tilemap_size) = world.get::<TilemapSize>(map_handler.get_map_entity(IVec2{x: 0, y: 0}).unwrap()) else {
            return (vec![], MovementNodes {
                move_nodes: HashMap::new(),
            });        };

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

        while !unvisited_nodes.is_empty() {
            unvisited_nodes.sort_by(|x, y| {
                x.move_cost
                    .unwrap()
                    .partial_cmp(&y.move_cost.unwrap())
                    .unwrap()
            });

            let Some(current_node) = unvisited_nodes.get(0) else {
                continue;
            };

            let neighbors = move_info.get_neighbors_tilepos(
                current_node.node_pos,
                self.diagonal_movement.is_diagonal(),
                tilemap_size,
            );

            let current_node = *current_node;

            'neighbors: for neighbor in neighbors.iter() {
                if visited_nodes.contains(neighbor) {
                    continue;
                }
                let Some(tile_entity) = tile_storage.get(neighbor) else {
                    continue;

                };

                move_info.add_node(neighbor, current_node);
                if tile_movement_cost_check(
                    *object_moving,
                    tile_entity,
                    neighbor,
                    &current_node.node_pos,
                    &mut move_info,
                    world,
                ) {
                } else {
                    continue 'neighbors;
                }
                // checks the tile against each of the move rules added if its false kill this loop
                for i in 0..movement_system.tile_move_checks.len() {
                    let check = movement_system.tile_move_checks[i].as_ref();
                    if check.is_valid_move(
                        *object_moving,
                        tile_entity,
                        neighbor,
                        &current_node.node_pos,
                        world,
                    ) {
                    } else {
                        continue 'neighbors;
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
        //move_info.move_nodes = move_info.move_nodes;
        (available_moves, move_info)
    }
}

/// A trait used to define a new check for a tile in a [`MovementCalculator`]s pathfinding algorithm.
/// Implement one of these traits for each separate logical check you want the MovementCalculator to
/// do to determine if a tile is a valid move or not.
///
/// # Example
/// Here is an example of a simple TileMoveCheck implementation. This impl provides a check for whether
/// or not a tile has space in the tile for the relevant objects stacking class
/// ```rust
/// use bevy::prelude::{Entity, World};
/// use bevy_ecs_tilemap::prelude::TilePos;
/// use bevy_ggf::mapping::tiles::{ObjectStackingClass, TileObjectStacks};
/// use bevy_ggf::movement::TileMoveCheck;
///
/// // Create a new struct for our TileMoveCheck
/// pub struct MoveCheckSpace;
///
/// // Implement the TileMoveCheck Trait for our struct
/// impl TileMoveCheck for MoveCheckSpace {
///     fn is_valid_move(
///         &self,
///         entity_moving: Entity,
///         tile_entity: Entity,
///         tile_pos: &TilePos,
///         last_tile_pos: &TilePos,
///         world: &World,
///     ) -> bool {
/// // Get the ObjectStackingClass component of our object that is trying to move
///         let Some(object_stack_class) = world.get::<ObjectStackingClass>(entity_moving) else {
/// // If the object doesnt have a stack class then we want to return false as this object should not be able to move
///             return false;
///         };
/// // Get the TileObjectStacks component of the tile that we are checking
///         let Some(tile_objects) = world.get::<TileObjectStacks>(tile_entity) else {
///             return false;
///         };
/// // Use the built in function on a TileObjectStacks struct to check if the tile has space for this objects stacking class
///         if tile_objects.has_space(object_stack_class) == true {
/// // If there is space then this object can move into the tile and we return true
///             return true;
///         }
/// // Else there is no space and we return false instead
///         return false;
///     }
/// }
/// ```
pub trait TileMoveCheck {
    fn is_valid_move(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        tile_pos: &TilePos,
        last_tile_pos: &TilePos,
        world: &World,
    ) -> bool;
}

/// Provided function that can be used in a [`MovementCalculator`] to keep track of the nodes in a pathfinding node,
/// their associated movement costs, and which is the node that has the shortest path to that specific
/// node. Will automatically compute all of the above.
pub fn tile_movement_cost_check(
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

    return if tile_node.move_cost.is_some() {
        if (move_from_tile_node.move_cost.unwrap()
            + *tile_movement_costs
                .movement_type_cost
                .get(object_movement.movement_type)
                .unwrap_or(&1) as i32)
            < (tile_node.move_cost.unwrap())
        {
            tile_node.move_cost = Some(
                move_from_tile_node.move_cost.unwrap()
                    + *tile_movement_costs
                        .movement_type_cost
                        .get(object_movement.movement_type)
                        .unwrap_or(&1) as i32,
            );
            tile_node.prior_node = move_from_tile_node.node_pos;
            true
        } else {
            false
        }
    } else if (move_from_tile_node.move_cost.unwrap()
        + *tile_movement_costs
            .movement_type_cost
            .get(object_movement.movement_type)
            .unwrap_or(&1) as i32)
        <= object_movement.move_points
    {
        tile_node.move_cost = Some(
            move_from_tile_node.move_cost.unwrap()
                + *tile_movement_costs
                    .movement_type_cost
                    .get(object_movement.movement_type)
                    .unwrap_or(&1) as i32,
        );
        tile_node.prior_node = move_from_tile_node.node_pos;
        true
    } else {
        false
    };
}

/// Simple enum to represent whether Diagonal Movement is enabled or disabled
#[derive(Clone, Eq, Hash, PartialEq, Default)]
pub enum DiagonalMovement {
    Enabled,
    #[default]
    Disabled,
}

impl DiagonalMovement {
    /// Returns true or false depending on whether Diagonal Movement is enabled or disabled
    pub fn is_diagonal(&self) -> bool {
        match self {
            DiagonalMovement::Enabled => true,
            DiagonalMovement::Disabled => false,
        }
    }
}

/// Resource that holds the TilePos of any available moves and the move nodes of whatever the [`calculate_move`]
/// function created
#[derive(Clone, Eq, PartialEq, Default, Resource)]
pub struct CurrentMovementInformation {
    pub available_moves: Vec<TilePos>,
    pub move_nodes: HashMap<TilePos, MoveNode>,
}

impl CurrentMovementInformation {
    /// Returns true or false if CurrentMovementInformation contains a move at the assigned TilePos
    pub fn contains_move(&self, new_pos: &TilePos) -> bool {
        self.available_moves.contains(new_pos)
    }

    pub fn clear_information(&mut self) {
        self.available_moves.clear();
        self.move_nodes.clear();
    }
}

/// Struct used in a [`MovementCalculator`] to hold the list of [`MoveNode`]
pub struct MovementNodes {
    pub move_nodes: HashMap<TilePos, MoveNode>,
}

impl MovementNodes {
    /// Adds a Node to the MovementNodes Hashmap. If the Hashmap already contains a node for the designated
    /// TilePos then it does nothing.
    ///
    /// The instantiated node contains a None value for the move_cost.
    pub fn add_node(&mut self, tile_pos: &TilePos, prior_node: MoveNode) {
        // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
        if self.move_nodes.contains_key(tile_pos) {
        } else {
            let node = MoveNode {
                node_pos: *tile_pos,
                prior_node: prior_node.node_pos,
                move_cost: None,
            };
            self.move_nodes.insert(*tile_pos, node);
        }
    }

    /// Gets the node at the specified TilePos and returns a mutable reference to it
    pub fn get_node_mut(&mut self, tile_pos: &TilePos) -> Option<&mut MoveNode> {
        // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
        self.move_nodes.get_mut(tile_pos)
    }

    /// Returns a mutable reference for both nodes specified and returns them in the same order
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

    /// Returns the TilePos for all the nodes neighbors. Will correctly work on edges where a TilePos
    /// is not valid. Will return diagonal nodes based on the diagonal_movement bool.
    pub fn get_neighbors_tilepos(
        &self,
        node_to_get_neighbors: TilePos,
        diagonal_movement: bool,
        tilemap_size: &TilemapSize,
    ) -> Vec<TilePos> {
        let mut neighbor_tiles: Vec<TilePos> = vec![];
        let origin_tile = node_to_get_neighbors;
        if let Some(north) =
            TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 + 1, tilemap_size)
        {
            neighbor_tiles.push(north);
        }
        if let Some(east) =
            TilePos::from_i32_pair(origin_tile.x as i32 + 1, origin_tile.y as i32, tilemap_size)
        {
            neighbor_tiles.push(east);
        }
        if let Some(south) =
            TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 - 1, tilemap_size)
        {
            neighbor_tiles.push(south);
        }
        if let Some(west) =
            TilePos::from_i32_pair(origin_tile.x as i32 - 1, origin_tile.y as i32, tilemap_size)
        {
            neighbor_tiles.push(west);
        }

        if diagonal_movement {
            if let Some(northwest) = TilePos::from_i32_pair(
                origin_tile.x as i32 - 1,
                origin_tile.y as i32 + 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(northwest);
            }
            if let Some(northeast) = TilePos::from_i32_pair(
                origin_tile.x as i32 + 1,
                origin_tile.y as i32 + 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(northeast);
            }
            if let Some(southeast) = TilePos::from_i32_pair(
                origin_tile.x as i32 + 1,
                origin_tile.y as i32 - 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(southeast);
            }
            if let Some(southwest) = TilePos::from_i32_pair(
                origin_tile.x as i32 - 1,
                origin_tile.y as i32 - 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(southwest);
            }
        }
        neighbor_tiles
    }
}

/// Represents a tile in a MovementNodes struct. Used to hold information relevant to movement calculation
#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub struct MoveNode {
    pub node_pos: TilePos,
    pub prior_node: TilePos,
    pub move_cost: Option<i32>,
}

/// An error that represents any MoveErrors
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum MoveError {
    NotValidMove(String),
}

impl Default for MoveError {
    fn default() -> Self {
        MoveError::NotValidMove(String::from("Invalid Move"))
    }
}

/// A move event. Used to conduct actions related to object movement
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

/// Handles all MoveBegin events. Uses the MovementSystem resource to calculate the move and update
/// the CurrentMoveInformation resource
fn handle_move_begin_events(world: &mut World) {
    let mut move_events_vec: Vec<MoveEvent> = vec![];

    // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
    // as if you were writing an ordinary system.
    let mut system_state: SystemState<EventReader<MoveEvent>> = SystemState::new(world);

    // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
    // system_state.get(&world) provides read-only versions of your system parameters instead.
    let mut move_events = system_state.get_mut(world);

    for event in move_events.iter() {
        if let MoveEvent::MoveBegin { object_moving } = event {
            move_events_vec.push(MoveEvent::MoveBegin {
                object_moving: *object_moving,
            });
        }
    }

    // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
    // as if you were writing an ordinary system.
    let mut system_state: SystemState<Res<MovementSystem>> = SystemState::new(world);

    // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
    // system_state.get(&world) provides read-only versions of your system parameters instead.
    let movement_system = system_state.get(world);

    let mut move_info: (Vec<TilePos>, MovementNodes) = (
        vec![],
        MovementNodes {
            move_nodes: HashMap::new(),
        },
    );

    for event in move_events_vec {
        if let MoveEvent::MoveBegin { object_moving } = event {
            move_info = movement_system.movement_calculator.calculate_move(
                &movement_system,
                &object_moving,
                world,
            );
        }
    }

    if !move_info.0.is_empty() {
        world.resource_scope(|_world, mut a: Mut<CurrentMovementInformation>| {
            a.available_moves = move_info.0;
            a.move_nodes = move_info.1.move_nodes;
        });
    }
}

/// Handles the TryMove events. Will check if the given TilePos is inside the CurrentMovementInformation
/// resource and will move the unit if so.
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
        if let MoveEvent::TryMoveObject {
            object_moving,
            new_pos,
        } = event
        {
            if movement_information.contains_move(new_pos) {
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
//TODO update this to use the MapHandler resource
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
    tile_query: &mut Query<(&mut TileObjectStacks, &mut TileObjects)>,
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
    let (_map, grid_size, map_type, mut tile_storage, map_transform) = tilemap_q.single_mut();

    // if a tile exists at the selected point
    return if let Some(tile_entity) = tile_storage.get(new_pos) {
        // if the tile has the needed components
        if let Ok((_tile_stack_rules, _tile_objects)) = tile_query.get(tile_entity) {
            remove_object_from_tile(
                *object_moving,
                object_stack_class,
                &mut tile_storage,
                tile_query,
                object_grid_position.tile_position,
            );
            add_object_to_tile(
                *object_moving,
                &mut object_grid_position,
                object_stack_class,
                &mut tile_storage,
                tile_query,
                *new_pos,
            );

            // have to transform the tiles position to the transformed position to place the object at the right point
            let tile_world_pos =
                tile_pos_to_centered_map_world_pos(new_pos, map_transform, grid_size, map_type);

            transform.translation = tile_world_pos.extend(5.0);

            Ok(MoveEvent::MoveComplete {
                object_moved: *object_moving,
            })
        } else {
            Err(MoveError::NotValidMove(String::from(
                "Tile does not have needed components",
            )))
        }
    } else {
        Err(MoveError::NotValidMove(String::from(
            "Move Position not valid",
        )))
    };
}

pub fn move_complete(object_moving: Entity) {}

/// implements TileMoveCheck. Provides a check for whether a tile has space for the object thats moving's
/// object stacking class
pub struct MoveCheckSpace;

impl TileMoveCheck for MoveCheckSpace {
    fn is_valid_move(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        _tile_pos: &TilePos,
        _last_tile_pos: &TilePos,
        world: &World,
    ) -> bool {
        let Some(object_stack_class) = world.get::<ObjectStackingClass>(entity_moving) else {
            return false;
        };
        let Some(tile_objects) = world.get::<TileObjectStacks>(tile_entity) else {
            return false;
        };

        tile_objects.has_space(object_stack_class)
    }
}

/// implements TileMoveCheck. Provides a check for whether an object is able to move in the given tiles
/// terrain or not
pub struct MoveCheckTerrain;

impl TileMoveCheck for MoveCheckTerrain {
    fn is_valid_move(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        _tile_pos: &TilePos,
        _last_tile_pos: &TilePos,
        world: &World,
    ) -> bool {
        let Some(object_movement) = world.get::<ObjectMovement>(entity_moving) else {
            return false;
        };
        let Some(tile_terrain_info) = world.get::<TileTerrainInfo>(tile_entity) else {
            return false;
        };

        object_movement
            .object_terrain_movement_rules
            .can_move_on_tile(tile_terrain_info)
    }
}

/// Struct used to define a new [`MovementType`]. MovementType represents how a unit moves and is used
/// for movement costs chiefly
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

// UNIT MOVEMENT STUFF

/// Basic Bundle that supplies all needed movement components for a unit
#[derive(Bundle)]
pub struct UnitMovementBundle {
    pub object_movement: ObjectMovement,
}

/// Required for an Object to move. Without this an object is unable to move.
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
/// allow or deny specific [`TerrainType`]s. Whatever bool you set for that specific [`TerrainType`] controls
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
        if let Some(terrain_type_rule) =
            self.terrain_type_rules.get(&tile_terrain_info.terrain_type)
        {
            return *terrain_type_rule;
        }

        self.terrain_class_rules
            .contains(&tile_terrain_info.terrain_type.terrain_class)
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
