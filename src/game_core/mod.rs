﻿//!

use crate::game_core::command::{GameCommand, GameCommandMeta, GameCommandQueue, GameCommands};
use crate::game_core::runner::GameRunner;
use crate::game_core::state::{DespawnedObjects, GameStateHandler, StateEvents, StateSystems};
use crate::mapping::terrain::TileTerrainInfo;
use crate::mapping::tiles::{
    ObjectStackingClass, Tile, TileObjectStacks, TileObjectStacksCount, TileObjects,
};
use crate::mapping::MapIdProvider;
use crate::movement::TileMovementCosts;
use crate::object::{
    Object, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectId, ObjectIdProvider, ObjectType,
};
use crate::player::{Player, PlayerList};
use bevy::app::{App, CoreSchedule, Plugin};
use bevy::ecs::world::EntityMut;
use bevy::prelude::{
    apply_system_buffers, Children, Commands, Component, FixedTime, IntoSystemAppConfig,
    IntoSystemConfig, Parent, ReflectComponent, ReflectResource, ResMut, Resource, Schedule, World,
};
use bevy::reflect::{FromType, GetTypeRegistration, Reflect, TypeRegistry, TypeRegistryInternal};
use bevy_ecs_tilemap::tiles::TilePos;
use chrono::{DateTime, Utc};
use parking_lot::{Mutex, RwLock};
use std::default::Default;
use std::sync::Arc;

pub mod command;
pub mod requests;
pub mod runner;
pub mod state;

pub struct BggfGamePlugin {}

impl Plugin for BggfGamePlugin {
    fn build(&self, app: &mut App) {}
}

/// Resource that is inserted into the game world to allow access in game systems
#[derive(Resource)]
pub struct GameTypeRegistry {
    pub type_registry: TypeRegistry,
}

/// Holds all the actual game information
#[derive(Resource)]
pub struct Game {
    /// A world that should hold all game state
    pub game_world: World,
    /// Holds component and resource registrations that will be diffed and updated
    pub type_registry: TypeRegistry,
    /// Holds updates to the game state
    pub game_state_handler: GameStateHandler,
    /// List of all players in the game
    pub player_list: PlayerList,
}

impl Game {
    pub fn get_entire_state(&mut self, for_player_id: Option<usize>) -> StateEvents {
        self.game_state_handler.get_entire_state(
            &mut self.game_world,
            for_player_id,
            &self.type_registry,
        )
    }

    pub fn get_state_diff(&mut self, for_player_id: usize) -> StateEvents {
        self.game_state_handler.get_state_diff(
            &mut self.game_world,
            for_player_id,
            &self.type_registry,
        )
    }

    pub fn execute_game_commands(&mut self) {}
}

/// Runtime that is implemented by the user to drive their game
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
    pub fn simulate(&mut self, mut game_data: &mut Game) {
        self.game_runner.simulate_game(&mut game_data.game_world);
    }
}

/// GameBuilder that creates a new game and sets it up correctly
#[derive(Resource)]
pub struct GameBuilder<GR>
where
    GR: GameRunner + 'static,
{
    pub game_runner: GR,
    pub game_world: World,
    pub setup_schedule: Schedule,
    pub type_registry: TypeRegistry,
    pub commands: Option<GameCommands>,
    pub next_player_id: usize,
    pub player_list: PlayerList,
}

impl<GR> GameBuilder<GR>
where
    GR: GameRunner,
{
    pub fn new_game(game_runner: GR) -> GameBuilder<GR> {
        let mut game_world = World::new();

        game_world.insert_resource(GameCommands::default());
        game_world.insert_resource(ObjectIdProvider::default());

        GameBuilder {
            game_runner,
            game_world,
            setup_schedule: GameBuilder::<GR>::default_setup_schedule(),
            type_registry: GameBuilder::<GR>::default_registry(),
            commands: Default::default(),
            next_player_id: 0,
            player_list: PlayerList { players: vec![] },
        }
    }
    pub fn new_game_with_commands(
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

        game_world.insert_resource(ObjectIdProvider::default());
        game_world.insert_resource(MapIdProvider::default());

        GameBuilder {
            game_runner,
            game_world,
            setup_schedule: GameBuilder::<GR>::default_setup_schedule(),
            type_registry: GameBuilder::<GR>::default_registry(),
            commands: Some(GameCommands {
                queue: GameCommandQueue {
                    queue: game_command_queue,
                },
                history: Default::default(),
            }),
            next_player_id: 0,
            player_list: PlayerList { players: vec![] },
        }
    }

    /// Removes the [`GameCommands`] from the game world and returns them. Make sure to reinsert the commands
    /// after using them
    pub fn remove_commands(&mut self) -> Option<GameCommands> {
        self.commands.take()
    }

    /// Inserts the given commands into the game world
    pub fn insert_commands(&mut self, game_commands: GameCommands) {
        self.commands = Some(game_commands);
    }

    /// Adds the default registry which has all the basic Bevy_GGF components and resources
    pub fn default_registry() -> TypeRegistry {
        TypeRegistry {
            internal: Arc::new(RwLock::new({
                let mut r = TypeRegistryInternal::new();
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

                // Other crates
                r.register::<TilePos>();
                let registration = r.get_mut(std::any::TypeId::of::<TilePos>()).unwrap();
                registration.insert(<ReflectComponent as FromType<TilePos>>::from_type());

                // tiles
                r.register::<Tile>();
                let registration = r.get_mut(std::any::TypeId::of::<Tile>()).unwrap();
                registration.insert(<ReflectComponent as FromType<Tile>>::from_type());

                r.register::<TileTerrainInfo>();
                let registration = r
                    .get_mut(std::any::TypeId::of::<TileTerrainInfo>())
                    .unwrap();
                registration.insert(<ReflectComponent as FromType<TileTerrainInfo>>::from_type());

                //r.register::<TileObjectStacks>();
                //let registration = r.get_mut(std::any::TypeId::of::<TileObjectStacks>()).unwrap();
                //registration.insert(<ReflectComponent as FromType<TileObjectStacks>>::from_type());
                r.register::<TileObjects>();
                r.register::<TileObjectStacksCount>();

                r.register::<TileObjects>();
                let registration = r.get_mut(std::any::TypeId::of::<TileObjects>()).unwrap();
                registration.insert(<ReflectComponent as FromType<TileObjects>>::from_type());

                r.register::<TileMovementCosts>();
                let registration = r
                    .get_mut(std::any::TypeId::of::<TileMovementCosts>())
                    .unwrap();
                registration.insert(<ReflectComponent as FromType<TileMovementCosts>>::from_type());

                //Objects
                r.register::<ObjectId>();
                let registration = r.get_mut(std::any::TypeId::of::<ObjectId>()).unwrap();
                registration.insert(<ReflectComponent as FromType<ObjectId>>::from_type());

                r.register::<ObjectClass>();

                r.register::<ObjectGroup>();

                r.register::<ObjectType>();

                r.register::<ObjectGridPosition>();
                let registration = r
                    .get_mut(std::any::TypeId::of::<ObjectGridPosition>())
                    .unwrap();
                registration
                    .insert(<ReflectComponent as FromType<ObjectGridPosition>>::from_type());

                r.register::<Object>();
                let registration = r.get_mut(std::any::TypeId::of::<Object>()).unwrap();
                registration.insert(<ReflectComponent as FromType<Object>>::from_type());

                r.register::<ObjectStackingClass>();
                let registration = r
                    .get_mut(std::any::TypeId::of::<ObjectStackingClass>())
                    .unwrap();
                registration
                    .insert(<ReflectComponent as FromType<ObjectStackingClass>>::from_type());

                r
            })),
        }
    }

    /// Registers a component which will be tracked, updated, and reported in state events
    pub fn register_component<Type>(&mut self)
    where
        Type: GetTypeRegistration + Reflect + Default + Component,
    {
        let mut registry = self.type_registry.write();
        registry.register::<Type>();

        let registration = registry.get_mut(std::any::TypeId::of::<Type>()).unwrap();
        registration.insert(<ReflectComponent as FromType<Type>>::from_type());
        drop(registry);
    }

    /// Registers a resource that will be tracked and reported as part of the state
    pub fn register_resource<Type>(&mut self)
    where
        Type: GetTypeRegistration + Reflect + Default + Resource,
    {
        let mut registry = self.type_registry.write();
        registry.register::<Type>();

        let registration = registry.get_mut(std::any::TypeId::of::<Type>()).unwrap();
        registration.insert(<ReflectResource as FromType<Type>>::from_type());
        drop(registry);
    }

    pub fn default_setup_schedule() -> Schedule {
        let mut schedule = Schedule::default();

        schedule
    }

    pub fn add_player(&mut self, needs_state: bool) -> (usize, EntityMut) {
        let new_player_id = self.next_player_id;
        self.next_player_id += 1;
        let player_entity = self
            .game_world
            .spawn(Player::new(new_player_id, needs_state));
        self.player_list
            .players
            .push(Player::new(new_player_id, needs_state));
        (new_player_id, player_entity)
    }

    pub fn build(mut self, mut main_world: &mut World) {
        self.setup_schedule.run(&mut self.game_world);


        main_world.insert_resource::<GameRuntime<GR>>(GameRuntime {
            game_runner: self.game_runner,
        });
        self.game_world.insert_resource(GameTypeRegistry {
            type_registry: self.type_registry.clone(),
        });
        self.game_world.insert_resource(DespawnedObjects {
            despawned_objects: Default::default(),
        });
        self.game_world.insert_resource(self.player_list.clone());


        if let Some(mut commands) = self.commands.as_mut() {
            commands.execute_buffer(&mut self.game_world);
        } else {
            self.commands = Some(GameCommands::default());
        }
        
        main_world.insert_resource(self.commands.unwrap());
        main_world.insert_resource::<Game>(Game {
            game_world: self.game_world,
            type_registry: self.type_registry,
            game_state_handler: Default::default(),
            player_list: self.player_list,
        });
    }
}
