use bevy::prelude::{Schedule, World};

/// The [`GameRunner`] represents the actual *game* logic that you want run whenever the game state
/// should be updated
pub trait GameRunner: Send + Sync {
    fn simulate_game(&mut self, world: &mut World);
}

pub struct TurnBasedGameRunner {
    turn_schedule: Schedule,
}

impl GameRunner for TurnBasedGameRunner {
    fn simulate_game(&mut self, world: &mut World) {
        self.turn_schedule.run(world);
    }
}
