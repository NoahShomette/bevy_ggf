pub mod object;
pub mod terrain;
pub mod tiles;

use crate::mapping::terrain::{TerrainType, TileTerrainInfo};
use crate::mapping::tiles::{
    GGFTileBundle, GGFTileObjectBundle, Tile, TileObjects, TileStackRules,
};
use crate::movement::TileMovementRules;
use crate::object::{ObjectGridPosition, ObjectInfo};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, TilemapBundle};
use rand;
use rand::Rng;

/// Map struct used to keep track of the general structure of the map. Holds a reference to the tilemap_entity
/// that this map info applies to
#[derive(Component)]
pub struct Map {
    pub tilemap_type: TilemapType,
    pub map_size: TilemapSize,
    pub tilemap_entity: Entity,
}

impl Map {
    /// Adds the given object to a tile
    pub fn add_object_to_tile(
        &self,
        object_to_add: Entity,
        object_query: &mut Query<(&mut ObjectGridPosition, &ObjectInfo)>,
        tile_storage: &mut TileStorage,
        tile_query: &mut Query<(&mut TileStackRules, &mut TileObjects)>,
        tile_pos_to_add: TilePos,
    ) {
        let (mut object_grid_position, object_info) = object_query.get_mut(object_to_add).unwrap();

        let tile_entity = tile_storage.get(&tile_pos_to_add).unwrap();
        if let Ok((mut tile_stack_rules, mut tile_objects)) = tile_query.get_mut(tile_entity) {
            if tile_stack_rules
                .has_space(&object_info.object_type.object_group.object_class)
            {
                tile_objects.entities_in_tile.push(object_to_add);
                object_grid_position.grid_position = IVec2 {
                    x: tile_pos_to_add.x as i32,
                    y: tile_pos_to_add.y as i32,
                };
                tile_stack_rules.increment_object_class_count(
                    &object_info.object_type.object_group.object_class,
                );

                info!("entities in tile: {}", tile_objects.entities_in_tile.len());
                info!(
                    "tile_stacks_rules_count: {:?}",
                    tile_stack_rules
                        .tile_stack_rules
                        .get(&object_info.object_type.object_group.object_class)
                        .unwrap()
                );
            } else {
                info!("NO SPACE IN TILE");
            }
        }
    }
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
        tile_stack_rules: TileStackRules,
    ) -> Map {
        let map_size = *tile_map_size;
        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_type = *tilemap_type;
        let tilemap_entity = commands.spawn_empty().id();

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

        commands.entity(tilemap_entity).insert(TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(map_texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            frustum_culling: FrustumCulling(true),
            ..Default::default()
        });

        Map {
            tilemap_type,
            map_size,
            tilemap_entity,
        }
    }
}
