use bevy::prelude::{App, World};
use bevy::MinimalPlugins;
use bevy_ggf::game_core::runner::GameRunner;
use bevy_ggf::game_core::{GameBuilder, GameType};
use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::movement::{GameBuilderMovementExt, TileMovementCosts};
use bevy_ggf::BggfDefaultPlugins;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(BggfDefaultPlugins);
    app.add_startup_system(setup);
    app.run();
}

#[derive(Default)]
pub struct TestRunner {}
impl GameRunner for TestRunner {
    fn simulate_game(&mut self, world: &mut World) {
        todo!()
    }
}
/*
TerrainType {
name: "Forest",
texture_index: 1,
terrain_class: &TERRAIN_CLASSES[0],
},

 */

pub const TERRAIN_CLASSES: &'static [TerrainClass] = &[
    TerrainClass { name: "Ground" },
    TerrainClass { name: "Water" },
];

fn setup(mut world: &mut World) {
    let mut game = GameBuilder::<TestRunner>::new_game(GameType::Networked, TestRunner::default());
    game.setup_movement(vec![(
        TerrainType {
            name: "Grassland",
            terrain_class: &TERRAIN_CLASSES[0],
        },
        TileMovementCosts {
            movement_type_cost: Default::default(),
        },
    )]);
    game.build(&mut world);
}
