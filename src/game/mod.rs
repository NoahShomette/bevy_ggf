use crate::game::command::{GameCommand, GameCommands};
use bevy::app::{App, Plugin};
use bevy::prelude::{Bundle, Component, Resource};

pub mod command;
pub mod runner;

pub struct BggfGamePlugin {}

impl Plugin for BggfGamePlugin {
    fn build(&self, app: &mut App) {}
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
