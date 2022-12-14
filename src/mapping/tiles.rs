// What does a tile need to hold?

// Terrain information - movement related stuff and all that like movement costs
// units in tile information - maybe just the entity?
// the sprite
// buildings in tile - which will be somewhat tied to the tile as I want buildings to basically be moveless units.
// Maybe buildings get a marker component or trait, then if you want something to be a building its the same
// stuff as a unit but they get that marker component/trait and it holds them in a separate spot

use crate::mapping::terrain::TileTerrainInfo;
use bevy::prelude::{Bundle, Component, Entity};
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
pub struct BggfTileBundle {
    /// Bevy_ecs_tilemap tile bundle
    pub tile_bundle: TileBundle,
    pub tile: Tile,
    pub tile_terrain_info: TileTerrainInfo,
}

#[derive(Bundle)]
pub struct BggfTileObjectBundle {
    pub tile_stack_rules: TileObjectStacks,
    pub tile_objects: TileObjects,
}

/// Marker component on map tiles for ease of query and accessing
#[derive(Component)]
pub struct Tile;

/// Defines a new stacking rule for objects based on a [`StackingClass`]. The count of objects in the tile is kept
/// using an [`ObjectStacksCount`] struct.
#[derive(Clone, Eq, PartialEq, Component)]
pub struct TileObjectStacks {
    pub tile_object_stacks: HashMap<&'static StackingClass, ObjectStacksCount>,
}

impl TileObjectStacks {
    pub fn new(stack_rules: Vec<(&'static StackingClass, ObjectStacksCount)>) -> TileObjectStacks {
        TileObjectStacks {
            tile_object_stacks: TileObjectStacks::new_terrain_type_rules(stack_rules),
        }
    }

    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn new_terrain_type_rules(
        stack_rules: Vec<(&'static StackingClass, ObjectStacksCount)>,
    ) -> HashMap<&'static StackingClass, ObjectStacksCount> {
        let mut hashmap: HashMap<&'static StackingClass, ObjectStacksCount> = HashMap::new();
        for rule in stack_rules.iter() {
            hashmap.insert(rule.0, rule.1);
        }
        hashmap
    }

    pub fn has_space(&self, object_class: &ObjectStackingClass) -> bool {
        return if let Some(tile_stack_count_max) =
            self.tile_object_stacks.get(object_class.stack_class)
        {
            tile_stack_count_max.current_count < tile_stack_count_max.max_count
        } else {
            false
        };
    }

    pub fn increment_object_class_count(&mut self, object_class: &ObjectStackingClass) {
        if let Some(tile_stack_count_max) =
            self.tile_object_stacks.get_mut(object_class.stack_class)
        {
            tile_stack_count_max.current_count += 1;
        }
    }

    pub fn decrement_object_class_count(&mut self, object_class: &ObjectStackingClass) {
        if let Some(tile_stack_count_max) =
            self.tile_object_stacks.get_mut(object_class.stack_class)
        {
            if tile_stack_count_max.current_count == 0 {
            } else {
                tile_stack_count_max.current_count -= 1;
            }
        }
    }
}

#[test] // This is kinda a useless test but whatever. new year new tests
fn test_tile_object_stacks() {
    const STACKING_CLASS_GROUND: StackingClass = StackingClass { name: "Ground" };

    let tile_object_stacks = TileObjectStacks::new(vec![(
        &STACKING_CLASS_GROUND,
        ObjectStacksCount {
            current_count: 0,
            max_count: 1,
        },
    )]);

    assert!(tile_object_stacks.has_space(&ObjectStackingClass {
        stack_class: &STACKING_CLASS_GROUND,
    },))
}

/// A StackingClass represents what kind of stack an object belongs to in a tile. This is used internally
/// in [`TileObjectStacks`]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StackingClass {
    pub name: &'static str,
}

/// A component to hold a [`StackingClass`].
#[derive(Clone, Eq, PartialEq, Hash, Debug, Component)]
pub struct ObjectStackingClass {
    pub stack_class: &'static StackingClass,
}

/// Wraps two u32s for use in a [`TileStackRules`] component. Used to keep track of the current_count
/// of objects belonging to that [`ObjectClass`] in the tile and the max_count allowed in the tile.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct ObjectStacksCount {
    pub current_count: u32,
    pub max_count: u32,
}

/// Simple Vec that holds Entities that are currently in the tile.
#[derive(Clone, Eq, PartialEq, Default, Component)]
pub struct TileObjects {
    pub entities_in_tile: Vec<Entity>,
}

impl TileObjects {
    /// Checks if the given entity is currently in this tile
    pub fn contains_object(&self, entity: Entity) -> bool {
        self.entities_in_tile.contains(&entity)
    }

    /// Adds the given entity
    pub fn add_object(&mut self, entity: Entity) {
        self.entities_in_tile.push(entity);
    }

    /// Removes the given entity
    pub fn remove_object(&mut self, entity: Entity) -> bool {
        let mut iter = self.entities_in_tile.iter();
        if let Some(position) = iter.position(|&i| i == entity) {
            self.entities_in_tile.remove(position);
            true
        } else {
            false
        }
    }
}
