use crate::game::Game;
use crate::mapping::tiles::{ObjectStackingClass, TileObjectStackingRules, TileObjects};
use crate::mapping::{tile_pos_to_centered_map_world_pos, MapHandler};
use crate::object::{Object, ObjectGridPosition};
use bevy::ecs::system::SystemState;
use bevy::log::info;
use bevy::math::IVec2;
use bevy::prelude::{Entity, Mut, Query, Res, Transform, With, Without, World};
use bevy_ecs_tilemap::prelude::{TilemapGridSize, TilemapType};
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use std::fmt::Debug;

/// Executes all stored game commands by calling the command queue execute buffer function
pub fn execute_game_commands_buffer(world: &mut World) {
    world.resource_scope(|world, mut game: Mut<Game>| {
        game.commands.execute_buffer(world);
    });
}

/// Executes all rollbacks requested - panics if a rollback fails
pub fn execute_game_rollbacks_buffer(world: &mut World) {
    world.resource_scope(|world, mut game: Mut<Game>| {
        while game.commands.history.rollbacks != 0 {
            if let Some(mut command) = game.commands.history.history.pop() {
                command.rollback(world).expect("Rollback failed");
                info!("Rollbacked: {:?}", command);
            }
            game.commands.history.rollbacks -= 1;
        }
    });
}

/// A base trait defining an action that affects the game. Define your own to implement your own
/// custom commands that will be automatically saved and executed
/// ```rust
///
///
/// ```
pub trait GameCommand: Send + Sync + Debug + GameCommandClone + 'static {
    /// Execute the command
    fn execute(&mut self, world: &mut World) -> Result<(), String>;
    fn rollback(&mut self, world: &mut World) -> Result<(), String>;
}

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
        T: 'static + GameCommand + Clone,
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
    rollbacks: u32,
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
}

/// A struct to hold, execute, and rollback [`GameCOmmand`]s. Use associated actions to access and
/// modify the game
#[derive(Default)]
pub struct GameCommands {
    pub queue: GameCommandQueue,
    pub history: GameCommandsHistory,
}

impl GameCommands {
    /// Drains the command buffer and attempts to execute each command. Will only push commands that
    /// succeed to the history. If commands dont succeed they are silently failed
    pub fn execute_buffer(&mut self, world: &mut World) {
        for mut command in self.queue.queue.drain(..).into_iter() {
            if let Ok(_) = command.execute(world) {
                info!("executed {:?}", command);
                self.history.push(command.clone());
            } else {
                info!("execution failed {:?}", command);
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
        object_entity: Entity,
        tile_pos: TilePos,
    ) -> AddObjectToTile {
        self.queue.push(AddObjectToTile {
            object_entity,
            tile_pos,
        });
        AddObjectToTile {
            object_entity,
            tile_pos,
        }
    }

    /// Removes the given entity from the given tile if the tile exists and the entity has the required components.
    /// Will silently fail if either of the above are invalid.
    /// Execute will *not* set the objects grid position - Rollback will
    pub fn remove_object_from_tile(
        &mut self,
        object_entity: Entity,
        tile_pos: TilePos,
    ) -> RemoveObjectFromTile {
        self.queue.push(RemoveObjectFromTile {
            object_entity,
            tile_pos,
        });
        RemoveObjectFromTile {
            object_entity,
            tile_pos,
        }
    }
}

/// Removes the given entity from the given tile if the tile exists and the entity has the required components.
/// Will silently fail if either of the above are invalid.
/// Execute will *not* set the objects grid position - Rollback will
#[derive(Clone, Debug)]
pub struct RemoveObjectFromTile {
    pub object_entity: Entity,
    pub tile_pos: TilePos,
}

impl GameCommand for RemoveObjectFromTile {
    fn execute(&mut self, mut world: &mut World) -> Result<(), String> {
        let mut system_state: SystemState<(
            Query<&ObjectStackingClass>,
            Res<MapHandler>,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<&TileStorage>,
        )> = SystemState::new(&mut world);
        let (mut object_query, map_handler, mut tile_query, tile_storage_query) =
            system_state.get_mut(&mut world);

        let Ok(object_stacking_class) = object_query.get_mut(self.object_entity) else {
            return Err(String::from("No object stacking class component found"));
        };
        let Ok(tile_storage) = tile_storage_query.get(map_handler.get_map_entity(IVec2::ZERO).unwrap()) else {
            return Err(String::from("No tile_storage found"));
        };
        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();
        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile stack rules found"));
        };

        tile_objects.remove_object(self.object_entity);
        tile_stack_rules.decrement_object_class_count(object_stacking_class);
        return Ok(());
    }

    fn rollback(&mut self, mut world: &mut World) -> Result<(), String> {
        let mut system_state: SystemState<(
            Query<(&mut ObjectGridPosition, &ObjectStackingClass)>,
            Res<MapHandler>,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<&TileStorage>,
        )> = SystemState::new(&mut world);

        let (mut object_query, map_handler, mut tile_query, tile_storage_query) =
            system_state.get_mut(&mut world);

        let Ok((mut object_grid_position, object_stacking_class)) = object_query.get_mut(self.object_entity) else {
            return Err(String::from("No object stacking class component found"));
        };
        let Ok(tile_storage) = tile_storage_query.get(map_handler.get_map_entity(IVec2::ZERO).unwrap()) else {
            return Err(String::from("No tile_storage found"));
        };

        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();

        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile stack rules found"));
        };

        tile_objects.add_object(self.object_entity);
        object_grid_position.tile_position = self.tile_pos;
        tile_stack_rules.increment_object_class_count(object_stacking_class);
        Ok(())
    }
}

/// Adds the given entity to the given tile if the tile exists and the entity has the required components.
/// Will silently fail if either of the above are invalid.
/// Rollback will *not* set the objects grid position
#[derive(Clone, Debug)]
pub struct AddObjectToTile {
    pub object_entity: Entity,
    pub tile_pos: TilePos,
}

impl GameCommand for AddObjectToTile {
    fn execute(&mut self, mut world: &mut World) -> Result<(), String> {
        let mut system_state: SystemState<(
            Query<
                (
                    &mut Transform,
                    &mut ObjectGridPosition,
                    &ObjectStackingClass,
                ),
                With<Object>,
            >,
            Query<(&TilemapGridSize, &TilemapType, &Transform), Without<Object>>,
            Res<MapHandler>,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<&TileStorage>,
        )> = SystemState::new(&mut world);

        let (mut object_query, mut map_query, map_handler, mut tile_query, tile_storage_query) =
            system_state.get_mut(&mut world);

        let Ok((mut transform, mut object_grid_position, object_stacking_class)) = object_query.get_mut(self.object_entity) else {
            return Err(String::from("Object components not found"));
        };
        let Ok(tile_storage) = tile_storage_query.get(map_handler.get_map_entity(IVec2::ZERO).unwrap()) else {
            return Err(String::from("No tile storage component not found"));
        };

        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();

        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile components found"));
        };

        let Ok((grid_size, map_type, map_transform)) = map_query.get(map_handler.get_map_entity(IVec2::ZERO).unwrap()) else {
            return Err(String::from("No map components found"));
        };

        tile_objects.add_object(self.object_entity);
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
            Query<&ObjectStackingClass>,
            Res<MapHandler>,
            Query<(&mut TileObjectStackingRules, &mut TileObjects)>,
            Query<&TileStorage>,
        )> = SystemState::new(&mut world);

        let (mut object_query, map_handler, mut tile_query, tile_storage_query) =
            system_state.get_mut(&mut world);

        let Ok(object_stacking_class) = object_query.get_mut(self.object_entity) else {
            return Err(String::from("No object stacking class component found"));
        };
        let Ok(tile_storage) = tile_storage_query.get(map_handler.get_map_entity(IVec2::ZERO).unwrap()) else {
            return Err(String::from("No tile storage component found"));
        };

        let tile_entity = tile_storage.get(&self.tile_pos).unwrap();

        let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) else {
            return Err(String::from("No tile components found"));
        };

        tile_objects.remove_object(self.object_entity);
        tile_stack_rules.decrement_object_class_count(object_stacking_class);
        Ok(())
    }
}
