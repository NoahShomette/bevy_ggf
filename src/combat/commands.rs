use crate::game::command::GameCommands;
use crate::game::GameId;
use crate::mapping::MapId;

pub trait GameCommandsExt{
    fn attack_object(attacking_object: GameId, defending_object: GameId, on_map: MapId) -> AttackObject;

}

impl GameCommandsExt for GameCommands{
    fn attack_object(attacking_object: GameId, defending_object: GameId, on_map: MapId) -> AttackObject {
        todo!()
    }
}


pub struct AttackObject{
    
}