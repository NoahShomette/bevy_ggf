use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::{BggfDefaultPlugins};

use bevy_ggf::mapping::Map;
use bevy_ggf::mapping::terrain::{TERRAIN_EXTENSION_TYPES, TerrainExtensionType};

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {

    let tilemap_size = TilemapSize { x: 100, y: 100 };
    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    let tilemap_type = TilemapType::Square;
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let map_texture_vec: Vec<TerrainExtensionType> = vec![
        TERRAIN_EXTENSION_TYPES[0],
        TERRAIN_EXTENSION_TYPES[1],
        TERRAIN_EXTENSION_TYPES[2],
        TERRAIN_EXTENSION_TYPES[3],
        TERRAIN_EXTENSION_TYPES[4],
        TERRAIN_EXTENSION_TYPES[5],
        TERRAIN_EXTENSION_TYPES[6],
    ];
    //let map_texture_vec: Vec<Box<dyn TerrainExtensionTraitBase>> = vec![Box::new(Grassland{}), Box::new(Hill{}), Box::new(Ocean{})];
    let map = Map::generate_random_map(
        &mut commands,
        &tilemap_size,
        &tilemap_type,
        &tilemap_tile_size,
        texture_handle,
        &map_texture_vec,
    );

    commands.spawn(map);
}

fn despawn_map(
    mut query: Query<(Entity, &Map)>,
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let (entity, map) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::A) {
        commands.entity(entity).despawn_descendants();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            window: WindowDescriptor {
                width: 1270.0,
                height: 720.0,
                title: String::from(
                    "Basic Example - Press Space to change Texture and H to show/hide tilemap.",
                ),
                ..Default::default()
            },
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(BggfDefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(despawn_map)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
