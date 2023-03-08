use bevy::prelude::{Resource, World};
use crate::game::GameId;

/// A battle resolver, this takes
#[derive(Resource)]
pub struct Combat<T> {
    pub attack_power_calculator: Box<dyn AttackPowerCalculator + Send + Sync>,
    pub battle_calculator: Box<dyn BattleCalculator<Result=T> + Send + Sync>,
}

pub enum BattleError {
    Message(String),
    InvalidComponents(String),
}

pub trait AttackPowerCalculator {
    /// Calculate and return the final object attack power to be applied to the opponent object
    /// - must return a number, even if its 0 in case of failure
    fn calculate_object_attack_power(&self, object_to_calculate: GameId, opponent_object: GameId, world: &mut World) -> u32;
}

pub trait BattleCalculator {
    type Result;

    fn resolve_combat(
        &mut self,
        world: &mut World,
        attacking_entity: GameId,
        defending_entity: GameId,
    ) -> Result<Self::Result, BattleError>;
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BattleResult<T> {
    attacking_object: GameId,
    defending_object: GameId,
    result: T,
}
