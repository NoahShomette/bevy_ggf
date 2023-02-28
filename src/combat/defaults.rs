use crate::combat::battle_resolver::{BattleCalculator, BattleError, BattleResolver, BattleResult};
use crate::combat::{BaseAttackPower, Health, AttackPower};
use crate::object::{ObjectInfo, ObjectType};
use bevy::prelude::{Component, Entity, World};
use bevy::utils::HashMap;

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
/// all default [`BattleResolver`]s.
#[derive(Clone, Eq, Debug, PartialEq)]
pub struct BasicBattleResult {
    pub defending_damage: u32,
    pub attacking_damage: u32,
}

pub struct BasicBattleCalculator {}

impl BattleCalculator for BasicBattleCalculator {
    type Result = BasicBattleResult;

    fn resolve_combat(
        &mut self,
        world: &World,
        attacking_entity: Entity,
        defending_entity: Entity,
    ) -> Result<Self::Result, BattleError> {
        let Some(attacking_ap) = world.get::<AttackPower>(attacking_entity) else {
            return Err(BattleError::InvalidComponents(String::from("Attacking Object did not have ObjectAttackPower Component")));
        };
        let Some(defending_ap) = world.get::<AttackPower>(defending_entity) else {
            return Err(BattleError::InvalidComponents(String::from("Defending Object did not have ObjectAttackPower Component")));
        };
        let Some(attacking_health) = world.get::<Health>(attacking_entity) else {
            return Err(BattleError::InvalidComponents(String::from("Attacking Object did not have Health Component")));
        };
        let Some(defending_health) = world.get::<Health>(defending_entity) else {
            return Err(BattleError::InvalidComponents(String::from("Defending Object did not have Health Component")));
        };

        let attacking_base_ap = attacking_ap.attack_power.get_base_attack_power(&world, attacking_entity, defending_entity);
        let defending_base_ap = defending_ap.attack_power.get_base_attack_power(&world, attacking_entity, defending_entity);

        return Err(BattleError::InvalidComponents(String::from("Defending Object did not have Health Component")));
    }
}
