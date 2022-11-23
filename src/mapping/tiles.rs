

// What does a tile need to hold?

// Terrain information - movement related stuff and all that like movement costs
// units in tile information - maybe just the entity?
// the sprite
// buildings in tile - which will be somewhat tied to the tile as I want buildings to basically be moveless units.
// Maybe buildings get a marker component or trait, then if you want something to be a building its the same
// stuff as a unit but they get that marker component/trait and it holds them in a separate spot


use bevy::prelude::{Bundle, Component};
use bevy_ecs_tilemap::prelude::TileBundle;
use crate::mapping::terrain::{TileTerrainInfo};

/// Bundle containing all the basic tile components needed for a tile.
///
/// ### Note
/// Does not contain components from other sections of the crate such as [TileMovementCosts](crate::movement::TileMovementCosts), if
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


