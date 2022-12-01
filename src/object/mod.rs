use crate::movement::UnitMovementBundle;
use crate::selection::SelectableEntity;
use bevy::prelude::{Bundle, Component, Entity, Resource};

// ObjectClass -> (Ground, Air, Water, Building, etc)
// ObjectGroup -> (Armor, Capital Ship, Helicopter)
// ObjectType -> (Light Tank, Battleship, Infantry, Unit Barracks, Wall)

/// Base bundle that provides all functionality for all subsystems in the crate
#[derive(Bundle)]
pub struct ObjectBundle {
    object_info: ObjectInfo,
    selectable: SelectableEntity,

    unit_movement_bundle: UnitMovementBundle,
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
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
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
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
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
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ObjectType {
    pub name: &'static str,
    pub object_group: &'static ObjectGroup,
}

/// Holds a reference to a [`ObjectType`]. Is used to explicitly determine what an entity is from other
/// Objects and enable logic based on a specific object type. Use this with a distinct [`ObjectType`] to
/// define objects that might have exact same components and stats but you want different
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct ObjectInfo {
    object_type: &'static ObjectType,
}

/// Resource holding all [`UnitType`]s that are used in the game
#[derive(Resource)]
pub struct GameObjectInfo {
    object_classes: Vec<ObjectClass>,
    object_groups: Vec<ObjectGroup>,
    object_types: Vec<ObjectType>,
}

#[derive(Component)]
pub struct TileObjectStackLimits<T> {
    stack_limit: u32,
    object_class_limited: T,
}

pub struct TileObjectEntities {
    pub entities_in_tile: Vec<Entity>,
}
