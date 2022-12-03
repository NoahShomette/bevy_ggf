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
    ObjectStackingClass, StackingClass, TileObjects, TileStackCountMax, TileObjectStacks,
};
use bevy_ggf::mapping::Map;
use bevy_ggf::movement::{MovementType, TileMovementCosts, TileMovementRules};
use bevy_ggf::object::{
    Object, ObjectBundle, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectInfo, ObjectType,
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
) {
    let tilemap_size = TilemapSize { x: 10, y: 10 };
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

    commands.spawn(map);

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
            texture: infantry_texture_handle,
            ..default()
        },
    });

}

fn add_object_to_tile(
    mut map_query: Query<(&mut Map)>,
    mut object_query: Query<(&mut ObjectGridPosition, &ObjectInfo, &ObjectStackingClass)>,
    mut tile_query: Query<(&mut TileObjectStacks, &mut TileObjects)>,
    mut object_entity_query: Query<Entity, With<ObjectGridPosition>>,
    mut tile_storage_query: Query<&mut TileStorage>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::A) {
        let map = map_query.single_mut();
        let entity = object_entity_query.single();
        let (mut object_grid_position, object_info, object_stack_class) =
            object_query.get_mut(entity).unwrap();
        let mut tile_storage = tile_storage_query.single_mut();
        let tile_pos = TilePos { x: 0, y: 0 };
        map.add_object_to_tile(
            entity,
            &mut object_grid_position,
            object_stack_class,
            &mut tile_storage,
            &mut tile_query,
            tile_pos,
        );
    }
}

fn remove_object_from_tile(
    mut map_query: Query<(&mut Map)>,
    mut object_query: Query<(&mut ObjectGridPosition, &ObjectInfo, &ObjectStackingClass)>,
    mut tile_query: Query<(&mut TileObjectStacks, &mut TileObjects)>,
    mut object_entity_query: Query<Entity, With<ObjectGridPosition>>,
    mut tile_storage_query: Query<&mut TileStorage>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::D) {
        let map = map_query.single_mut();
        let entity = object_entity_query.single();
        let (mut object_grid_position, object_info, object_stack_class) =
            object_query.get_mut(entity).unwrap();
        let mut tile_storage = tile_storage_query.single_mut();
        let tile_pos = TilePos { x: 0, y: 0 };
        map.remove_object_from_tile(
            entity,
            &object_stack_class,
            &mut tile_storage,
            &mut tile_query,
            tile_pos,
        );
    }
}

fn move_unit_to_tile_clicked(
    mut map_query: Query<(&mut Map)>,
    mut object_query: Query<
        (
            &mut Transform,
            &mut ObjectGridPosition,
            &ObjectInfo,
            &ObjectStackingClass,
        ),
        With<Object>,
    >,
    mut tile_query: Query<(&mut TileObjectStacks, &mut TileObjects)>,
    mut object_entity_query: Query<Entity, With<ObjectGridPosition>>,
    mut tilemap_q: Query<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &mut TileStorage,
            &Transform,
        ),
        Without<Object>,
    >,
    keyboard_input: Res<Input<KeyCode>>,
    mut click_event_reader: EventReader<ClickEvent>,
) {
    let entity = object_entity_query.single();

    let (mut transform, mut object_grid_position, object_info, object_stack_class) =
        object_query.get_mut(entity).unwrap();

    for event in click_event_reader.iter() {
        match event {
            ClickEvent::Click {
                world_pos,
                selected_entity,
            } => {
                info!("World Pos: {}", world_pos);
                for (map_size, grid_size, map_type, mut tile_storage, map_transform) in
                    tilemap_q.iter_mut()
                {
                    let cursor_in_map_pos: Vec2 = {
                        // Extend the cursor_pos vec3 by 1.0
                        let world_pos = world_pos.extend(0.0);
                        let cursor_pos = Vec4::from((world_pos, 1.0));
                        let cursor_in_map_pos =
                            map_transform.compute_matrix().inverse() * cursor_pos;
                        cursor_in_map_pos.xy()
                    };

                    if let Some(tile_pos) =
                        TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
                    {
                        let map = map_query.single_mut();
                        let tile_entity = tile_storage.get(&tile_pos).unwrap();
                        if let Ok((tile_stack_rules, tile_objects)) =
                            tile_query.get(tile_entity)
                        {
                            if tile_stack_rules.has_space(&object_stack_class) {
                                map.remove_object_from_tile(
                                    entity,
                                    &object_stack_class,
                                    &mut tile_storage,
                                    &mut tile_query,
                                    object_grid_position.grid_position,
                                );

                                map.add_object_to_tile(
                                    entity,
                                    &mut object_grid_position,
                                    &object_stack_class,
                                    &mut tile_storage,
                                    &mut tile_query,
                                    tile_pos,
                                );

                                let tile_world_pos =
                                    tile_pos.center_in_world(grid_size, map_type).extend(0.0);

                                let tile_adjusted_world_position: Vec2 = {
                                    // Extend the cursor_pos vec3 by 1.0

                                    let cursor_pos = Vec4::from((tile_world_pos, -1.0));
                                    let cursor_in_map_pos =
                                        map_transform.compute_matrix().inverse() * cursor_pos;
                                    cursor_in_map_pos.xy()
                                };
                                transform.translation = tile_adjusted_world_position.extend(5.0);
                            }
                            else{
                                info!("TILE FULL");
                            }
                        }
                    } else {
                        info!("Tile pos from world position didnt find a tile");
                    }
                }
            }
            _ => {}
        }
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
        .add_system(move_unit_to_tile_clicked)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
