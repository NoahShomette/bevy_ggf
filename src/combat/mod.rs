//!

use bevy::app::App;
use bevy::prelude::{Component, Entity, Plugin, World};
use bevy_ecs_tilemap::tiles::TilePos;

pub mod backend;
pub mod battle_resolver;
pub mod defaults;

pub struct BggfCombatPlugin {}

impl Plugin for BggfCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CombatEvent>();
    }
}

impl Default for BggfCombatPlugin {
    fn default() -> Self {
        BggfCombatPlugin {}
    }
}

/// Command events. Send an event to conduct the specified action correlating to the event.
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum CombatEvent {
    CalculateAttacks {
        attacking_entity: Entity,
    },
    Attack {
        attacking_entity: Entity,
        defending_entity: Entity,
        attack_info: ValidAttack,
    },
}

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct AvailableAttacks {}

#[derive(Clone, Eq, Hash, Debug, PartialEq)]
pub struct ValidAttack {
    pub target_entity: Entity,
    pub target_tile_position: TilePos,
    pub requires_move: Option<Vec<TilePos>>,
}

/// The health of an object. Without a Health component an object is not able to be attacked or killed.
/// Objects with a health component can be attacked and will be returned as valid targets by relevant
/// systems
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

pub trait BaseAttackPower {
    /// Returns the *base* attack power of the unit. This should be the base power, unmodified by any
    /// buffs, nerfs, or other modifiers.
    fn get_base_attack_power(&self, world: &World, entity: Entity, opponent_entity: Entity) -> u32;
}

/// Marker component denoting this unit as having attacked.
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct ObjectAttacked;

/// Component that holds an [`BaseAttackPower`] trait object. Attach this to objects that should deal damage
/// in combat
#[derive(Component)]
pub struct AttackPower {
    attack_power: Box<dyn BaseAttackPower + Send + Sync>,
}

/// Marks this object as NOT being attackable, can not be targeted or attacked
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct NonAttackable;

/// Marks this object as being invulnerable. Will not take damage during combat but can be attacked
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct Invulnerable;
