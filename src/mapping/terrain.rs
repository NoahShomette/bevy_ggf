//!

// two types of terrain. TerrainBaseType, TerrainExtensionType -
// TerrainBaseType is a type like Ground, Water,
// whatever else. TerrainExtensionType is like grassland, mountain, forests, hill forest, ocean, etc

// Terrains should work like this - you define the base type, and then you define the extension
// types of that base type. The extension types are only definable or assignable from the base type
// Maybe the defined base type holds a vec of the extension types or something.
// You can't assign a base type to anything as its more of a category really rather than
// a specific terrain. But you should be able to access the base type from an extension type

// information we need to hold in terrain
// Terrain type (both base and extension
//       - but the extension should be derived from the base type in such a way that we are really
//         only dealing with the extensions but we can access the base type from the extension as needed),
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
    pub terrain_extension_type: TerrainExtensionType,
}

/// Defines a new Terrain Base Type representing a category of Terrain types - eg Ground, Water, Etc
#[derive(Eq, Hash, PartialEq)]
pub struct TerrainBaseType {
    pub name: &'static str,
}

/// Defines a new Terrain Extension Type that is a derivative of the assigned terrain_base_type
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct TerrainExtensionType {
    pub name: &'static str,
    pub texture_index: u32,
    pub terrain_base_type: &'static TerrainBaseType,
}

