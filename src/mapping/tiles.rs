use bevy::prelude::{App, Component, IVec2};
use typetag::private::serde::{Deserialize, Serialize};
use downcast_rs::{Downcast, impl_downcast};

// How to 

// two types of terrain. TerrainBaseType and TerrainExtensionType - TerrainBaseType is a type like Ground, Water,
// whatever else. TerrainExtensionType is like grass, mountain, forest, ocean, etc. 
// The two types work on 


// Terrains should work like this - you define the base type, and then you define the extension 
// types of that base type. The extension types are only definable or assignable from the base type
// Maybe the defined base type holds a vec of the 

// information we need to hold in terrain
// Terrain type (both base and extension
//       - but the extension should be derived from the base type in such a way that we are really
//         only dealing with the extensions but we can access the base type from the extension as needed), 
// movement cost for each movement type - track, wheeled, etc (this would need to be extensible as well), 
// 
// 
//
//

pub struct TileTerrain{
    
}

pub trait AppTerrainSystem{
    
    fn add_terrain_to_app();
}

impl AppTerrainSystem for App{
    fn add_terrain_to_app() {
        todo!()
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct MapTile{
    tile_position: IVec2,
}


pub trait TerrainBaseTypeCore : Downcast{
}
impl_downcast!(TerrainBaseTypeCore);

pub trait TerrainBaseType : TerrainBaseTypeCore{
    const NAME: &'static str;
    
}


pub trait TerrainExtensionTypeCore : Downcast{
}
impl_downcast!(TerrainExtensionTypeCore);


pub trait TerrainExtensionType : TerrainExtensionTypeCore{
    const NAME: &'static str;
    //const BASE_TYPE: &'static dyn TerrainBaseType;
}