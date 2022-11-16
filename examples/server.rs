use bevy::{DefaultPlugins, MinimalPlugins};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::{App, ImagePlugin};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_ggf::networking::{GGFClient, GGFServer};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(GGFServer)
        .run();
}
