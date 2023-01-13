use crate::mapping::tiles::{ObjectStackingClass, TileObjectStackingRules, TileObjects};
use crate::mapping::{
    add_object_to_tile, remove_object_from_tile, tile_pos_to_centered_map_world_pos, Map,
};
use crate::movement::{
    AvailableMove, CurrentMovementInformation, MoveError, MoveEvent, MovementSystem,
    ObjectMovement, TileMovementCosts,
};
use crate::object::{Object, ObjectGridPosition};
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    info, Entity, EventReader, EventWriter, Mut, ParamSet, Query, Res, ResMut, Transform, With,
    Without, World,
};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapGridSize, TilemapSize, TilemapType};

/// Provided function that can be used in a [`MovementCalculator`](crate::movement::MovementCalculator) to keep track of the nodes in a pathfinding node,
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

/// Struct used in a [`MovementCalculator`](crate::movement::MovementCalculator) to hold the list of [`MoveNode`]
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

//TODO refactor this to have a field declaring it a valid move. then use that to filter the movenodes
// into available moves
/// Represents a tile in a MovementNodes struct. Used to hold information relevant to movement calculation
#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub struct MoveNode {
    pub node_pos: TilePos,
    pub prior_node: TilePos,
    pub move_cost: Option<i32>,
}

// main events
// MoveBegin
// MoveCalculated (Vec<TilePos>)
// MoveObject
// MoveComplete

/// Handles all MoveBegin events. Uses the MovementSystem resource to calculate the move and update
/// the CurrentMoveInformation resource
pub(crate) fn handle_move_begin_events(world: &mut World) {
    let mut move_events_vec: Vec<MoveEvent> = vec![];

    let mut system_state: SystemState<EventReader<MoveEvent>> = SystemState::new(world);
    let mut move_events = system_state.get_mut(world);

    for event in move_events.iter() {
        if let MoveEvent::MoveBegin { object_moving } = event {
            move_events_vec.push(MoveEvent::MoveBegin {
                object_moving: *object_moving,
            });
        }
    }

    let mut system_state: SystemState<Res<MovementSystem>> = SystemState::new(world);
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
            for (tile_pos, move_node) in move_info.1.move_nodes.iter() {
                if move_info.0.contains(&move_node.node_pos) {
                    a.available_moves.insert(
                        *tile_pos,
                        AvailableMove {
                            tile_pos: move_node.node_pos,
                            prior_tile_pos: move_node.prior_node,
                            move_cost: move_node
                                .move_cost
                                .expect("All valid moves should have a move cost"),
                        },
                    );
                }
                // if the move node is the starting node we want to add it to the available moves.
                if move_node.move_cost == Some(0) {
                    a.available_moves.insert(
                        *tile_pos,
                        AvailableMove {
                            tile_pos: move_node.node_pos,
                            prior_tile_pos: move_node.node_pos,
                            move_cost: 0,
                        },
                    );
                }
            }
        });
    }
}

/// Handles the TryMove events. Will check if the given TilePos is inside the CurrentMovementInformation
/// resource and will move the unit if so.
pub(crate) fn handle_try_move_events(
    mut move_events: ParamSet<(EventReader<MoveEvent>, EventWriter<MoveEvent>)>,
    mut object_query: Query<
        (
            &mut Transform,
            &mut ObjectGridPosition,
            &ObjectStackingClass,
        ),
        With<Object>,
    >,
    mut tile_query: Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
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
    tile_query: &mut Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
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
