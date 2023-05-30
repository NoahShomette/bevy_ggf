use bevy::prelude::{Resource, Schedule, SystemSet, World};
use crate::game_core::Game;

/// Runtime that is implemented by the user to drive their game
#[derive(Resource)]
pub struct GameRuntime<T>
    where
        T: GameRunner,
{
    pub game_runner: T,
    pub framework_pre_schedule: Schedule,
    pub framework_post_schedule: Schedule,
}

impl<T> GameRuntime<T>
    where
        T: GameRunner,
{
    pub fn simulate(&mut self, mut game_data: &mut Game) {
        self.framework_pre_schedule.run(&mut game_data.game_world);
        self.game_runner.simulate_game(&mut game_data.game_world);
        self.framework_post_schedule.run(&mut game_data.game_world);
    }
}

// SystemSet for the GameRunner FrameworkPostSchedule
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub enum PostBaseSets {
    CommandFlush,
    Pre,
    Main,
    Post
}

// SystemSet for the GameRunner FrameworkPreSchedule
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub enum PreBaseSets {
    CommandFlush,
    Pre,
    Main,
    Post
}

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
