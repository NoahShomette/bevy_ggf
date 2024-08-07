//! # Bevy_GGF
//! Bevy Grid Game Framework (Bevy_ggf), is a framework for creating grid based tactics and strategy
//! games in the Bevy game engine. This framework is intended to provide a massive and modular
//! jumpstart to anyone who intends to make a game in the style of Advanced Wars and Final Fantasy
//! initially, and then eventually strategy games like Civilization.
//!
//!

use crate::combat::BggfCombatPlugin;
use crate::mapping::BggfMappingPlugin;
use crate::movement::BggfMovementPlugin;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::PluginGroup;

pub mod combat;
pub mod game_core;
pub mod mapping;
pub mod movement;
pub mod object;
pub mod pathfinding;
pub mod player;

pub struct BggfDefaultPlugins;

impl PluginGroup for BggfDefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BggfMovementPlugin::default())
            .add(BggfMappingPlugin)
            .add(BggfCombatPlugin::default())
    }
}
