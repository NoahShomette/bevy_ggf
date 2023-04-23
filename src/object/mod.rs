//! bevy_ggf contains a built in method to represent units, buildings, and anything else that is not
//! a tile and resides on the map. This system is built on top of Bevy_ECS and is based on the entity
//! component system.

use crate::mapping::tiles::ObjectStackingClass;
use crate::movement::ObjectMovementBundle;
use bevy::prelude::{Bundle, Component, ReflectComponent, Resource};
use bevy::reflect::{FromReflect, Reflect};
use bevy_ecs_tilemap::prelude::TilePos;
use serde::{Deserialize, Serialize};

// Default Components that we should have for objects
// These are separated simply to ease development and thought process. Any component for any object can
// go on any object. Theres no fundamental difference between a building and a unit, just controlling what they can
// do

// Generic Components
/*
Health
Team
Sprite/Animation
State?
ObjectType
GridPos
Attackable

EnemyTeamMovementAllowed
AlliedTeamMovementProhibited

 */

// Unit Components
/*
Movement
UnitStats
Combat

 */

// Building Components
/*
BuildOptions
AddResourceOnTurn
 */

// ObjectClass -> (Ground, Air, Water, Building, etc)
// ObjectGroup -> (Armor, Capital Ship, Helicopter)
// ObjectType -> (Light Tank, Battleship, Infantry, Unit Barracks, Wall)
#[derive(Bundle, Clone)]
pub struct ObjectMinimalBundle {
    pub object: Object,
    pub object_info: ObjectInfo,
    pub object_grid_position: ObjectGridPosition,
    pub object_stacking_class: ObjectStackingClass,
}

/// Base bundle that provides all functionality for all subsystems in the crate
#[derive(Bundle, Clone)]
pub struct ObjectCoreBundle {
    // items that are in the minimal bundle items first
    pub object: Object,
    pub object_info: ObjectInfo,
    pub object_grid_position: ObjectGridPosition,
    pub object_stacking_class: ObjectStackingClass,
    //
    //pub unit_movement_bundle: UnitMovementBundle,
}

/// Base bundle that provides all functionality for all subsystems in the crate
#[derive(Bundle, Clone)]
pub struct UnitBundle {
    // items that are in the minimal bundle items first
    pub object: Object,
    pub object_info: ObjectInfo,
    pub object_grid_position: ObjectGridPosition,
    pub object_stacking_class: ObjectStackingClass,

    //
    pub unit_movement_bundle: ObjectMovementBundle,
}

/// A resource inserted into the world to provide consistent unique ids to keep track of game
/// entities through potential spawns, despawns, and other shenanigans.
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Resource, Reflect, FromReflect)]
pub struct ObjectIdProvider {
    pub last_id: usize,
}

impl Default for ObjectIdProvider {
    fn default() -> Self {
        ObjectIdProvider { last_id: 0 }
    }
}

impl ObjectIdProvider {
    pub fn next_id_component(&mut self) -> ObjectId {
        ObjectId { id: self.next_id() }
    }

    pub fn next_id(&mut self) -> usize {
        self.last_id = self.last_id.saturating_add(1);
        self.last_id
    }

    pub fn remove_last_id(&mut self) {
        self.last_id = self.last_id.saturating_sub(1);
    }
}

/// Provides a way to track entities through potential despawns, spawns, and other shenanigans. Use
/// this to reference entities and then query for the entity that it is attached to.
#[derive(
    Default,
    Clone,
    Copy,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct ObjectId {
    pub id: usize,
}

///Marker component for an entity signifying it as an Object
#[derive(Default, Clone, Copy, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Object;

impl Object {
    // texture, tile_pos, stacking type, object type
    pub fn spawn() {}
}

/// Defines a new distinct ObjectClass. ObjectClass is used to represent the base class of an Object.
///
/// ## Example
/// (Ground, Air, Water, Building, etc)
/// ```rust
/// use bevy_ggf::object::ObjectClass;
///
/// pub const OBJECT_CLASS: &'static [ObjectClass] = &[
///     ObjectClass { name: String::from("Ground") },
///     ObjectClass { name: String::from("Air") },
///     ObjectClass { name: String::from("Water") },
///     ObjectClass { name: String::from("Building") },
/// ];
/// ```
#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct ObjectClass {
    pub name: String,
}

/// Defines a new distinct ObjectGroup. ObjectGroup is used to represent the group that an Object
/// "belongs" too.
///
/// ## Example
/// (Armor, Capital Ship, Helicopter, Factory, UnitProductionBuilding)
/// ```rust
/// use bevy_ggf::object::{ObjectClass, ObjectGroup};
///
/// pub const OBJECT_CLASS_GROUND: ObjectClass = ObjectClass{name: String::from("Ground")};
/// pub const OBJECT_GROUP_INFANTRY: ObjectGroup = ObjectGroup{name: String::from("Infantry"), object_class: OBJECT_CLASS_GROUND};
///
/// ```
#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct ObjectGroup {
    pub name: String,
    pub object_class: ObjectClass,
}

/// Defines a new distinct ObjectType. Each ObjectType should represent a distinct and unique type of
/// object.
/// Add the [`ObjectInfo`] wrapper component to an entity to use.
///
/// ## Example
/// Example items that might fall under this type and their potential trees
///
/// [`ObjectType`] / [`ObjectGroup`] / [`ObjectClass`]
/// - Rifleman < Infantry < Ground
/// - LightTank < Armor < Ground
/// - Battleship < Capital Ship < Water
/// - Submarine < Submersable < Water
/// - UnitBarracks < ProductionBuilding < Building
/// - Wall < Fortification < Building
///
/// ---
///
/// ```rust
/// use bevy_ggf::object::{ObjectClass, ObjectGroup, ObjectType};
///
/// // Declare an ObjectClass to use in our ObjectGroup
/// pub const OBJECT_CLASS_GROUND: ObjectClass = ObjectClass{name: String::from("Ground")};
///
/// // Declare an ObjectGroup using our ObjectClass
/// pub const OBJECT_GROUP_INFANTRY: ObjectGroup = ObjectGroup{name: String::from("Infantry"), object_class: OBJECT_CLASS_GROUND};
///
/// // Declare our ObjectType using the ObjectGroup which uses in itself the ObjectClass
/// pub const OBJECT_TYPE_RIFLEMAN: ObjectType = ObjectType{name: String::from("Rifleman"), object_group: OBJECT_GROUP_INFANTRY};
///
/// // We can access an ObjectTypes genealogy through it.
/// pub fn get_object_type_genealogy(){
///     let object_type_group = OBJECT_TYPE_RIFLEMAN.object_group;
///     let object_type_class = object_type_group.object_class;
/// }
///
/// ```
#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct ObjectType {
    pub name: String,
    pub object_group: ObjectGroup,
}

/// Holds a reference to a [`ObjectType`]. Is used to explicitly determine what an entity is from other
/// Objects and enable logic based on a specific object type. Use this with a distinct [`ObjectType`] to
/// define objects that might have exact same components and stats but you want different
#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    PartialEq,
    Debug,
    Component,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(Component)]
pub struct ObjectInfo {
    pub object_type: ObjectType,
}

/// Resource holding all [`ObjectType`]s that are used in the game
#[derive(Resource, Reflect, FromReflect)]
#[allow(dead_code)]
pub struct GameObjectInfo {
    object_classes: Vec<ObjectClass>,
    object_groups: Vec<ObjectGroup>,
    object_types: Vec<ObjectType>,
}

/// The position of the Object on the Tilemap.
#[derive(Default, Clone, Copy, Eq, Hash, PartialEq, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ObjectGridPosition {
    pub tile_position: TilePos,
}

// TODO: Implement building objects eventually
/// Allows this object to build other objects. Not currently implemented
#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Component,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(Component)]
struct Builder {
    pub can_build: Vec<ObjectType>,
}
