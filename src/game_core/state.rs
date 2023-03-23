use crate::mapping::tiles::Tile;
use crate::object::{ObjectGridPosition, ObjectId};
use crate::team::PlayerId;
use bevy::prelude::{Reflect, ReflectComponent, SystemSet, World};
use bevy::reflect::{TypeRegistry, TypeRegistryArc};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;

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
    pub fn get_state(&mut self, mut world: &World, type_registry: &TypeRegistryArc) -> StateEvents {
        let mut state: StateEvents = StateEvents {
            players: vec![],
            resources: vec![],
            tiles: vec![],
            despawned_objects: vec![],
        };
        // We make a temporary map of tiles so that as we get objects from the world we can insert
        // them into the right tiles information
        let mut tiles: HashMap<TilePos, TileState> = HashMap::new();
        let mut objects: HashMap<TilePos, Vec<ObjectState>> = HashMap::new();

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
                        tiles.insert(
                            *tile_pos,
                            TileState {
                                tile_pos: *tile_pos,
                                components,
                                objects_in_tile: vec![],
                            },
                        );
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
                        if let Some(objects) = objects.get_mut(&tile_pos.tile_position){
                            objects.push(ObjectState {
                                object_id: *object_id,
                                components,
                                object_grid_position: *tile_pos,
                            },)
                        }else{
                            objects.insert(
                                tile_pos.tile_position,
                                vec![ObjectState {
                                    object_id: *object_id,
                                    components,
                                    object_grid_position: *tile_pos,
                                }],
                            );
                        }
                        
                    }
                }
            }
        }

        

        for (_, mut tile) in tiles.drain() {
            if let Some(objects) = objects.get_mut(&tile.tile_pos){
                for object in objects.drain(..) {
                    tile.objects_in_tile.push(object);
                }
            }
            state.tiles.push(tile);
        }

        state
    }

    pub fn get_state_diff(&mut self, mut world: &World, type_registry: &TypeRegistry) {}

    pub fn get_updates(&mut self) -> Option<StateEvents> {
        let mut has_state = false;
        let mut new_events = StateEvents {
            players: vec![],
            resources: vec![],
            tiles: vec![],
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


/// Contains the state of a player, identified by a [`PlayerId`] component
#[derive(Debug)]
pub struct PlayerState {
    pub player_id: PlayerId,
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
    pub objects_in_tile: Vec<ObjectState>,
}

/// A list of all changed states that occured during the last simulation tick
#[derive(Debug, Default)]
pub struct StateEvents {
    pub players: Vec<PlayerState>,
    pub resources: Vec<ResourceState>,
    pub tiles: Vec<TileState>,
    pub despawned_objects: Vec<ObjectId>,
}
