pub mod object;
pub mod terrain;
pub mod tiles;

use crate::mapping::terrain::{TerrainType, TileTerrainInfo};
use crate::mapping::tiles::{
    GGFTileBundle, GGFTileObjectBundle, ObjectStackingClass, Tile, TileObjectStacks, TileObjects,
};
use crate::movement::TileMovementRules;
use crate::object::{Object, ObjectGridPosition};
use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, TilemapBundle};
use rand;
use rand::Rng;

/// Bundle for Mapping
pub struct BggfMappingBundle;

impl Plugin for BggfMappingBundle {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateMapTileObject>()
            .add_system(update_map_tile_object_event);
    }
}

/// Map struct used to keep track of the general structure of the map. Holds a reference to the tilemap_entity
/// that this map info applies to
#[derive(Component)]
pub struct Map {
    pub tilemap_type: TilemapType,
    pub map_size: TilemapSize,
    pub tilemap_entity: Entity,
}

impl Map {
    pub fn generate_random_map(
        mut commands: &mut Commands,
        tile_map_size: &TilemapSize,
        tilemap_type: &TilemapType,
        tilemap_tile_size: &TilemapTileSize,
        map_texture_handle: Handle<Image>,
        map_terrain_vec: &Vec<TerrainType>,
        tile_movement_rules: ResMut<TileMovementRules>,
        tile_stack_rules: TileObjectStacks,
    ) -> Entity {
        let map_size = *tile_map_size;
        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_type = *tilemap_type;
        let tilemap_entity = commands.spawn_empty().id();
        info!("{:?}", tile_movement_rules.movement_cost_rules);

        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let tile_pos = TilePos { x, y };
                let mut rng = rand::thread_rng();
                let tile_texture_index = rng.gen_range(0..map_terrain_vec.len());
                let texture_index = &map_terrain_vec[tile_texture_index];
                let tile_movement_costs = tile_movement_rules
                    .movement_cost_rules
                    .get(&map_terrain_vec[tile_texture_index])
                    .unwrap();

                let tile_entity = commands
                    .spawn(GGFTileBundle {
                        tile_bundle: TileBundle {
                            position: tile_pos,
                            texture_index: TileTextureIndex(texture_index.texture_index),
                            tilemap_id: TilemapId(tilemap_entity),
                            ..Default::default()
                        },
                        tile: Tile,
                        tile_terrain_info: TileTerrainInfo {
                            terrain_type: map_terrain_vec[tile_texture_index].clone(),
                        },
                    })
                    .insert(GGFTileObjectBundle {
                        tile_stack_rules: tile_stack_rules.clone(),
                        tile_objects: TileObjects::default(),
                    })
                    .insert(tile_movement_costs.clone())
                    .id();

                tile_storage.set(&tile_pos, tile_entity);
            }
        }

        let tile_size = *tilemap_tile_size;
        let grid_size: TilemapGridSize = tile_size.into();
        let map_type = TilemapType::default();

        commands
            .entity(tilemap_entity)
            .insert(TilemapBundle {
                grid_size,
                map_type,
                size: map_size,
                storage: tile_storage,
                texture: TilemapTexture::Single(map_texture_handle),
                tile_size,
                transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
                frustum_culling: FrustumCulling(true),
                ..Default::default()
            })
            .insert(Map {
                tilemap_type,
                map_size,
                tilemap_entity,
            })
            .id()
    }
}

/// Adds the given object to a tile while keeping the TileObjectStacks component of the tile up to date
///
/// Will Panic if tile_pos isnt a valid tile position in [`TileStorage`]

// Look at having this return a result with an error message
pub fn add_object_to_tile(
    object_to_add: Entity,
    object_grid_position: &mut ObjectGridPosition,
    object_stack_class: &ObjectStackingClass,
    tile_storage: &mut TileStorage,
    tile_query: &mut Query<(&mut TileObjectStacks, &mut TileObjects)>,
    tile_pos_to_add: TilePos,
) {
    let tile_entity = tile_storage.get(&tile_pos_to_add).unwrap();
    if let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) {
        tile_objects.add_object(object_to_add);
        object_grid_position.tile_position = tile_pos_to_add;
        tile_stack_rules.increment_object_class_count(&object_stack_class);

        info!("entities in tile: {}", tile_objects.entities_in_tile.len());
        info!(
                "tile_stacks_rules_count: {:?}",
                tile_stack_rules
                    .tile_object_stacks
                    .get(&object_stack_class.stack_class)
                    .unwrap()
            );
    }
}

/// Will Panic if object_to_add isn't an entity in the given [`TileStorage`]
pub fn remove_object_from_tile(
    object_to_remove: Entity,
    object_stack_class: &ObjectStackingClass,
    tile_storage: &mut TileStorage,
    tile_query: &mut Query<(&mut TileObjectStacks, &mut TileObjects)>,
    tile_pos_to_remove: TilePos,
) {
    let tile_entity = tile_storage.get(&tile_pos_to_remove).unwrap();
    if let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) {
        tile_objects.remove_object(object_to_remove);
        tile_stack_rules.decrement_object_class_count(&object_stack_class);

        info!("entities in tile: {}", tile_objects.entities_in_tile.len());
        info!(
                "tile_stacks_rules_count: {:?}",
                tile_stack_rules
                    .tile_object_stacks
                    .get(&object_stack_class.stack_class)
                    .unwrap()
            );
    }
}

//TODO Decide if this enum is actually needed. Might be helpful sometimes but the move unit function can probably serve the same use mostly. Except it cant just remove a unit from the tile by events
pub enum UpdateMapTileObject {
    Add{
        object_entity: Entity, 
        tile_pos: TilePos,
    },
    Remove{
        object_entity: Entity,
        tile_pos: TilePos,
    },
}

fn update_map_tile_object_event(
    mut update_event: EventReader<UpdateMapTileObject>,
    mut object_query: Query<
        (
            &mut ObjectGridPosition,
            &ObjectStackingClass,
        ),
        With<Object>,
    >,
    mut tile_query: Query<(&mut TileObjectStacks, &mut TileObjects)>,
    mut tilemap_q: Query<
        (
            &mut Map,
            &mut TileStorage,
            &Transform,
        ),
        Without<Object>,
    >,
) {
    for event in update_event.iter() {
        // gets the map components
        let (map,mut tile_storage, map_transform) = tilemap_q.single_mut();

        match event {
            UpdateMapTileObject::Add { object_entity, tile_pos } => {
                // gets the components needed to move the object
                let (mut object_grid_position, object_stack_class) =
                    object_query.get_mut(*object_entity).unwrap();
                // if a tile exists at the selected point
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    // if the tile has the needed components
                    if let Ok((_tile_stack_rules, _tile_objects)) = tile_query.get(tile_entity) {
                        add_object_to_tile(
                            *object_entity,
                            &mut object_grid_position,
                            &object_stack_class,
                            &mut tile_storage,
                            &mut tile_query,
                            *tile_pos,
                        );
                    }
                }
            }
            UpdateMapTileObject::Remove { object_entity, tile_pos } => {
                let (object_grid_position, object_stack_class) =
                    object_query.get_mut(*object_entity).unwrap();
                // if a tile exists at the selected point
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    // if the tile has the needed components
                    if let Ok((_tile_stack_rules, _tile_objects)) = tile_query.get(tile_entity) {
                        remove_object_from_tile(
                            *object_entity,
                            &object_stack_class,
                            &mut tile_storage,
                            &mut tile_query,
                            object_grid_position.tile_position,
                        );
                    }
                }
            }
        }
    }
}

// Translates a Vec2 world_position to a new Vec2 relative to the maps transform.
pub fn world_pos_to_map_transform_pos(world_pos: &Vec2, map_transform: &Transform) -> Vec2 {
    let transformed_pos: Vec2 = {
        // Extend the cursor_pos vec3 by 1.0
        let world_pos = world_pos.extend(0.0);
        let world_pos_4 = Vec4::from((world_pos, 1.0));
        let transformed_pos = map_transform.compute_matrix().inverse() * world_pos_4;
        transformed_pos.xy()
    };
    transformed_pos
}

// Translates a Vec2 world_position to a new Vec2 relative to the maps transform.
pub fn world_pos_to_tile_pos(
    world_pos: &Vec2,
    map_transform: &Transform,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) -> Option<TilePos> {
    let transformed_pos: Vec2 = {
        // Extend the cursor_pos vec3 by 1.0
        let world_pos = world_pos.extend(0.0);
        let world_pos_4 = Vec4::from((world_pos, 1.0));
        let transformed_pos = map_transform.compute_matrix().inverse() * world_pos_4;
        transformed_pos.xy()
    };

    let tile_pos_result = TilePos::from_world_pos(&transformed_pos, map_size, grid_size, map_type);
    match tile_pos_result {
        None => None,
        Some(tile_pos) => Some(tile_pos),
    }
}

pub fn tile_pos_to_centered_map_world_pos(
    tile_pos: &TilePos,
    map_transform: &Transform,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
) -> Vec2 {
    let tile_world_pos = tile_pos.center_in_world(grid_size, map_type).extend(0.0);

    let transformed_pos: Vec2 = {
        // Extend the cursor_pos vec3 by 1.0
        let tile_pos_4 = Vec4::from((tile_world_pos, -1.0));
        let transformed_pos = map_transform.compute_matrix().inverse() * tile_pos_4;
        transformed_pos.xy()
    };
    transformed_pos
}
