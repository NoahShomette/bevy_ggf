use crate::mapping::terrain::TileTerrainInfo;
use crate::mapping::tiles::{ObjectStackingClass, TileObjectStacks};
use crate::mapping::MapHandler;
use crate::movement::backend::{tile_movement_cost_check, MoveNode, MovementNodes};
use crate::movement::{
    DiagonalMovement, MovementCalculator, MovementSystem, ObjectMovement, TileMoveCheck,
};
use crate::object::ObjectGridPosition;
use bevy::prelude::{Entity, IVec2, Res, World};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapSize};

// BUILT IN IMPLEMENTATIONS

/// Built in struct with an implementation for a [`MovementCalculator`] for a simple square based map.
/// The pathfinding algorithm is an implementation of Djikstras.
/// Contains a field for a [`DiagonalMovement`] enum. The pathfinding algorithm will include diagonal
/// tiles based on this enum.
#[derive(Clone)]
pub struct SquareMovementCalculator {
    pub diagonal_movement: DiagonalMovement,
}

impl MovementCalculator for SquareMovementCalculator {
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
                    if !check.is_valid_move(
                        *object_moving,
                        tile_entity,
                        neighbor,
                        &current_node.node_pos,
                        world,
                    ) {
                        continue 'neighbors;
                    }
                }

                // if none of them return false and cancel the loop then we can infer that we are able to move into that neighbor
                // we add the neighbor to the list of unvisited nodes and then push the neighbor to the available moves list
                unvisited_nodes.push(*move_info.get_node_mut(neighbor).expect(
                    "Is safe because we know we add the node in at the beginning of this loop",
                )); //
                available_moves.push(*neighbor);
            }

            unvisited_nodes.remove(0);
            visited_nodes.push(current_node.node_pos);
        }
        (available_moves, move_info)
    }
}

/// implements TileMoveCheck. Provides a check for whether a tile has space for the object that's moving
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
