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

use bevy::prelude::{Component, ReflectComponent};
use bevy::reflect::{FromReflect, Reflect};
use serde::{Deserialize, Serialize};

/// Component holding the tile terrain info needed by any built in logic.
/// Terrain type
#[derive(Default, Component, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct TileTerrainInfo {
    pub terrain_type: TerrainType,
}

/// Defines a new TerrainClass representing a category of [`TerrainType`]s. Used to specify different
/// class or categories of terrain. Eg Ground, Water, Etc
#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    PartialEq,
    Debug,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct TerrainClass {
    pub name: String,
}

/// Defines a new TerrainType that is considered a derivative of the assigned terrain_class
#[derive(
    Default,
    Clone,
    Hash,
    Eq,
    PartialEq,
    Debug,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct TerrainType {
    pub name: String,
    pub terrain_class: TerrainClass,
}
