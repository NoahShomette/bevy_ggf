use crate::mapping::MapId;
use crate::movement::{AvailableMove, ObjectMovement, TileMoveChecks, TileMovementCosts};
use crate::object::ObjectGridPosition;
use crate::pathfinding::{MapNode, PathfindAlgorithm, PathfindCallback, PathfindMap};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Entity, Query, World};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};

#[derive(Clone, Copy)]
pub struct Node {
    pub node_pos: TilePos,
    pub prior_node_pos: TilePos,
    pub move_cost: u32,
    pub valid_move: bool,
    pub calculated: bool,
}

impl From<Node> for AvailableMove {
    /// Converts the MoveNode to AvailableMove. It will set move_cost to zero if the given move node
    /// does not have a move cost set.
    fn from(node: Node) -> Self {
        Self {
            tile_pos: node.node_pos,
            move_cost: node.move_cost as i32,
            prior_tile_pos: node.prior_node_pos,
        }
    }
}

impl MapNode for Node {
    type NodePos = TilePos;
    type MapNode = Node;

    fn previous_node_pos(&self) -> Self::NodePos {
        self.prior_node_pos
    }

    fn set_previous_node(&mut self, node_pos: Self::NodePos) {
        self.prior_node_pos = node_pos;
    }

    fn cost(&self) -> u32 {
        self.move_cost
    }

    fn set_cost(&mut self, cost: u32) {
        self.move_cost = cost;
    }
}

pub struct DijkstraSquare {
    pub diagonals: bool,
    pub nodes: HashMap<TilePos, Node>,
}

impl PathfindAlgorithm<TilePos, Node, ObjectMovement> for DijkstraSquare {
    type PathfindOutput = Vec<AvailableMove>;

    fn pathfind<
        CB: PathfindCallback<TilePos>,
        PM: PathfindMap<TilePos, Node, Vec<AvailableMove>, ObjectMovement>,
    >(
        &mut self,
        on_map: MapId,
        pathfind_entity: Entity,
        mut world: &mut World,
        node_validity_checks: &mut TileMoveChecks,
        pathfind_callback: &mut Option<CB>,
        pathfind_map: &mut PM,
    ) -> Self::PathfindOutput {
        let mut system_state: SystemState<(
            Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
            Query<&ObjectGridPosition>,
        )> = SystemState::new(world);
        let (mut tile_storage_query, object_query) = system_state.get_mut(world);

        let Ok(object_grid_position) = object_query.get(pathfind_entity) else {
            return vec![];
        };

        let Some((_, _, tile_storage, tilemap_size)) = tile_storage_query
            .iter_mut()
            .find(|(_, id, _, _)| id == &&on_map)
        else {
            return vec![];
        };

        let tile_storage = tile_storage.clone();
        let tilemap_size = tilemap_size.clone();

        pathfind_map.new_pathfind_map(object_grid_position.tile_position.into());

        let mut available_moves: Vec<TilePos> = vec![];

        // unvisited nodes
        let mut unvisited_nodes: Vec<Node> = vec![Node {
            node_pos: object_grid_position.tile_position.into(),
            prior_node_pos: object_grid_position.tile_position.into(),
            move_cost: 0,
            valid_move: false,
            calculated: false,
        }];
        let mut visited_nodes: Vec<TilePos> = vec![];

        while !unvisited_nodes.is_empty() {
            unvisited_nodes.sort_by(|x, y| x.move_cost.partial_cmp(&y.move_cost).unwrap());

            let Some(current_node) = unvisited_nodes.get(0) else {
                continue;
            };

            let neighbor_pos = pathfind_map.get_neighbors(current_node.node_pos, &tilemap_size);

            let current_node = *current_node;
            let mut neighbors: Vec<(TilePos, Entity)> = vec![];
            for neighbor in neighbor_pos.iter() {
                let Some(tile_entity) = tile_storage.get(neighbor) else {
                    continue;
                };
                neighbors.push((*neighbor, tile_entity));
            }

            'neighbors: for neighbor in neighbors.iter() {
                if visited_nodes.contains(&neighbor.0) {
                    continue;
                }

                pathfind_map.new_node(neighbor.0, current_node);

                if !pathfind_map.node_cost_calculation(
                    pathfind_entity,
                    neighbor.1,
                    neighbor.0,
                    current_node.node_pos,
                    world,
                ) {
                    let _ = pathfind_map.set_calculated_node(neighbor.0);
                    continue 'neighbors;
                }

                if !node_validity_checks.check_tile_move_checks(
                    pathfind_entity,
                    neighbor.1,
                    &neighbor.0,
                    &current_node.node_pos,
                    world,
                ) {
                    let _ = pathfind_map.set_calculated_node(neighbor.0);
                    continue 'neighbors;
                }

                let _ = pathfind_map.set_valid_node(neighbor.0);
                let _ = pathfind_map.set_calculated_node(neighbor.0);

                // if none of them return false and cancel the loop then we can infer that we are able to move into that neighbor
                // we add the neighbor to the list of unvisited nodes and then push the neighbor to the available moves list
                unvisited_nodes.push(pathfind_map.get_node_mut(neighbor.0).expect(
                    "Is safe because we know we add the node in at the beginning of this loop",
                ).clone());
                available_moves.push(neighbor.0);

                if let Some(callback) = pathfind_callback {
                    callback.foreach_tile(
                        pathfind_entity,
                        neighbor.1,
                        neighbor.0,
                        pathfind_map.get_node(neighbor.0).unwrap().cost(),
                        &mut world,
                    );
                }
            }

            unvisited_nodes.remove(0);
            visited_nodes.push(current_node.node_pos);
        }

        pathfind_map.get_output()
    }
}

pub struct PathfindMapDijkstra {
    pub map: HashMap<TilePos, Node>,
    pub diagonals: bool,
}

impl PathfindMap<TilePos, Node, Vec<AvailableMove>, ObjectMovement> for PathfindMapDijkstra {
    fn new_pathfind_map(&mut self, starting_pos: TilePos) {
        let mut map: HashMap<TilePos, Node> = HashMap::default();

        // insert the starting node at the moving objects grid position
        map.insert(
            starting_pos,
            Node {
                node_pos: starting_pos,
                prior_node_pos: starting_pos,
                move_cost: 0,
                valid_move: true,
                calculated: false,
            },
        );

        self.map = map;
    }

    fn node_cost_calculation(
        &mut self,
        entity_moving: Entity,
        tile_entity: Entity,
        tile_pos: TilePos,
        move_from_tile_pos: TilePos,
        world: &World,
    ) -> bool {
        let Some(object_movement) = world.get::<ObjectMovement>(entity_moving) else {
            return false;
        };
        let Some(tile_movement_costs) = world.get::<TileMovementCosts>(tile_entity) else {
            return false;
        };

        let Some([tile_node, move_from_tile_node]) =
            self.map.get_many_mut([&tile_pos, &move_from_tile_pos])
        else {
            return false;
        };

        return if tile_node.calculated {
            if (move_from_tile_node.move_cost
                + *tile_movement_costs
                    .movement_type_cost
                    .get(&object_movement.movement_type)
                    .unwrap_or(&1))
                < (tile_node.move_cost)
            {
                tile_node.move_cost = move_from_tile_node.move_cost
                    + *tile_movement_costs
                        .movement_type_cost
                        .get(&object_movement.movement_type)
                        .unwrap_or(&1);
                tile_node.prior_node_pos = move_from_tile_node.node_pos;
                true
            } else {
                false
            }
        } else if (move_from_tile_node.move_cost
            + *tile_movement_costs
                .movement_type_cost
                .get(&object_movement.movement_type)
                .unwrap_or(&1))
            <= object_movement.move_points as u32
        {
            tile_node.move_cost = move_from_tile_node.move_cost
                + *tile_movement_costs
                    .movement_type_cost
                    .get(&object_movement.movement_type)
                    .unwrap_or(&1);
            tile_node.prior_node_pos = move_from_tile_node.node_pos;
            true
        } else {
            false
        };
    }

    fn get_neighbors(&self, node_pos: TilePos, tilemap_size: &TilemapSize) -> Vec<TilePos> {
        let mut neighbor_tiles: Vec<TilePos> = vec![];
        let origin_tile = node_pos;
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

        if self.diagonals {
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

    fn get_node_mut(&mut self, node_pos: TilePos) -> Option<&mut Node> {
        self.map.get_mut(&node_pos)
    }

    fn get_node(&self, node_pos: TilePos) -> Option<&Node> {
        self.map.get(&node_pos)
    }

    fn new_node(&mut self, new_node_pos: TilePos, prior_node: Node) {
        if !self.map.contains_key(&new_node_pos) {
            let node = Node {
                node_pos: new_node_pos,
                prior_node_pos: prior_node.node_pos,
                move_cost: 0,
                valid_move: false,
                calculated: false,
            };
            self.map.insert(new_node_pos, node);
        }
    }

    fn set_valid_node(&mut self, node_pos: TilePos) -> Result<(), String> {
        return if let Some(node) = self.get_node_mut(node_pos) {
            node.valid_move = true;
            Ok(())
        } else {
            Err(String::from("Error getting node"))
        };
    }

    fn set_calculated_node(&mut self, node_pos: TilePos) -> Result<(), String> {
        return if let Some(node) = self.get_node_mut(node_pos) {
            node.calculated = true;
            Ok(())
        } else {
            Err(String::from("Error getting node"))
        };
    }

    fn get_output(&mut self) -> Vec<AvailableMove> {
        let mut available_moves: Vec<AvailableMove> = vec![];
        for (_, node) in self.map.iter() {
            if node.valid_move {
                available_moves.push(AvailableMove::from(*node));
            }
        }
        available_moves
    }
}
