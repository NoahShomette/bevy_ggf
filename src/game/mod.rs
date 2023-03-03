use crate::camera::GGFCamera2dBundle;
use crate::game::command::{GameCommand, GameCommands};
use crate::movement::TerrainMovementCosts;
use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Bundle, Camera2d, Camera2dBundle, Commands, Component, ComputedVisibility, Entity, Image, Local, Mesh, Query, Res, ResMut, Resource, TextureAtlas, World};
use bevy::render::extract_component::ExtractComponent;
use bevy::render::RenderStage::Extract as ExtractStage;
use bevy::render::{Extract, MainWorld, RenderApp};
use bevy::sprite::{ColorMaterial, Mesh2dHandle, Sprite};
use std::ops::{Deref, DerefMut};
use std::thread::spawn;
use bevy::asset::Handle;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::log::info;
use bevy::pbr::wireframe::{Wireframe, WireframeConfig};

pub mod command;
pub mod runner;

pub struct BggfGamePlugin {}

impl Plugin for BggfGamePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Bundle)]
pub struct GameBundle {
    pub game_id: GameId,
    pub game_commands: GameCommands,
}

impl GameBundle {
    pub fn spawn_new_game(game_id: usize, render_game: bool) -> GameBundle {
        let game_id = GameId::new(game_id);
        GameBundle {
            game_id: GameId::new(game_id.game_id),
            game_commands: GameCommands::new(game_id),
        }
    }
}

/// The id of the given game. Is both assigned as a component to the Game entity in the main world
/// and is inserted as a resource into the game world.
#[derive(Component, Clone, Copy, Hash, Ord, PartialOrd, PartialEq, Eq, Default, Resource)]
pub struct GameId {
    pub game_id: usize,
}

impl GameId {
    pub fn new(game_id: usize) -> GameId {
        GameId { game_id }
    }
}


#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct Id {
    id: usize,
}

/// A resource inserted into the game world to provide consistent unique ids to keep track of game
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
    pub fn next_id_component(&mut self) -> Id {
        Id { id: self.next_id() }
    }

    pub fn next_id(&mut self) -> usize {
        self.last_id = self.last_id.saturating_add_signed(1);
        self.last_id
    }
}
