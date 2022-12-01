pub mod tiles;
pub mod terrain;

use std::process::id;
use crate::mapping::tiles::{GGFTileBundle, Tile};
use crate::movement::{MovementType, TileMovementCosts, TileMovementRules};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, TilemapBundle};
use rand;
use rand::Rng;
use crate::mapping::terrain::{TerrainExtensionType, TileTerrainInfo};

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
        map_terrain_vec: &Vec<TerrainExtensionType>,
        tile_movement_rules: ResMut<TileMovementRules>
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

                let tile_movement_costs = tile_movement_rules.movement_cost_rules.get(&map_terrain_vec[tile_texture_index]).unwrap();
                
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
                            terrain_extension_type: map_terrain_vec[tile_texture_index].clone(),
                        },
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
