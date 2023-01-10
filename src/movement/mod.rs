pub mod backend;
pub mod defaults;

use crate::mapping::terrain::{TerrainClass, TerrainType, TileTerrainInfo};
use crate::movement::backend::{
    handle_move_begin_events, handle_try_move_events, MoveNode, MovementNodes,
};
use bevy::prelude::{
    App, Bundle, Component, CoreStage, Entity, IntoSystemDescriptor, Plugin, Res, Resource, World,
};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapType};

/// Core plugin for the bevy_ggf Movement System. Contains basic needed functionality.
/// Does not contain a MovementSystem. You have to insert that yourself
///
pub struct BggfMovementPlugin;

impl Plugin for BggfMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMovementRules>()
            .init_resource::<CurrentMovementInformation>()
            .add_event::<MoveEvent>()
            .add_event::<MoveError>()
            .add_system_to_stage(CoreStage::PostUpdate, handle_move_begin_events.at_end())
            .add_system(handle_try_move_events);
    }
}

/// Defines a MovementSystem. This resource is used to calculate movement, define the list of checks
/// for the [`MovementCalculator`], and holds the [`TilemapType`]
#[derive(Resource)]
pub struct MovementSystem {
    pub movement_calculator: Box<dyn MovementCalculator>,
    pub map_type: TilemapType,
    pub tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
}

impl MovementSystem {
    fn new(
        map_type: TilemapType,
        movement_calculator: Box<dyn MovementCalculator>,
        tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
    ) -> MovementSystem {
        MovementSystem {
            movement_calculator,
            map_type,
            tile_move_checks,
        }
    }

    fn register_movement_system(
        app: &mut App,
        map_type: TilemapType,
        movement_calculator: Box<dyn MovementCalculator>,
        tile_move_checks: Vec<Box<dyn TileMoveCheck + Send + Sync>>,
    ) {
        let movement_system = MovementSystem::new(map_type, movement_calculator, tile_move_checks);
        app.world.insert_resource(movement_system);
    }
}

/// A trait defining a new MovementCalculator - define the [`calculate_move`] fn in order to control
/// exactly how the movement works. Add this to a [`MovementSystem`] and insert that as a resource
/// to define your movement system
///
/// Bevy_GGF contains a MovementCalculator for Square based maps called [`SquareMovementSystem`]
pub trait MovementCalculator: 'static + Send + Sync {
    /// The main function of a [`MovementCalculator`]. This is called when a [`MoveEvent`] is received
    /// and the result is pushed into the [`CurrentMovementInformation`] Resource automatically. Use
    /// this function to define your own movement algorithm.
    fn calculate_move(
        &self,
        movement_system: &Res<MovementSystem>,
        object_moving: &Entity,
        world: &World,
    ) -> (Vec<TilePos>, MovementNodes);
}

/// A trait used to define a new check for a tile in a [`MovementCalculator`]s pathfinding algorithm.
/// Implement one of these traits for each separate logical check you want the MovementCalculator to
/// do to determine if a tile is a valid move or not.
///
/// # Example
/// Here is an example of a simple TileMoveCheck implementation. This impl provides a check for whether
/// or not a tile has space in the tile for the relevant objects stacking class
/// ```rust
/// use bevy::prelude::{Entity, World};
/// use bevy_ecs_tilemap::prelude::TilePos;
/// use bevy_ggf::mapping::tiles::{ObjectStackingClass, TileObjectStacks};
/// use bevy_ggf::movement::TileMoveCheck;
///
/// // Create a new struct for our TileMoveCheck
/// pub struct MoveCheckSpace;
///
/// // Implement the TileMoveCheck Trait for our struct
/// impl TileMoveCheck for MoveCheckSpace {
///     fn is_valid_move(
///         &self,
///         entity_moving: Entity,
///         tile_entity: Entity,
///         tile_pos: &TilePos,
///         last_tile_pos: &TilePos,
///         world: &World,
///     ) -> bool {
/// // Get the ObjectStackingClass component of our object that is trying to move
///         let Some(object_stack_class) = world.get::<ObjectStackingClass>(entity_moving) else {
/// // If the object doesnt have a stack class then we want to return false as this object should not be able to move
///             return false;
///         };
/// // Get the TileObjectStacks component of the tile that we are checking
///         let Some(tile_objects) = world.get::<TileObjectStacks>(tile_entity) else {
///             return false;
///         };
/// // Use the built in function on a TileObjectStacks struct to check if the tile has space for this objects stacking class
/// // If there is space then this object can move into the tile and we return true
/// // Else there is no space and we return false instead
///         tile_objects.has_space(object_stack_class)
///     }
/// }
/// ```
pub trait TileMoveCheck {
    fn is_valid_move(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        tile_pos: &TilePos,
        last_tile_pos: &TilePos,
        world: &World,
    ) -> bool;
}

/// Resource that holds the TilePos of any available moves and the move nodes of whatever the [`calculate_move`]
/// function created
#[derive(Clone, Eq, PartialEq, Default, Debug, Resource)]
pub struct CurrentMovementInformation {
    pub available_moves: HashMap<TilePos, AvailableMove>,
}

impl CurrentMovementInformation {
    /// Returns true or false if CurrentMovementInformation contains a move at the assigned TilePos
    pub fn contains_move(&self, new_pos: &TilePos) -> bool {
        self.available_moves.contains_key(new_pos)
    }

    pub fn clear_information(&mut self) {
        self.available_moves.clear();
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub struct AvailableMove {
    pub tile_pos: TilePos,
    pub prior_tile_pos: TilePos,
    pub move_cost: i32,
}

/// A move event. Used to conduct actions related to object movement
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum MoveEvent {
    MoveBegin {
        object_moving: Entity,
    },
    MoveCalculated {
        available_moves: Vec<TilePos>,
    },
    TryMoveObject {
        object_moving: Entity,
        new_pos: TilePos,
    },
    MoveComplete {
        object_moved: Entity,
    },
}

/// An error that represents any MoveErrors
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum MoveError {
    NotValidMove(String),
}

impl Default for MoveError {
    fn default() -> Self {
        MoveError::NotValidMove(String::from("Invalid Move"))
    }
}

/// Simple enum to represent whether Diagonal Movement is enabled or disabled
#[derive(Clone, Eq, Hash, PartialEq, Default)]
pub enum DiagonalMovement {
    Enabled,
    #[default]
    Disabled,
}

impl DiagonalMovement {
    /// Returns true or false depending on whether Diagonal Movement is enabled or disabled
    pub fn is_diagonal(&self) -> bool {
        match self {
            DiagonalMovement::Enabled => true,
            DiagonalMovement::Disabled => false,
        }
    }
}

/// Struct used to define a new [`MovementType`]. MovementType represents how a unit moves and is used
/// for movement costs chiefly
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct MovementType {
    pub name: &'static str,
}

/// Component that must be added to a tile in order to define that tiles movement cost.
///
/// Contains a hashmap that holds a reference to a [`MovementType`] as a key and a u32 as the value. The u32 is used
/// in pathfinding as the cost to move into that tile.
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub struct TileMovementCosts {
    pub movement_type_cost: HashMap<&'static MovementType, u32>,
}

impl TileMovementCosts {
    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn new(rules: Vec<(&'static MovementType, u32)>) -> TileMovementCosts {
        let mut hashmap: HashMap<&'static MovementType, u32> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0, rule.1);
        }
        TileMovementCosts {
            movement_type_cost: hashmap,
        }
    }
    pub fn calculate_unit_move_cost(&self) {}
}

/// Defines a resource that will hold all [`TileMovementCosts`] - references to a specific TileMovementCosts
/// are stored in each tile as their current cost.
#[derive(Resource, Default, Debug)]
pub struct TileMovementRules {
    pub movement_cost_rules: HashMap<TerrainType, TileMovementCosts>,
}

// UNIT MOVEMENT STUFF

/// Basic Bundle that supplies all needed movement components for a unit
#[derive(Bundle)]
pub struct UnitMovementBundle {
    pub object_movement: ObjectMovement,
}

/// Required for an Object to move. Without this an object is unable to move.
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub struct ObjectMovement {
    pub move_points: i32,
    pub movement_type: &'static MovementType,
    pub object_terrain_movement_rules: ObjectTerrainMovementRules,
}

/// Defines what type of terrain an object can move onto. Place into an [`ObjectMovement`] component to
/// define what tiles the object can move into
///
/// # Logic
/// The rules are evaluated in a two step process. terrain_type_rules first, and then terrain_class_rules second
///
/// - terrain_type_rules should be considered an exception to terrain_class_rules and only used if you want to
/// allow or deny specific [`TerrainType`]s. Whatever bool you set for that specific [`TerrainType`] controls
/// whether that tile is a valid move tile or not. Rules in this will be followed over any TerrainClass
/// rules.
/// - terrain_class_rules should be the first option used when assigning what terrain an object can
/// move on and only using terrain_type_rules if you need to make an exception. Every [`TerrainClass`]
/// added to terrain_class_rules denotes that the object can move onto any TerrainTypes that has a reference
/// to that TerrainClass.
///
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ObjectTerrainMovementRules {
    pub terrain_class_rules: Vec<&'static TerrainClass>,
    pub terrain_type_rules: HashMap<&'static TerrainType, bool>,
}

impl ObjectTerrainMovementRules {
    /// Creates a new [`ObjectTerrainMovementRules`] from the provided [`TerrainClass`] vec and [`TerrainType`] rules
    pub fn new(
        terrain_classes: Vec<&'static TerrainClass>,
        terrain_type_rules: Vec<(&'static TerrainType, bool)>,
    ) -> ObjectTerrainMovementRules {
        ObjectTerrainMovementRules {
            terrain_class_rules: terrain_classes,
            terrain_type_rules: ObjectTerrainMovementRules::new_terrain_type_rules(
                terrain_type_rules,
            ),
        }
    }

    /// Returns true if the object can move onto that tiles terrain. Returns false if it cannot
    ///
    /// # Logic
    /// It checks self.terrain_type_rules for a rule for the tiles [`TerrainType`]. If it finds a rule
    /// it returns that directly. If it doesn't find a rule it checks if self.terrain_class_rules
    /// contains a reference to the tiles [`TerrainClass`]. If it does then it returns true. Else
    /// it returns false.
    pub fn can_move_on_tile(&self, tile_terrain_info: &TileTerrainInfo) -> bool {
        if let Some(terrain_type_rule) =
            self.terrain_type_rules.get(&tile_terrain_info.terrain_type)
        {
            return *terrain_type_rule;
        }

        self.terrain_class_rules
            .contains(&tile_terrain_info.terrain_type.terrain_class)
    }

    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn new_terrain_type_rules(
        rules: Vec<(&'static TerrainType, bool)>,
    ) -> HashMap<&'static TerrainType, bool> {
        let mut hashmap: HashMap<&'static TerrainType, bool> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0, rule.1);
        }
        hashmap
    }

    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn add_terrain_class_rule(&mut self, rule: &'static TerrainClass) {
        self.terrain_class_rules.push(rule);
    }
}

#[test]
fn test_terrain_rules() {
    const TERRAIN_CLASSES: &'static [TerrainClass] = &[
        TerrainClass { name: "Ground" },
        TerrainClass { name: "Water" },
    ];

    const TERRAIN_TYPES: &'static [TerrainType] = &[
        TerrainType {
            name: "Grassland",
            texture_index: 0,
            terrain_class: &TERRAIN_CLASSES[0],
        },
        TerrainType {
            name: "Forest",
            texture_index: 1,
            terrain_class: &TERRAIN_CLASSES[0],
        },
        TerrainType {
            name: "Mountain",
            texture_index: 2,
            terrain_class: &TERRAIN_CLASSES[0],
        },
    ];
    let movement_rules = ObjectTerrainMovementRules::new(
        vec![&TERRAIN_CLASSES[0], &TERRAIN_CLASSES[1]],
        vec![(&TERRAIN_TYPES[2], false)],
    );

    let tile_terrain_info = TileTerrainInfo {
        terrain_type: TERRAIN_TYPES[2],
    };

    // this expression should be negative because in the given ObjectTerrainMovementRules TERRAIN_TYPES[2] is set to false
    assert_eq!(movement_rules.can_move_on_tile(&tile_terrain_info), false);
}

/// Marker component signifying that the unit has moved and cannot move anymore
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct ObjectMoved;
