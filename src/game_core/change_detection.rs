use crate::game_core::state::{Changed, DespawnedObjects, ResourceChangeTracking};
use crate::object::{Object, ObjectId};
use bevy::prelude::{
    Commands, Component, DespawnRecursiveExt, DetectChanges, Entity, FromReflect, Mut, Query,
    Reflect, RemovedComponents, ResMut, Resource, With, World,
};

#[derive(Component)]
pub struct DespawnObject;

/// System automatically inserted into the GameRunner::game_post_schedule to automatically handle despawning
/// objects and updating the DespawnedObjects resource
pub fn despawn_objects(
    mut commands: Commands,
    query: Query<(Entity, &ObjectId), (With<DespawnObject>, With<Object>)>,
    mut despawn_objects: ResMut<DespawnedObjects>,
) {
    for (entity, object_id) in query.iter() {
        despawn_objects
            .despawned_objects
            .insert(*object_id, Changed::default());

        commands.entity(entity).despawn_recursive();
    }
}

/// For every entity containing the given component that has changed, inserts a Changed::default() component
pub fn track_component_changes<C: Component>(
    mut commands: Commands,
    query: Query<Entity, bevy::prelude::Changed<C>>,
    mut removed_components: RemovedComponents<C>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Changed::default());
    }

    for entity in removed_components.iter() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(Changed::default());
        }
    }
}

/// Checks if the given resource has changed and if so inserts its ComponentId into the
/// ResourceChangeTracking resource
pub fn track_resource_changes<R: Resource>(world: &mut World) {
    world.resource_scope(|world, resource: Mut<R>| {
        if resource.is_changed() {
            let component_id = world.components().resource_id::<R>().unwrap_or_else(|| {
                panic!("resource does not exist: {}", std::any::type_name::<R>())
            });

            world.resource_scope(|world, mut resources: Mut<ResourceChangeTracking>| {
                if let Some(_) = resources.resources.get(&component_id) {
                    resources.resources.insert(component_id, Changed::default());
                } else {
                    resources.resources.insert(component_id, Changed::default());
                }
            });
        }
    });
}
#[derive(Default, Component, Reflect, FromReflect)]
struct TestComponent(u32);

// TODO: write tests for this
#[test]
fn test_component_change_tracking() {
    let mut world = World::new();
    let mut game = GameBuilder::<TurnBasedGameRunner>::new_game(TurnBasedGameRunner {
        turn_schedule: Default::default(),
    });
    game.register_component::<TestComponent>();
    game.build(&mut world);

    let mut game = world.remove_resource::<Game>().unwrap();
    let mut game_runtime = world
        .remove_resource::<GameRuntime<TurnBasedGameRunner>>()
        .unwrap();

    let entity = game
        .game_world
        .spawn_empty()
        .insert(TestComponent(0))
        .insert(ObjectId { id: 0 })
        .insert(ObjectGridPosition {
            tile_position: Default::default(),
        })
        .id();

    game_runtime.simulate(&mut game.game_world);

    let mut first_state =
        game.game_state_handler
            .get_state_diff(&mut game.game_world, 0, &game.type_registry);

    let mut entity_mut = game.game_world.entity_mut(entity);
    let mut component = entity_mut.get_mut::<TestComponent>().unwrap();
    component.0 += 1;

    game_runtime.simulate(&mut game.game_world);

    let mut second_state =
        game.game_state_handler
            .get_state_diff(&mut game.game_world, 0, &game.type_registry);

    let components = first_state.objects.pop().unwrap().components;

    let test_component_1 = components
        .iter()
        .find(|item| {
            if let Some(_) = <TestComponent as FromReflect>::from_reflect(&*item.clone_value()) {
                return true;
            }
            false
        })
        .unwrap();
    let Some(test_component_1) =
        <TestComponent as FromReflect>::from_reflect(&*test_component_1.clone_value())
    else {
        panic!("Couldn't find component")
    };

    let components = second_state.objects.pop().unwrap().components;

    let test_component_2 = components
        .iter()
        .find(|item| {
            if let Some(_) = <TestComponent as FromReflect>::from_reflect(&*item.clone_value()) {
                return true;
            }
            false
        })
        .unwrap();
    let Some(test_component_2) =
        <TestComponent as FromReflect>::from_reflect(&*test_component_2.clone_value())
    else {
        panic!("Couldn't find component")
    };

    assert_eq!(test_component_1.0, 0);
    assert_eq!(test_component_2.0, 1);
}

#[derive(Default, Resource, Reflect, FromReflect)]
struct TestResource(u32);

#[test]
fn test_resource_change_tracking() {
    let mut world = World::new();
    let mut game = GameBuilder::<TurnBasedGameRunner>::new_game(TurnBasedGameRunner {
        turn_schedule: Default::default(),
    });
    game.register_resource::<TestResource>();
    game.build(&mut world);

    let mut game = world.remove_resource::<Game>().unwrap();
    let mut game_runtime = world
        .remove_resource::<GameRuntime<TurnBasedGameRunner>>()
        .unwrap();

    game.game_world.insert_resource(TestResource(0));

    game_runtime.simulate(&mut game.game_world);

    let mut first_state =
        game.game_state_handler
            .get_state_diff(&mut game.game_world, 0, &game.type_registry);

    game.game_world
        .resource_scope(|_, mut resource: Mut<TestResource>| {
            resource.0 += 1;
        });

    game_runtime.simulate(&mut game.game_world);

    let mut second_state =
        game.game_state_handler
            .get_state_diff(&mut game.game_world, 0, &game.type_registry);

    let resource = first_state.resources.pop().unwrap();

    let Some(test_component_1) =
        <TestResource as FromReflect>::from_reflect(&*resource.resource.clone_value())
    else {
        panic!("Couldn't find component")
    };

    let resource = second_state.resources.pop().unwrap();

    let Some(test_component_2) =
        <TestResource as FromReflect>::from_reflect(&*resource.resource.clone_value())
    else {
        panic!("Couldn't find component")
    };

    assert_eq!(test_component_1.0, 0);
    assert_eq!(test_component_2.0, 1);
}
