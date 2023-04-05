use bevy::prelude::{
    App, ClearColor, Color, IntoSystemAppConfigs, IntoSystemConfig, Mut, Schedule, World,
};
use bevy::{DefaultPlugins, MinimalPlugins};
use bevy_ecs_tilemap::prelude::{TilemapSize, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::command::GameCommands;
use bevy_ggf::game_core::runner::GameRunner;
use bevy_ggf::game_core::state::StateThing;
use bevy_ggf::game_core::{Game, GameBuilder, GameRuntime};
use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{StackingClass, TileObjectStacks, TileObjectStacksCount};
use bevy_ggf::mapping::{MapCommandsExt, MapId};
use bevy_ggf::movement::{GameBuilderMovementExt, MovementType, TileMovementCosts};
use bevy_ggf::object::{ObjectClass, ObjectGroup, ObjectType};
use bevy_ggf::BggfDefaultPlugins;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(BggfDefaultPlugins);

    app.add_startup_system(setup);
    app.add_system(simulate);
    app.run();
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

    let tilemap_size = TilemapSize { x: 100, y: 100 };
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
            stacking_class_ground,
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

    let spawn_map_command = game_commands.spawn_object(
        (TilePos { x: 50, y: 50 }),
        TilePos { x: 50, y: 50 },
        MapId { id: 1 },
    );

    let mut game = GameBuilder::<TestRunner>::new_game_with_commands(
        vec![Box::new(spawn_map_command)],
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

    game.build(&mut world);
}

fn simulate(mut world: &mut World) {
    world.resource_scope(|world, mut game: Mut<Game>| {
        world.resource_scope(|world, mut game_runtime: Mut<GameRuntime<TestRunner>>| {
            game_runtime.game_runner.simulate_game(&mut game.game_world);
        });
        let game_state = game.get_entire_state();
        for state in game_state {
        }
    });
}
