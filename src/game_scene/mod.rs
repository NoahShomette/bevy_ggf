// two main scenes maybe initially - MainMenuScene, GameScene
// Basically all entities spawned should be hooked to either of these, whichever is current, and 
// then when changing scenes everything is gone

use bevy::app::{App, Plugin};
use bevy::ecs::query::WorldQuery;
use bevy::prelude::{Bundle, Commands, Component, Entity, Handle, Query, Resource, With};
use bevy::scene::DynamicScene;

pub enum GameState{
    MainMenu,
    Playing
}

#[derive(Resource)]
pub struct GameStruct{
    game_state: GameState,
    main_menu_scene: Handle<DynamicScene>,
    playing_scene: DynamicScene,
}

impl Default for GameStruct{
    fn default() -> Self {
        GameStruct{
            game_state: GameState::MainMenu,
            main_menu_scene: Default::default(),
            playing_scene: Default::default()
        }
    }
}

impl GameStruct{
    fn startup(mut commands: Commands){
        let game_state = GameState::MainMenu;
        //commands.init_resource::<GameStruct>();
    }
}
