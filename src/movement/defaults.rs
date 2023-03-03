use crate::game::GameId;
use crate::mapping::terrain::TileTerrainInfo;
use crate::mapping::tiles::{ObjectStackingClass, TileObjectStackingRules, TileObjects};
use crate::movement::backend::{tile_movement_cost_check, MoveNode, MovementNodes};
use crate::movement::{
    DiagonalMovement, MovementCalculator, MovementSystem, ObjectMovement, ObjectTypeMovementRules,
    TileMoveCheck, TileMoveCheckMeta, TileMoveChecks,
};
use crate::object::{Object, ObjectGridPosition, ObjectInfo};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Entity, IVec2, Query, Res, Transform, With, Without, World};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapSize, TilemapType};
use crate::mapping::MapId;

// BUILT IN IMPLEMENTATIONS

/// Built in struct with an implementation for a [`MovementCalculator`](crate::movement::MovementCalculator) for a simple square based map.
/// The pathfinding algorithm is an implementation of Djikstras.
/// Contains a field for a [`DiagonalMovement`] enum. The pathfinding algorithm will include diagonal
/// tiles based on this enum.
#[derive(Clone)]
pub struct SquareMovementCalculator {
    pub diagonal_movement: DiagonalMovement,
}

#[rustfmt::skip] // rustfmt breaking ci
impl MovementCalculator for SquareMovementCalculator {
    fn calculate_move(
        &self,
        tile_move_checks: &TileMoveChecks,
        map_type: TilemapType,
        on_map: MapId,
        object_moving: Entity,
        world: &mut World,
    ) -> MovementNodes {

        let mut system_state: SystemState<
            (Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
            Query<&ObjectGridPosition>)
        > = SystemState::new(world);
        let (mut tile_storage_query, mut object_query) =
            system_state.get_mut(world);
        
        
        let Ok(object_grid_position) = object_query.get(object_moving) else{
            return MovementNodes {
                move_nodes: HashMap::new(),
            };
        };
        
        let Some((_, _, tile_storage, tilemap_size)) = tile_storage_query
            .iter_mut()
            .find(|(_, id, _, _)| id == &&on_map)else{
            return MovementNodes {
                move_nodes: HashMap::new(),
            };
        };
        
        let tile_storage = tile_storage.clone();
        let tilemap_size = tilemap_size.clone();

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
                valid_move: true,
            },
        );

        // unvisited nodes
        let mut unvisited_nodes: Vec<MoveNode> = vec![MoveNode {
            node_pos: object_grid_position.tile_position,
            prior_node: object_grid_position.tile_position,
            move_cost: Some(0),
            valid_move: false,
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

            let neighbor_pos = move_info.get_neighbors_tilepos(
                current_node.node_pos,
                self.diagonal_movement.is_diagonal(),
                &tilemap_size,
            );

            let current_node = *current_node;
            let mut neighbors: Vec<(TilePos, Entity)> = vec![];
            for neighbor in neighbor_pos.iter(){
                let Some(tile_entity) = tile_storage.get(neighbor) else {
                    continue;
                };
                neighbors.push((*neighbor, tile_entity));
            }


            'neighbors: for neighbor in neighbors.iter() {
                if visited_nodes.contains(&neighbor.0) {
                    continue;
                }
  

                move_info.add_node(&neighbor.0, current_node);

                if !tile_movement_cost_check(
                    object_moving,
                    neighbor.1,
                    &neighbor.0,
                    &current_node.node_pos,
                    &mut move_info,
                    world,
                ){
                    continue 'neighbors;
                }

                if !tile_move_checks.check_tile_move_checks(
                    object_moving,
                    neighbor.1,
                    &neighbor.0,
                    &current_node.node_pos,
                    world,
                ) {
                    continue 'neighbors;
                }


                let _ = move_info.set_valid_move(&neighbor.0);

                // if none of them return false and cancel the loop then we can infer that we are able to move into that neighbor
                // we add the neighbor to the list of unvisited nodes and then push the neighbor to the available moves list
                unvisited_nodes.push(*move_info.get_node_mut(&neighbor.0).expect(
                    "Is safe because we know we add the node in at the beginning of this loop",
                )); //
                available_moves.push(neighbor.0);
            }

            unvisited_nodes.remove(0);
            visited_nodes.push(current_node.node_pos);
        }
        move_info
    }
}

/// implements TileMoveCheck. Provides a check for whether a tile has space for the object that's moving
/// object stacking class
pub struct MoveCheckSpace;

impl TileMoveCheck for MoveCheckSpace {
    fn is_valid_move(
        &self,
        moving_entity: Entity,
        tile_entity: Entity,
        _checking_tile_pos: &TilePos,
        _move_from_tile_pos: &TilePos,
        world: &mut World,
    ) -> bool {
        let Some(object_stack_class) = world.get::<ObjectStackingClass>(moving_entity) else {
            return false;
        };
        let Some(tile_objects) = world.get::<TileObjectStackingRules>(tile_entity) else {
            return false;
        };

        tile_objects.has_space(object_stack_class)
    }
}

/// implements TileMoveCheck. Provides a check for whether an object is able to move in the given tile
/// based on the tiles terrain and the objects in the tile
pub struct MoveCheckAllowedTile;

impl TileMoveCheck for MoveCheckAllowedTile {
    fn is_valid_move(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        _tile_pos: &TilePos,
        _last_tile_pos: &TilePos,
        world: &mut World,
    ) -> bool {
        let mut system_state: SystemState<(
            Query<(
                Entity,
                &GameId,
                Option<&ObjectTypeMovementRules>,
                Option<&ObjectMovement>,
                Option<&ObjectInfo>,
            )>,
            Query<(&TileTerrainInfo, &TileObjects)>,
        )> = SystemState::new(world);
        let (mut object_query, mut tile_query) = system_state.get_mut(world);

        let Ok((entity, object_id, object_type_movement_rules, object_movement, object_info)) = object_query.get(entity_moving) else{
            return false
        };

        let Ok((tile_terrain_info, tile_objects)) = tile_query.get(tile_entity) else{
            return false
        };

        // if the moving object has the optional type movement rules
        if let Some(object_type_movement_rules) = object_type_movement_rules {
            // get the tiles object holder
            // for each object in the holder we feed its info into the ObjectTypeMovementRules
            // and return the bool if its there, else we just ignore it
            for tile_object in tile_objects.entities_in_tile.iter() {
                let Some((_, _, _, _, object_info)) = object_query
                        .iter()
                        .find(|(_, id, _, _, _)| id == &tile_object) else{
                        return true;
                    };
                if let Some(object_info) = object_info {
                    if let Some(bool) = object_type_movement_rules.can_move_on_tile(object_info) {
                        return bool;
                    }
                }
            }
        };
        if let Some(object_movement) = object_movement {
            object_movement
                .object_terrain_movement_rules
                .can_move_on_tile(tile_terrain_info)
        } else {
            return false;
        }
    }
}
