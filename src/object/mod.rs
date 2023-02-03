//! bevy_ggf contains a built in method to represent units, buildings, and anything else that is not
//! a tile and resides on the map. This system is built on top of Bevy_ECS and is based on the entity
//! component system.

use crate::mapping::tiles::ObjectStackingClass;
use crate::movement::ObjectMovementBundle;
use crate::selection::SelectableEntity;
use bevy::prelude::{Bundle, Component, Resource, SpriteBundle};
use bevy_ecs_tilemap::prelude::TilePos;

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
#[derive(Bundle)]
pub struct ObjectMinimalBundle {
    pub object: Object,
    pub object_info: ObjectInfo,
    pub selectable: SelectableEntity,
    pub object_grid_position: ObjectGridPosition,
    pub object_stacking_class: ObjectStackingClass,
}

/// Base bundle that provides all functionality for all subsystems in the crate
#[derive(Bundle)]
pub struct ObjectCoreBundle {
    // items that are in the minimal bundle items first
    pub object: Object,
    pub object_info: ObjectInfo,
    pub selectable: SelectableEntity,
    pub object_grid_position: ObjectGridPosition,
    pub object_stacking_class: ObjectStackingClass,

    //
    pub sprite_bundle: SpriteBundle,
    //pub unit_movement_bundle: UnitMovementBundle,
}

/// Base bundle that provides all functionality for all subsystems in the crate
#[derive(Bundle)]
pub struct UnitBundle {
    // items that are in the minimal bundle items first
    pub object: Object,
    pub object_info: ObjectInfo,
    pub selectable: SelectableEntity,
    pub object_grid_position: ObjectGridPosition,
    pub object_stacking_class: ObjectStackingClass,

    //
    pub sprite_bundle: SpriteBundle,
    pub unit_movement_bundle: ObjectMovementBundle,
}

///Marker component for an entity signifying it as an Object
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
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
///     ObjectClass { name: "Ground" },
///     ObjectClass { name: "Air" },
///     ObjectClass { name: "Water" },
///     ObjectClass { name: "Building" },
/// ];
/// ```
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub struct ObjectClass {
    pub name: &'static str,
}

/// Defines a new distinct ObjectGroup. ObjectGroup is used to represent the group that an Object
/// "belongs" too.
///
/// ## Example
/// (Armor, Capital Ship, Helicopter, Factory, UnitProductionBuilding)
/// ```rust
/// use bevy_ggf::object::{ObjectClass, ObjectGroup};
///
/// pub const OBJECT_CLASS_GROUND: ObjectClass = ObjectClass{name: "Ground"};
/// pub const OBJECT_GROUP_INFANTRY: ObjectGroup = ObjectGroup{name: "Infantry", object_class: &OBJECT_CLASS_GROUND};
///
/// ```
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub struct ObjectGroup {
    pub name: &'static str,
    pub object_class: &'static ObjectClass,
}

/// Defines a new distinct ObjectType. ObjectType is
/// used to explicitly differentiate different types of objects and enable logic based on an objects type.
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
/// pub const OBJECT_CLASS_GROUND: ObjectClass = ObjectClass{name: "Ground"};
///
/// // Declare an ObjectGroup using our ObjectClass
/// pub const OBJECT_GROUP_INFANTRY: ObjectGroup = ObjectGroup{name: "Infantry", object_class: &OBJECT_CLASS_GROUND};
///
/// // Declare our ObjectType using the ObjectGroup which uses in itself the ObjectClass
/// pub const OBJECT_TYPE_RIFLEMAN: ObjectType = ObjectType{name: "Rifleman", object_group: &OBJECT_GROUP_INFANTRY};
///
/// // We can access an ObjectTypes genealogy through it.
/// pub fn get_object_type_genealogy(){
///     let object_type_group = OBJECT_TYPE_RIFLEMAN.object_group;
///     let object_type_class = object_type_group.object_class;
/// }
///
/// ```
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub struct ObjectType {
    pub name: &'static str,
    pub object_group: &'static ObjectGroup,
}

/// Holds a reference to a [`ObjectType`]. Is used to explicitly determine what an entity is from other
/// Objects and enable logic based on a specific object type. Use this with a distinct [`ObjectType`] to
/// define objects that might have exact same components and stats but you want different
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Component)]
pub struct ObjectInfo {
    pub object_type: &'static ObjectType,
}

/// Resource holding all [`ObjectType`]s that are used in the game
#[derive(Resource)]
#[allow(dead_code)]
pub struct GameObjectInfo {
    object_classes: Vec<ObjectClass>,
    object_groups: Vec<ObjectGroup>,
    object_types: Vec<ObjectType>,
}

/// The position of the Object on the Tilemap.
#[derive(Component)]
pub struct ObjectGridPosition {
    pub tile_position: TilePos,
}
