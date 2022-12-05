use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::Vec4Swizzles;
use bevy::prelude::Keyframes::Translation;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::camera::ClickEvent;
use bevy_ggf::BggfDefaultPlugins;
use std::thread::spawn;

use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{
    ObjectStackingClass, StackingClass, TileObjectStacks, TileObjects, TileStackCountMax,
};
use bevy_ggf::mapping::{
    tile_pos_to_centered_map_world_pos, world_pos_to_map_transform_pos, world_pos_to_tile_pos, Map,
};
use bevy_ggf::movement::{
    MoveCompleteEvent, MoveObjectEvent, MovementType, TileMovementCosts, TileMovementRules,
};
use bevy_ggf::object::{
    Object, ObjectBundle, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectInfo, ObjectType,
};
use bevy_ggf::selection::{SelectObjectEvent, SelectableEntity, SelectedObject};

pub const OBJECT_CLASS_GROUND: ObjectClass = ObjectClass { name: "Ground" };
pub const OBJECT_GROUP_INFANTRY: ObjectGroup = ObjectGroup {
    name: "Infantry",
    object_class: &OBJECT_CLASS_GROUND,
};
pub const OBJECT_TYPE_RIFLEMAN: ObjectType = ObjectType {
    name: "Rifleman",
    object_group: &OBJECT_GROUP_INFANTRY,
};

pub const STACKING_CLASS_GROUND: StackingClass = StackingClass { name: "Ground" };

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
    mut move_event_writer: EventWriter<MoveObjectEvent>,
) {
    let tilemap_size = TilemapSize { x: 200, y: 200 };
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
    tile_stack_hashmap.insert(
        &STACKING_CLASS_GROUND,
        TileStackCountMax {
            current_count: 0,
            max_count: 1,
        },
    );
    let tile_stack_rules: TileObjectStacks = TileObjectStacks {
        tile_object_stacks: tile_stack_hashmap,
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

    let infantry_texture_handle: Handle<Image> = asset_server.load("infantry_single_sprite.png");
    let tile_size = tilemap_tile_size;
    let grid_size: TilemapGridSize = tile_size.into();
    let tile_pos = TilePos::new(0, 0);

    let entity = commands.spawn(ObjectBundle {
        object: Object,
        object_info: ObjectInfo {
            object_type: &OBJECT_TYPE_RIFLEMAN,
        },
        selectable: SelectableEntity,
        object_grid_position: ObjectGridPosition {
            grid_position: TilePos::new(0, 0),
        },
        object_stacking_class: ObjectStackingClass {
            stack_class: &STACKING_CLASS_GROUND,
        },
        sprite_bundle: SpriteBundle {
            transform: Transform {
                translation: tile_pos
                    .center_in_world(&grid_size, &tilemap_type)
                    .extend(5.0),
                ..default()
            },
            texture: infantry_texture_handle.clone(),
            ..default()
        },
    });
    move_event_writer.send(MoveObjectEvent {
        object_moving: entity.id(),
        new_pos: TilePos::new(0, 0),
    });

    let entity = commands.spawn(ObjectBundle {
        object: Object,
        object_info: ObjectInfo {
            object_type: &OBJECT_TYPE_RIFLEMAN,
        },
        selectable: SelectableEntity,
        object_grid_position: ObjectGridPosition {
            grid_position: TilePos::new(10, 10),
        },
        object_stacking_class: ObjectStackingClass {
            stack_class: &STACKING_CLASS_GROUND,
        },
        sprite_bundle: SpriteBundle {
            transform: Transform {
                translation: tile_pos
                    .center_in_world(&grid_size, &tilemap_type)
                    .extend(5.0),
                ..default()
            },
            texture: infantry_texture_handle.clone(),
            ..default()
        },
    });
    move_event_writer.send(MoveObjectEvent {
        object_moving: entity.id(),
        new_pos: TilePos::new(10, 10),
    });

    let entity = commands.spawn(ObjectBundle {
        object: Object,
        object_info: ObjectInfo {
            object_type: &OBJECT_TYPE_RIFLEMAN,
        },
        selectable: SelectableEntity,
        object_grid_position: ObjectGridPosition {
            grid_position: TilePos::new(5, 5),
        },
        object_stacking_class: ObjectStackingClass {
            stack_class: &STACKING_CLASS_GROUND,
        },
        sprite_bundle: SpriteBundle {
            transform: Transform {
                translation: tile_pos
                    .center_in_world(&grid_size, &tilemap_type)
                    .extend(5.0),
                ..default()
            },
            texture: infantry_texture_handle.clone(),
            ..default()
        },
    });
    move_event_writer.send(MoveObjectEvent {
        object_moving: entity.id(),
        new_pos: TilePos::new(5, 5),
    });
}

fn select_and_move_unit_to_tile_clicked(
    selected_entity: Res<SelectedObject>,
    map_transform: Query<(&Transform, &TilemapSize, &TilemapGridSize, &TilemapType), With<Map>>,
    mut move_event_writer: EventWriter<MoveObjectEvent>,
    mut click_event_reader: EventReader<ClickEvent>,
    mut select_object_event_writer: EventWriter<SelectObjectEvent>,
) {
    let (transform, map_size, grid_size, map_type) = map_transform.single();

    for event in click_event_reader.iter() {
        match event {
            ClickEvent::Click { world_pos } => {
                info!("World Pos: {}", world_pos);
                if let Some(selected_entity) = selected_entity.selected_entity {
                    if let Some(tile_pos) =
                        world_pos_to_tile_pos(&world_pos, transform, map_size, grid_size, map_type)
                    {
                        move_event_writer.send(MoveObjectEvent {
                            object_moving: selected_entity,
                            new_pos: tile_pos,
                        });
                    }
                } else {
                    if let Some(tile_pos) =
                        world_pos_to_tile_pos(&world_pos, transform, map_size, grid_size, map_type)
                    {
                        select_object_event_writer.send(SelectObjectEvent { tile_pos });
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_move_complete_event(
    mut selected_object: ResMut<SelectedObject>,
    mut event_reader: EventReader<MoveCompleteEvent>,
) {
    
    for event in event_reader.iter(){
        selected_object.selected_entity = None;
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
        .add_system(select_and_move_unit_to_tile_clicked)
        .add_system(handle_move_complete_event)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
