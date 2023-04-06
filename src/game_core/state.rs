use crate::mapping::tiles::Tile;
use crate::object::{ObjectGridPosition, ObjectId};
use crate::player::{Player, PlayerList};
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    Commands, Component, Entity, FromReflect, Mut, Query, Reflect, ReflectComponent, Resource,
    SystemSet, With, World,
};
use bevy::reflect::{TypeRegistry, TypeRegistryArc};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};
use std::process::id;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum StateSystems {
    CommandFlush,
    State,
}

#[derive(Default)]
pub struct GameStateHandler {
    state_events: StateEvents,
}

// Should be able to call get_state to get the entire game state, and then get state diff to get only
// the state that changed since last time this system was run
impl GameStateHandler {
    /// returns the entire game state in a vec
    pub fn get_entire_state(
        &mut self,
        mut world: &World,
        for_player_id: Option<usize>,
        type_registry: &TypeRegistryArc,
    ) -> StateEvents {
        let mut state: StateEvents = StateEvents {
            players: vec![],
            resources: vec![],
            tiles: vec![],
            objects: vec![],
            despawned_objects: vec![],
        };
        let type_registry = type_registry.read();

        for archetype in world.archetypes().iter() {
            for entity in archetype.entities() {
                let entity_id = entity.entity();

                // tiles
                if let Some(tile) = world.get::<Tile>(entity_id) {
                    let mut components: Vec<Box<dyn Reflect>> = vec![];
                    // fill the component vectors of rollback entities
                    for component_id in archetype.components() {
                        let reflect_component = world
                            .components()
                            .get_info(component_id)
                            .and_then(|info| type_registry.get(info.type_id().unwrap()))
                            .and_then(|registration| registration.data::<ReflectComponent>());
                        if let Some(reflect_component) = reflect_component {
                            if let Some(component) =
                                reflect_component.reflect(world.entity(entity_id))
                            {
                                components.push(component.clone_value());
                            }
                        }
                    }

                    if let Some(tile_pos) = world.get::<TilePos>(entity_id) {
                        state.tiles.push(TileState {
                            tile_pos: *tile_pos,
                            components,
                        });
                    }
                }

                if let Some(object_id) = world.get::<ObjectId>(entity_id) {
                    let mut components: Vec<Box<dyn Reflect>> = vec![];
                    // fill the component vectors of rollback entities
                    for component_id in archetype.components() {
                        let reflect_component = world
                            .components()
                            .get_info(component_id)
                            .and_then(|info| type_registry.get(info.type_id().unwrap()))
                            .and_then(|registration| registration.data::<ReflectComponent>());
                        if let Some(reflect_component) = reflect_component {
                            if let Some(component) =
                                reflect_component.reflect(world.entity(entity_id))
                            {
                                components.push(component.clone_value());
                            }
                        }
                    }

                    if let Some(tile_pos) = world.get::<ObjectGridPosition>(entity_id) {
                        state.objects.push(ObjectState {
                            object_id: *object_id,
                            components,
                            object_grid_position: *tile_pos,
                        })
                    }
                }
            }
        }

        state
    }

    pub fn get_state_diff(
        &mut self,
        world: &mut World,
        for_player_id: usize,
        type_registry: &TypeRegistry,
    ) -> StateEvents {
        let mut state: StateEvents = StateEvents {
            players: vec![],
            resources: vec![],
            tiles: vec![],
            objects: vec![],
            despawned_objects: vec![],
        };

        let type_registry = type_registry.read();
        let mut query = world.query_filtered::<Entity, With<Changed>>();

        let entities: Vec<Entity> = query.iter(world).collect();

        for entity in entities.iter() {
            let entity = *entity;
            let mut entity_mut = world.entity_mut(entity);
            let mut changed = entity_mut.get_mut::<Changed>().unwrap();
            if changed.check_and_register_seen(for_player_id) {
                continue;
            }
            if let Some(_) = world.get::<Tile>(entity) {
                let mut components: Vec<Box<dyn Reflect>> = vec![];
                for component in world.inspect_entity(entity).iter() {
                    let reflect_component = type_registry
                        .get(component.type_id().unwrap())
                        .and_then(|registration| registration.data::<ReflectComponent>());
                    if let Some(reflect_component) = reflect_component {
                        if let Some(component) = reflect_component.reflect(world.entity(entity)) {
                            components.push(component.clone_value());
                        }
                    }
                }

                if let Some(tile_pos) = world.get::<TilePos>(entity) {
                    state.tiles.push(TileState {
                        tile_pos: *tile_pos,
                        components,
                    });
                }
            }

            if let Some(object_id) = world.get::<ObjectId>(entity) {
                let mut components: Vec<Box<dyn Reflect>> = vec![];
                for component in world.inspect_entity(entity).iter() {
                    let reflect_component = type_registry
                        .get(component.type_id().unwrap())
                        .and_then(|registration| registration.data::<ReflectComponent>());
                    if let Some(reflect_component) = reflect_component {
                        if let Some(component) = reflect_component.reflect(world.entity(entity)) {
                            components.push(component.clone_value());
                        }
                    }
                }

                if let Some(tile_pos) = world.get::<ObjectGridPosition>(entity) {
                    state.objects.push(ObjectState {
                        object_id: *object_id,
                        components,
                        object_grid_position: *tile_pos,
                    })
                }
            }
        }

        world.resource_scope(|world, mut despawned_objects: Mut<DespawnedObjects>| {
            for (id, mut changed) in despawned_objects.despawned_objects.iter_mut() {
                if changed.check_and_register_seen(for_player_id) {
                    state.despawned_objects.push(*id);
                }
            }
        });

        state
    }

    /// Simple function that will clear all changed components that have been fully seen
    pub fn clear_changed(&mut self, world: &mut World, player_list: &PlayerList) {
        let mut system_state: SystemState<(Query<(Entity, &Changed)>, Commands)> =
            SystemState::new(world);
        let (changed_query, mut commands) = system_state.get(world);
        for (entity, changed) in changed_query.iter() {
            if changed.all_seen(&player_list.players) {
                commands.entity(entity).remove::<Changed>();
            }
        }

        world.resource_scope(|world, mut despawned_objects: Mut<DespawnedObjects>| {
            let mut index_to_remove: Vec<ObjectId> = vec![];
            for (id, mut changed) in despawned_objects.despawned_objects.iter_mut() {
                if changed.all_seen(&player_list.players) {
                    index_to_remove.push(*id);
                }
            }
            for id in index_to_remove {
                despawned_objects.despawned_objects.remove(&id);
            }
        });
        system_state.apply(world);
    }

    pub fn get_updates(&mut self) -> Option<StateEvents> {
        let mut has_state = false;
        let mut new_events = StateEvents {
            players: vec![],
            resources: vec![],
            tiles: vec![],
            objects: vec![],
            despawned_objects: vec![],
        };
        if !self.state_events.players.is_empty() {
            has_state = true;
            new_events.players = self.state_events.players.drain(..).collect();
        }
        if !self.state_events.resources.is_empty() {
            has_state = true;
            new_events.resources = self.state_events.resources.drain(..).collect();
        }
        if !self.state_events.tiles.is_empty() {
            has_state = true;
            new_events.tiles = self.state_events.tiles.drain(..).collect();
        }
        if !self.state_events.objects.is_empty() {
            has_state = true;
            new_events.objects = self.state_events.objects.drain(..).collect();
        }
        if !self.state_events.despawned_objects.is_empty() {
            has_state = true;
            new_events.despawned_objects = self.state_events.despawned_objects.drain(..).collect();
        }

        if has_state {
            return Some(new_events);
        } else {
            return None;
        }
    }
}

/// An individual state change of a specific *thing*, Object, Tile, Resource, or Player. It is an enum
/// that matches the specific [`StateThing`] that was changed. Each enum variant contains the
/// information needed to enact that which includes Ids, the kind of
/// change represented by [`ChangeType`], and the reflected state itself

/// Contains the state of a player, identified by a [`Player`] component
#[derive(Debug)]
pub struct PlayerState {
    pub player_id: Player,
    pub components: Vec<Box<dyn Reflect>>,
}

/// Contains the state of a [`Resource`]
#[derive(Debug)]
pub struct ResourceState {
    pub resource: Box<dyn Reflect>,
}

/// Contains an objects state, identified via its [`ObjectId`] component
#[derive(Debug)]
pub struct ObjectState {
    pub object_id: ObjectId,
    pub object_grid_position: ObjectGridPosition,
    pub components: Vec<Box<dyn Reflect>>,
}

/// Contains the entire state of a Tile, identified by its [`TilePos`] component, and all the Objects
/// in that tile
#[derive(Debug)]
pub struct TileState {
    pub tile_pos: TilePos,
    pub components: Vec<Box<dyn Reflect>>,
}

/// A list of all changed states that occured during the last simulation tick
#[derive(Debug, Default)]
pub struct StateEvents {
    pub players: Vec<PlayerState>,
    pub resources: Vec<ResourceState>,
    pub tiles: Vec<TileState>,
    pub objects: Vec<ObjectState>,
    pub despawned_objects: Vec<ObjectId>,
}

#[derive(
    Default, Clone, Eq, Debug, PartialEq, Component, Reflect, FromReflect, Serialize, Deserialize,
)]
pub struct Changed {
    pub players_seen: Vec<usize>,
}

impl Changed {
    pub fn all_seen(&self, players: &Vec<Player>) -> bool {
        for player in players.iter() {
            if player.needs_state && !self.players_seen.contains(&player.id()) {
                return false;
            }
        }
        true
    }

    pub fn check_and_register_seen(&mut self, id: usize) -> bool {
        return if self.players_seen.contains(&id) {
            true
        } else {
            self.players_seen.push(id);
            false
        };
    }

    pub fn was_seen(&mut self, id: usize) -> bool {
        return self.players_seen.contains(&id);
    }
}

/// Resource inserted into the world that will be used to drive sending despawned object updates
#[derive(Clone, Eq, Debug, PartialEq, Resource, Reflect, FromReflect, Serialize, Deserialize)]
pub struct DespawnedObjects {
    pub despawned_objects: HashMap<ObjectId, Changed>,
}
