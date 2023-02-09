use crate::object::ObjectType;
use bevy::prelude::Component;
use bevy::utils::HashMap;

pub mod backend;
pub mod defaults;

/// The health of an object. Without a Health component an object is not able to be attacked or killed.
/// Objects with a health component can be attacked
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct Health {
    pub current_health: u32,
    pub max_health: u32,
    pub on_death: OnDeath,
}

impl Health {
    /// Reduces current_health by the specified amount, down to a maximum of 0
    pub fn take_damage(&mut self, damage_amount: u32) {
        self.current_health = self.current_health.saturating_sub(damage_amount);
    }

    /// Increases current_health by the specified amount, up to the maximum specified by max_health
    pub fn heal(&mut self, heal_amount: u32) {
        self.current_health = self.current_health.saturating_add(heal_amount);

        if self.current_health > self.max_health {
            self.current_health = self.max_health;
        }
    }
}

/// Specifies what will happen to the object when it is killed in battle
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub enum OnDeath {
    /// Destroys the object when killed
    Destroy,
    /// Captures the object when killed, converting it to the killing team and restoring it to the
    /// specified health
    Capture { restore_at_health: u32 },
}

///
#[derive(Clone, Eq, Debug, PartialEq, Component)]
pub struct AttackPower {
    attack_type: AttackType,
}

#[derive(Clone, Eq, Debug, PartialEq)]
pub enum AttackType {
    Object { damage: HashMap<ObjectType, u32> },
    Strength { strength: u32 },
}

/// Marks this object as NOT being attackable, can not be targeted or attacked
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct NonAttackable;

/// Marks this object as being invulnerable. Will not take damage during combat but can be attacked
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct Invulnerable;
