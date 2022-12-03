// What does a tile need to hold?

// Terrain information - movement related stuff and all that like movement costs
// units in tile information - maybe just the entity?
// the sprite
// buildings in tile - which will be somewhat tied to the tile as I want buildings to basically be moveless units.
// Maybe buildings get a marker component or trait, then if you want something to be a building its the same
// stuff as a unit but they get that marker component/trait and it holds them in a separate spot

use crate::mapping::terrain::TileTerrainInfo;
use bevy::prelude::{Bundle, Commands, Component, Entity};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::TileBundle;

/// Bundle containing all the basic tile components needed for a tile.
///
/// ### Note
/// Does not contain components from other sections of the crate such as [TileMovementCosts](crate::movement::TileMovementCosts), if
/// you want one of those use one of the super bundles in prelude. If you need to include other
/// components in every tile and one of the super bundles wont work it's recommended to create your
/// own super bundles
#[derive(Bundle)]
pub struct GGFTileBundle {
    /// Bevy_ecs_tilemap tile bundle
    pub tile_bundle: TileBundle,
    pub tile: Tile,
    pub tile_terrain_info: TileTerrainInfo,
}

#[derive(Bundle)]
pub struct GGFTileObjectBundle {
    pub tile_stack_rules: TileObjectStacks,
    pub tile_objects: TileObjects,
}

/// Marker component on map tiles for ease of query and accessing
#[derive(Component)]
pub struct Tile;

/// Defines a new TileStackRule based on an [`ObjectClass`]. The count of objects in the tile is kept
/// using [`TileStackCountMax`].
#[derive(Clone, Eq, PartialEq, Component)]
pub struct TileObjectStacks {
    pub tile_object_stacks: HashMap<&'static StackingClass, TileStackCountMax>,
}

impl TileObjectStacks {
    pub fn has_space(&self, object_class: &ObjectStackingClass) -> bool {
        if let Some(tile_stack_count_max) = self.tile_object_stacks.get(object_class.stack_class)
        {
            if tile_stack_count_max.current_count < tile_stack_count_max.max_count {
                return true;
            }
        } else {
            return false;
        }
        return false;
    }

    pub fn increment_object_class_count(&mut self, object_class: &ObjectStackingClass) {
        if let Some(tile_stack_count_max) = self.tile_object_stacks.get_mut(object_class.stack_class)
        {
            tile_stack_count_max.current_count += 1;
        }
    }

    pub fn decrement_object_class_count(&mut self, object_class: &ObjectStackingClass) {
        if let Some(tile_stack_count_max) = self.tile_object_stacks.get_mut(object_class.stack_class)
        {
            if tile_stack_count_max.current_count == 0{
                
            }else{
                tile_stack_count_max.current_count -= 1;
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StackingClass {
    pub name: &'static str,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Component)]
pub struct ObjectStackingClass {
    pub stack_class: &'static StackingClass,
}

/// Wraps two u32s for use in a [`TileStackRules`] component. Used to keep track of the current_count
/// of objects belonging to that [`ObjectClass`] in the tile and the max_count allowed in the tile.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TileStackCountMax {
    pub current_count: u32,
    pub max_count: u32,
}

/// Simple Vec that holds Entities that are currently in the tile. Is not sorted by [`ObjectClasses`]
#[derive(Clone, Eq, PartialEq, Default, Component)]
pub struct TileObjects {
    pub entities_in_tile: Vec<Entity>,
}

impl TileObjects {
    
    pub fn contains_object(&self, entity: Entity) -> bool {
        return if self.entities_in_tile.contains(&entity) {
            true
        } else {
            false
        };
    }

    pub fn add_object(&mut self, entity: Entity){
        self.entities_in_tile.push(entity);
    }

    pub fn remove_object(&mut self, entity: Entity) -> bool {
        let mut iter = self.entities_in_tile.iter();
        return if let Some(position) = iter.position(|&i| i == entity) {
            self.entities_in_tile.remove(position);
            true
        } else {
            false
        };
    }
}
