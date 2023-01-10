use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::camera::{ClickEvent, CursorWorldPos};
use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{
    ObjectStackingClass, ObjectStacksCount, StackingClass, TileObjectStacks,
};
use bevy_ggf::mapping::{
    tile_pos_to_centered_map_world_pos, world_pos_to_tile_pos, Map, MapHandler, UpdateMapTileObject,
};
use bevy_ggf::movement::defaults::{MoveCheckSpace, MoveCheckTerrain, SquareMovementCalculator};
use bevy_ggf::movement::{
    CurrentMovementInformation, DiagonalMovement, MoveEvent, MovementSystem, MovementType,
    ObjectMovement, ObjectTerrainMovementRules, TileMovementCosts, TileMovementRules,
    UnitMovementBundle,
};
use bevy_ggf::object::{
    Object, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectInfo, ObjectType,
    UnitBundle,
};
use bevy_ggf::selection::{
    ClearSelectedObject, CurrentSelectedObject, SelectableEntity, TrySelectEvents,
};
use bevy_ggf::BggfDefaultPlugins;

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
    map_handler: ResMut<MapHandler>,
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
        TERRAIN_TYPES[2],
        //TERRAIN_TYPES[3],
        TERRAIN_TYPES[4],
        //TERRAIN_TYPES[5],
        //TERRAIN_TYPES[6],
    ];

    for terrain_extension_type in terrain_extension_types.iter() {
        match terrain_extension_type.name {
            "Grassland" => {
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts::new(vec![(&MOVEMENT_TYPES[0], 1), (&MOVEMENT_TYPES[1], 1)]),
                );
            }
            "Forest" => {
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts::new(vec![(&MOVEMENT_TYPES[0], 1), (&MOVEMENT_TYPES[1], 2)]),
                );
            }
            "Mountain" => {
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts::new(vec![(&MOVEMENT_TYPES[0], 3), (&MOVEMENT_TYPES[1], 3)]),
                );
            }
            "Hill" => {
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts::new(vec![(&MOVEMENT_TYPES[0], 2), (&MOVEMENT_TYPES[1], 2)]),
                );
            }
            "Sand" => {
                tile_movement_rules.movement_cost_rules.insert(
                    *terrain_extension_type,
                    TileMovementCosts::new(vec![(&MOVEMENT_TYPES[0], 2), (&MOVEMENT_TYPES[1], 1)]),
                );
            }
            &_ => {}
        }
    }

    let tile_stack_rules = TileObjectStacks::new(vec![(
        &STACKING_CLASS_GROUND,
        ObjectStacksCount {
            current_count: 0,
            max_count: 2,
        },
    )]);

    //let map_texture_vec: Vec<Box<dyn TerrainExtensionTraitBase>> = vec![Box::new(Grassland{}), Box::new(Hill{}), Box::new(Ocean{})];
    Map::generate_random_map(
        &mut commands,
        map_handler,
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

    let movement_rules = ObjectTerrainMovementRules::new(
        vec![&TERRAIN_CLASSES[0], &TERRAIN_CLASSES[1]],
        vec![(&TERRAIN_TYPES[2], false)],
    );

    let entity = commands.spawn(UnitBundle {
        object: Object,
        object_info: ObjectInfo {
            object_type: &OBJECT_TYPE_RIFLEMAN,
        },
        selectable: SelectableEntity,
        object_grid_position: ObjectGridPosition {
            tile_position: TilePos::new(0, 0),
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
                move_points: 10,
                movement_type: &MOVEMENT_TYPES[0],
                object_terrain_movement_rules: movement_rules.clone(),
            },
        },
    });
    move_event_writer.send(UpdateMapTileObject::Add {
        object_entity: entity.id(),
        tile_pos: TilePos::new(0, 0),
    });

    let entity = commands.spawn(UnitBundle {
        object: Object,
        object_info: ObjectInfo {
            object_type: &OBJECT_TYPE_RIFLEMAN,
        },
        selectable: SelectableEntity,
        object_grid_position: ObjectGridPosition {
            tile_position: TilePos::new(1, 1),
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

fn handle_right_click(
    mut click_event_reader: EventReader<ClickEvent>,
    mut clear_select_object_event_writer: EventWriter<ClearSelectedObject>,
) {
    for event in click_event_reader.iter() {
        match event {
            ClickEvent::RightClick { world_pos: _ } => {
                clear_select_object_event_writer.send(ClearSelectedObject)
            }
            _ => {}
        }
    }
}

fn select_and_move_unit_to_tile_clicked(
    selected_entity: Res<CurrentSelectedObject>,
    map_transform: Query<(&Transform, &TilemapSize, &TilemapGridSize, &TilemapType), With<Map>>,
    moving_object: Query<&ObjectGridPosition>,
    mut move_event_writer: EventWriter<MoveEvent>,
    mut click_event_reader: EventReader<ClickEvent>,
    mut select_object_event_writer: EventWriter<TrySelectEvents>,

) {
    let (transform, map_size, grid_size, map_type) = map_transform.single();

    for event in click_event_reader.iter() {
        match event {
            ClickEvent::Click { world_pos } => {
                info!("World Pos: {}", world_pos);
                if let Some(selected_entity) = selected_entity.object_entity {
                    if let Ok(object_tile_pos) = moving_object.get(selected_entity) {
                        if let Some(tile_pos) = world_pos_to_tile_pos(
                            &world_pos, transform, map_size, grid_size, map_type,
                        ) {
                            if object_tile_pos.tile_position != tile_pos {
                                move_event_writer.send(MoveEvent::TryMoveObject {
                                    object_moving: selected_entity,
                                    new_pos: tile_pos,
                                });
                            } else{
                                select_object_event_writer.send(TrySelectEvents::TilePos(tile_pos));
                            }
                        }
                    }
                } else {
                    if let Some(tile_pos) =
                        world_pos_to_tile_pos(&world_pos, transform, map_size, grid_size, map_type)
                    {
                        select_object_event_writer.send(TrySelectEvents::TilePos(tile_pos));
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_move_complete_event(
    mut selected_object: ResMut<CurrentSelectedObject>,
    mut event_reader: EventReader<MoveEvent>,
) {
    for event in event_reader.iter() {
        match event {
            MoveEvent::MoveComplete { .. } => {
                selected_object.object_entity = None;
            }
            _ => {}
        }
    }
}

fn handle_move_sprites(
    movement_info: Res<CurrentMovementInformation>,
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
    sprite_handle_exists: Local<bool>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let (_map, grid_size, map_type, _tile_storage, map_transform) = tilemap_q.single_mut();
    if *sprite_handle_exists != true {
        *sprite_handle = asset_server.load("movement_sprite.png");
    }
    if movement_info.available_moves.len() > 0 {
        if sprite_entities.len() == 0 {
            for i in movement_info.available_moves.iter() {
                let sprite = commands.spawn(SpriteBundle {
                    transform: Transform {
                        translation: tile_pos_to_centered_map_world_pos(
                            i.0,
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
    movement_information: Res<CurrentMovementInformation>,
    map_transform: Query<(&Transform, &TilemapSize, &TilemapGridSize, &TilemapType), With<Map>>,

    mut sprite_entities: Local<Vec<Entity>>,
    mut sprite_handle: Local<Handle<Image>>,
    sprite_handle_exists: Local<bool>,
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
        let movement_info = movement_information.into_inner();
        if movement_info.contains_move(&tile_pos) {
            // get move node from movement information for this tile. follow the line back
            if let Some(node) = movement_info.available_moves.get(&tile_pos) {
                let mut reached_player = false;
                let sprite = commands.spawn(SpriteBundle {
                    transform: Transform {
                        translation: tile_pos_to_centered_map_world_pos(
                            &tile_pos, transform, grid_size, map_type,
                        )
                        .extend(6.0),
                        ..default()
                    },
                    texture: sprite_handle.clone(),
                    ..default()
                });
                sprite_entities.push(sprite.id());
                let mut current_node = *node;
                while reached_player == false {
                    let new_node_pos = current_node.prior_tile_pos;
                    if let Some(new_node) = movement_info.available_moves.get(&new_node_pos) {
                        let sprite = commands.spawn(SpriteBundle {
                            transform: Transform {
                                translation: tile_pos_to_centered_map_world_pos(
                                    &new_node_pos,
                                    transform,
                                    grid_size,
                                    map_type,
                                )
                                .extend(6.0),
                                ..default()
                            },
                            texture: sprite_handle.clone(),
                            ..default()
                        });
                        sprite_entities.push(sprite.id());
                        current_node = *new_node;

                        if new_node.move_cost == 0 {
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
        .insert_resource(MovementSystem{
            movement_calculator: Box::new(SquareMovementCalculator { diagonal_movement: DiagonalMovement::Disabled }),
            map_type: TilemapType::Square,
            tile_move_checks: vec![Box::new(MoveCheckSpace), Box::new(MoveCheckTerrain)],
        })
        .add_startup_system(startup)
        .add_system(select_and_move_unit_to_tile_clicked)
        .add_system(handle_move_complete_event)
        .add_system(handle_move_sprites)
        .add_system(show_move_path)
        .add_system(handle_right_click)

        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //.add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
