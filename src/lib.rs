//! # Bevy_GGF
//! Bevy Grid Game Framework (Bevy_ggf), is a framework for creating grid based tactics and strategy
//! games in the Bevy game engine. This framework is intended to provide a massive and modular
//! jumpstart to anyone who intends to make a game in the style of Advanced Wars and Final Fantasy
//! initially, and then eventually strategy games like Civilization.
//!
//!

use crate::camera::BggfCameraPlugin;
use crate::selection::BggfSelectionPlugin;
use bevy::app::{App, Plugin, PluginGroupBuilder};
use bevy::prelude::PluginGroup;
use iyes_loopless::prelude::AppLooplessStateExt;
use crate::movement::BggfMovementPlugin;

pub mod camera;
mod helpers;
pub mod mapping;
pub mod movement;
pub mod selection;
pub mod object;

pub struct BggfCorePlugin;

impl Plugin for BggfCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::MainMenu);
    }
}

pub struct BggfDefaultPlugins;

impl PluginGroup for BggfDefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BggfCorePlugin)
            .add(BggfCameraPlugin)
            .add(BggfSelectionPlugin)
            .add(BggfMovementPlugin)
    }
}

/// The 3 states that a Bevy_GGF game can be in. If you require more or any unique states then submit
/// an issue on github and I'll figure out how to make that happen!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    InGame,
    Editor,
}
