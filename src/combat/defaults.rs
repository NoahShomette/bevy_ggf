use crate::combat::AttackPower;
use crate::object::{ObjectInfo, ObjectType};
use bevy::prelude::{Component, Entity, World};
use bevy::utils::HashMap;

/// A simple default struct implementing [`AttackPower`]. Holds a hashmap that must contain a reference
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

impl AttackPower for ObjectAP {
    fn get_attack_power(&self, world: &World, _: Entity, opponent_entity: Entity) -> u32 {
        let Some(object_info) = world.get::<ObjectInfo>(opponent_entity) else {
            return self.default_attack_power;
        };
        let Some(ap) = self.attack_power.get(object_info.object_type) else {
            return self.default_attack_power;
        };
        return *ap;
    }
}

/// A simple default struct implementing [`AttackPower`]. Returns a single u32 representing that
/// objects attack power
pub struct UniversalAP {
    attack_power: u32,
}

impl AttackPower for UniversalAP {
    fn get_attack_power(&self, _: &World, _: Entity, _: Entity) -> u32 {
        self.attack_power
    }
}
