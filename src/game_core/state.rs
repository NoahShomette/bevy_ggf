use crate::mapping::tiles::Tile;
use crate::object::{ObjectGridPosition, ObjectId};
use crate::player::{Player, PlayerList};
use bevy::ecs::component::{ComponentId, ComponentInfo};
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    Commands, Component, Entity, FromReflect, Mut, Query, Reflect, Resource, SystemSet, With, World,
};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};
use std::any::Any;

use super::saving::{ComponentBinaryState, GameSerDeRegistry, SaveId};

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
    pub fn get_entire_state(&mut self, world: &mut World) -> StateEvents {
        let mut state: StateEvents = StateEvents {
            players: vec![],
            resources: vec![],
            tiles: vec![],
            objects: vec![],
            despawned_objects: vec![],
        };

        let mut query = world.query_filtered::<(
            &dyn SaveId,
            Option<&Tile>,
            Option<&TilePos>,
            Option<&ObjectId>,
            Option<&ObjectGridPosition>,
        ), With<Changed>>();

        for (saveable_components, opt_tile, opt_tilepos, opt_object_id, opt_object_grid_pos) in
            query.iter_mut(world)
        {
            if opt_tile.is_some() {
                let mut components: Vec<ComponentBinaryState> = vec![];
                for component in saveable_components.iter() {
                    if let Some((id, binary)) = component.save() {
                        components.push(ComponentBinaryState {
                            id,
                            component: binary,
                        });
                    }
                }

                if let Some(tile_pos) = opt_tilepos {
                    state.tiles.push(TileState {
                        tile_pos: *tile_pos,
                        components,
                    });
                }
            }

            if let Some(object_id) = opt_object_id {
                let mut components: Vec<ComponentBinaryState> = vec![];
                for component in saveable_components.iter() {
                    if let Some((id, binary)) = component.save() {
                        components.push(ComponentBinaryState {
                            id,
                            component: binary,
                        });
                    }
                }

                if let Some(tile_pos) = opt_object_grid_pos {
                    state.objects.push(ObjectState {
                        object_id: *object_id,
                        components,
                        object_grid_position: *tile_pos,
                    })
                }
            }
        }
        state
    }

    pub fn get_state_diff(&mut self, world: &mut World, for_player_id: usize) -> StateEvents {
        let mut state: StateEvents = StateEvents {
            players: vec![],
            resources: vec![],
            tiles: vec![],
            objects: vec![],
            despawned_objects: vec![],
        };

        let mut query = world.query_filtered::<(
            &dyn SaveId,
            &mut Changed,
            Option<&Tile>,
            Option<&TilePos>,
            Option<&ObjectId>,
            Option<&ObjectGridPosition>,
        ), With<Changed>>();

        for (
            saveable_components,
            mut changed,
            opt_tile,
            opt_tilepos,
            opt_object_id,
            opt_object_grid_pos,
        ) in query.iter_mut(world)
        {
            if changed.was_seen(for_player_id) {
                continue;
            }
            if opt_tile.is_some() {
                let mut components: Vec<ComponentBinaryState> = vec![];
                for component in saveable_components.iter() {
                    if let Some((id, binary)) = component.save() {
                        components.push(ComponentBinaryState {
                            id,
                            component: binary,
                        });
                    }
                }

                if let Some(tile_pos) = opt_tilepos {
                    state.tiles.push(TileState {
                        tile_pos: *tile_pos,
                        components,
                    });
                }
            }

            if let Some(object_id) = opt_object_id {
                let mut components: Vec<ComponentBinaryState> = vec![];
                for component in saveable_components.iter() {
                    if let Some((id, binary)) = component.save() {
                        components.push(ComponentBinaryState {
                            id,
                            component: binary,
                        });
                    }
                }

                if let Some(tile_pos) = opt_object_grid_pos {
                    state.objects.push(ObjectState {
                        object_id: *object_id,
                        components,
                        object_grid_position: *tile_pos,
                    })
                }
            }

            if let Some(player) = world.get::<Player>(entity) {
                let mut components: Vec<Box<dyn Reflect>> = vec![];
                for component in world.inspect_entity(entity).iter() {
                    let reflect_component = type_registry
                        .get(ComponentInfo::type_id(component).unwrap())
                        .and_then(|registration| registration.data::<ReflectComponent>());
                    if let Some(reflect_component) = reflect_component {
                        if let Some(component) = reflect_component.reflect(world.entity(entity)) {
                            components.push(component.clone_value());
                        }
                    }
                }

                state.players.push(PlayerState {
                    player_id: *player,
                    components,
                })
            }
        }

        world.resource_scope(|world, mut despawned_objects: Mut<DespawnedObjects>| {
            for (id, mut changed) in despawned_objects.despawned_objects.iter_mut() {
                if !changed.check_and_register_seen(for_player_id) {
                    state.despawned_objects.push(*id);
                }
            }
        });

        world.resource_scope(|world, mut resources: Mut<ResourceChangeTracking>| {
            for (id, changed) in resources.resources.iter_mut() {
                if !changed.check_and_register_seen(for_player_id) {
                    // go through all resources and clone those that are registered
                    for (component_id, _) in world.storages().resources.iter() {
                        let reflect_component = world
                            .storages()
                            .resources
                            .get(component_id)
                            .and_then(|info| type_registry.get(info.type_id().to_owned()))
                            .and_then(|registration| registration.data::<ReflectResource>());
                        if let Some(reflect_resource) = reflect_component {
                            if let Some(resource) = reflect_resource.reflect(world) {
                                if component_id == *id {
                                    state.resources.push(ResourceState {
                                        resource: resource.clone_value(),
                                    })
                                }
                            }
                        }
                    }
                }
            }
        });

        state
    }

    /// Simple function that will clear all changed components that have been fully seen as well as
    /// the DespawnedObjects resource and the ResourceChangeTracking resource
    pub fn clear_changed(&mut self, world: &mut World, player_list: &PlayerList) {
        let mut system_state: SystemState<(Query<(Entity, &Changed)>, Commands)> =
            SystemState::new(world);
        let (changed_query, mut commands) = system_state.get(world);
        for (entity, changed) in changed_query.iter() {
            if changed.all_seen(&player_list.players) {
                commands.entity(entity).remove::<Changed>();
            }
        }

        world.resource_scope(|_world, mut despawned_objects: Mut<DespawnedObjects>| {
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

        world.resource_scope(
            |_world, mut resource_change_tracking: Mut<ResourceChangeTracking>| {
                let mut index_to_remove: Vec<ComponentId> = vec![];
                for (id, mut changed) in resource_change_tracking.resources.iter_mut() {
                    if changed.all_seen(&player_list.players) {
                        index_to_remove.push(*id);
                    }
                }
                for id in index_to_remove {
                    resource_change_tracking.resources.remove(&id);
                }
            },
        );

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

        return if has_state { Some(new_events) } else { None };
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
    pub components: Vec<ComponentBinaryState>,
}

/// Contains the entire state of a Tile, identified by its [`TilePos`] component, and all the Objects
/// in that tile
#[derive(Debug)]
pub struct TileState {
    pub tile_pos: TilePos,
    pub components: Vec<ComponentBinaryState>,
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
    /// Checks if all players that are marked as needs_state have been registered and returns the result
    pub fn all_seen(&self, players: &Vec<Player>) -> bool {
        for player in players.iter() {
            if player.needs_state && !self.players_seen.contains(&player.id()) {
                return false;
            }
        }
        true
    }

    /// Checks if the given player id has already been registered and returns the result. If the player
    /// id hasn't seen the changes then it marks it as seen and returns false. If the player id has seen
    /// the changes then it does nothing and returns true.
    pub fn check_and_register_seen(&mut self, id: usize) -> bool {
        return if self.players_seen.contains(&id) {
            true
        } else {
            self.players_seen.push(id);
            false
        };
    }

    /// Checks if the given player id has been registered and returns the results
    pub fn was_seen(&mut self, id: usize) -> bool {
        return self.players_seen.contains(&id);
    }
}

/// Resource inserted into the world that will be used to drive sending despawned object updates
#[derive(Clone, Eq, Debug, PartialEq, Resource, Reflect, FromReflect, Serialize, Deserialize)]
pub struct DespawnedObjects {
    pub despawned_objects: HashMap<ObjectId, Changed>,
}

/// Resource inserted into the world that will be used to drive sending despawned object updates
#[derive(Clone, Eq, Debug, PartialEq, Resource)]
pub struct ResourceChangeTracking {
    pub resources: HashMap<ComponentId, Changed>,
}
