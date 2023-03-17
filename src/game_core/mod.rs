use crate::game_core::command::{GameCommand, GameCommandMeta, GameCommandQueue, GameCommands};
use bevy::app::{App, Plugin};
use bevy::prelude::{Component, Resource, Schedule, World};
use chrono::{DateTime, Utc};

pub mod command;
pub mod runner;

pub struct BggfGamePlugin {}

impl Plugin for BggfGamePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub enum GameType {
    Networked,
    Local,
}

pub trait GameAppExt {
    fn new_game(&mut self, game_type: GameType) -> &mut Self;
    fn new_game_with_commands(
        &mut self,
        game_type: GameType,
        commands: Vec<Box<dyn GameCommand>>,
    ) -> &mut Self;
}

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Resource)]
pub struct Game<T> {
    pub game_type: GameType,
    pub game_runner: T,
    pub game_world: World,
}

impl GameAppExt for App {
    fn new_game(&mut self, game_type: GameType) -> &mut Self {
        self.insert_resource(GameCommands::default())
            .insert_resource(GameIdProvider::default())
            .insert_resource(Game {
                game_type,
                tick_schedule: Default::default(),
                game_world: Default::default(),
            });

        self
    }
    fn new_game_with_commands(
        &mut self,
        game_type: GameType,
        commands: Vec<Box<dyn GameCommand>>,
    ) -> &mut Self {
        let mut game_command_queue: Vec<GameCommandMeta> = vec![];

        for command in commands.into_iter() {
            let utc: DateTime<Utc> = Utc::now();
            game_command_queue.push(GameCommandMeta {
                command,
                command_time: utc,
            })
        }

        self.insert_resource(GameCommands {
            queue: GameCommandQueue {
                queue: game_command_queue,
            },
            history: Default::default(),
        })
        .insert_resource(GameIdProvider::default())
        .insert_resource(Game { game_type });

        self
    }
}

/// A resource inserted into the world to provide consistent unique ids to keep track of game
/// entities through potential spawns, despawns, and other shenanigans.
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Resource)]
pub struct GameIdProvider {
    pub last_id: usize,
}

impl Default for GameIdProvider {
    fn default() -> Self {
        GameIdProvider { last_id: 0 }
    }
}

impl GameIdProvider {
    pub fn next_id_component(&mut self) -> GameId {
        GameId { id: self.next_id() }
    }

    pub fn next_id(&mut self) -> usize {
        self.last_id = self.last_id.saturating_add(1);
        self.last_id
    }

    pub fn remove_last_id(&mut self) {
        self.last_id = self.last_id.saturating_sub(1);
    }
}

/// Provides a way to track entities through potential despawns, spawns, and other shenanigans. Use
/// this to reference entities and then query for the entity that it is attached to.
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct GameId {
    id: usize,
}
