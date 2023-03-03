//!

pub mod backend;
pub mod defaults;

use crate::game::command::{AddObjectToTile, GameCommand, GameCommands, RemoveObjectFromTile};
use crate::game::GameId;
use crate::mapping::terrain::{TerrainClass, TerrainType, TileTerrainInfo};
use crate::movement::backend::{
    add_object_moved_component_on_moves, handle_move_begin_events, MoveNode, MovementNodes,
};
use crate::movement::MoveEvent::TryMoveObject;
use crate::object::{ObjectClass, ObjectGroup, ObjectInfo, ObjectType};
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    info, App, Bundle, Component, CoreStage, Entity, EventReader, EventWriter,
    IntoSystemDescriptor, Plugin, Query, Res, Resource, World,
};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapType};

/// Core plugin for the bevy_ggf Movement System. Contains basic needed functionality.
/// Does not contain a MovementSystem. You have to insert that yourself
///
pub struct BggfMovementPlugin {
    pub add_defaults_core: bool,
    pub add_defaults_extra: bool,
}

impl Plugin for BggfMovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainMovementCosts>()
            .add_event::<ClearObjectAvailableMoves>()
            .add_event::<MoveEvent>()
            .add_event::<MoveError>();
        if self.add_defaults_core {
            app.add_system_to_stage(CoreStage::PostUpdate, handle_move_begin_events.at_end());
        }
        if self.add_defaults_extra {
            app.add_system(add_object_moved_component_on_moves);
        }
    }
}

impl Default for BggfMovementPlugin {
    fn default() -> Self {
        Self {
            add_defaults_core: true,
            add_defaults_extra: true,
        }
    }
}

/// An extension trait for [GameCommands] with movement related commands.
pub trait MoveCommandsExt {
    fn move_object(
        &mut self,
        object_moving: GameId,
        on_map: GameId,
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
        object_moving: GameId,
        on_map: GameId,
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

#[derive(Clone, Debug)]
pub struct MoveObject {
    object_moving: GameId,
    on_map: GameId,
    current_pos: TilePos,
    new_pos: TilePos,
    attempt: bool,
}

impl GameCommand for MoveObject {
    fn execute(
        &mut self,
        mut world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
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
                let mut system_state: SystemState<Query<(Entity, &GameId)>> =
                    SystemState::new(&mut world);

                let mut object_query = system_state.get_mut(&mut world);

                let Some((entity, id)) = object_query
                    .iter_mut()
                    .find(|(_, id)| id == &&self.object_moving);

                if let Some(movement_information) = world.get::<CurrentMovementInformation>(entity)
                {
                    return if movement_information.contains_move(&self.new_pos) {
                        remove.execute(world)?;
                        add.execute(world)?;

                        let mut system_state: SystemState<EventWriter<MoveEvent>> =
                            SystemState::new(world);
                        let mut move_event = system_state.get_mut(world);

                        move_event.send(MoveEvent::MoveComplete {
                            object_moved: self.object_moving,
                        });

                        system_state.apply(world);
                        return Ok(None);
                    } else {
                        info!("TilePos not in movement info");
                        Err(String::from("Tile_pos not in movement information"))
                    };
                } else {
                    info!("Object has no movemement information");
                    Err(String::from("Object has no movemement information"))
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
                return Ok(None);
            }
        };
    }

    fn rollback(
        &mut self,
        world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
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

        return Ok(Some(Box::new(MoveObject {
            object_moving: self.object_moving,
            on_map: self.on_map,
            current_pos: self.current_pos,
            new_pos: self.new_pos,
            attempt: false,
        })));
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
    /// Helper function that will loop through each [`TileMoveCheck`] in the movement system and return
    /// false if any *one* was false, or true if all were true.
    pub fn check_tile_move_checks(
        &self,
        entity_moving: Entity,
        tile_entity: Entity,
        tile_pos: &TilePos,
        last_tile_pos: &TilePos,
        world: &World,
    ) -> bool {
        for i in 0..self.tile_move_checks.len() {
            let check = self.tile_move_checks[i].as_ref();
            if !check.is_valid_move(entity_moving, tile_entity, tile_pos, last_tile_pos, world) {
                return false;
            }
        }
        true
    }

    /// Unused currently. Kept for future reference and potential implementation
    #[allow(dead_code)]
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
    /// Unused currently. Kept for future reference and potential implementation
    #[allow(dead_code)]
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

/// A trait defining a new MovementCalculator - define the [`calculate_move`](MovementCalculator::calculate_move) fn in order to control
/// exactly how the movement works. Add this to a [`MovementSystem`] and insert that as a resource
/// to define your movement system
///
/// Bevy_GGF contains a series of default MovementCalculators, detailed in [`defaults`] including one
/// that implements Advance Wars style movement for square based maps called [`SquareMovementCalculator`](defaults::SquareMovementCalculator)
pub trait MovementCalculator: 'static + Send + Sync {
    /// The main function of a [`MovementCalculator`]. This is called when a [`MoveEvent`] is received
    /// and all [`MoveNode`](backend::MoveNode) with valid_move marked true will be
    /// pushed into the [`CurrentMovementInformation`] Resource automatically. Use
    /// this function to define your own movement algorithm.
    fn calculate_move(
        &self,
        movement_system: &Res<MovementSystem>,
        object_moving: &Entity,
        world: &World,
    ) -> MovementNodes;
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
/// use bevy_ggf::mapping::tiles::{ObjectStackingClass, TileObjectStackingRules};
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
///         let Some(tile_objects) = world.get::<TileObjectStackingRules>(tile_entity) else {
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

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub struct AvailableMove {
    pub tile_pos: TilePos,
    pub prior_tile_pos: TilePos,
    pub move_cost: i32,
}

impl From<MoveNode> for AvailableMove {
    /// Converts the MoveNode to AvailableMove. It will panic if the given MoveNode does not have
    /// a move_cost.
    fn from(node: MoveNode) -> Self {
        AvailableMove {
            tile_pos: node.node_pos,
            prior_tile_pos: node.prior_node,
            move_cost: node.move_cost.expect("move_cost cannot be None"),
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
        object_moving: GameId,
    },
    MoveCalculated {
        available_moves: Vec<TilePos>,
    },
    TryMoveObject {
        object_moving: GameId,
        new_pos: TilePos,
    },
    MoveComplete {
        object_moved: GameId,
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

/// Defines a resource that will hold all [`TileMovementCosts`] related to TerrainTypes - references to a specific TileMovementCosts
/// are stored in each tile as their current cost using the [`TileMovementCosts`] component.
#[derive(Resource, Default, Debug)]
pub struct TerrainMovementCosts {
    pub movement_cost_rules: HashMap<TerrainType, TileMovementCosts>,
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
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub struct ObjectMovement {
    pub move_points: i32,
    pub movement_type: &'static MovementType,
    pub object_terrain_movement_rules: ObjectTerrainMovementRules,
}

//TODO: Update this to just a guaranteed ordered vec - ordered meaning the first element
/// Resource that holds a Hashmap of [`AvailableMove`] structs. These structs should represent verified
/// valid moves and are updated when the [`MoveEvent::MoveBegin`] event is processed.
#[derive(Clone, Eq, PartialEq, Default, Debug, Component)]
pub struct CurrentMovementInformation {
    pub available_moves: HashMap<TilePos, AvailableMove>,
}

impl CurrentMovementInformation {
    /// Returns true or false if CurrentMovementInformation contains a move at the assigned TilePos
    pub fn contains_move(&self, new_pos: &TilePos) -> bool {
        self.available_moves.contains_key(new_pos)
    }

    /// Clears the moves from the collection
    pub fn clear_moves(&mut self) {
        self.available_moves.clear();
    }
}

pub struct ClearObjectAvailableMoves {
    object: Entity,
}

fn clear_selected_object(
    mut clear_object_reader: EventReader<ClearObjectAvailableMoves>,
    mut current_movement_information: Query<&mut CurrentMovementInformation>,
) {
    for event in clear_object_reader.iter() {
        if let Ok(mut info) = current_movement_information.get_mut(event.object) {
            info.clear_moves();
        }
    }
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
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub struct ObjectTypeMovementRules {
    object_class_rules: HashMap<&'static ObjectClass, bool>,
    object_group_rules: HashMap<&'static ObjectGroup, bool>,
    object_type_rules: HashMap<&'static ObjectType, bool>,
}

impl ObjectTypeMovementRules {
    /// Creates a new [`ObjectTypeMovementRules`] from the three provided vecs holding a tuple containing
    /// one each of the following, [`ObjectClass`], [`ObjectGroup`], [`ObjectType`], and a bool. The
    /// bool fed in with the corresponding object info controls whether the object that has this component
    /// is allowed to move onto the given tile.
    pub fn new(
        object_class_rules: Vec<(&'static ObjectClass, bool)>,
        object_group_rules: Vec<(&'static ObjectGroup, bool)>,
        object_type_rules: Vec<(&'static ObjectType, bool)>,
    ) -> ObjectTypeMovementRules {
        ObjectTypeMovementRules {
            object_class_rules: ObjectTypeMovementRules::new_class_rules_hashmaps(
                object_class_rules,
            ),
            object_group_rules: ObjectTypeMovementRules::new_group_rules_hashmaps(
                object_group_rules,
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
        type_rules: Vec<(&'static ObjectType, bool)>,
    ) -> HashMap<&'static ObjectType, bool> {
        let mut type_hashmap: HashMap<&'static ObjectType, bool> = HashMap::new();

        for rule in type_rules.iter() {
            type_hashmap.insert(rule.0, rule.1);
        }
        type_hashmap
    }

    /// Helper function to create a hashmap of [`ObjectGroup`] rules for Object Movement.
    pub fn new_group_rules_hashmaps(
        group_rules: Vec<(&'static ObjectGroup, bool)>,
    ) -> HashMap<&'static ObjectGroup, bool> {
        let mut group_hashmap: HashMap<&'static ObjectGroup, bool> = HashMap::new();

        for rule in group_rules.iter() {
            group_hashmap.insert(rule.0, rule.1);
        }
        group_hashmap
    }

    /// Helper function to create a hashmap of [`ObjectClass`] rules for Object Movement.
    pub fn new_class_rules_hashmaps(
        class_rules: Vec<(&'static ObjectClass, bool)>,
    ) -> HashMap<&'static ObjectClass, bool> {
        let mut class_hashmap: HashMap<&'static ObjectClass, bool> = HashMap::new();

        for rule in class_rules.iter() {
            class_hashmap.insert(rule.0, rule.1);
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
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ObjectTerrainMovementRules {
    terrain_class_rules: Vec<&'static TerrainClass>,
    terrain_type_rules: HashMap<&'static TerrainType, bool>,
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
            .contains(&tile_terrain_info.terrain_type.terrain_class)
    }

    /// Helper function to create a hashmap of [`TerrainType`] rules for Object Movement.
    pub fn new_terrain_type_rules(
        rules: Vec<(&'static TerrainType, bool)>,
    ) -> HashMap<&'static TerrainType, bool> {
        let mut hashmap: HashMap<&'static TerrainType, bool> = HashMap::new();
        for rule in rules.iter() {
            hashmap.insert(rule.0, rule.1);
        }
        hashmap
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

    // this expression should be negative because in the given ObjectTerrainMovementRules TERRAIN_TYPES[2]
    // is set to false
    assert_eq!(movement_rules.can_move_on_tile(&tile_terrain_info), false);
}

//TODO: When we have some form of scheduling, make this go away by default at the beginning of the
// players turn
/// Marker component signifying that the unit has moved and cannot move anymore
#[derive(Clone, Copy, Eq, Hash, PartialEq, Component)]
pub struct ObjectMoved;
