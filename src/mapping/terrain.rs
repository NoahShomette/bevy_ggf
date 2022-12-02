//!

// two types of terrain. TerrainClass, TerrainType -
// TerrainClass is a type like Ground, Water,
// whatever else. TerrainType is like grassland, mountain, forests, hill forest, ocean, etc

// Terrains should work like this - you define the TerrainClass, and then you define the
// TerrainTypes of that TerrainClass
// You can't assign a TerrainClass to anything as its more of a category really rather than
// a specific terrain. But you should be able to access the TerrainClass from a TerrainType

// information we need to hold in terrain
// TerrainType - holds a reference to its TerrainClass
// movement cost for each movement type - track, wheeled, etc (this would need to be extensible as well),
//
//
//
//

use bevy::prelude::Component;

/// Component holding the tile terrain info needed by any built in logic.
/// Terrain type
#[derive(Component)]
pub struct TileTerrainInfo {
    pub terrain_type: TerrainType,
}

/// Defines a new TerrainClass representing a category of [`TerrainType`]s. Used to specify different 
/// class or categories of terrain. Eg Ground, Water, Etc
#[derive(Eq, Hash, PartialEq)]
pub struct TerrainClass {
    pub name: &'static str,
}

/// Defines a new TerrainType that is considered a derivative of the assigned terrain_class
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct TerrainType {
    pub name: &'static str,
    pub texture_index: u32,
    pub terrain_class: &'static TerrainClass,
}

