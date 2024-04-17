use crate::mapping::tiles::Tile;
use crate::object::{ObjectGridPosition, ObjectId};
use crate::player::Player;
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    Commands, Component, Entity, FromReflect, Mut, Query, Reflect, Resource, SystemSet, With, World,
};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};

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
        }

        world.resource_scope(|world, mut despawned_objects: Mut<DespawnedObjects>| {
            for (id, mut changed) in despawned_objects.despawned_objects.iter_mut() {
                if changed.was_seen(for_player_id) {
                    state.despawned_objects.push(*id);
                }
            }
        });

        state
    }

    /// Simple function that will clear all changed components that have been fully seen
    pub fn clear_changed(&mut self, world: &mut World) {
        let mut system_state: SystemState<(Query<(Entity, &Changed)>, Commands)> =
            SystemState::new(world);
        let (changed_query, mut commands) = system_state.get(world);
        for (entity, changed) in changed_query.iter() {
            if changed.all_seen() {
                commands.entity(entity).remove::<Changed>();
            }
        }
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

#[derive(Clone, Eq, Debug, PartialEq, Component, Reflect, FromReflect, Serialize, Deserialize)]
pub struct Changed {
    pub players_seen: HashMap<usize, bool>,
}

impl Changed {
    pub fn all_seen(&self) -> bool {
        for (_, bool) in self.players_seen.iter() {
            if !bool {
                return false;
            }
        }
        true
    }

    pub fn was_seen(&mut self, id: usize) -> bool {
        let was_seen = self
            .players_seen
            .get(&id)
            .expect("Changed Components must include every PlayerId");
        let was_seen = *was_seen;
        self.players_seen.insert(id, true);

        was_seen
    }
}

/// Resource inserted into the world that will be used to drive sending despawned object updates
#[derive(Clone, Eq, Debug, PartialEq, Resource, Reflect, FromReflect, Serialize, Deserialize)]
pub struct DespawnedObjects {
    pub despawned_objects: HashMap<ObjectId, Changed>,
}
