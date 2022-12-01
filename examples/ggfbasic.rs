use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::{BggfDefaultPlugins};

use bevy_ggf::mapping::Map;
use bevy_ggf::mapping::terrain::{TerrainBaseType, TerrainExtensionType};
use bevy_ggf::movement::{MovementType, TileMovementCosts, TileMovementRules};


pub const MOVEMENT_TYPES: &'static [MovementType] = &[
    MovementType { name: "Normal" },
    MovementType { name: "Tread" },
];

pub const TERRAIN_BASE_TYPES: &'static [TerrainBaseType] = &[
    TerrainBaseType { name: "Ground" },
    TerrainBaseType { name: "Water" },
];

pub const TERRAIN_EXTENSION_TYPES: &'static [TerrainExtensionType] = &[
    TerrainExtensionType {
        name: "Grassland",
        texture_index: 0,
        terrain_base_type: &TERRAIN_BASE_TYPES[0],
    },
    TerrainExtensionType {
        name: "Forest",
        texture_index: 1,
        terrain_base_type: &TERRAIN_BASE_TYPES[0],
    },
    TerrainExtensionType {
        name: "Mountain",
        texture_index: 2,
        terrain_base_type: &TERRAIN_BASE_TYPES[0],
    },
    TerrainExtensionType {
        name: "Hill",
        texture_index: 3,
        terrain_base_type: &TERRAIN_BASE_TYPES[0],
    },
    TerrainExtensionType {
        name: "Sand",
        texture_index: 4,
        terrain_base_type: &TERRAIN_BASE_TYPES[0],
    },
    TerrainExtensionType {
        name: "CoastWater",
        texture_index: 5,
        terrain_base_type: &TERRAIN_BASE_TYPES[1],
    },
    TerrainExtensionType {
        name: "Ocean",
        texture_index: 6,
        terrain_base_type: &TERRAIN_BASE_TYPES[1],
    },
];

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut tile_movement_rules: ResMut<TileMovementRules>) {

    let tilemap_size = TilemapSize { x: 100, y: 100 };
    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    let tilemap_type = TilemapType::Square;
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let terrain_extension_types: Vec<TerrainExtensionType> = vec![
        TERRAIN_EXTENSION_TYPES[0],
        TERRAIN_EXTENSION_TYPES[1],
        TERRAIN_EXTENSION_TYPES[2],
        TERRAIN_EXTENSION_TYPES[3],
        TERRAIN_EXTENSION_TYPES[4],
        TERRAIN_EXTENSION_TYPES[5],
        TERRAIN_EXTENSION_TYPES[6],
    ];

    let mut tile_movement_cost: HashMap<&MovementType, u32> = HashMap::new();
    tile_movement_cost.insert(&MOVEMENT_TYPES[0], 1);
    tile_movement_cost.insert(&MOVEMENT_TYPES[1], 1);
    
    for terrain_extension_type in terrain_extension_types.iter() {
        tile_movement_rules.movement_cost_rules.insert(*terrain_extension_type, TileMovementCosts{ movement_type_cost: tile_movement_cost.clone() });
    }
    
    //let map_texture_vec: Vec<Box<dyn TerrainExtensionTraitBase>> = vec![Box::new(Grassland{}), Box::new(Hill{}), Box::new(Ocean{})];
    let map = Map::generate_random_map(
        &mut commands,
        &tilemap_size,
        &tilemap_type,
        &tilemap_tile_size,
        texture_handle,
        &terrain_extension_types,
        tile_movement_rules,
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
