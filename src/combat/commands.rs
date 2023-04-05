use crate::game_core::command::{GameCommands};
use crate::mapping::MapId;
use crate::object::ObjectId;

pub trait GameCommandsExt{
    fn attack_object(attacking_object: ObjectId, defending_object: ObjectId, on_map: MapId) -> AttackObject;

}

impl GameCommandsExt for GameCommands{
    fn attack_object(attacking_object: ObjectId, defending_object: ObjectId, on_map: MapId) -> AttackObject {
        todo!()
    }
}


pub struct AttackObject{
    
}