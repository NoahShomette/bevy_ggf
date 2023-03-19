//!

use crate::game_core::command::{GameCommand, GameCommandMeta, GameCommandQueue, GameCommands};
use crate::game_core::runner::GameRunner;
use crate::game_core::state::{GameStateHandler, get_state_diff, StateEvents, StateSystems};
use crate::movement::MovementSystems;
use crate::object::{ObjectClass, ObjectGroup, ObjectId, ObjectIdProvider, ObjectType};
use bevy::app::{App, Plugin};
use bevy::prelude::{apply_system_buffers, Children, Component, IntoSystemSetConfig, IntoSystemConfig, Parent, ReflectComponent, ReflectResource, Resource, Schedule, World};
use bevy::reflect::{FromType, GetTypeRegistration, Reflect, TypeRegistry, TypeRegistryInternal};
use bevy_ecs_tilemap::tiles::TilePos;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::default::Default;
use std::sync::Arc;

pub mod command;
pub mod runner;
pub mod state;

pub struct BggfGamePlugin {}

impl Plugin for BggfGamePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub enum GameType {
    Networked,
    Local,
}

/// Meta information on the game.
#[derive(Resource)]
pub struct GameInfo {
    pub game_type: GameType,
    pub type_registry: TypeRegistry,
    /// Holds
    pub systems_schedule: Schedule,
}

impl GameInfo {}

#[derive(Resource)]
pub struct GameData {
    pub game_world: World,
    pub game_state_handler: GameStateHandler,
}

impl GameData {}

#[derive(Debug, Resource)]
pub struct GameRuntime<T>
where
    T: GameRunner,
{
    pub game_runner: T,
}

impl<T> GameRuntime<T>
where
    T: GameRunner,
{
    pub fn simulate(&mut self, mut game_data: &mut GameData) {
        self.game_runner.simulate_game(&mut game_data.game_world);
    }
}

#[derive(Resource)]
pub struct GameBuilder<GR>
where
    GR: GameRunner + 'static,
{
    pub game_type: GameType,
    pub game_runner: GR,
    pub game_world: World,
    pub setup_schedule: Schedule,
    pub systems_schedule: Schedule,
    pub type_registry: TypeRegistry,
}

impl<GR> GameBuilder<GR>
where
    GR: GameRunner,
{
    pub fn new_game(game_type: GameType, game_runner: GR) -> GameBuilder<GR> {
        let mut game_world = World::new();

        game_world.insert_resource(GameCommands::default());
        game_world.insert_resource(ObjectIdProvider::default());

        GameBuilder {
            game_type,
            game_runner,
            game_world,
            setup_schedule: GameBuilder::<GR>::default_setup_schedule(),
            systems_schedule: Default::default(),
            type_registry: GameBuilder::<GR>::default_registry(),
        }
    }
    pub fn new_game_with_commands(
        game_type: GameType,
        commands: Vec<Box<dyn GameCommand>>,
        game_runner: GR,
    ) -> GameBuilder<GR> {
        let mut game_command_queue: Vec<GameCommandMeta> = vec![];

        for command in commands.into_iter() {
            let utc: DateTime<Utc> = Utc::now();
            game_command_queue.push(GameCommandMeta {
                command,
                command_time: utc,
            })
        }

        let mut game_world = World::new();

        game_world.insert_resource(GameCommands {
            queue: GameCommandQueue {
                queue: game_command_queue,
            },
            history: Default::default(),
        });
        game_world.insert_resource(ObjectIdProvider::default());

        GameBuilder {
            game_type,
            game_runner,
            game_world,
            setup_schedule: GameBuilder::<GR>::default_setup_schedule(),
            systems_schedule: Default::default(),
            type_registry: GameBuilder::<GR>::default_registry(),
        }
    }

    pub fn default_registry() -> TypeRegistry {
        TypeRegistry {
            internal: Arc::new(RwLock::new({
                let mut r = TypeRegistryInternal::empty();
                // `Parent` and `Children` must be registered so that their `ReflectMapEntities`
                // data may be used.
                //
                // While this is a little bit of a weird spot to register these, are the only
                // Bevy core types implementing `MapEntities`, so for now it's probably fine to
                // just manually register these here.
                //
                // The user can still register any custom types with `register_rollback_type()`.
                r.register::<Parent>();
                r.register::<Children>();

                r.register::<TilePos>();

                //defaults
                r.register::<ObjectId>();
                r.register::<ObjectClass>();
                r.register::<ObjectGroup>();
                r.register::<ObjectType>();

                r
            })),
        }
    }

    pub fn register_component<Type>(self) -> Self
    where
        Type: GetTypeRegistration + Reflect + Default + Component,
    {
        let mut registry = self.type_registry.write();
        registry.register::<Type>();

        let registration = registry.get_mut(std::any::TypeId::of::<Type>()).unwrap();
        registration.insert(<ReflectComponent as FromType<Type>>::from_type());
        drop(registry);
        self
    }

    pub fn register_resource<Type>(self) -> Self
    where
        Type: GetTypeRegistration + Reflect + Default + Resource,
    {
        let mut registry = self.type_registry.write();
        registry.register::<Type>();

        let registration = registry.get_mut(std::any::TypeId::of::<Type>()).unwrap();
        registration.insert(<ReflectResource as FromType<Type>>::from_type());
        drop(registry);
        self
    }

    pub fn default_setup_schedule() -> Schedule {
        let mut schedule = Schedule::default();
        
        
        schedule
            .configure_sets((StateSystems::CommandFlush, StateSystems::State))
            .add_system(apply_system_buffers.in_set(StateSystems::CommandFlush))
            .add_system(get_state_diff.after(StateSystems::State));

        schedule
    }

    pub fn build(mut self, mut world: &mut World) {
        self.setup_schedule.run(&mut self.game_world);

        world.insert_resource::<GameRuntime<GR>>(GameRuntime {
            game_runner: self.game_runner,
        });
        world.insert_resource::<GameData>(GameData {
            game_world: self.game_world,
            game_state_handler: Default::default(),
        });
        world.insert_resource::<GameInfo>(GameInfo {
            game_type: self.game_type,
            type_registry: self.type_registry,
            systems_schedule: self.systems_schedule,
        });
    }
}
