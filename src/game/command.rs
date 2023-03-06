//! Any actions that affect the game world should be specified as a [`GameCommand`] and submitted to
//! through the [`GameCommands`] to enable saving, rollback, and more.
//!
//! To use in a system request the [`GameCommands`] Resource, get the commands field, and call a defined
//! command or submit a custom command using commands.add().
//! ```rust
//! use bevy::prelude::{Bundle, ResMut, World};
//! use bevy_ecs_tilemap::prelude::TilePos;
//! use bevy_ggf::game::command::{GameCommand, GameCommands};
//! use bevy_ggf::mapping::MapId;
//!
//! #[derive(Bundle)]
//! pub struct CustomBundle{
//!     // Whatever components you want in your bundle
//! }
//!     
//! fn spawn_object_built_in_command(
//!     // Request the GameCommands Resource - all actions in the game should be communicated through
//!     // this
//!     mut game_commands: ResMut<GameCommands>,
//! ){
//!     // Call whatever command on GameCommands - Add your own commands by writing an extension trait
//!     // and implementing that for GameCommands//!
//!
//!     game_commands.spawn_object(CustomBundle, TilePos::new(1, 1), MapId{id: 0});
//! }
//!
//! // Create a struct for your custom command, use this to store whatever data you need to execute
//! // and rollback the commands
//! #[derive(Clone, Debug)]
//! struct MyCustomCommand;
//!
//! // Impl GameCommand for your struct
//! impl GameCommand for MyCustomCommand{
//!     fn execute(&mut self, world: &mut World) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
//!         todo!() // Implement whatever your custom command should do here
//!     }
//!
//!     fn rollback(&mut self, world: &mut World) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
//!         todo!() // Implement how to reverse your custom command - you can use your struct to save
//!                 // any data you might need, like the GameId of an entity spawned
//!     }
//! }
//!
//! fn spawn_object_custom_command(
//!    mut game: ResMut<GameCommands>,
//! ){
//!     game.commands.add(MyCustomCommand);
//! }
//!
//! ```

use crate::game::{GameId, GameIdProvider};
use crate::mapping::tiles::{ObjectStackingClass, TileObjectStackingRules, TileObjects};
use crate::mapping::{tile_pos_to_centered_map_world_pos, MapId};
use crate::object::{Object, ObjectGridPosition};
use bevy::ecs::system::SystemState;
use bevy::log::info;
use bevy::prelude::{
    Bundle, DespawnRecursiveExt, Entity, Mut, Query, Resource, Transform, With, Without, World,
};
use bevy_ecs_tilemap::prelude::{TilemapGridSize, TilemapType};
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use std::fmt::Debug;
use chrono::{DateTime, Utc};

/// Executes all stored game commands by calling the command queue execute buffer function
pub fn execute_game_commands_buffer(world: &mut World) {
    world.resource_scope(|world, mut game: Mut<GameCommands>| {
        game.execute_buffer(world);
    });
}

/// Executes all rollbacks requested - panics if a rollback fails
pub fn execute_game_rollbacks_buffer(world: &mut World) {
    world.resource_scope(|world, mut game: Mut<GameCommands>| {
        while game.history.rollbacks != 0 {
            if let Some(mut command) = game.history.pop() {
                let new_command = command.rollback(world).expect("Rollback failed");
                if let Some(command) = new_command {
                    game.history.rolledback_history.push(command);
                } else {
                    game.history.rolledback_history.push(command);
                }
                info!("Rollbacked command");
            }
            game.history.rollbacks -= 1;
        }
    });
}

/// Executes all rollforwards requested - panics if an execute fails
pub fn execute_game_rollforward_buffer(world: &mut World) {
    world.resource_scope(|world, mut game: Mut<GameCommands>| {
        while game.history.rollforwards != 0 {
            if let Some(mut command) = game.history.rolledback_history.pop() {
                if let Ok(new_command) = command.execute(world) {
                    info!("Rolledforward command");
                    if let Some(command) = new_command {
                        game.history.push(command.clone());
                    } else {
                        game.history.push(command.clone());
                    }
                } else {
                    info!("Rolledforward failed");
                }
            }
            game.history.rollforwards -= 1;
        }
    });
}

pub enum CommandType {
    System,
    Player,
}

pub struct GameCommandMeta {
    command: Box<dyn GameCommand>,
    command_time: DateTime<Utc>,
    //command_type: CommandType,
}

/// A base trait defining an action that affects the game. Define your own to implement your own
/// custom commands that will be automatically saved, executed, and rolledback
/// ```rust
/// use bevy::prelude::World;
/// use bevy_ggf::game::command::GameCommand;
/// #[derive(Clone, Debug)]
///  struct MyCustomCommand;
///
///  impl GameCommand for MyCustomCommand{fn execute(&mut self, world: &mut World) -> Result<(), String> {
///          todo!() // Implement whatever your custom command should do here
///      }
///
///  fn rollback(&mut self, world: &mut World) -> Result<(), String> {
///          todo!() // Implement how to reverse your custom command
///      }
///  }
///
/// ```
pub trait GameCommand: Send + GameCommandClone + Sync + 'static {
    /// Execute the command
    fn execute(&mut self, world: &mut World) -> Result<Option<Box<dyn GameCommand>>, String>;
    fn rollback(&mut self, world: &mut World) -> Result<Option<Box<dyn GameCommand>>, String>;
}

/* TODO: Figure out if a closure is possible. Probably not since we have two functions, but either way
 it would be nice if we can but they can still do whatever they need otherwise
impl<F> GameCommand for F
    where
        F: FnOnce(&mut World) + Sync + Copy + Debug + GameCommandClone + Send + 'static,
{
    fn execute(self: &mut F, world: &mut World) -> Result<(), String> {
        Ok(self(world))
    }
    fn rollback(self: &mut F, world: &mut World) -> Result<(), String> {
        Ok(self(world))
    }
}

 */

impl Clone for Box<dyn GameCommand> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Helper trait to clone boxed Game Commands
pub trait GameCommandClone {
    fn clone_box(&self) -> Box<dyn GameCommand>;
}

impl<T> GameCommandClone for T
where
    T: 'static + GameCommand + Clone + ?Sized,
{
    fn clone_box(&self) -> Box<dyn GameCommand> {
        Box::new(self.clone())
    }
}

/// The queue of pending [`GameCommand`]s. Doesn't do anything until executed
#[derive(Default)]
pub struct GameCommandQueue {
    pub queue: Vec<Box<dyn GameCommand>>,
}

impl GameCommandQueue {
    /// Push a new command to the end of the queue
    pub fn push<C>(&mut self, command: C)
    where
        C: GameCommand,
    {
        self.queue.push(Box::from(command));
    }

    /// Take the last command in the queue. Returns None if queue is empty
    pub fn pop(&mut self) -> Option<Box<dyn GameCommand>> {
        self.queue.pop()
    }
}

/// The history of all commands sent for this [`Game`] instance - if a command rollback occurs the
/// command is discarded from the history. This means that the history contains only the commands
/// that led to this instance of the game
#[derive(Default)]
pub struct GameCommandsHistory {
    pub history: Vec<Box<dyn GameCommand>>,
    pub rolledback_history: Vec<Box<dyn GameCommand>>,
    rollbacks: u32,
    rollforwards: u32,
}

impl GameCommandsHistory {
    /// Push a command to the end of the history vec
    pub fn push(&mut self, command: Box<dyn GameCommand>) {
        self.history.push(command);
    }

    /// Take the last command in the queue. Returns None if queue is empty
    pub fn pop(&mut self) -> Option<Box<dyn GameCommand>> {
        self.history.pop()
    }

    /// Push a command to the end of the history vec
    pub fn push_rollback_history(&mut self, command: Box<dyn GameCommand>) {
        self.rolledback_history.push(command);
    }

    /// Take the last command in the queue. Returns None if queue is empty
    pub fn pop_rollback_history(&mut self) -> Option<Box<dyn GameCommand>> {
        self.rolledback_history.pop()
    }

    pub fn clear_rollback_history(&mut self) {
        self.rolledback_history.clear();
    }
}

/// A struct to hold, execute, and rollback [`GameCommand`]s. Use associated actions to access and
/// modify the game
#[derive(Default, Resource)]
pub struct GameCommands {
    pub queue: GameCommandQueue,
    pub history: GameCommandsHistory,
}

impl GameCommands {
    pub fn new() -> Self {
        GameCommands {
            queue: Default::default(),
            history: Default::default(),
        }
    }

    /// Drains the command buffer and attempts to execute each command. Will only push commands that
    /// succeed to the history. If commands dont succeed they are silently failed
    pub fn execute_buffer(&mut self, world: &mut World) {
        for mut command in self.queue.queue.drain(..).into_iter() {
            if let Ok(new_command) = command.execute(world) {
                self.history.clear_rollback_history();
                info!("executed Command");
                if let Some(command) = new_command {
                    self.history.push(command.clone());
                } else {
                    self.history.push(command.clone());
                }
            } else {
                info!("execution failed ");
            }
        }
    }

    /// Request a single rollback - The game will attempt to rollback the next time
    /// [`execute_game_rollbacks_buffer`] is called
    pub fn rollback_one(&mut self) {
        self.history.rollbacks += 1;
    }

    /// Request a specific number of rollbacks - The game will attempt these rollbacks the next time
    /// [`execute_game_rollbacks_buffer`] is called
    pub fn rollback_amount(&mut self, amount: u32) {
        self.history.rollbacks += amount;
    }

    pub fn rollforward(&mut self, amount: u32) {
        self.history.rollforwards += amount;
    }

    /// Add a custom command to the queue
    pub fn add<T>(&mut self, command: T) -> T
    where
        T: GameCommand + Clone,
    {
        self.queue.push(command.clone());
        command
    }

    /// Adds the given entity to the given tile if the tile exists and the entity has the required components.
    /// Will silently fail if either of the above are invalid.
    /// Rollback will *not* set the objects grid position
    ///
    /// This is unchecked and forceful. It will do so even
    /// if the move is invalid. It is the callers responsibility to ensure that it is valid
    pub fn add_object_to_tile(
        &mut self,
        object_entity: GameId,
        on_map: MapId,
        tile_pos: TilePos,
    ) -> AddObjectToTile {
        self.queue.push(AddObjectToTile {
            object_game_id: object_entity,
            on_map,
            tile_pos,
        });
        AddObjectToTile {
            object_game_id: object_entity,
            on_map,
            tile_pos,
        }
    }

    /// Removes the given entity from the given tile if the tile exists and the entity has the required components.
    /// Will silently fail if either of the above are invalid.
    /// Execute will *not* set the objects grid position - Rollback will
    pub fn remove_object_from_tile(
        &mut self,
        object_game_id: GameId,
        on_map: MapId,
        tile_pos: TilePos,
    ) -> RemoveObjectFromTile {
        self.queue.push(RemoveObjectFromTile {
            object_game_id,
            on_map,
            tile_pos,
        });
        RemoveObjectFromTile {
            object_game_id,
            on_map,
            tile_pos,
        }
    }

    pub fn spawn_object<T>(&mut self, bundle: T, tile_pos: TilePos, on_map: MapId) -> SpawnObject<T>
    where
        T: Bundle + Clone,
    {
        self.queue.push(SpawnObject {
            bundle: bundle.clone(),
            tile_pos,
            on_map,
            object_game_id: None,
        });
        SpawnObject {
            bundle,
            tile_pos,
            on_map,
            object_game_id: None,
        }
    }
    pub fn despawn_object(&mut self) {}
}

/// Removes the given entity from the given tile if the tile exists and the entity has the required components.
/// Will silently fail if either of the above are invalid.
/// Execute will *not* set the objects grid position - Rollback will
#[derive(Clone, Debug)]
pub struct RemoveObjectFromTile {
    pub object_game_id: GameId,
    pub on_map: MapId,
    pub tile_pos: TilePos,
}

impl GameCommand for RemoveObjectFromTile {
    fn execute(
        &mut self,
        mut world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
        let mut system_state: SystemState<(
            Query<(&GameId, &ObjectStackingClass)>,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<(&MapId, &TileStorage)>,
        )> = SystemState::new(&mut world);
        let (mut object_query, mut tile_query, mut tile_storage_query) =
            system_state.get_mut(&mut world);

        let Some((_, object_stacking_class)) = object_query
            .iter_mut()
            .find(|(id, _)| id == &&self.object_game_id)else {
            return Err(String::from("No object components found"));
        };
        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&self.on_map)else {
            return Err(String::from("No tile components found"));
        };

        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();
        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile stack rules found"));
        };

        tile_objects.remove_object(self.object_game_id);
        tile_stack_rules.decrement_object_class_count(object_stacking_class);
        return Ok(None);
    }

    fn rollback(
        &mut self,
        mut world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
        let mut system_state: SystemState<(
            Query<(&GameId, &mut ObjectGridPosition, &ObjectStackingClass)>,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<(&MapId, &TileStorage)>,
        )> = SystemState::new(&mut world);

        let (mut object_query, mut tile_query, mut tile_storage_query) =
            system_state.get_mut(&mut world);

        let Some((_, mut object_grid_position, object_stacking_class)) = object_query
            .iter_mut()
            .find(|(id, _, _)| id == &&self.object_game_id)else {
            return Err(String::from("No object components found"));
        };
        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&self.on_map)else {
            return Err(String::from("No tile components found found"));
        };

        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();

        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile stack rules found"));
        };

        tile_objects.add_object(self.object_game_id);
        object_grid_position.tile_position = self.tile_pos;
        tile_stack_rules.increment_object_class_count(object_stacking_class);
        Ok(None)
    }
}

/// Adds the given entity to the given tile if the tile exists and the entity has the required components.
/// Will silently fail if either of the above are invalid.
/// Rollback will *not* set the objects grid position or change the position of the objects transform
#[derive(Clone, Debug)]
pub struct AddObjectToTile {
    pub object_game_id: GameId,
    pub on_map: MapId,
    pub tile_pos: TilePos,
}

impl GameCommand for AddObjectToTile {
    fn execute(
        &mut self,
        mut world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
        let mut system_state: SystemState<(
            Query<
                (
                    &GameId,
                    &mut Transform,
                    &mut ObjectGridPosition,
                    &ObjectStackingClass,
                ),
                With<Object>,
            >,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<(
                Entity,
                &MapId,
                &TileStorage,
                &TilemapGridSize,
                &TilemapType,
                &Transform,
                Without<Object>,
            )>,
        )> = SystemState::new(&mut world);

        let (mut object_query, mut tile_query, mut tile_storage_query) =
            system_state.get_mut(&mut world);

        let Some((_, mut transform, mut object_grid_position, object_stacking_class)) =
            object_query
                .iter_mut()
                .find(|(id, _, _, _)| id == &&self.object_game_id) else {
            return Err(String::from(format!("No Object Components found for GameId: {:?}", self.object_game_id)));
        };
        let Some((entity, _, tile_storage, grid_size, map_type, map_transform, _)) = tile_storage_query
            .iter_mut()
            .find(|(_, id, _, _, _,_, _)| id == &&self.on_map) else {
            return Err(String::from(format!("No Map Components found for GameId: {:?}", self.on_map)));
        };

        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();

        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile components found"));
        };

        tile_objects.add_object(self.object_game_id);
        object_grid_position.tile_position = self.tile_pos;
        tile_stack_rules.increment_object_class_count(object_stacking_class);

        // have to transform the tiles position to the transformed position to place the object at the right point
        let tile_world_pos =
            tile_pos_to_centered_map_world_pos(&self.tile_pos, map_transform, grid_size, map_type);

        transform.translation = tile_world_pos.extend(5.0);
        Ok(None)
    }

    fn rollback(
        &mut self,
        mut world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
        let mut system_state: SystemState<(
            Query<(&GameId, &ObjectStackingClass)>,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<(&MapId, &TileStorage)>,
        )> = SystemState::new(&mut world);

        let (mut object_query, mut tile_query, mut tile_storage_query) =
            system_state.get_mut(&mut world);

        let Some((_, object_stacking_class)) = object_query
            .iter_mut()
            .find(|(id, _)| id == &&self.object_game_id)else {
            return Err(String::from("No object components found found"));
        };
        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&self.on_map)else {
            return Err(String::from("No tile components found"));
        };

        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();

        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile components found"));
        };

        tile_objects.remove_object(self.object_game_id);
        tile_stack_rules.decrement_object_class_count(object_stacking_class);
        Ok(None)
    }
}

#[derive(Clone, Debug)]
pub struct SpawnObject<T>
where
    T: Bundle,
{
    pub bundle: T,
    pub tile_pos: TilePos,
    pub on_map: MapId,
    pub object_game_id: Option<GameId>,
}

impl<T> GameCommand for SpawnObject<T>
where
    T: Bundle + Clone,
{
    fn execute(
        &mut self,
        world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
        // Assign a new id as we un assign the id when we rollback
        let id = world.resource_mut::<GameIdProvider>().next_id_component();

        world.spawn(self.bundle.clone()).insert(id);

        let mut add = AddObjectToTile {
            object_game_id: id,
            on_map: self.on_map,
            tile_pos: self.tile_pos,
        };
        let _ = add.execute(world);
        return Ok(Some(Box::new(SpawnObject {
            bundle: self.bundle.clone(),
            tile_pos: self.tile_pos,
            on_map: self.on_map,
            object_game_id: Some(id),
        })));
    }

    fn rollback(
        &mut self,
        mut world: &mut World,
    ) -> Result<Option<Box<(dyn GameCommand + 'static)>>, String> {
        let mut system_state: SystemState<Query<(Entity, &GameId)>> = SystemState::new(&mut world);
        let mut object_query = system_state.get_mut(&mut world);

        let Some((entity, _)) = object_query.iter_mut().find(|(_, id)| {
            id == &&self
                .object_game_id
                .expect("Rollback can only be called after execute which returns an entity id")
        })else {
            return Err(String::from("No object components found"));
        };

        let mut remove = RemoveObjectFromTile {
            object_game_id: self
                .object_game_id
                .expect("Rollback can only be called after execute which returns an entity id"),
            on_map: self.on_map,
            tile_pos: self.tile_pos,
        };
        let _ = remove.execute(world);
        world.entity_mut(entity).despawn_recursive();
        world.resource_mut::<GameIdProvider>().remove_last_id();

        return Ok(None);
    }
}
