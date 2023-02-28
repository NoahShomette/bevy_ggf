use bevy::app::{App, Plugin};
use bevy::prelude::Resource;
use crate::game::command::{GameCommands, GameCommandsHistory};

pub mod command;
pub mod runner;

pub struct BggfGamePlugin {}

impl Plugin for BggfGamePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource, Default)]
pub struct Game {
    pub commands: GameCommands,
}