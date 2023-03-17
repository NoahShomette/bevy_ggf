//! Any actions that affect the game world should be specified as a [`GameCommand`] and submitted to
//! through the [`GameCommands`] to enable saving, rollback, and more. A command should be entirely
//! self contained, everything needed to accurately recreate the command should be included. A command
//! **cannot** rely on any actions outside of it, only data. Eg, for MoveObject, you can't rely on
//! the moving object having an up to date [`CurrentMovementInformation`](crate::movement::CurrentMovementInformation)
//! component, you must calculate the move in the command
//!
//! To use in a system, request the [`GameCommands`] Resource, get the commands field, and call a defined
//! command or submit a custom command using commands.add().
//! ```rust
//! use bevy::prelude::{Bundle, ResMut, World};
//! use bevy_ecs_tilemap::prelude::TilePos;
//! use bevy_ggf::game_core::command::{GameCommand, GameCommands};
//! use bevy_ggf::mapping::MapId;
//!
//! #[derive(Bundle, Default)]
//! pub struct CustomBundle{
//!     // Whatever components you want in your bundle - GameCommands::spawn_object will automatically
//!     // insert the GameId struct with the next id
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
//!     game_commands.spawn_object(CustomBundle::default(), TilePos::new(1, 1), MapId{id: 0});
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
//!                 // any data you might need, like the GameId of an entity spawned, the transform
//!                 // that the entity was at before, etc
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

use crate::mapping::tiles::{ObjectStackingClass, TileObjectStackingRules, TileObjects};
use crate::mapping::{tile_pos_to_centered_map_world_pos, MapId};
use crate::object::{Object, ObjectGridPosition};
use bevy::ecs::system::SystemState;
use bevy::log::info;
use bevy::prelude::{Bundle, Component, DespawnRecursiveExt, Entity, Mut, Query, Reflect, ReflectComponent, Resource, Transform, With, Without, World};
use bevy_ecs_tilemap::prelude::{TilemapGridSize, TilemapType};
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use chrono::{DateTime, Utc};
use std::fmt::Debug;
use std::process::id;
use crate::game_core::{Game, GameId, GameIdProvider, GameType};

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
                command.command.rollback(world).expect("Rollback failed");
                game.history.rolledback_history.push(command);
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
                if let Ok(_) = command.command.execute(world) {
                    game.history.push(command.clone());
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

#[derive(Clone)]
pub struct GameCommandMeta {
    pub command: Box<dyn GameCommand>,
    pub command_time: DateTime<Utc>,
    //command_type: CommandType,
}

/// A base trait defining an action that affects the game. Define your own to implement your own
/// custom commands that will be automatically saved, executed, and rolledback. The rollback function
/// **MUST** exactly roll the world back to as it was, excluding entity IDs.
/// ```rust
/// use bevy::prelude::World;
/// use bevy_ggf::game_core::command::GameCommand;
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
    fn execute(&mut self, world: &mut World) -> Result<(), String>;
    
    #[cfg(feature = "command_rollback")]
    fn rollback(&mut self, world: &mut World) -> Result<(), String>;
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
    pub queue: Vec<GameCommandMeta>,
}

impl GameCommandQueue {
    /// Push a new command to the end of the queue
    pub fn push<C>(&mut self, command: C)
    where
        C: GameCommand,
    {
        let utc: DateTime<Utc> = Utc::now();
        let command_meta = GameCommandMeta {
            command: Box::from(command),
            command_time: utc,
        };
        self.queue.push(command_meta);
    }

    /// Take the last command in the queue. Returns None if queue is empty
    pub fn pop(&mut self) -> Option<GameCommandMeta> {
        self.queue.pop()
    }
}

/// The history of all commands sent for this [`Game`] instance - if a command rollback occurs the
/// command is discarded from the history. This means that the history contains only the commands
/// that led to this instance of the game
#[derive(Default)]
pub struct GameCommandsHistory {
    pub history: Vec<GameCommandMeta>,
    pub rolledback_history: Vec<GameCommandMeta>,
    rollbacks: u32,
    rollforwards: u32,
}

impl GameCommandsHistory {
    /// Push a command to the end of the history vec
    pub fn push(&mut self, command: GameCommandMeta) {
        self.history.push(command);
    }

    /// Take the last command in the queue. Returns None if queue is empty
    pub fn pop(&mut self) -> Option<GameCommandMeta> {
        self.history.pop()
    }

    /// Push a command to the end of the history vec
    pub fn push_rollback_history(&mut self, command: GameCommandMeta) {
        self.rolledback_history.push(command);
    }

    /// Take the last command in the queue. Returns None if queue is empty
    pub fn pop_rollback_history(&mut self) -> Option<GameCommandMeta> {
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
    /// succeed to the history. If commands dont succeed they are silently failed.
    /// If [`Game`].game_type is set to Networked: Automatically checks if the new commands occured
    /// before any old commands and will rollback the world and then replay commands to ensure proper
    /// timeline
    pub fn execute_buffer(&mut self, world: &mut World) {
        let mut temp_rb_commands: Vec<GameCommandMeta> = vec![];
        for mut command in self.queue.queue.drain(..).into_iter() {
            match world.resource::<Game>().game_type {
                GameType::Networked => {
                    let mut amount_to_rollback = 0;
                    'old_check: for old_command in self.history.history.iter().rev() {
                        if command.command_time < old_command.command_time {
                            amount_to_rollback += 1;
                        } else {
                            break 'old_check;
                        }
                    }

                    for mut rb_command in self
                        .history
                        .history
                        .drain(
                            self.history.history.len() - amount_to_rollback
                                ..self.history.history.len(),
                        )
                        .into_iter()
                    {
                        rb_command
                            .command
                            .rollback(world)
                            .expect("Failed to rollback command");
                        temp_rb_commands.push(rb_command);
                    }

                    if let Ok(_) = command.command.execute(world) {
                        self.history.push(command);
                    } else {
                        info!("execution failed ");
                    }

                    for mut rb_command in temp_rb_commands.drain(..).into_iter() {
                        rb_command
                            .command
                            .execute(world)
                            .expect("Failed to rollback command");
                        self.history.history.push(rb_command);
                    }
                }
                GameType::Local => {
                    if let Ok(_) = command.command.execute(world) {
                        self.history.push(command);
                    } else {
                        info!("execution failed ");
                    }
                }
            }

            self.history.clear_rollback_history();
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
    pub fn despawn_object(&mut self, on_map: MapId, object_game_id: GameId) -> DespawnObject {
        self.queue.push(DespawnObject {
            on_map,
            object_game_id,
            tile_pos: None,
        });
        DespawnObject {
            object_game_id,
            on_map,
            tile_pos: None,
        }
    }
}

/// Removes the given entity from the given tile if the tile exists and the entity has the required components.
/// Will silently fail if either of the above are invalid.
/// Execute will *not* set the objects grid position - Rollback will.
/// This should be used with [AddObjectToTile] command to enable true reversing as needed. Look
///  at [SpawnObject] as an example of how to do this.
#[derive(Clone, Debug)]
pub struct RemoveObjectFromTile {
    pub object_game_id: GameId,
    pub on_map: MapId,
    pub tile_pos: TilePos,
}

impl GameCommand for RemoveObjectFromTile {
    fn execute(&mut self, mut world: &mut World) -> Result<(), String> {
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
        return Ok(());
    }

    fn rollback(&mut self, mut world: &mut World) -> Result<(), String> {
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
        Ok(())
    }
}

/// Adds the given entity to the given tile if the tile exists and the entity has the required components.
/// Will silently fail if either of the above are invalid.
/// Rollback will *not* set the objects grid position or change the position of the objects transform
/// This should be used with [RemoveObjectFromTile] command to enable true reversing as needed. Look
/// at [SpawnObject] as an example of how to do this.
#[derive(Clone, Debug)]
pub struct AddObjectToTile {
    pub object_game_id: GameId,
    pub on_map: MapId,
    pub tile_pos: TilePos,
}

impl GameCommand for AddObjectToTile {
    fn execute(&mut self, mut world: &mut World) -> Result<(), String> {
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
        Ok(())
    }

    fn rollback(&mut self, mut world: &mut World) -> Result<(), String> {
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
        Ok(())
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
    fn execute(&mut self, world: &mut World) -> Result<(), String> {
        // Assign a new id as we un assign the id when we rollback
        let id = world.resource_mut::<GameIdProvider>().next_id_component();

        world.spawn(self.bundle.clone()).insert(id);

        let mut add = AddObjectToTile {
            object_game_id: id,
            on_map: self.on_map,
            tile_pos: self.tile_pos,
        };
        let _ = add.execute(world);
        self.object_game_id = Some(id);
        return Ok(());
    }

    fn rollback(&mut self, mut world: &mut World) -> Result<(), String> {
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

        return Ok(());
    }
}

#[derive(Clone, Debug)]
pub struct DespawnObject {
    pub on_map: MapId,
    pub object_game_id: GameId,
    pub tile_pos: Option<TilePos>,
    //pub object_components: Option<Vec<>>
}

impl GameCommand for DespawnObject {
    fn execute(&mut self, world: &mut World) -> Result<(), String> {
        let mut system_state: SystemState<Query<(Entity, &GameId, &TilePos)>> =
            SystemState::new(world);
        let mut object_query = system_state.get_mut(world);

        let Some((entity, _, tile_pos)) = object_query.iter_mut().find(|(_, id, _)| {
            id == &&self
                .object_game_id
        })else {
            return Err(String::from("No object components found"));
        };

        let tile_pos = *tile_pos;

        world.despawn(entity);

        let mut remove = RemoveObjectFromTile {
            object_game_id: self.object_game_id,
            on_map: self.on_map,
            tile_pos,
        };
        let _ = remove.execute(world);

        self.tile_pos = Some(tile_pos);
        return Ok(());
    }

    fn rollback(&mut self, mut world: &mut World) -> Result<(), String> {
        let mut system_state: SystemState<Query<(Entity, &GameId)>> = SystemState::new(&mut world);
        let mut object_query = system_state.get_mut(&mut world);

        let Some((entity, _)) = object_query.iter_mut().find(|(_, id)| {
            id == &&self
                .object_game_id
        })else {
            return Err(String::from("No object components found"));
        };

        let mut remove = RemoveObjectFromTile {
            object_game_id: self.object_game_id,
            on_map: self.on_map,
            tile_pos: self.tile_pos.expect("Tile Pos must be set on execution"),
        };
        let _ = remove.execute(world);
        world.entity_mut(entity).despawn_recursive();
        world.resource_mut::<GameIdProvider>().remove_last_id();

        return Ok(());
    }
}
