use bevy::prelude::{App, Mut, Schedule, World};
use bevy::MinimalPlugins;
use bevy_ecs_tilemap::prelude::{TilemapSize, TilemapTileSize, TilemapType};
use bevy_ggf::game_core::command::GameCommands;
use bevy_ggf::game_core::runner::GameRunner;
use bevy_ggf::game_core::{GameBuilder, GameData, GameInfo, GameRuntime, GameType};
use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{StackingClass, TileObjectStackingRules, TileObjectStacksCount};
use bevy_ggf::mapping::{MapCommandsExt, SpawnRandomMap};
use bevy_ggf::movement::{GameBuilderMovementExt, MovementType, TileMovementCosts};
use bevy_ggf::object::{ObjectClass, ObjectGroup, ObjectType};
use bevy_ggf::BggfDefaultPlugins;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(BggfDefaultPlugins);
    app.add_startup_system(setup);
    app.add_systems(simulate);
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

pub const STACKING_CLASS_GROUND: StackingClass = StackingClass { name: "Ground" };
pub const STACKING_CLASS_BUILDING: StackingClass = StackingClass { name: "Building" };

pub const MOVEMENT_TYPES: &'static [MovementType] = &[
    MovementType { name: "Infantry" },
    MovementType { name: "Tread" },
];

pub const TERRAIN_CLASSES: &'static [TerrainClass] = &[
    TerrainClass { name: "Ground" },
    TerrainClass { name: "Water" },
];

pub const TERRAIN_TYPES: &'static [TerrainType] = &[
    TerrainType {
        name: "Grassland",
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Forest",
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Mountain",
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Hill",
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "Sand",
        terrain_class: &TERRAIN_CLASSES[0],
    },
    TerrainType {
        name: "CoastWater",
        terrain_class: &TERRAIN_CLASSES[1],
    },
    TerrainType {
        name: "Ocean",
        terrain_class: &TERRAIN_CLASSES[1],
    },
];

fn setup(mut world: &mut World) {

    let OBJECT_CLASS_GROUND: ObjectClass = ObjectClass {
        name: String::from("Ground"),
    };
    let OBJECT_GROUP_INFANTRY: ObjectGroup = ObjectGroup {
        name: String::from("Infantry"),
        object_class: OBJECT_CLASS_GROUND.clone(),
    };
    let OBJECT_TYPE_RIFLEMAN: ObjectType = ObjectType {
        name: String::from("Rifleman"),
        object_group: OBJECT_GROUP_INFANTRY.clone(),
    };

    let OBJECT_CLASS_BUILDING: ObjectClass = ObjectClass {
        name: String::from("Building"),
    };
    let OBJECT_GROUP_IMPROVEMENTS: ObjectGroup = ObjectGroup {
        name: String::from("OBJECT_CLASS_BUILDING"),
        object_class: OBJECT_CLASS_GROUND,
    };
    let OBJECT_TYPE_BRIDGE: ObjectType = ObjectType {
        name: String::from("Bridge"),
        object_group: OBJECT_GROUP_INFANTRY,
    };
    
    let tilemap_size = TilemapSize { x: 100, y: 100 };
    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let tilemap_type = TilemapType::Square;

    let terrain_extension_types: Vec<TerrainType> = vec![
        TERRAIN_TYPES[0],
        TERRAIN_TYPES[1],
        TERRAIN_TYPES[2],
        //TERRAIN_TYPES[3],
        TERRAIN_TYPES[4],
        //TERRAIN_TYPES[5],
        //TERRAIN_TYPES[6],
    ];

    let tile_stack_rules = TileObjectStackingRules::new(vec![
        (
            &STACKING_CLASS_GROUND,
            TileObjectStacksCount {
                current_count: 0,
                max_count: 1,
            },
        ),
        (
            &STACKING_CLASS_BUILDING,
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

    let mut game = GameBuilder::<TestRunner>::new_game_with_commands(
        GameType::Networked,
        vec![Box::new(spawn_map_command)],
        TestRunner::default(),
    );
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

fn simulate(mut world: &mut World) {
    world.resource_scope(|world, mut game_schedule: Mut<GameData>| {
        world.resource_scope(|world, mut game_info: Mut<GameRuntime<TestRunner>>| {
            game_info
                .game_runner
                .simulate_game(&mut game_schedule.game_world);
        });
    });
}
