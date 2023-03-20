use bevy::prelude::{Schedule, World};

/// The [`GameRunner`] represents the actual *game* logic that you want run whenever the game state
/// should be updated, independently of GameCommands
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

pub struct RealTimeGameRunner {
    ticks: usize,
    tick_schedule: Schedule,
}

impl GameRunner for RealTimeGameRunner {
    fn simulate_game(&mut self, world: &mut World) {
        self.ticks = self.ticks.saturating_add(1);
        self.tick_schedule.run(world);
    }
}
