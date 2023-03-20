use bevy::input::keyboard::KeyboardInput;
use bevy::input::Input;
use bevy::prelude::{
    App, ClearColor, Color, IntoSystemAppConfigs, IntoSystemConfig, KeyCode, Local, Mut, Res,
    ResMut, Resource, Schedule, World,
};
use bevy::{DefaultPlugins, MinimalPlugins};
use bevy_ascii_terminal::{
    AutoCamera, Border, Terminal, TerminalBundle, TerminalPlugin, TileFormatter,
};
use bevy_ecs_tilemap::prelude::{TilemapSize, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::command::CommandType::Player;
use bevy_ggf::game_core::command::GameCommands;
use bevy_ggf::game_core::runner::GameRunner;
use bevy_ggf::game_core::state::StateThing;
use bevy_ggf::game_core::{Game, GameBuilder, GameRuntime};
use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{
    ObjectStackingClass, StackingClass, TileObjectStacks, TileObjectStacksCount,
};
use bevy_ggf::mapping::{MapCommandsExt, MapId};
use bevy_ggf::movement::defaults::SquareMovementCalculator;
use bevy_ggf::movement::{GameBuilderMovementExt, MoveCommandsExt, MovementType, ObjectMovement, ObjectTerrainMovementRules, TileMovementCosts};
use bevy_ggf::object::{ObjectClass, ObjectGridPosition, ObjectGroup, ObjectId, ObjectType};
use bevy_ggf::BggfDefaultPlugins;
use bevy_ggf::game_core::state::StateThing::Object;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(BggfDefaultPlugins);
    app.add_plugin(TerminalPlugin)
        .insert_resource(ClearColor(Color::BLACK));
    app.add_startup_system(setup);
    app.add_system(simulate);
    app.add_system(handle_input);
    app.run();
}

#[derive(Resource)]
pub struct PlayerPos {
    pub object_grid_position: ObjectGridPosition,
}

#[derive(Default)]
pub struct TestRunner {
    schedule: Schedule,
}
impl GameRunner for TestRunner {
    fn simulate_game(&mut self, world: &mut World) {
        self.schedule.run(world);
        println!("Ran world")
    }
}

fn handle_input(
    mut game: ResMut<Game>,
    mut game_commands: ResMut<GameCommands>,
    mut input: ResMut<Input<KeyCode>>,
    old_player_pos: Option<Res<PlayerPos>>,
) {
    if let Some(player_pos) = old_player_pos {
        if input.just_pressed(KeyCode::A) {
            let _ = game_commands.move_object(
                ObjectId { id: 1 },
                MapId { id: 1 },
                player_pos.object_grid_position.tile_position,
                TilePos {
                    x: player_pos.object_grid_position.tile_position.x.saturating_sub(1),
                    y: player_pos.object_grid_position.tile_position.y,
                },
                false,
            );
        }
        if input.just_pressed(KeyCode::S) {
            let _ = game_commands.move_object(
                ObjectId { id: 1 },
                MapId { id: 1 },
                player_pos.object_grid_position.tile_position,
                TilePos {
                    x: player_pos.object_grid_position.tile_position.x,
                    y: player_pos.object_grid_position.tile_position.y.saturating_sub(1),
                },
                true,
            );
        }
        if input.just_pressed(KeyCode::D) {
            let _ = game_commands.move_object(
                ObjectId { id: 1 },
                MapId { id: 1 },
                player_pos.object_grid_position.tile_position,
                TilePos {
                    x: player_pos.object_grid_position.tile_position.x.saturating_add(1),
                    y: player_pos.object_grid_position.tile_position.y,
                },
                true,
            );
        }
        if input.just_pressed(KeyCode::W) {
            let _ = game_commands.move_object(
                ObjectId { id: 1 },
                MapId { id: 1 },
                player_pos.object_grid_position.tile_position,
                TilePos {
                    x: player_pos.object_grid_position.tile_position.x,
                    y: player_pos.object_grid_position.tile_position.y.saturating_add(1),
                },
                true,
            );
        }
    }
}

fn setup(mut world: &mut World) {
    let stacking_class_ground: StackingClass = StackingClass {
        name: String::from("Ground"),
    };
    let stacking_class_building: StackingClass = StackingClass {
        name: String::from("Building"),
    };

    let MOVEMENT_TYPES: Vec<MovementType> = vec![
        MovementType {
            name: String::from("Infantry"),
        },
        MovementType {
            name: String::from("Tread"),
        },
    ];

    let TERRAIN_CLASSES: Vec<TerrainClass> = vec![
        TerrainClass {
            name: String::from("Ground"),
        },
        TerrainClass {
            name: String::from("Water"),
        },
    ];

    let TERRAIN_TYPES: Vec<TerrainType> = vec![
        TerrainType {
            name: String::from("Grassland"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TerrainType {
            name: String::from("Forest"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TerrainType {
            name: String::from("Mountain"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TerrainType {
            name: String::from("Hill"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TerrainType {
            name: String::from("Sand"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TerrainType {
            name: String::from("CoastWater"),
            terrain_class: TERRAIN_CLASSES[1].clone(),
        },
        TerrainType {
            name: String::from("Ocean"),
            terrain_class: TERRAIN_CLASSES[1].clone(),
        },
    ];

    let object_class_ground: ObjectClass = ObjectClass {
        name: String::from("Ground"),
    };
    let object_group_infantry: ObjectGroup = ObjectGroup {
        name: String::from("Infantry"),
        object_class: object_class_ground.clone(),
    };
    let object_type_rifleman: ObjectType = ObjectType {
        name: String::from("Rifleman"),
        object_group: object_group_infantry.clone(),
    };

    let object_class_building: ObjectClass = ObjectClass {
        name: String::from("Building"),
    };
    let object_group_improvements: ObjectGroup = ObjectGroup {
        name: String::from("object_class_building"),
        object_class: object_class_ground,
    };
    let object_type_bridge: ObjectType = ObjectType {
        name: String::from("Bridge"),
        object_group: object_group_infantry,
    };

    let tilemap_size = TilemapSize { x: 50, y: 50 };
    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let tilemap_type = TilemapType::Square;

    let terrain_extension_types: Vec<TerrainType> = vec![
        TERRAIN_TYPES[0].clone(),
        TERRAIN_TYPES[1].clone(),
        TERRAIN_TYPES[2].clone(),
        //TERRAIN_TYPES[3].clone(),
        TERRAIN_TYPES[4].clone(),
        //TERRAIN_TYPES[5].clone(),
        //TERRAIN_TYPES[6].clone(),
    ];

    let tile_stack_rules = TileObjectStacks::new(vec![
        (
            stacking_class_ground.clone(),
            TileObjectStacksCount {
                current_count: 0,
                max_count: 1,
            },
        ),
        (
            stacking_class_building,
            TileObjectStacksCount {
                current_count: 0,
                max_count: 1,
            },
        ),
    ]);

    let mut game_commands = GameCommands::new();

    let spawn_map_command = game_commands.generate_random_map(
        tilemap_size,
        tilemap_type,
        tilemap_tile_size,
        terrain_extension_types,
        tile_stack_rules,
    );

    let player_spawn_pos = TilePos { x: 10, y: 10 };

    let spawn_object = game_commands.spawn_object(
        (
            ObjectGridPosition {
                tile_position: player_spawn_pos,
            },
            ObjectMovement {
                move_points: 5,
                movement_type: MOVEMENT_TYPES[0].clone(),
                object_terrain_movement_rules: ObjectTerrainMovementRules::new(vec![TERRAIN_CLASSES[0].clone()], vec![]),
            },
            ObjectStackingClass {
                stack_class: stacking_class_ground,
            },
            bevy_ggf::object::Object,
        ),
        player_spawn_pos,
        MapId { id: 1 },
    );
    world.insert_resource(PlayerPos {
        object_grid_position: ObjectGridPosition {
            tile_position: player_spawn_pos,
        },
    });

    let mut game = GameBuilder::<TestRunner>::new_game_with_commands(
        vec![Box::new(spawn_map_command), Box::new(spawn_object)],
        TestRunner::default(),
    );

    game.setup_movement(vec![(
        TerrainType {
            name: String::from("Grassland"),
            terrain_class: TERRAIN_CLASSES[0].clone(),
        },
        TileMovementCosts {
            movement_type_cost: Default::default(),
        },
    )]);

    game.with_movement_calculator(
        SquareMovementCalculator {
            diagonal_movement: Default::default(),
        },
        vec![],
        tilemap_type,
    );

    game.build(&mut world);

    let mut term =
        Terminal::new([tilemap_size.x, tilemap_size.y]).with_border(Border::single_line());

    world.spawn((TerminalBundle::from(term), AutoCamera));
}

fn simulate(mut world: &mut World) {
    world.resource_scope(|world, mut game: Mut<Game>| {
        world.resource_scope(|world, mut game_runtime: Mut<GameRuntime<TestRunner>>| {
            world.resource_scope(|world, mut game_commands: Mut<GameCommands>| {
                game_commands.execute_buffer(&mut game.game_world);
            });
            game_runtime.game_runner.simulate_game(&mut game.game_world);
        });
        let game_state = game.get_new_state();
        let mut player_pos = world.remove_resource::<PlayerPos>().unwrap();
        let mut term = world.query::<&mut Terminal>().single_mut(world);
        for state in game_state {
            match state {
                StateThing::Object {
                    change_type,
                    object_id,
                    object_grid_position,
                    components,
                } => {
                    term.put_char(
                        [
                            object_grid_position.tile_position.x,
                            object_grid_position.tile_position.y,
                        ],
                        'P'.fg(Color::WHITE).bg(Color::BLUE),
                    );
                    player_pos.object_grid_position.tile_position =
                        object_grid_position.tile_position;
                }
                StateThing::Tile {
                    change_type,
                    tile_pos,
                    components,
                } => {
                    term.put_char([tile_pos.x, tile_pos.y], 'H'.fg(Color::GREEN));
                }
                StateThing::Resource { .. } => {}
                StateThing::Player {
                    change_type,
                    player_id,
                    components,
                } => {}
            }
        }

        world.insert_resource(player_pos);
    });
}
