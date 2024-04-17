//!

use crate::game_core::change_detection::{
    despawn_objects, track_component_changes, track_resource_changes,
};
use crate::game_core::command::{GameCommand, GameCommandMeta, GameCommandQueue, GameCommands};
use crate::game_core::runner::{GameRunner, GameRuntime, PostBaseSets, PreBaseSets};
use crate::game_core::state::{
    DespawnedObjects, GameStateHandler, ResourceChangeTracking, StateEvents,
};
use crate::mapping::terrain::TileTerrainInfo;
use crate::mapping::tiles::{
    ObjectStackingClass, Tile, TileObjectStacksCount, TileObjects, TilePosition,
};
use crate::mapping::MapIdProvider;
use crate::movement::TileMovementCosts;
use crate::object::{
    Object, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectId, ObjectIdProvider, ObjectInfo,
    ObjectType,
};
use crate::player::{Player, PlayerList, PlayerMarker};
use bevy::app::{App, Plugin};
use bevy::ecs::world::EntityMut;
use bevy::prelude::*;
use bevy::reflect::{FromType, GetTypeRegistration, Reflect, TypeRegistry, TypeRegistryInternal};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_trait_query::RegisterExt;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::default::Default;

use self::saving::{GameSerDeRegistry, SaveId};

pub mod change_detection;
pub mod command;
pub mod requests;
pub mod runner;
pub mod save_id_implementations;
pub mod saving;
pub mod state;

pub struct BggfGamePlugin {}

impl Plugin for BggfGamePlugin {
    fn build(&self, app: &mut App) {}
}

/// Holds all the actual game information
#[derive(Resource)]
pub struct Game {
    /// A world that should hold all game state
    pub game_world: World,
    /// Holds component and resource registrations that will be diffed and updated
    pub component_registry: GameSerDeRegistry,
    /// Holds updates to the game state
    pub game_state_handler: GameStateHandler,
    /// List of all players in the game
    pub player_list: PlayerList,
}

impl Game {
    pub fn get_entire_state(&mut self) -> StateEvents {
        self.game_state_handler
            .get_entire_state(&mut self.game_world)
    }

    pub fn get_state_diff(&mut self, for_player_id: usize) -> StateEvents {
        self.game_state_handler
            .get_state_diff(&mut self.game_world, for_player_id)
    }

    pub fn clear_changed(&mut self) {
        self.game_state_handler
            .clear_changed(&mut self.game_world, &self.player_list);
    }

    pub fn execute_game_commands(&mut self) {}
}

/// GameBuilder that creates a new game and sets it up correctly
#[derive(Resource)]
pub struct GameBuilder<GR>
where
    GR: GameRunner + 'static,
{
    pub game_runner: GR,
    /// A schedule that is run before the GameRunner::simulate_game function
    pub game_pre_schedule: Schedule,
    /// A schedule that is run after the GameRunner::simulate_game function
    pub game_post_schedule: Schedule,
    pub game_world: World,
    /// A schedule that is run as the last item before inserting the Game Resource during setup. Use
    /// this for systems that must be run once when the game is setup and only then
    pub setup_schedule: Schedule,
    pub game_serde_registry: GameSerDeRegistry,
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
            game_pre_schedule: GameBuilder::<GR>::default_game_pre_schedule(),
            game_post_schedule: GameBuilder::<GR>::default_game_post_schedule(),
            game_world,
            setup_schedule: GameBuilder::<GR>::default_setup_schedule(),
            game_serde_registry: GameSerDeRegistry::default_registry(),
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
            game_pre_schedule: GameBuilder::<GR>::default_game_pre_schedule(),
            game_post_schedule: GameBuilder::<GR>::default_game_post_schedule(),
            game_world,
            setup_schedule: GameBuilder::<GR>::default_setup_schedule(),
            game_serde_registry: GameSerDeRegistry::default_registry(),
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
    pub fn add_default_registrations(&mut self) {
        self.game_world
            .register_component_as::<dyn SaveId, TilePosition>();
        self.game_world.register_component_as::<dyn SaveId, Tile>();
        self.game_world
            .register_component_as::<dyn SaveId, TileTerrainInfo>();
        self.game_world
            .register_component_as::<dyn SaveId, TileObjects>();
        self.game_world
            .register_component_as::<dyn SaveId, TileMovementCosts>();
        self.game_world
            .register_component_as::<dyn SaveId, ObjectId>();
        self.game_world
            .register_component_as::<dyn SaveId, ObjectGridPosition>();
        self.game_world
            .register_component_as::<dyn SaveId, Object>();
        self.game_world
            .register_component_as::<dyn SaveId, ObjectStackingClass>();
        self.game_world
            .register_component_as::<dyn SaveId, PlayerMarker>();
    }

    pub fn default_components_track_changes(&mut self) {
        self.register_component_track_changes::<Parent>();
        self.register_component_track_changes::<Children>();

        self.register_component_track_changes::<TilePos>();
        self.register_component_track_changes::<Tile>();
        self.register_component_track_changes::<TileTerrainInfo>();
        self.register_component_track_changes::<TileObjects>();
        //self.register_component_track_changes::<TileObjectStacks>();
        self.register_component_track_changes::<TileMovementCosts>();

        self.register_component_track_changes::<ObjectId>();
        self.register_component_track_changes::<ObjectGridPosition>();
        self.register_component_track_changes::<Object>();
        self.register_component_track_changes::<ObjectStackingClass>();
        self.register_component_track_changes::<ObjectInfo>();

        self.register_component_track_changes::<PlayerMarker>();
    }

    /// Inserts a system into GameRunner::game_post_schedule that will track the specified Component
    /// and insert a Changed::default() component when it detects a change
    pub fn register_component_track_changes<C>(&mut self)
    where
        C: Component,
    {
        self.game_post_schedule
            .add_system(track_component_changes::<C>.in_base_set(PostBaseSets::Main));
    }

    /// Registers a resource which will be tracked, updated, and reported in state events
    pub fn register_resource_track_changes<R>(&mut self)
    where
        R: Resource,
    {
        self.game_post_schedule
            .add_system(track_resource_changes::<R>.in_base_set(PostBaseSets::Main));
    }

    /// Registers a component which will be tracked, updated, and reported in state events. Also adds
    /// the component to change detection
    pub fn register_component<Type>(&mut self)
    where
        Type: Component + SaveId + Serialize + DeserializeOwned,
    {
        self.game_serde_registry.register_component::<Type>();
        self.game_world.register_component_as::<dyn SaveId, Type>();
    }

    pub fn default_setup_schedule() -> Schedule {
        let mut schedule = Schedule::default();

        schedule
    }
    pub fn default_game_pre_schedule() -> Schedule {
        let mut schedule = Schedule::default();
        schedule
            .configure_sets(
                (
                    PreBaseSets::Pre,
                    PreBaseSets::PreCommandFlush,
                    PreBaseSets::Main,
                    PreBaseSets::MainCommandFlush,
                    PreBaseSets::Post,
                    PreBaseSets::PostCommandFlush,
                )
                    .chain(),
            )
            .add_system(apply_system_buffers.in_base_set(PreBaseSets::PreCommandFlush))
            .add_system(apply_system_buffers.in_base_set(PreBaseSets::MainCommandFlush))
            .add_system(apply_system_buffers.in_base_set(PreBaseSets::PostCommandFlush));

        schedule
    }

    pub fn default_game_post_schedule() -> Schedule {
        let mut schedule = Schedule::default();
        schedule
            .configure_sets(
                (
                    PostBaseSets::PreCommandFlush,
                    PostBaseSets::Pre,
                    PostBaseSets::MainCommandFlush,
                    PostBaseSets::Main,
                    PostBaseSets::PostCommandFlush,
                    PostBaseSets::Post,
                )
                    .chain(),
            )
            .add_system(apply_system_buffers.in_base_set(PostBaseSets::PreCommandFlush))
            .add_system(apply_system_buffers.in_base_set(PostBaseSets::MainCommandFlush))
            .add_system(apply_system_buffers.in_base_set(PostBaseSets::PostCommandFlush));

        schedule.add_system(despawn_objects.in_base_set(PostBaseSets::Pre));
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

    pub fn build(mut self, main_world: &mut World) {
        self.setup_schedule.run(&mut self.game_world);
        main_world.insert_resource::<GameRuntime<GR>>(GameRuntime {
            game_runner: self.game_runner,
            game_pre_schedule: self.game_pre_schedule,
            game_post_schedule: self.game_post_schedule,
        });
        self.game_world
            .insert_resource(self.game_serde_registry.clone());
        self.game_world.insert_resource(DespawnedObjects {
            despawned_objects: Default::default(),
        });
        self.game_world.insert_resource(ResourceChangeTracking {
            resources: Default::default(),
        });
        self.game_world.insert_resource(self.player_list.clone());

        if let Some(commands) = self.commands.as_mut() {
            commands.execute_buffer(&mut self.game_world);
        } else {
            self.commands = Some(GameCommands::default());
        }

        main_world.insert_resource(self.commands.unwrap());

        self.setup_schedule.run(&mut self.game_world);

        main_world.insert_resource::<Game>(Game {
            game_world: self.game_world,
            component_registry: self.game_serde_registry,
            game_state_handler: Default::default(),
            player_list: self.player_list,
        });
    }
}
