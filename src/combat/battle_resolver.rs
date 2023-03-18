use bevy::prelude::{Resource, World};
use crate::object::ObjectId;

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
    fn calculate_object_attack_power(&self, object_to_calculate: ObjectId, opponent_object: ObjectId, world: &mut World) -> u32;
}

pub trait BattleCalculator {
    type Result;

    fn resolve_combat(
        &mut self,
        world: &mut World,
        attacking_entity: ObjectId,
        defending_entity: ObjectId,
    ) -> Result<Self::Result, BattleError>;
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BattleResult<T> {
    attacking_object: ObjectId,
    defending_object: ObjectId,
    result: T,
}
