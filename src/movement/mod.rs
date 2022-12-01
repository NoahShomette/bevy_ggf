use bevy::app::App;
use crate::mapping::terrain::{TerrainBaseType, TerrainExtensionType};
use bevy::prelude::{Bundle, Component, Plugin, Resource};
use bevy::utils::HashMap;

/// Movement System

pub struct BggfMovementPlugin;

impl Plugin for BggfMovementPlugin{
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMovementRules>();
    }
}

// just quick example of a movement system might work for a unit
struct UnitMovementRules {
    terrain_base_rules: HashMap<&'static TerrainBaseType, bool>,
    terrain_extension_rules: HashMap<&'static TerrainExtensionType, bool>,
}

/*
fn test() {
    let mut movement_rules = UnitMovementRules {
        terrain_base_rules: HashMap::new(),
        terrain_extension_rules: HashMap::new(),
    };

    movement_rules
        .terrain_base_rules
        .insert(&TERRAIN_BASE_TYPES[0], true);
    movement_rules
        .terrain_extension_rules
        .insert(&TERRAIN_EXTENSION_TYPES[2], false);
}

 */

/// Component that must be added to a tile in order to define that tiles movement cost.
///
/// Contains a hashmap that holds a reference to a [`MovementType`] as a key and a u32 as the value. The u32 is used
/// in pathfinding as the cost to move into that tile.
#[derive(Clone, Eq, PartialEq, Component)]
pub struct TileMovementCosts {
    pub movement_type_cost: HashMap<&'static MovementType, u32>,
}

/// Struct used to define a new [`MovementType`]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MovementType {
    pub name: &'static str,
}

/// Defines a resource that will hold all [`TileMovementCosts`] - references to a specific TileMovementCosts
/// are stored in each tile as their current cost.
#[derive(Resource, Default)]
pub struct TileMovementRules {
    pub movement_cost_rules: HashMap<TerrainExtensionType, TileMovementCosts>,
}


//UNIT MOVEMENT STUFF

/// Basic Bundle that supplies all needed movement components for a unit
#[derive(Bundle)]
pub struct UnitMovementBundle{
    unit_movement_type: UnitMovementType,
}

/// Holds a reference to a units [`MovementType`]. A MovementType is used to define what kind of movement
/// costs that the unit uses during movement
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct UnitMovementType {
    movement_type: &'static MovementType,
}
