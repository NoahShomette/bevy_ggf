use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::Vec4Swizzles;
use bevy::prelude::KeyCode::L;
use bevy::prelude::Keyframes::Translation;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::camera::{ClickEvent, CursorWorldPos};
use bevy_ggf::BggfDefaultPlugins;
use std::thread::spawn;

use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{
    ObjectStackingClass, StackingClass, TileObjectStacks, TileObjects, TileStackCountMax,
};
use bevy_ggf::mapping::{
    tile_pos_to_centered_map_world_pos, world_pos_to_map_transform_pos, world_pos_to_tile_pos, Map,
    UpdateMapTileObject,
};
use bevy_ggf::movement::{
    MoveEvent, MovementInformation, MovementType, ObjectMovement, ObjectTerrainMovementRules,
    TileMovementCosts, TileMovementRules, UnitMovementBundle,
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

pub const TERRAIN_CLASSES: &'static [TerrainClass] = &[
    TerrainClass { name: "Ground" },
    TerrainClass { name: "Water" },
];

pub const TERRAIN_TYPES: &'static [TerrainType] = &[
    TerrainType {
        name: "Grassland",
        texture_index: 0,
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Forest",
        texture_index: 1,
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Mountain",
        texture_index: 2,
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Hill",
        texture_index: 3,
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Sand",
        texture_index: 4,
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "CoastWater",
        texture_index: 5,
        terrain_class: &TERRAIN_CLASSES[1],
    },
    TerrainType {
        name: "Ocean",
        texture_index: 6,
        terrain_class: &TERRAIN_CLASSES[1],
    },
];

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_movement_rules: ResMut<TileMovementRules>,
    mut move_event_writer: EventWriter<UpdateMapTileObject>,
) {
    let tilemap_size = TilemapSize { x: 100, y: 100 };
    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    let tilemap_type = TilemapType::Square;
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let terrain_extension_types: Vec<TerrainType> = vec![
        TERRAIN_TYPES[0],
        TERRAIN_TYPES[1],
        //TERRAIN_TYPES[2],
        //TERRAIN_TYPES[3],
        TERRAIN_TYPES[4],
        //TERRAIN_TYPES[5],
        //TERRAIN_TYPES[6],
    ];

    let grass = TERRAIN_TYPES[0];
    let forest = TERRAIN_TYPES[1];
    let mountain = TERRAIN_TYPES[2];
    let hill = TERRAIN_TYPES[3];
    let sand = TERRAIN_TYPES[4];

    for terrain_extension_type in terrain_extension_types.iter() {
        match terrain_extension_type.name {
            "Grassland" => {
                let mut tile_movement_cost: HashMap<&MovementType, u32> = HashMap::new();
                tile_movement_cost.insert(&MOVEMENT_TYPES[0], 1);
                tile_movement_cost.insert(&MOVEMENT_TYPES[1], 1);
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts {
                        movement_type_cost: tile_movement_cost.clone(),
                    },
                );
            }
            "Forest" => {
                let mut tile_movement_cost: HashMap<&MovementType, u32> = HashMap::new();
                tile_movement_cost.insert(&MOVEMENT_TYPES[0], 1);
                tile_movement_cost.insert(&MOVEMENT_TYPES[1], 2);
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts {
                        movement_type_cost: tile_movement_cost.clone(),
                    },
                );
            }
            "Mountain" => {
                let mut tile_movement_cost: HashMap<&MovementType, u32> = HashMap::new();
                tile_movement_cost.insert(&MOVEMENT_TYPES[0], 3);
                tile_movement_cost.insert(&MOVEMENT_TYPES[1], 3);
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts {
                        movement_type_cost: tile_movement_cost.clone(),
                    },
                );
            }
            "Hill" => {
                let mut tile_movement_cost: HashMap<&MovementType, u32> = HashMap::new();
                tile_movement_cost.insert(&MOVEMENT_TYPES[0], 2);
                tile_movement_cost.insert(&MOVEMENT_TYPES[1], 2);
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts {
                        movement_type_cost: tile_movement_cost.clone(),
                    },
                );
            }
            "Sand" => {
                let mut tile_movement_cost: HashMap<&MovementType, u32> = HashMap::new();
                tile_movement_cost.insert(&MOVEMENT_TYPES[0], 1);
                tile_movement_cost.insert(&MOVEMENT_TYPES[1], 2);
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts {
                        movement_type_cost: tile_movement_cost.clone(),
                    },
                );
            }
            &_ => {}
        }
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

    let movement_rules = ObjectTerrainMovementRules {
        terrain_class_rules: vec![&TERRAIN_CLASSES[0], &TERRAIN_CLASSES[1]],
        terrain_type_rules: ObjectTerrainMovementRules::new_terrain_type(vec![(
            &TERRAIN_TYPES[2],
            false,
        )]),
    };

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
        unit_movement_bundle: UnitMovementBundle {
            object_movement: ObjectMovement {
                move_points: 20,
                movement_type: &MOVEMENT_TYPES[0],
                object_terrain_movement_rules: movement_rules.clone(),
            },
        },
    });
    move_event_writer.send(UpdateMapTileObject::Add {
        object_entity: entity.id(),
        tile_pos: TilePos::new(0, 0),
    });

    let entity = commands.spawn(ObjectBundle {
        object: Object,
        object_info: ObjectInfo {
            object_type: &OBJECT_TYPE_RIFLEMAN,
        },
        selectable: SelectableEntity,
        object_grid_position: ObjectGridPosition {
            grid_position: TilePos::new(1, 1),
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
        unit_movement_bundle: UnitMovementBundle {
            object_movement: ObjectMovement {
                move_points: 20,
                movement_type: &MOVEMENT_TYPES[0],
                object_terrain_movement_rules: movement_rules.clone(),
            },
        },
    });
    move_event_writer.send(UpdateMapTileObject::Add {
        object_entity: entity.id(),
        tile_pos: TilePos::new(1, 1),
    });
}

fn select_and_move_unit_to_tile_clicked(
    selected_entity: Res<SelectedObject>,
    map_transform: Query<(&Transform, &TilemapSize, &TilemapGridSize, &TilemapType), With<Map>>,
    mut move_event_writer: EventWriter<MoveEvent>,
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
                        move_event_writer.send(MoveEvent::TryMoveObject {
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
    mut event_reader: EventReader<MoveEvent>,
) {
    for event in event_reader.iter() {
        match event {
            MoveEvent::MoveComplete { .. } => {
                selected_object.selected_entity = None;
            }
            _ => {}
        }
    }
}

fn handle_move_sprites(
    movement_info: Res<MovementInformation>,
    mut tilemap_q: Query<
        (
            &mut Map,
            &TilemapGridSize,
            &TilemapType,
            &mut TileStorage,
            &Transform,
        ),
        Without<Object>,
    >,
    mut sprite_entities: Local<Vec<Entity>>,
    mut sprite_handle: Local<Handle<Image>>,
    mut sprite_handle_exists: Local<bool>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let (map, grid_size, map_type, mut tile_storage, map_transform) = tilemap_q.single_mut();
    if *sprite_handle_exists != true {
        *sprite_handle = asset_server.load("movement_sprite.png");
    }
    if movement_info.available_moves.len() > 0 {
        if sprite_entities.len() == 0 {
            for i in movement_info.available_moves.iter() {
                let sprite = commands.spawn(SpriteBundle {
                    transform: Transform {
                        translation: tile_pos_to_centered_map_world_pos(
                            i,
                            map_transform,
                            grid_size,
                            map_type,
                        )
                        .extend(4.0),
                        ..default()
                    },
                    texture: sprite_handle.clone(),
                    ..default()
                });
                sprite_entities.push(sprite.id());
            }
        }
    } else {
        for sprite_entity in sprite_entities.iter() {
            commands.entity(*sprite_entity).despawn();
        }
        sprite_entities.clear();
    }
}

fn show_move_path(
    cursor_world_pos: Res<CursorWorldPos>,
    movement_information: Res<MovementInformation>,
    map_transform: Query<(&Transform, &TilemapSize, &TilemapGridSize, &TilemapType), With<Map>>,

    mut sprite_entities: Local<Vec<Entity>>,
    mut sprite_handle: Local<Handle<Image>>,
    mut sprite_handle_exists: Local<bool>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if *sprite_handle_exists != true {
        *sprite_handle = asset_server.load("dot.png");
    }
    let (transform, map_size, grid_size, map_type) = map_transform.single();
    if let Some(tile_pos) = world_pos_to_tile_pos(
        &cursor_world_pos.cursor_world_pos,
        transform,
        map_size,
        grid_size,
        map_type,
    ) {
        for sprite_entity in sprite_entities.iter() {
            commands.entity(*sprite_entity).despawn();
        }
        sprite_entities.clear();
        if movement_information.available_moves.contains(&tile_pos) {
            // get move node from movmeent information for this tile. follow the line back
            if let Some(node) = movement_information.move_nodes.get(&tile_pos) {
                let mut reached_player = false;
                let sprite = commands.spawn(SpriteBundle {
                    transform: Transform {
                        translation: tile_pos_to_centered_map_world_pos(
                            &tile_pos, transform, grid_size, map_type,
                        )
                        .extend(4.0),
                        ..default()
                    },
                    texture: sprite_handle.clone(),
                    ..default()
                });
                sprite_entities.push(sprite.id());
                let mut current_node = *node;
                while reached_player == false {
                    let new_node_pos = current_node.prior_node;
                    if let Some(new_node) = movement_information.move_nodes.get(&new_node_pos) {
                        let sprite = commands.spawn(SpriteBundle {
                            transform: Transform {
                                translation: tile_pos_to_centered_map_world_pos(
                                    &new_node_pos,
                                    transform,
                                    grid_size,
                                    map_type,
                                )
                                .extend(4.0),
                                ..default()
                            },
                            texture: sprite_handle.clone(),
                            ..default()
                        });
                        sprite_entities.push(sprite.id());
                        current_node = *new_node;

                        if new_node.move_cost.unwrap() == 0 {
                            reached_player = true;
                        }
                    }
                }
            }
        } else {
            for sprite_entity in sprite_entities.iter() {
                commands.entity(*sprite_entity).despawn();
            }
            sprite_entities.clear();
        }
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
        .add_system(handle_move_sprites)
        .add_system(show_move_path)
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //.add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
