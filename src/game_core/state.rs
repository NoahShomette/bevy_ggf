use bevy::prelude::Reflect;
use bevy_ecs_tilemap::tiles::TilePos;
use crate::object::ObjectId;

pub enum StateType{
    Object{
        object_id: ObjectId
    },
    Tile{
        tile_pos: TilePos
    },
    Resource,
    Player{
        object_id: ObjectId
    },
}

pub enum ChangeType {
    Modified,
    Spawned,
    Despawned,
}

pub struct StateEvent {
    state: Vec<StateChange>
}

pub struct StateChange{
    state_type: StateType,
    change_type: ChangeType,
    updated_state: Vec<Box<dyn Reflect>>
}
