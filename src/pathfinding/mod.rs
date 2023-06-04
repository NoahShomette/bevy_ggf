mod algorithms;

use crate::mapping::{Map, MapId};
use crate::movement::TileMoveChecks;
use bevy::prelude::{Component, Entity, World};
use bevy_ecs_tilemap::prelude::TilemapSize;
use std::marker::PhantomData;
use std::path::Iter;

use crate::pathfinding;
pub use algorithms::dijkstra;
pub use algorithms::dijkstra::DijkstraSquare;

/// What are the main parts of a pathfinding system that we want to support
/// 1. The actual pathfinding and generation - we need to use bevy_ecs_tilemap to access tiles and offer
/// them up to the pathfinding for several reasons
/// - For each tile - is this tile valid - eg can we move into this tile and should we evaluate its
///     neighbors or should we discard this tile and move on
/// - if the tile is valid is there anything we want to do to the tile or with the tile? eg if its clrs
/// then I want to use the pathfinder to do the color conflicts so for each tile I want to run a set of custom game logic

pub struct PathfindInstance<
    PF: PathfindAlgorithm<NodePos, MapNode, CostComponent>,
    PM: PathfindMap<NodePos, MapNode, PF::PathfindOutput, CostComponent>,
    CB: PathfindCallback<NodePos>,
    NodePos,
    MapNode,
    CostComponent: Component,
> {
    pub pathfind_algorithm: PF,
    pub node_validity_checks: TileMoveChecks,
    pub pathfind_callback: Option<CB>,
    pub pathfind_map: PM,
    phantom_data: PhantomData<NodePos>,
    phantom_data_2: PhantomData<MapNode>,
}

impl<PF, PM, NodePos, MapNode, CB, CostComponent: Component>
    PathfindInstance<PF, PM, CB, NodePos, MapNode, CostComponent>
where
    PF: PathfindAlgorithm<NodePos, MapNode, CostComponent>,
    PM: PathfindMap<NodePos, MapNode, PF::PathfindOutput, CostComponent>,
    CB: PathfindCallback<NodePos>,
{
    /// Construct a new pathfind instance
    pub fn new(
        pathfind_algorithm: PF,
        node_validity_checks: TileMoveChecks,
        pathfind_callback: Option<CB>,
        pathfind_map: PM,
    ) -> Self {
        Self {
            pathfind_algorithm,
            node_validity_checks,
            pathfind_callback,
            pathfind_map,
            phantom_data: Default::default(),
            phantom_data_2: Default::default(),
        }
    }

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
            &mut self.pathfind_map,
        )
    }
}

/// Core trait that represents the base algorithm
pub trait PathfindAlgorithm<NodePos, MapNode, CostComponent: Component> {
    /// The output that your pathfind algorithm will output.
    type PathfindOutput;
    /// The main component of the pathfinding system. This is what computes the pathfinder and runs
    /// the rest of the components of the systems
    fn pathfind<
        CB: PathfindCallback<NodePos>,
        PM: PathfindMap<NodePos, MapNode, Self::PathfindOutput, CostComponent>,
    >(
        &mut self,
        on_map: MapId,
        pathfind_entity: Entity,
        world: &mut World,
        node_validity_checks: &mut TileMoveChecks,
        pathfind_callback: &mut Option<CB>,
        pathfind_map: &mut PM,
    ) -> Self::PathfindOutput;
}

pub trait PathfindMap<NodePos, MapNode, PathfindOutput, CostComponent: Component> {
    fn new_pathfind_map(&mut self, starting_pos: NodePos);

    fn node_cost_calculation(
        &mut self,
        pathfinding_entity: Entity,
        node_entity: Entity,
        node_pos: NodePos,
        starting_node_pos: NodePos,
        world: &World,
    ) -> bool;

    fn get_neighbors(&self, node_pos: NodePos, tilemap_size: &TilemapSize) -> Vec<NodePos>;

    fn get_node_mut(&mut self, node_pos: NodePos) -> Option<&mut MapNode>;

    fn new_node(&mut self, new_node_pos: NodePos, prior_node: MapNode);
    fn set_valid_node(&mut self, node_pos: NodePos) -> Result<(), String>;
    fn set_calculated_node(&mut self, node_pos: NodePos) -> Result<(), String>;
    fn get_output(&mut self) -> PathfindOutput;
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

pub trait PathfindCallback<NodePos> {
    fn foreach_tile(
        &mut self,
        pathfinding_entity: Entity,
        node_entity: Entity,
        node_pos: NodePos,
        world: &mut World,
    );
}
