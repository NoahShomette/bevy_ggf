use bevy::utils::HashMap;
use crate::mapping::tiles::{TERRAIN_BASE_TYPES, TERRAIN_EXTENSION_TYPES, TerrainBaseType, TerrainExtensionType};

//just quick example of a movement system might work for a unit
struct Unit{
    
    base_terrain_rules: HashMap<&'static TerrainBaseType, bool>,
    extension_terrain_rules: HashMap<&'static TerrainExtensionType, bool>,

}
fn test(){
 
    let mut new_unit = Unit{
        base_terrain_rules: HashMap::new(),
        extension_terrain_rules: HashMap::new()
    };
    
    new_unit.base_terrain_rules.insert(&TERRAIN_BASE_TYPES[0], true);
    new_unit.extension_terrain_rules.insert(&TERRAIN_EXTENSION_TYPES[2], false);

}