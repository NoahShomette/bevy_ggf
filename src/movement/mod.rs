use bevy::prelude::Component;
use bevy::utils::HashMap;
use crate::mapping::tiles::{TERRAIN_BASE_TYPES, TERRAIN_EXTENSION_TYPES, TerrainBaseType, TerrainExtensionType};

/// Contains all Movement Related

// just quick example of a movement system might work for a unit
struct MovementRules{
    base_terrain_rules: HashMap<&'static TerrainBaseType, bool>,
    extension_terrain_rules: HashMap<&'static TerrainExtensionType, bool>,

}
fn test(){

    let mut movement_rules = MovementRules{
        base_terrain_rules: HashMap::new(),
        extension_terrain_rules: HashMap::new()
    };

    movement_rules.base_terrain_rules.insert(&TERRAIN_BASE_TYPES[0], true);
    movement_rules.extension_terrain_rules.insert(&TERRAIN_EXTENSION_TYPES[2], false);

}

/// Component that must be added to a tile in order to define that tiles movement cost.
/// 
/// Contains a hashmap that holds a reference to a [`MovementType`] as a key and a u32 as the value. The u32 is used
/// in pathfinding as the cost to move into that tile.
#[derive(Clone, Eq, PartialEq, Component)]
pub struct TileMovementCosts {
    pub movement_type_cost: HashMap<&'static  MovementType, u32>
}

/// Struct used to define a new [`MovementType`]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MovementType{
    pub name: &'static str,
}

pub const MOVEMENT_TYPES: &'static [MovementType] = &[
    MovementType{ name: "Normal" },
    MovementType{ name: "Tread" }

];