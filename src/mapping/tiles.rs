// two types of terrain. TerrainBaseType and TerrainExtensionType - TerrainBaseType is a type like Ground, Water,
// whatever else. TerrainExtensionType is like grass, mountain, forest, ocean, etc.
// The two types work on a derivitive basis. The extension types are derivitives of a base type.

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

// What does a tile need to hold?

// Terrain information - movement related stuff and all that like movement costs
// units in tile information - maybe just the entity?
// the sprite
// buildings in tile - which will be somewhat tied to the tile as I want buildings to basically be moveless units.
// Maybe buildings get a marker component or trait, then if you want something to be a building its the same
// stuff as a unit but they get that marker component/trait and it holds them in a separate spot


use bevy::prelude::{Bundle, Component, Entity, reflect_trait, Resource};
use bevy_ecs_tilemap::prelude::TileBundle;

/// Bundle containing all the basic tile components needed for a tile.
///
/// ### Note
/// Does not contain components from other sections of the crate such as [`TileMovementCosts`], if
/// you want one of those use one of the super bundles in prelude. If you need to include other 
/// components in every tile and one of the super bundles wont work it's recommended to create your
/// own super bundles
#[derive(Bundle)]
pub struct GGFTileBundle{
    /// Bevy_ecs_tilemap tile bundle
    pub tile_bundle: TileBundle,
    pub tile : Tile,
    pub tile_terrain_info: TileTerrainInfo,
}

/// Marker component on map tiles for ease of query and accessing
#[derive(Component)]
pub struct Tile;

/// Component holding the tile terrain info needed by any built in logic.
/// Terrain type
#[derive(Component)]
pub struct TileTerrainInfo {
    pub terrain_extension_type: TerrainExtensionType

}

/// Defines a new Terrain Base Type representing a category of Terrain types - eg Ground, Water, Etc
#[derive(Eq, Hash, PartialEq)]
pub struct TerrainBaseType{
    pub name: &'static str,
}

// Defines a new Terrain Extension Type that is part of the 
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct TerrainExtensionType{
    pub name: &'static str,
    pub texture_index: u32,
    pub terrain_base_type: &'static TerrainBaseType,
}




pub const TERRAIN_BASE_TYPES: &'static [TerrainBaseType] = &[
    TerrainBaseType{ name: "Ground" },
    TerrainBaseType{ name: "Water" }

];

pub const TERRAIN_EXTENSION_TYPES: &'static [TerrainExtensionType] = &[
    TerrainExtensionType{ name: "Grassland", texture_index: 0, terrain_base_type: &TERRAIN_BASE_TYPES[0] },
    TerrainExtensionType{ name: "Forest", texture_index: 1, terrain_base_type: &TERRAIN_BASE_TYPES[0] },
    TerrainExtensionType{ name: "Mountain", texture_index: 2, terrain_base_type: &TERRAIN_BASE_TYPES[0] },
    TerrainExtensionType{ name: "Hill", texture_index: 3, terrain_base_type: &TERRAIN_BASE_TYPES[0] },
    TerrainExtensionType{ name: "Sand", texture_index: 4, terrain_base_type: &TERRAIN_BASE_TYPES[0] },
    TerrainExtensionType{ name: "CoastWater", texture_index: 5, terrain_base_type: &TERRAIN_BASE_TYPES[1] },
    TerrainExtensionType{ name: "Ocean", texture_index: 6, terrain_base_type: &TERRAIN_BASE_TYPES[1] },
];
