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


use bevy::prelude::{Component, Entity, reflect_trait, Resource};
use downcast_rs::DowncastSync;

/// Marker component on map tiles for ease of query and accessing
#[derive(Component)]
pub struct Tile;

/// Marker component on map tiles for ease of query and accessing
//#[derive(Resource)]
//pub struct TerrainInfo(HashMap<dyn TerrainExtensionTraitBase, dyn TerrainBaseTraitBase>);

/// Component holding the tile terrain info needed by any built in logic.
/// Terrain type
#[derive(Component)]
pub struct TileTerrainInfo {
    //pub(crate) terrain_extension_type: Box<dyn TerrainExtensionTraitBase>
    pub terrain_extension_type: TerrainExtensionType

}

#[derive(Eq, Hash, PartialEq)]
pub struct TerrainBaseType{
    pub name: &'static str,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
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


pub trait TerrainBaseTraitBase: DowncastSync {}
downcast_rs::impl_downcast!(sync TerrainBaseTraitBase);

pub trait TerrainBaseTrait: TerrainBaseTraitBase {
    /// A unique name to identify your Terrain Base Type, this needs to be unique __across all included crates__
    ///
    /// A good combination is crate name + struct name
    const NAME: &'static str;
}

/// You will also have to mark it with either [`ServerMessage`] or [`ClientMessage`] (or both)
/// to signal which direction this message can be sent.
pub trait TerrainExtensionTraitBase: DowncastSync {
    
    fn return_name(&self) -> &str;
    fn return_texture_id(&self) -> &u32;
    fn return_terrain_base_trait(&self) -> &dyn TerrainBaseTraitBase;
    
}
downcast_rs::impl_downcast!(sync TerrainExtensionTraitBase);

pub trait TerrainExtensionTrait: TerrainExtensionTraitBase {
    /// A unique name to identify your Terrain Extension Type, this needs to be unique __across all included crates__
    ///
    /// A good combination is crate name + struct name
    const NAME: &'static str;
    const TEXTURE_ID: &'static u32;
    const TERRAIN_BASE: &'static dyn TerrainBaseTraitBase;
    
    fn return_name(&self) -> &str{
        Self::NAME
    }
    fn return_texture_id(&self) -> &u32{
        Self::TEXTURE_ID
    }
}

pub struct GroundTerrainBase {}

impl TerrainBaseTraitBase for GroundTerrainBase {}
impl TerrainBaseTrait for GroundTerrainBase {
    const NAME: &'static str = "Bevy_ggf_GroundTerrainBase";
}

pub struct WaterTerrainBase {}

impl TerrainBaseTraitBase for WaterTerrainBase {}
impl TerrainBaseTrait for WaterTerrainBase {
    const NAME: &'static str = "Bevy_ggf_WaterTerrainBase";
}

pub struct Grassland {}

impl TerrainExtensionTraitBase for Grassland {
    fn return_name(&self) -> &str {
        "Bevy_ggf_Grassland"    
    }

    fn return_texture_id(&self) -> &u32 {
        &0
    }

    fn return_terrain_base_trait(&self) -> &dyn TerrainBaseTraitBase {
        &GroundTerrainBase{}
    }
}


pub struct Hill {}

impl TerrainExtensionTraitBase for Hill {
    fn return_name(&self) -> &str {
        "Bevy_ggf_Hill"
    }

    fn return_texture_id(&self) -> &u32 {
        &3
    }

    fn return_terrain_base_trait(&self) -> &dyn TerrainBaseTraitBase {
        &GroundTerrainBase{}
    }
}


pub struct Ocean {}

impl TerrainExtensionTraitBase for Ocean {
    fn return_name(&self) -> &str {
        "Bevy_ggf_Ocean"
    }

    fn return_texture_id(&self) -> &u32 {
        &6
    }

    fn return_terrain_base_trait(&self) -> &dyn TerrainBaseTraitBase {
        &WaterTerrainBase{}
    }
}

