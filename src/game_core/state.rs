use crate::object::ObjectId;
use crate::team::PlayerId;
use bevy::prelude::{apply_system_buffers, FromReflect, Reflect, Schedule, SystemSet, World};
use bevy::reflect::TypeRegistry;
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum StateSystems{
    CommandFlush,
    State,
}

#[derive(Default)]
pub struct GameStateHandler{
    state_events: StateEvents,
}

/// Gets the state diff of the given world from the last time it was ran
pub fn get_state_diff(mut world: &mut World){
    
}

// Should be able to call get_state to get the entire game state, and then get state diff to get only
// the state that changed since last time this system was run
impl GameStateHandler {
    pub fn get_state(&mut self, mut world: & World, type_registry: &TypeRegistry){
    }
    
    pub fn get_state_diff(&mut self, mut world: & World, type_registry: &TypeRegistry){
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
#[derive(Clone, Copy, Debug, Hash, Eq, PartialOrd, PartialEq, Ord, Reflect, FromReflect, Serialize, Deserialize)]
pub enum ChangeType {
    Modified,
    Spawned,
    Despawned,
}

/// A list of all state things that occured during the last simulation tick
#[derive(Debug, Default)]
pub struct StateEvents {
    state: Vec<StateThing>,
}
