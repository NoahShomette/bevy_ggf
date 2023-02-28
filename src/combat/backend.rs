use crate::combat::{ObjectAttacked};
use bevy::prelude::{Commands, EventReader};
use crate::combat::battle_resolver::BattleResult;

/// Adds the [`ObjectAttacked`] component to any entity that is sent through the [`CombatResultEvent::AttackResult`]
/// event.
pub fn add_object_attacked_component_on_attacks(
    //mut move_events: EventReader<BattleResult>,
    mut commands: Commands,
) {
    //for event in move_events.iter() {}
}
