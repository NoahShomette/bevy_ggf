// What does a tile need to hold?

// Terrain information - movement related stuff and all that like movement costs
// units in tile information - maybe just the entity?
// the sprite
// buildings in tile - which will be somewhat tied to the tile as I want buildings to basically be moveless units.
// Maybe buildings get a marker component or trait, then if you want something to be a building its the same
// stuff as a unit but they get that marker component/trait and it holds them in a separate spot

use crate::mapping::terrain::TileTerrainInfo;
use crate::object::ObjectId;
use bevy::prelude::{Bundle, Component, ReflectComponent};
use bevy::reflect::{FromReflect, Reflect};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::TilemapId;
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};

/// Bundle containing all the basic tile components needed for a tile.
///
/// ### Note
/// Does not contain components from other sections of the crate such as [TileMovementCosts](crate::movement::TileMovementCosts), if
/// you want one of those use one of the super bundles in prelude. If you need to include other
/// components in every tile and one of the super bundles wont work it's recommended to create your
/// own super bundles
#[derive(Bundle)]
pub struct BggfTileBundle {
    pub tile: Tile,
    pub tile_terrain_info: TileTerrainInfo,
    pub tile_pos: TilePos,
    pub tilemap_id: TilemapId,
}

#[derive(Bundle)]
pub struct BggfTileObjectBundle {
    pub tile_stack_rules: TileObjectStacks,
    pub tile_objects: TileObjects,
}

/// Marker component on map tiles for ease of query and accessing
#[derive(Default, Component, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Tile;

/// Marker component on map tiles for ease of query and accessing
#[derive(
    Default,
    Clone,
    Copy,
    Eq,
    Hash,
    PartialEq,
    Debug,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct TilePosition {
    pub x: u32,
    pub y: u32,
}

impl Into<TilePos> for TilePosition {
    fn into(self) -> TilePos {
        TilePos::new(self.x, self.y)
    }
}

impl From<TilePos> for TilePosition {
    fn from(value: TilePos) -> Self {
        TilePosition::new(value.x, value.y)
    }
}

impl TilePosition {
    pub fn new(x: u32, y: u32) -> TilePosition {
        TilePosition { x: x, y: y }
    }
}

/// Defines a new stacking rule for objects based on a [`StackingClass`]. The count of objects in the tile is kept
/// using an [`TileObjectStacksCount`] struct.
#[derive(
    Default,
    Clone,
    Eq,
    PartialEq,
    Component,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(Component)]
pub struct TileObjectStacks {
    pub tile_object_stacks: HashMap<StackingClass, TileObjectStacksCount>,
}

impl TileObjectStacks {
    pub fn new(stack_rules: Vec<(StackingClass, TileObjectStacksCount)>) -> TileObjectStacks {
        TileObjectStacks {
            tile_object_stacks: TileObjectStacks::new_terrain_type_rules(stack_rules),
        }
    }

    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn new_terrain_type_rules(
        stack_rules: Vec<(StackingClass, TileObjectStacksCount)>,
    ) -> HashMap<StackingClass, TileObjectStacksCount> {
        let mut hashmap: HashMap<StackingClass, TileObjectStacksCount> = HashMap::new();
        for rule in stack_rules.iter() {
            hashmap.insert(rule.0.clone(), rule.1);
        }
        hashmap
    }

    pub fn has_space(&self, object_class: &ObjectStackingClass) -> bool {
        return if let Some(tile_stack_count_max) =
            self.tile_object_stacks.get(&object_class.stack_class)
        {
            tile_stack_count_max.current_count < tile_stack_count_max.max_count
        } else {
            false
        };
    }

    pub fn increment_object_class_count(&mut self, object_class: &ObjectStackingClass) {
        if let Some(tile_stack_count_max) =
            self.tile_object_stacks.get_mut(&object_class.stack_class)
        {
            tile_stack_count_max.current_count += 1;
        }
    }

    #[rustfmt::skip] // rustfmt breaking ci
    pub fn decrement_object_class_count(&mut self, object_class: &ObjectStackingClass) {
        if let Some(tile_stack_count_max) = self
            .tile_object_stacks
            .get_mut(&object_class.stack_class)
        {
            if tile_stack_count_max.current_count == 0 {} else {
                tile_stack_count_max.current_count -= 1;
            }
        }
    }
}

#[rustfmt::skip] // rustfmt breaking ci
#[test] // This is kinda a useless test but whatever. new year new tests
fn test_tile_object_stacks() {
    let stacking_class_ground: StackingClass = StackingClass { name: String::from("Ground") };

    let tile_object_stacking_rules = TileObjectStacks::new(vec![(
        stacking_class_ground.clone(),
        TileObjectStacksCount {
            current_count: 0,
            max_count: 1,
        },
    )]);

    assert!(tile_object_stacking_rules.has_space(&ObjectStackingClass {
        stack_class: stacking_class_ground.clone(),
    }, ))
}

/// A StackingClass represents what kind of stack an object belongs to in a tile. This is used internally
/// in [`TileObjectStacks`]
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
pub struct StackingClass {
    pub name: String,
}

/// A component to hold a [`StackingClass`].
#[derive(
    Default,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Component,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(Component)]
pub struct ObjectStackingClass {
    pub stack_class: StackingClass,
}

/// Wraps two u32s for use in a [`TileObjectStacks`] component. Used to keep track of the current_count
/// of objects belonging to that [`ObjectStackingClass`] in the tile and the max_count allowed in the tile.
#[derive(
    Default,
    Clone,
    Copy,
    Eq,
    Hash,
    PartialEq,
    Debug,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct TileObjectStacksCount {
    pub current_count: u32,
    pub max_count: u32,
}

/// Simple Vec that holds the [`ObjectId`] of all Objects that are currently in the tile.
#[derive(
    Clone,
    Eq,
    PartialEq,
    Default,
    Component,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(Component)]
pub struct TileObjects {
    pub entities_in_tile: Vec<ObjectId>,
}

impl TileObjects {
    /// Checks if the given entity is currently in this tile
    pub fn contains_object(&self, entity: ObjectId) -> bool {
        self.entities_in_tile.contains(&entity)
    }

    /// Adds the given entity
    pub fn add_object(&mut self, entity: ObjectId) {
        self.entities_in_tile.push(entity);
    }

    /// Removes the given entity
    pub fn remove_object(&mut self, entity: ObjectId) -> bool {
        let mut iter = self.entities_in_tile.iter();
        if let Some(position) = iter.position(|&i| i == entity) {
            self.entities_in_tile.remove(position);
            true
        } else {
            false
        }
    }
}
