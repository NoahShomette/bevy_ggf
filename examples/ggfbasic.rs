use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::Keyframes::Translation;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::BggfDefaultPlugins;
use std::thread::spawn;

use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{ObjectStackingClass, StackingClass, TileObjects, TileStackCountMax, TileStackRules};
use bevy_ggf::mapping::Map;
use bevy_ggf::movement::{MovementType, TileMovementCosts, TileMovementRules};
use bevy_ggf::object::{
    ObjectBundle, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectInfo, ObjectType,
};
use bevy_ggf::selection::SelectableEntity;


pub const OBJECT_CLASS_GROUND: ObjectClass = ObjectClass { name: "Ground" };
pub const OBJECT_GROUP_INFANTRY: ObjectGroup = ObjectGroup {
    name: "Infantry",
    object_class: &OBJECT_CLASS_GROUND,
};
pub const OBJECT_TYPE_RIFLEMAN: ObjectType = ObjectType {
    name: "Rifleman",
    object_group: &OBJECT_GROUP_INFANTRY,
};

pub const STACKING_CLASS_GROUND: StackingClass = StackingClass{name: "Ground"};

pub const MOVEMENT_TYPES: &'static [MovementType] = &[
    MovementType { name: "Infantry" },
    MovementType { name: "Tread" },
];


pub const TERRAIN_BASE_TYPES: &'static [TerrainClass] = &[
    TerrainClass { name: "Ground" },
    TerrainClass { name: "Water" },
];

pub const TERRAIN_EXTENSION_TYPES: &'static [TerrainType] = &[
    TerrainType {
        name: "Grassland",
        texture_index: 0,
        terrain_class: &TERRAIN_BASE_TYPES[0],
    },
    TerrainType {
        name: "Forest",
        texture_index: 1,
        terrain_class: &TERRAIN_BASE_TYPES[0],
    },
    TerrainType {
        name: "Mountain",
        texture_index: 2,
        terrain_class: &TERRAIN_BASE_TYPES[0],
    },
    TerrainType {
        name: "Hill",
        texture_index: 3,
        terrain_class: &TERRAIN_BASE_TYPES[0],
    },
    TerrainType {
        name: "Sand",
        texture_index: 4,
        terrain_class: &TERRAIN_BASE_TYPES[0],
    },
    TerrainType {
        name: "CoastWater",
        texture_index: 5,
        terrain_class: &TERRAIN_BASE_TYPES[1],
    },
    TerrainType {
        name: "Ocean",
        texture_index: 6,
        terrain_class: &TERRAIN_BASE_TYPES[1],
    },
];

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_movement_rules: ResMut<TileMovementRules>,
) {
    let tilemap_size = TilemapSize { x: 100, y: 100 };
    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    let tilemap_type = TilemapType::Square;
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let terrain_extension_types: Vec<TerrainType> = vec![
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
        tile_movement_rules.movement_cost_rules.insert(
            *terrain_extension_type,
            TileMovementCosts {
                movement_type_cost: tile_movement_cost.clone(),
            },
        );
    }

    let mut tile_stack_hashmap: HashMap<&StackingClass, TileStackCountMax> = HashMap::new();
    tile_stack_hashmap.insert(&STACKING_CLASS_GROUND, TileStackCountMax{ current_count: 0, max_count: 1 });
    let tile_stack_rules: TileStackRules = TileStackRules {
        tile_stack_rules: tile_stack_hashmap,
    };

    //let map_texture_vec: Vec<Box<dyn TerrainExtensionTraitBase>> = vec![Box::new(Grassland{}), Box::new(Hill{}), Box::new(Ocean{})];
    let map = Map::generate_random_map(
        &mut commands,
        &tilemap_size,
        &tilemap_type,
        &tilemap_tile_size,
        texture_handle,
        &terrain_extension_types,
        tile_movement_rules,
        tile_stack_rules,
    );

    commands.spawn(map);

    let infantry_texture_handle: Handle<Image> = asset_server.load("infantry_single_sprite.png");

    commands.spawn(ObjectBundle {
        object_info: ObjectInfo {
            object_type: &OBJECT_TYPE_RIFLEMAN,
        },
        selectable: SelectableEntity,
        object_grid_position: ObjectGridPosition {
            grid_position: Default::default(),
        },
        object_stacking_class: ObjectStackingClass { stack_class: &STACKING_CLASS_GROUND },
        sprite_bundle: SpriteBundle {
            transform: Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 5.0,
                },
                ..default()
            },
            texture: infantry_texture_handle,
            ..default()
        },
    });
}

fn add_object_to_tile(
    mut map_query: Query<(&mut Map)>,
    mut object_query: Query<(&mut ObjectGridPosition, &ObjectInfo, &ObjectStackingClass)>,
    mut tile_query: Query<(&mut TileStackRules, &mut TileObjects)>,
    mut object_entity_query: Query<Entity, With<ObjectGridPosition>>,
    mut tile_storage_query: Query<&mut TileStorage>,
    keyboard_input: Res<Input<KeyCode>>,

) {
    if keyboard_input.just_pressed(KeyCode::A) {
        let map = map_query.single_mut();
        let entity = object_entity_query.single();
        let mut tile_storage = tile_storage_query.single_mut();
        let tile_pos = TilePos { x: 0, y: 0 };
        map.add_object_to_tile(entity, &mut object_query, &mut tile_storage, &mut tile_query, tile_pos);
        
    }
    
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
        .add_system(add_object_to_tile)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
