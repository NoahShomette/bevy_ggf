use crate::combat::battle_resolver::{
    AttackPowerCalculator, BattleCalculator, BattleError, BattleResult, Combat,
};
use crate::combat::{AttackPower, BaseAttackPower, Health, OnDeath};
use crate::object::{ObjectId, ObjectInfo, ObjectType};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Component, Entity, Mut, Query, ResMut, World};
use bevy::utils::HashMap;
use crate::combat::commands::GameCommandsExt;
use crate::game_core::command::GameCommands;

/// A simple default struct implementing [`BaseAttackPower`]. Holds a hashmap that must contain a reference
/// to every ObjectType in the game. Returns the u32 saved in the hashmap corresponding to the given
/// opponent_entity
#[derive(Clone, Eq, Debug, PartialEq, Component)]
pub struct ObjectAP {
    attack_power: HashMap<ObjectType, u32>,
    /// The attack_power to return if the opponent doesn't have an [`ObjectInfo`] component or there
    /// isn't a reference to the opponents ObjectType in the attack_power hashmap
    default_attack_power: u32,
}

impl ObjectAP {
    /// Helper function to create a new ObjectAP component with the given vec of ObjectType and u32
    /// tuples
    pub fn new(attack_powers: Vec<(ObjectType, u32)>) -> ObjectAP {
        let mut hashmap: HashMap<ObjectType, u32> = HashMap::new();
        for (object_type, attack_power) in attack_powers {
            hashmap.insert(object_type, attack_power);
        }
        ObjectAP {
            attack_power: hashmap,
            default_attack_power: 0,
        }
    }
}

impl BaseAttackPower for ObjectAP {
    fn get_base_attack_power(&self, world: &World, _: Entity, opponent_entity: Entity) -> u32 {
        let Some(object_info) = world.get::<ObjectInfo>(opponent_entity) else {
            return self.default_attack_power;
        };
        let Some(ap) = self.attack_power.get(object_info.object_type) else {
            return self.default_attack_power;
        };
        return *ap;
    }
}

/// A simple default struct implementing [`BaseAttackPower`]. Returns a single u32 representing that
/// objects attack power
pub struct UniversalAP {
    attack_power: u32,
}

impl BaseAttackPower for UniversalAP {
    fn get_base_attack_power(&self, _: &World, _: Entity, _: Entity) -> u32 {
        self.attack_power
    }
}

/// Basic battle result usable in [`BattleResult`] if you only need/want to know damage. Works with
/// all default [`Combat`]s.
#[derive(Clone, Eq, Debug, PartialEq)]
pub struct BasicBattleResult {
    pub defending_damage_dealt: u32,
    pub attacking_damage_dealt: u32,
}

pub struct BasicObjectAPCalculator;

impl AttackPowerCalculator for BasicObjectAPCalculator {
    fn calculate_object_attack_power(
        &self,
        object_to_calculate: ObjectId,
        opponent_object: ObjectId,
        world: &mut World,
    ) -> u32 {
        let mut system_state: SystemState<Query<(Entity, &ObjectId, &AttackPower)>> =
            SystemState::new(world);
        let object_query = system_state.get(world);

        let Some((main_entity, _, main_ap)) = object_query.iter().find(|(_, id, _)| {
            id == &&object_to_calculate
        })else {
            return 0;
        };

        let Some((opponent_entity, _, _)) = object_query.iter().find(|(_, id, _)| {
            id == &&opponent_object
        })else {
            return 0;
        };

        return main_ap
            .attack_power
            .get_base_attack_power(&world, main_entity, opponent_entity);
    }
}

pub struct BasicBattleCalculator {}

impl BattleCalculator for BasicBattleCalculator {
    type Result = BasicBattleResult;

    fn resolve_combat(
        &mut self,
        world: &mut World,
        attacking_id: ObjectId,
        defending_id: ObjectId,
    ) -> Result<Self::Result, BattleError> {
        let mut attacking_ap = 0;
        let mut defending_ap = 0;

        world.resource_scope(|world, combat: Mut<Combat<Self::Result>>| {
            attacking_ap = combat
                .attack_power_calculator
                .calculate_object_attack_power(attacking_id, defending_id, world);
            defending_ap = combat
                .attack_power_calculator
                .calculate_object_attack_power(defending_id, attacking_id, world);
        });

        let mut system_state: SystemState<(Query<(Entity, &ObjectId, &mut Health)>, ResMut<GameCommands>)> =
            SystemState::new(world);
        let (mut object_query, mut game_commands) = system_state.get_mut(world);

        let Some((attacking_entity, _, mut attacking_health)) = object_query.iter_mut().find(|(_, id, _)| {
            id == &&attacking_id
        })else {
            return Err(BattleError::Message(String::from("Attacking Object not found in query")));
        };

        attacking_health.damage(attacking_ap);

        if attacking_health.current_health <= 0 {
            match attacking_health.on_death {
                OnDeath::Destroy => {
                    //game_commands.despawn_object(/* MapId */, /* GameId */);
                }
                OnDeath::Capture { .. } => {}
            }
        }

        let Some((defending_entity, _, mut defending_health)) = object_query.iter_mut().find(|(_, id, _)| {
            id == &&defending_id
        })else {
            return Err(BattleError::Message(String::from("Defending Object not found in query")));
        };

        defending_health.damage(attacking_ap);

        return Ok(Self::Result {
            attacking_damage_dealt: attacking_ap,
            defending_damage_dealt: defending_ap,
        });
    }
}
