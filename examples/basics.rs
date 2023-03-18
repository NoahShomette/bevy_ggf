use bevy::prelude::{App, World};
use bevy::MinimalPlugins;
use bevy_ggf::game_core::runner::GameRunner;
use bevy_ggf::game_core::{GameBuilder, GameType};
use bevy_ggf::BggfDefaultPlugins;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    let mut game = GameBuilder::<TestRunner>::new_game(GameType::Networked, TestRunner::default());
    game.build(&mut app);

    app.add_plugins(BggfDefaultPlugins);
    app.run();
}

#[derive(Default)]
pub struct TestRunner {}
impl GameRunner for TestRunner {
    fn simulate_game(&mut self, world: &mut World) {
        todo!()
    }
}
