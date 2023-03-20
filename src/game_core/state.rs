﻿use crate::mapping::tiles::Tile;
use crate::object::ObjectId;
use crate::team::PlayerId;
use bevy::prelude::{
    apply_system_buffers, FromReflect, Reflect, ReflectComponent, Schedule, SystemSet, World,
};
use bevy::reflect::{TypeRegistry, TypeRegistryArc};
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum StateSystems {
    CommandFlush,
    State,
}

#[derive(Default)]
pub struct GameStateHandler {
    state_events: StateEvents,
}

/// Gets the state diff of the given world from the last time it was ran
pub fn get_state_diff(mut world: &mut World) {}

// Should be able to call get_state to get the entire game state, and then get state diff to get only
// the state that changed since last time this system was run
impl GameStateHandler {
    /// returns the entire game state in a vec
    pub fn get_state(
        &mut self,
        mut world: &World,
        type_registry: &TypeRegistryArc,
    ) -> Vec<StateThing> {
        let mut state: Vec<StateThing> = vec![];
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
                        state.push(StateThing::Tile {
                            change_type: ChangeType::NoChange,
                            tile_pos: *tile_pos,
                            components,
                        })
                    } else{
                        state.push(StateThing::Tile {
                            change_type: ChangeType::NoChange,
                            tile_pos: Default::default(),
                            components,
                        })
                    }
                      
                }
            }
        }

        state
    }

    pub fn get_state_diff(&mut self, mut world: &World, type_registry: &TypeRegistry) {}

    pub fn get_updates(&mut self) -> Option<StateEvents> {
        if !self.state_events.state.is_empty() {
            let new_events = StateEvents {
                state: self.state_events.state.drain(..).collect(),
            };
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
#[derive(Debug)]
pub enum StateThing {
    Object {
        change_type: ChangeType,
        object_id: ObjectId,
        components: Vec<Box<dyn Reflect>>,
    },
    Tile {
        change_type: ChangeType,
        tile_pos: TilePos,
        components: Vec<Box<dyn Reflect>>,
    },
    Resource {
        change_type: ChangeType,
        resource: Box<dyn Reflect>,
    },
    Player {
        player_id: PlayerId,
        change_type: ChangeType,
        components: Vec<Box<dyn Reflect>>,
    },
}

/// What type of change occured
#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    Eq,
    PartialOrd,
    PartialEq,
    Ord,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
pub enum ChangeType {
    NoChange,
    Modified,
    Spawned,
    Despawned,
}

/// A list of all state things that occured during the last simulation tick
#[derive(Debug, Default)]
pub struct StateEvents {
    state: Vec<StateThing>,
}
