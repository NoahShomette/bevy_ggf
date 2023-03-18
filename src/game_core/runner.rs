use bevy::prelude::{Schedule, World};

pub trait GameRunner: Send + Sync {
    fn simulate_game(&mut self, world: &mut World);
}

pub struct TurnBasedGameRunner {
    turn_schedule: Schedule,
}

impl GameRunner for TurnBasedGameRunner {
    fn simulate_game(&mut self, world: &mut World) {
        self.turn_schedule.run_once(world);
    }
}
