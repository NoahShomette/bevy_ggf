//! # Bevy_GGF
//! Bevy Grid Game Framework (Bevy_ggf), is a framework for creating grid based tactics and strategy
//! games in the Bevy game engine. This framework is intended to provide a massive and modular
//! jumpstart to anyone who intends to make a game in the style of Advanced Wars and Final Fantasy
//! initially, and then eventually strategy games like Civilization.
//!
//!

use crate::camera::BggfCameraPlugin;
use crate::mapping::BggfMappingPlugin;
use crate::movement::BggfMovementPlugin;
use crate::selection::BggfSelectionPlugin;
use bevy::app::{App, Plugin, PluginGroupBuilder};
use bevy::prelude::PluginGroup;
use iyes_loopless::prelude::AppLooplessStateExt;

pub mod camera;
pub mod mapping;
pub mod movement;
pub mod object;
pub mod selection;
pub mod combat;
pub mod team;

pub struct BggfCorePlugin;

impl Plugin for BggfCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::Menu);
    }
}

pub struct BggfDefaultPlugins;

impl PluginGroup for BggfDefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BggfCorePlugin)
            .add(BggfCameraPlugin)
            .add(BggfSelectionPlugin)
            .add(BggfMovementPlugin::default())
            .add(BggfMappingPlugin)
    }
}

/// The 2 overall states that a Bevy_GGF game can be in. If you think there should be more then submit
/// an issue on github and it can be discussed!
///
/// These two states are used for general logic and running the base game systems. Your game should always
/// be in one of these states.
///
/// ## Menu
///
/// Represents a menu outside of a game. Eg, the main menu, or an after game screen
///
/// ## `InGame`
///
/// Represents any time you are in a game and game logic should happen. Eg, starting a match, etc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    Menu,
    InGame,
}
