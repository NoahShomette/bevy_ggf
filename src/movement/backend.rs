use crate::movement::{
    AvailableMove, CurrentMovementInformation, MoveEvent, MovementSystem, ObjectMoved,
    ObjectMovement, TileMovementCosts,
};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Commands, Entity, EventReader, Mut, Query, Res, World};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize};
use crate::object::ObjectId;

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

    let Some((tile_node, move_from_tile_node)) = movement_nodes.get_two_node_mut(tile_pos, move_from_tile_pos) else {
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

/// Struct to be used in a [`MovementCalculator`](crate::movement::MovementCalculator) to hold the
/// list of [`MoveNode`]s. This is not to be used for any other purpose than calculating movement
/// and will be converted into an [`AvailableMove`] struct to be used outside the movement calculater
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
                valid_move: false,
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

    pub fn set_valid_move(&mut self, node_pos_to_update: &TilePos) -> Result<(), String> {
        return if let Some(node) = self.get_node_mut(node_pos_to_update) {
            node.valid_move = true;
            Ok(())
        } else {
            Err(String::from("Error getting node"))
        };
    }
}

/// Represents a tile in a MovementNodes struct. Used to hold information relevant to movement calculation
#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub struct MoveNode {
    pub node_pos: TilePos,
    pub prior_node: TilePos,
    pub move_cost: Option<i32>,
    pub valid_move: bool,
}

impl MoveNode {
    pub fn set_cost(&mut self, new_cost: i32) {
        self.move_cost = Some(new_cost);
    }
}

// main events
// MoveBegin
// MoveCalculated (Vec<TilePos>)
// MoveObject
// MoveComplete

/// Handles all MoveBegin events. Uses the MovementSystem resource to calculate the move and update
/// the CurrentMoveInformation resource
pub(crate) fn handle_move_begin_events(mut world: &mut World) {
    let mut move_events_vec: Vec<MoveEvent> = vec![];
    let mut system_state: SystemState<EventReader<MoveEvent>> = SystemState::new(world);
    let mut move_events = system_state.get_mut(world);

    for event in move_events.iter() {
        if let MoveEvent::MoveBegin {
            object_moving,
            on_map,
        } = event
        {
            move_events_vec.push(MoveEvent::MoveBegin {
                object_moving: *object_moving,
                on_map: *on_map,
            });
        }
    }
    let mut moves: HashMap<Entity, MovementNodes> = HashMap::new();

    world.resource_scope(|world, movement_system: Mut<MovementSystem>| {
        for event in move_events_vec {
            if let MoveEvent::MoveBegin {
                object_moving,
                on_map,
            } = event
            {
                let mut system_state: SystemState<Query<(Entity, &ObjectId)>> =
                    SystemState::new(world);
                let mut object_query = system_state.get_mut(world);
                let Some((entity, _)) = object_query
                    .iter_mut()
                    .find(|(_, id)| id == &&object_moving) else {
                    continue;
                };

                let move_info = movement_system.movement_calculator.calculate_move(
                    &movement_system.tile_move_checks,
                    movement_system.map_type,
                    on_map,
                    entity,
                    world,
                );

                moves.insert(entity, move_info);
            }
        }
    });

    let mut system_state: SystemState<Commands> = SystemState::new(world);
    let mut commands = system_state.get_mut(world);

    for (entity, move_nodes) in moves.iter() {
        if !move_nodes.move_nodes.is_empty() {
            let mut moves: HashMap<TilePos, AvailableMove> = HashMap::new();

            for (tile_pos, move_node) in move_nodes.move_nodes.iter() {
                if move_node.valid_move {
                    moves.insert(*tile_pos, AvailableMove::from(*move_node));
                }
            }

            commands.entity(*entity).insert(CurrentMovementInformation {
                available_moves: moves,
            });
        }
    }

    system_state.apply(world);
}

/// Adds the [`ObjectMoved`] component to any entity that is sent through the [`MoveEvent::MoveComplete`]
/// event.
pub fn add_object_moved_component_on_moves(
    mut move_events: EventReader<MoveEvent>,
    mut object_query: Query<(Entity, &ObjectId)>,
    mut commands: Commands,
) {
    for event in move_events.iter() {
        if let MoveEvent::MoveComplete { object_moved } = event {
            let Some((entity, _)) = object_query.iter_mut().find(|(_, id)| id == &object_moved) else{
                continue;
            };
            commands.entity(entity).insert(ObjectMoved);
        }
    }
}
