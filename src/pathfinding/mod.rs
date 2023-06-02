mod algorithms;

use crate::mapping::MapId;
use crate::movement::TileMoveChecks;
use bevy::prelude::{Entity, World};
use bevy_ecs_tilemap::prelude::TilemapSize;

pub use algorithms::dijkstra;
pub use algorithms::dijkstra::DijkstraSquare;

/// What are the main parts of a pathfinding system that we want to support
/// 1. The actual pathfinding and generation - we need to use bevy_ecs_tilemap to access tiles and offer
/// them up to the pathfinding for several reasons
/// - For each tile - is this tile valid - eg can we move into this tile and should we evaluate its
///     neighbors or should we discard this tile and move on
/// - if the tile is valid is there anything we want to do to the tile or with the tile? eg if its clrs
/// then I want to use the pathfinder to do the color conflicts so for each tile I want to run a set of custom game logic

pub struct PathfindInstance<PF: PathfindAlgorithm, CB: PathfindCallback> {
    pub pathfind_algorithm: PF,
    pub node_validity_checks: TileMoveChecks,
    pub pathfind_callback: Option<CB>,
}

impl<PF, CB> PathfindInstance<PF, CB>
where
    PF: PathfindAlgorithm,
    CB: PathfindCallback,
{
    /// The main function of a
    pub fn pathfind(
        &mut self,
        on_map: MapId,
        pathfind_entity: Entity,
        mut world: &mut World,
    ) -> PF::PathfindOutput {
        self.pathfind_algorithm.pathfind(
            on_map,
            pathfind_entity,
            world,
            &mut self.node_validity_checks,
            &mut self.pathfind_callback,
        )
    }
}

/// Core trait that represents the base algorithm
pub trait PathfindAlgorithm {
    /// The output that your pathfind algorithm will output.
    type PathfindOutput;
    type PathfindMap;
    type MapNode: MapNode;
    type NodePos;
    /// The main component of the pathfinding system. This is what computes the pathfinder and runs
    /// the rest of the components of the systems
    fn pathfind<CB: PathfindCallback>(
        &mut self,
        on_map: MapId,
        pathfind_entity: Entity,
        world: &mut World,
        node_validity_checks: &mut TileMoveChecks,
        pathfind_callback: &mut Option<CB>,
    ) -> Self::PathfindOutput;

    fn new_pathfind_map(starting_pos: Self::NodePos) -> Self::PathfindMap;

    fn node_cost_calculation(
        pathfinding_entity: Entity,
        node_entity: Entity,
        node_pos: Self::NodePos,
        starting_node_pos: Self::NodePos,
        pathfind_map: &mut Self::PathfindMap,
        world: &World,
    ) -> bool;

    fn get_neighbors(
        &self,
        node_pos: Self::NodePos,
        tilemap_size: &TilemapSize,
    ) -> Vec<Self::NodePos>;

    fn get_node_mut(&mut self, node_pos: Self::NodePos) -> Option<&mut Self::MapNode>;

    fn new_node(&mut self, new_node_pos: Self::NodePos, prior_node: Self::MapNode);
    fn set_valid_node(&mut self, node_pos: Self::NodePos) -> Result<(), String>;
    fn set_calculated_node(&mut self, node_pos: Self::NodePos) -> Result<(), String>;
}

/// Trait that represents a node in the graph
pub trait MapNode {
    type NodePos;
    type MapNode;

    /// Returns the node that led to this node
    fn previous_node_pos(&self) -> Self::NodePos;

    /// sets the previous node that led to this one
    fn set_previous_node(&mut self, node: Self::NodePos);

    fn cost(&mut self) -> u32;

    fn set_cost(&mut self, cost: u32);
}

pub trait PathfindCallback {
    type NodePos;

    fn foreach_tile(
        &mut self,
        pathfinding_entity: Entity,
        node_entity: Entity,
        node_pos: Self::NodePos,
        world: &mut World,
    );
}
