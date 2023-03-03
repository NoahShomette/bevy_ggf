pub mod object;
pub mod terrain;
pub mod tiles;

use std::process::id;
use crate::game::command::{GameCommand, GameCommands};
use crate::game::{GameId, GameIdProvider};
use crate::mapping::terrain::{TerrainType, TileTerrainInfo};
use crate::mapping::tiles::{
    BggfTileBundle, BggfTileObjectBundle, Tile, TileObjectStackingRules,
    TileObjects,
};
use crate::movement::TerrainMovementCosts;
use bevy::ecs::system::SystemState;
use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, TilemapBundle};
use rand;
use rand::Rng;

/// Bundle for Mapping
pub struct BggfMappingPlugin;

impl Plugin for BggfMappingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MapSpawned>().add_event::<MapDeSpawned>();
    }
}

pub struct MapSpawned{
    map_game_id: GameId,
}

pub struct MapDeSpawned{
    map_game_id: GameId,
}

/// Map struct used to keep track of the general structure of the map. Holds a reference to the tilemap_entity
/// that this map info applies to
#[derive(Component)]
pub struct Map {
    pub tilemap_type: TilemapType,
    pub map_size: TilemapSize,
    pub tilemap_entity: Entity,
}

pub trait MapCommandsExt {
    fn generate_random_map(
        &mut self,
        tile_map_size: TilemapSize,
        tilemap_type: TilemapType,
        tilemap_tile_size: TilemapTileSize,
        map_texture_handle: Handle<Image>,
        map_terrain_vec: Vec<TerrainType>,
        tile_stack_rules: TileObjectStackingRules,
    ) -> SpawnRandomMap;
}

impl MapCommandsExt for GameCommands {
    fn generate_random_map(
        &mut self,
        tile_map_size: TilemapSize,
        tilemap_type: TilemapType,
        tilemap_tile_size: TilemapTileSize,
        map_texture_handle: Handle<Image>,
        map_terrain_type_vec: Vec<TerrainType>,
        tile_stack_rules: TileObjectStackingRules,
    ) -> SpawnRandomMap {
        self.queue.push(
            SpawnRandomMap {
                tile_map_size,
                tilemap_type,
                tilemap_tile_size,
                map_texture_handle: map_texture_handle.clone(),
                map_terrain_type_vec: map_terrain_type_vec.clone(),
                tile_stack_rules: tile_stack_rules.clone(),
                spawned_map_id: None,
            });
        SpawnRandomMap {
            tile_map_size,
            tilemap_type,
            tilemap_tile_size,
            map_texture_handle,
            map_terrain_type_vec,
            tile_stack_rules,
            spawned_map_id: None,
        }
    }
}

#[derive(Clone)]
pub struct SpawnRandomMap {
    tile_map_size: TilemapSize,
    tilemap_type: TilemapType,
    tilemap_tile_size: TilemapTileSize,
    map_texture_handle: Handle<Image>,
    map_terrain_type_vec: Vec<TerrainType>,
    tile_stack_rules: TileObjectStackingRules,
    spawned_map_id: Option<GameId>,
}

impl GameCommand for SpawnRandomMap {
    fn execute(&mut self, world: &mut World) -> Result<Option<Box<dyn GameCommand>>, String> {
        let terrain_movement_costs = world.resource::<TerrainMovementCosts>();
        let terrain_movement_costs = terrain_movement_costs.clone();

        let map_size = self.tile_map_size;
        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_type = self.tilemap_type;
        let tilemap_entity = world.spawn_empty().id();

        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let tile_pos = TilePos { x, y };
                let mut rng = rand::thread_rng();
                let tile_texture_index = rng.gen_range(0..self.map_terrain_type_vec.len());
                let texture_index = self.map_terrain_type_vec[tile_texture_index];
                let tile_movement_costs = terrain_movement_costs
                    .movement_cost_rules
                    .get(&self.map_terrain_type_vec[tile_texture_index])
                    .unwrap();

                let tile_entity = world
                    .spawn(BggfTileBundle {
                        tile_bundle: TileBundle {
                            position: tile_pos,
                            texture_index: TileTextureIndex(texture_index.texture_index),
                            tilemap_id: TilemapId(tilemap_entity),
                            ..Default::default()
                        },
                        tile: Tile,
                        tile_terrain_info: TileTerrainInfo {
                            terrain_type: self.map_terrain_type_vec[tile_texture_index],
                        },
                    })
                    .insert(BggfTileObjectBundle {
                        tile_stack_rules: self.tile_stack_rules.clone(),
                        tile_objects: TileObjects::default(),
                    })
                    .insert(tile_movement_costs.clone())
                    .id();

                tile_storage.set(&tile_pos, tile_entity);
            }
        }

        let tile_size = self.tilemap_tile_size;
        let grid_size: TilemapGridSize = tile_size.into();
        let map_type = TilemapType::default();

        let mut object_id_provider = world.resource_mut::<GameIdProvider>();
        let id = object_id_provider.next_id_component();
        
        world.send_event::<MapSpawned>(MapSpawned{
            map_game_id: id,
        });
        
        world
            .entity_mut(tilemap_entity)
            .insert(TilemapBundle {
                grid_size,
                map_type,
                size: map_size,
                storage: tile_storage,
                texture: TilemapTexture::Single(self.map_texture_handle.clone_weak()),
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
            .insert(id);
        Ok(Some(Box::new(SpawnRandomMap {
            tile_map_size: self.tile_map_size,
            tilemap_type,
            tilemap_tile_size: self.tilemap_tile_size,
            map_texture_handle: self.map_texture_handle.clone(),
            map_terrain_type_vec: self.map_terrain_type_vec.clone(),
            tile_stack_rules: self.tile_stack_rules.clone(),
            spawned_map_id: Some(id),
        })))
    }

    fn rollback(&mut self, mut world: &mut World) -> Result<Option<Box<dyn GameCommand>>, String> {
        let mut system_state: SystemState<(Query<(Entity, &GameId, &TileStorage)>, Commands)> =
            SystemState::new(&mut world);

        let (mut object_query, mut commands) = system_state.get_mut(&mut world);

        let Some((entity, _, tile_storage)) = object_query.iter_mut().find(|(_, id, _)| id == &&self.spawned_map_id.expect("Rollback can only be called after execute which returns an entity id")) else {
            return Err(String::from("No entity found"));
        };

        for entity in tile_storage.iter().filter(|option| option.is_some()) {
            commands.entity(entity.unwrap()).despawn_recursive();
        }
        system_state.apply(world);
        world.entity_mut(entity).despawn_recursive();

        world.send_event::<MapDeSpawned>(MapDeSpawned{
            map_game_id: self.spawned_map_id.unwrap(),
        });
        
        return Ok(Some(Box::new(SpawnRandomMap {
            tile_map_size: self.tile_map_size,
            tilemap_type: self.tilemap_type,
            tilemap_tile_size: self.tilemap_tile_size,
            map_texture_handle: self.map_texture_handle.clone(),
            map_terrain_type_vec: self.map_terrain_type_vec.clone(),
            tile_stack_rules: self.tile_stack_rules.clone(),
            spawned_map_id: None,
        })));
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

    TilePos::from_world_pos(&transformed_pos, map_size, grid_size, map_type)
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
