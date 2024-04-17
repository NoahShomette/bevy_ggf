//!

pub mod backend;
pub mod defaults;

use crate::game_core::command::{AddObjectToTile, GameCommand, GameCommands, RemoveObjectFromTile};
use crate::game_core::runner::GameRunner;
use crate::game_core::GameBuilder;
use crate::mapping::terrain::{TerrainClass, TerrainType, TileTerrainInfo};
use crate::mapping::MapId;
use crate::movement::backend::{MoveNode, MovementNodes};
use crate::object::{ObjectClass, ObjectGroup, ObjectId, ObjectInfo, ObjectType};
use crate::player::PlayerList;
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    info, App, Bundle, Component, Entity, EventWriter, Events, IntoSystemConfig,
    IntoSystemSetConfig, IntoSystemSetConfigs, Mut, Plugin, Query, Reflect, ReflectComponent,
    Resource, SystemSet, World,
};
use bevy::reflect::FromReflect;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapType};
use serde::{Deserialize, Serialize};

/// Core plugin for the bevy_ggf Movement System. Contains basic needed functionality.
/// Does not contain a MovementSystem. You have to insert that yourself
///
pub struct BggfMovementPlugin;

impl Plugin for BggfMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveEvent>().add_event::<MoveError>();
    }
}

impl Default for BggfMovementPlugin {
    fn default() -> Self {
        Self
    }
}

pub trait GameBuilderMovementExt {
    fn with_movement_calculator<MC>(
        &mut self,
        movement_calculator: MC,
        tile_move_checks: Vec<TileMoveCheckMeta>,
        map_type: TilemapType,
    ) where
        MC: MovementCalculator,
        Self: Sized;

    fn setup_movement(&mut self, tile_movement_costs: Vec<(TerrainType, TileMovementCosts)>)
    where
        Self: Sized;
}

impl<T: GameRunner + 'static> GameBuilderMovementExt for GameBuilder<T>
where
    T: GameRunner + 'static,
{
    fn with_movement_calculator<MC>(
        &mut self,
        movement_calculator: MC,
        tile_move_checks: Vec<TileMoveCheckMeta>,
        map_type: TilemapType,
    ) where
        MC: MovementCalculator,
        Self: Sized,
    {
        self.game_world.insert_resource(MovementSystem {
            movement_calculator: Box::new(movement_calculator),
            map_type,
            tile_move_checks: TileMoveChecks { tile_move_checks },
        });
    }

    fn setup_movement(&mut self, tile_movement_costs: Vec<(TerrainType, TileMovementCosts)>)
    where
        Self: Sized,
    {
        self.game_world
            .insert_resource(TerrainMovementCosts::from_vec(tile_movement_costs));

        self.game_world.init_resource::<Events<MoveEvent>>();
        self.game_world.init_resource::<Events<MoveError>>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum MovementSystems {
    Parallel,
    CommandFlush,
}

/// An extension trait for [GameCommands] with movement related commands.
pub trait MoveCommandsExt {
    fn move_object(
        &mut self,
        object_moving: ObjectId,
        on_map: MapId,
        current_pos: TilePos,
        new_pos: TilePos,
        attempt: bool,
    ) -> MoveObject;
}

impl MoveCommandsExt for GameCommands {
    /// Moves an object if the object has a [`CurrentMovementInformation`] struct and that contains
    /// the [`TilePos`] that the object is moving too
    fn move_object(
        &mut self,
        object_moving: ObjectId,
        on_map: MapId,
        current_pos: TilePos,
        new_pos: TilePos,
        attempt: bool,
    ) -> MoveObject {
        self.queue.push(MoveObject {
            object_moving,
            on_map,
            current_pos,
            new_pos,
            attempt,
        });
        MoveObject {
            object_moving,
            on_map,
            current_pos,
            new_pos,
            attempt,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct MoveObject {
    object_moving: ObjectId,
    on_map: MapId,
    current_pos: TilePos,
    new_pos: TilePos,
    attempt: bool,
}

impl GameCommand for MoveObject {
    fn execute(&mut self, mut world: &mut World) -> Result<(), String> {
        let mut remove = RemoveObjectFromTile {
            object_game_id: self.object_moving,
            on_map: self.on_map,
            tile_pos: self.current_pos,
        };
        let mut add = AddObjectToTile {
            object_game_id: self.object_moving,
            on_map: self.on_map,
            tile_pos: self.new_pos,
        };

        return match self.attempt {
            true => {
                let mut system_state: SystemState<Query<(Entity, &ObjectId)>> =
                    SystemState::new(&mut world);

                let mut object_query = system_state.get_mut(&mut world);

                let Some((entity, id)) = object_query
                    .iter_mut()
                    .find(|(_, id)| id == &&self.object_moving)
                else {
                    return Err(String::from("Objet not found"));
                };

                let mut moves: HashMap<TilePos, AvailableMove> = HashMap::new();

                world.resource_scope(|world, movement_system: Mut<MovementSystem>| {
                    let moves_info = movement_system.movement_calculator.calculate_move(
                        &movement_system.tile_move_checks,
                        movement_system.map_type,
                        self.on_map,
                        entity,
                        world,
                    );

                    for (tile_pos, move_node) in moves_info.move_nodes.iter() {
                        if move_node.valid_move {
                            moves.insert(*tile_pos, AvailableMove::from(*move_node));
                        }
                    }
                });

                if moves.contains_key(&self.new_pos) {
                    remove.execute(world)?;
                    add.execute(world)?;

                    let mut system_state: SystemState<EventWriter<MoveEvent>> =
                        SystemState::new(world);
                    let mut move_event = system_state.get_mut(world);

                    move_event.send(MoveEvent::MoveComplete {
                        object_moved: self.object_moving,
                    });

                    system_state.apply(world);
                    Ok(())
                } else {
                    info!("Tile_pos not a valid move");
                    Err(String::from("Tile_pos not a valid move"))
                }
            }
            false => {
                remove.execute(world)?;
                add.execute(world)?;

                let mut system_state: SystemState<EventWriter<MoveEvent>> = SystemState::new(world);
                let mut move_event = system_state.get_mut(world);

                move_event.send(MoveEvent::MoveComplete {
                    object_moved: self.object_moving,
                });

                system_state.apply(world);
                Ok(())
            }
        };
    }

    fn rollback(&mut self, world: &mut World) -> Result<(), String> {
        let mut remove = RemoveObjectFromTile {
            object_game_id: self.object_moving,
            on_map: self.on_map,
            tile_pos: self.new_pos,
        };
        let mut add = AddObjectToTile {
            object_game_id: self.object_moving,
            on_map: self.on_map,
            tile_pos: self.current_pos,
        };

        remove.execute(world)?;
        add.execute(world)?;

        return Ok(());
    }
}

/// Defines a MovementSystem. This resource is used to calculate movement, define the list of checks
/// for the [`MovementCalculator`], and holds the [`TilemapType`]
#[derive(Resource)]
pub struct MovementSystem {
    pub movement_calculator: Box<dyn MovementCalculator>,
    pub map_type: TilemapType,
    pub tile_move_checks: TileMoveChecks,
}

impl MovementSystem {
    /// Unused currently. Kept for future reference and potential implementation
    #[allow(dead_code)]
    fn new(
        map_type: TilemapType,
        movement_calculator: Box<dyn MovementCalculator>,
        tile_move_checks: Vec<TileMoveCheckMeta>,
    ) -> MovementSystem {
        MovementSystem {
            movement_calculator,
            map_type,
            tile_move_checks: TileMoveChecks { tile_move_checks },
        }
    }
    /// Unused currently. Kept for future reference and potential implementation
    #[allow(dead_code)]
    fn register_movement_system(
        app: &mut App,
        map_type: TilemapType,
        movement_calculator: Box<dyn MovementCalculator>,
        tile_move_checks: Vec<TileMoveCheckMeta>,
    ) {
        let movement_system = MovementSystem::new(map_type, movement_calculator, tile_move_checks);
        app.world.insert_resource(movement_system);
    }
}

/// A trait defining a new MovementCalculator - define the [`calculate_move`](MovementCalculator::calculate_move) fn in order to control
/// exactly how the movement works. Add this to a [`MovementSystem`] and insert that as a resource
/// to define your movement system
///
/// Bevy_GGF contains a series of default MovementCalculators, detailed in [`defaults`] including one
/// that implements Advance Wars style movement for square based maps called [`SquareMovementCalculator`](defaults::SquareMovementCalculator)
pub trait MovementCalculator: 'static + Send + Sync {
    /// The main function of a [`MovementCalculator`]. This is called when a [`MoveEvent`] is received
    /// and all [`MoveNode`](MoveNode) with valid_move marked true will be
    /// pushed into the [`CurrentMovementInformation`] Resource automatically. Use
    /// this function to define your own movement algorithm.
    fn calculate_move(
        &self,
        tile_move_checks: &TileMoveChecks,
        map_type: TilemapType,
        on_map: MapId,
        object_moving: Entity,
        world: &mut World,
    ) -> MovementNodes;
}

pub struct TileMoveChecks {
    pub tile_move_checks: Vec<TileMoveCheckMeta>,
}

impl TileMoveChecks {
    /// Helper function that will loop through each [`TileMoveCheck`] in the movement system and return
    /// false if any *one* was false, or true if all were true.
    pub fn check_tile_move_checks(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        tile_pos: &TilePos,
        last_tile_pos: &TilePos,
        world: &mut World,
    ) -> bool {
        for i in 0..self.tile_move_checks.len() {
            let check = self.tile_move_checks[i].check.as_ref();
            if !check.is_valid_move(entity_moving, tile_entity, tile_pos, last_tile_pos, world) {
                return false;
            }
        }
        true
    }
}

pub struct TileMoveCheckMeta {
    pub check: Box<dyn TileMoveCheck + Send + Sync>,
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
///         world: &mut World,
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
        world: &mut World,
    ) -> bool;
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub struct AvailableMove {
    pub tile_pos: TilePos,
    pub prior_tile_pos: TilePos,
    pub move_cost: i32,
}

impl From<MoveNode> for AvailableMove {
    /// Converts the MoveNode to AvailableMove. It will set move_cost to zero if the given move node
    /// does not have a move cost set.
    fn from(node: MoveNode) -> Self {
        AvailableMove {
            tile_pos: node.node_pos,
            prior_tile_pos: node.prior_node,
            move_cost: node.move_cost.unwrap_or(0),
        }
    }
}

/// A move event. Used to conduct actions related to object movement
/// - [Self::MoveBegin] represents starting a move. By default, this will run the [`handle_move_begin_events`]
/// which will calculate the available moves for the given unit.
/// - [Self::MoveCalculated] is intended to run after a move has been calculated with the current units
/// available moves. __This is not used currently. Instead available moves are pushed straight to the
/// [`CurrentMovementInformation`] resource.__
/// - [Self::TryMoveObject] is sent when you want to try to move an object to a specific tile. Send
/// the object thats trying to move and the tile you want it to move to. By default is handles by
/// [`handle_try_move_events`]
/// - [Self::MoveComplete] is sent if the [Self::TryMoveObject] event was successful.
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum MoveEvent {
    MoveBegin {
        object_moving: ObjectId,
        on_map: MapId,
    },
    MoveCalculated {
        available_moves: Vec<TilePos>,
    },
    TryMoveObject {
        object_moving: ObjectId,
        new_pos: TilePos,
    },
    MoveComplete {
        object_moved: ObjectId,
    },
}

/// An error that represents any MoveErrors
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum MoveError {
    InvalidMove(String),
}

impl Default for MoveError {
    fn default() -> Self {
        MoveError::InvalidMove(String::from("Invalid Move"))
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

/// A resource that holds all the [`MovementType`]s in the game. The types are stored in a hashmap
/// with the key being the name given to the MovementType
#[derive(Clone, Eq, PartialEq, Debug, Resource)]
pub struct MovementTypes {
    pub movement_types: HashMap<String, MovementType>,
}

impl MovementTypes {
    pub fn insert(&mut self, movement_type: MovementType) {
        self.movement_types
            .insert(movement_type.name.clone(), movement_type.clone());
    }

    pub fn insert_vec(&mut self, movement_types: Vec<MovementType>) {
        for movement_type in movement_types {
            self.movement_types
                .insert(movement_type.name.clone(), movement_type.clone());
        }
    }
}

/// Struct used to define a new [`MovementType`]. MovementType represents how a unit moves and is used
/// for movement costs chiefly
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
pub struct MovementType {
    pub name: String,
}

/// Component that must be added to a tile in order to define that tiles movement cost.
///
/// Contains a hashmap that holds a reference to a [`MovementType`] as a key and a u32 as the value. The u32 is used
/// in pathfinding as the cost to move into that tile.
#[derive(
    Default,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Component,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(Component)]
pub struct TileMovementCosts {
    pub movement_type_cost: HashMap<MovementType, u32>,
}

impl TileMovementCosts {
    /// Helper function to create a hashmap of TerrainType rules for Object Movement.
    pub fn new(rules: Vec<(MovementType, u32)>) -> TileMovementCosts {
        let mut hashmap: HashMap<MovementType, u32> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0.clone(), rule.1);
        }
        TileMovementCosts {
            movement_type_cost: hashmap,
        }
    }
    pub fn calculate_unit_move_cost(&self) {}
}

/// Defines a resource that will hold all [`TileMovementCosts`] related to TerrainTypes - references to a specific TileMovementCosts
/// are stored in each tile as their current cost using the [`TileMovementCosts`] component.
#[derive(Resource, Default, Debug)]
pub struct TerrainMovementCosts {
    pub movement_cost_rules: HashMap<TerrainType, TileMovementCosts>,
}

impl TerrainMovementCosts {
    /// Creates a new [`TerrainMovementCosts`] struct from a vec of [`TerrainType`] and [`TileMovementCosts`]
    pub fn from_vec(
        terrain_movement_costs: Vec<(TerrainType, TileMovementCosts)>,
    ) -> TerrainMovementCosts {
        let mut hashmap: HashMap<TerrainType, TileMovementCosts> = HashMap::new();
        for (terrain_type, tile_movement_costs) in terrain_movement_costs {
            hashmap.insert(terrain_type, tile_movement_costs);
        }

        Self {
            movement_cost_rules: hashmap,
        }
    }
}

// UNIT MOVEMENT STUFF

/// Basic Bundle that supplies all required movement components for an object
#[derive(Bundle, Clone)]
pub struct ObjectMovementBundle {
    pub object_movement: ObjectMovement,
}

/// Component that allows an object to move. Defines three things:
///
/// **move_points** - Represents how far an object can move.
/// **movement_type - what kinda MovementType that the attached object uses
/// **object_terrain_movement_rules** - defines a list of rules based on TerrainType and TerrainClass
/// that the object follows. If you want to declare movement rules based on the type of object that
/// is in the tile that is getting checked, use ObjectTypeMovementRules
#[derive(
    Default,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Component,
    Reflect,
    FromReflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(Component)]
pub struct ObjectMovement {
    pub move_points: i32,
    pub movement_type: MovementType,
    pub object_terrain_movement_rules: ObjectTerrainMovementRules,
}

/// Optional component that can be attached to an object to define rules related to that objects movement
/// on other objects. Eg, allowing objects to move over water using bridges. In this situation bridges
/// would be another object.
///
/// The order is [`ObjectType`] > [`ObjectGroup`] > [`ObjectClass`]
///
/// # NOTE
/// These rules override [`ObjectTerrainMovementRules`].
///
/// If using the built in [`MoveCheckAllowedTile`](defaults::MoveCheckAllowedTile) implementation,
/// these rules ignore [`ObjectStackingClass`](crate::mapping::tiles::ObjectStackingClass).
///
#[derive(Default, Clone, Eq, PartialEq, Debug, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct ObjectTypeMovementRules {
    object_class_rules: HashMap<ObjectClass, bool>,
    object_group_rules: HashMap<ObjectGroup, bool>,
    object_type_rules: HashMap<ObjectType, bool>,
}

impl ObjectTypeMovementRules {
    /// Creates a new [`ObjectTypeMovementRules`] from the three provided vecs holding a tuple containing
    /// one each of the following, [`ObjectClass`], [`ObjectGroup`], [`ObjectType`], and a bool. The
    /// bool fed in with the corresponding object info controls whether the object that has this component
    /// is allowed to move onto the given tile.
    pub fn new(
        object_class_rules: Vec<(ObjectClass, bool)>,
        object_group_rules: Vec<(ObjectGroup, bool)>,
        object_type_rules: Vec<(ObjectType, bool)>,
    ) -> ObjectTypeMovementRules {
        ObjectTypeMovementRules {
            object_class_rules: ObjectTypeMovementRules::new_class_rules_hashmaps(
                object_class_rules.clone(),
            ),
            object_group_rules: ObjectTypeMovementRules::new_group_rules_hashmaps(
                object_group_rules.clone(),
            ),
            object_type_rules: ObjectTypeMovementRules::new_type_rules_hashmaps(object_type_rules),
        }
    }

    /// Returns an option if there is a rule for any of the object type, group, or class given.
    ///
    /// # Logic
    /// It checks each set of rules for a match to the object information in the given [`ObjectInfo`].
    /// If one is found, it returns the bool associated with it. If none are found it returns None.
    ///
    /// The order is [`ObjectType`] > [`ObjectGroup`] > [`ObjectClass`] - returning on the first rule
    /// found. Therefore you can use the layers of an object type to grant increasing specificity for
    /// what other objects an object can walk on.
    ///
    pub fn can_move_on_tile(&self, object_info: &ObjectInfo) -> Option<bool> {
        if let Some(rule) = self.object_type_rules.get(&object_info.object_type) {
            return Some(*rule);
        }
        if let Some(rule) = self
            .object_group_rules
            .get(&object_info.object_type.object_group)
        {
            return Some(*rule);
        }
        if let Some(rule) = self
            .object_class_rules
            .get(&object_info.object_type.object_group.object_class)
        {
            return Some(*rule);
        }

        None
    }

    /// Helper function to create a hashmap of [`ObjectType`] rules for Object Movement.
    pub fn new_type_rules_hashmaps(
        type_rules: Vec<(ObjectType, bool)>,
    ) -> HashMap<ObjectType, bool> {
        let mut type_hashmap: HashMap<ObjectType, bool> = HashMap::new();

        for rule in type_rules.iter() {
            type_hashmap.insert(rule.0.clone(), rule.1);
        }
        type_hashmap
    }

    /// Helper function to create a hashmap of [`ObjectGroup`] rules for Object Movement.
    pub fn new_group_rules_hashmaps(
        group_rules: Vec<(ObjectGroup, bool)>,
    ) -> HashMap<ObjectGroup, bool> {
        let mut group_hashmap: HashMap<ObjectGroup, bool> = HashMap::new();

        for rule in group_rules.iter() {
            group_hashmap.insert(rule.0.clone(), rule.1);
        }
        group_hashmap
    }

    /// Helper function to create a hashmap of [`ObjectClass`] rules for Object Movement.
    pub fn new_class_rules_hashmaps(
        class_rules: Vec<(ObjectClass, bool)>,
    ) -> HashMap<ObjectClass, bool> {
        let mut class_hashmap: HashMap<ObjectClass, bool> = HashMap::new();

        for rule in class_rules.iter() {
            class_hashmap.insert(rule.0.clone(), rule.1);
        }

        class_hashmap
    }
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
#[derive(
    Default, Clone, Eq, PartialEq, Debug, Reflect, FromReflect, serde::Deserialize, serde::Serialize,
)]
pub struct ObjectTerrainMovementRules {
    terrain_class_rules: Vec<TerrainClass>,
    terrain_type_rules: HashMap<TerrainType, bool>,
}

impl ObjectTerrainMovementRules {
    /// Creates a new [`ObjectTerrainMovementRules`] from the provided [`TerrainClass`] vec and [`TerrainType`] rules
    pub fn new(
        terrain_classes: Vec<TerrainClass>,
        terrain_type_rules: Vec<(TerrainType, bool)>,
    ) -> ObjectTerrainMovementRules {
        ObjectTerrainMovementRules {
            terrain_class_rules: terrain_classes,
            terrain_type_rules: ObjectTerrainMovementRules::new_terrain_type_rules(
                terrain_type_rules,
            ),
        }
    }

    /// Returns true if the object can move onto the given tiles terrain. Returns false if it cannot
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
            .contains(&&tile_terrain_info.terrain_type.terrain_class)
    }

    /// Helper function to create a hashmap of [`TerrainType`] rules for Object Movement.
    pub fn new_terrain_type_rules(rules: Vec<(TerrainType, bool)>) -> HashMap<TerrainType, bool> {
        let mut hashmap: HashMap<TerrainType, bool> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0.clone(), rule.1);
        }
        hashmap
    }
}

#[test]
fn test_terrain_rules() {
    let TERRAIN_CLASSES: Vec<TerrainClass> = vec![
        TerrainClass {
            name: String::from("Ground"),
        },
        TerrainClass {
            name: String::from("Water"),
        },
    ];

    let TERRAIN_TYPES: Vec<TerrainType> = vec![
        TerrainType {
            name: String::from("Grassland"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TerrainType {
            name: String::from("Forest"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TerrainType {
            name: String::from("Mountain"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
    ];
    let movement_rules = ObjectTerrainMovementRules::new(
        vec![TERRAIN_CLASSES[0].clone(), TERRAIN_CLASSES[1].clone()],
        vec![(TERRAIN_TYPES[2].clone(), false)],
    );

    let tile_terrain_info = TileTerrainInfo {
        terrain_type: TERRAIN_TYPES[2].clone(),
    };

    // this expression should be negative because in the given ObjectTerrainMovementRules TERRAIN_TYPES[2]
    // is set to false
    assert_eq!(movement_rules.can_move_on_tile(&tile_terrain_info), false);
}

//TODO: When we have some form of scheduling, make this go away by default at the beginning of the
// players turn
/// Marker component signifying that the unit has moved and cannot move anymore
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct ObjectMoved;
