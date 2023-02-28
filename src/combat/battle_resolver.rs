use bevy::prelude::{Entity, Resource, World};

/// A battle resolver. This is required by the battle resolver systems. Make sure you add the [`BattleResult`]
/// as an event as well.
#[derive(Resource)]
pub struct BattleResolver<T> {
    pub battle_calculator: Box<dyn BattleCalculator<Result=T> + Send + Sync>,
}

pub enum BattleError {
    InvalidComponents(String),
}

pub trait BattleCalculator {
    type Result;

    fn resolve_combat(
        &mut self,
        world: &World,
        attacking_entity: Entity,
        defending_entity: Entity,
    ) -> Result<Self::Result, BattleError>;
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BattleResult<T> {
    attacking_entity: Entity,
    defending_entity: Entity,
    result: T,
}

pub struct CombatResolverTest {}

pub fn handle_battle_events() {}
