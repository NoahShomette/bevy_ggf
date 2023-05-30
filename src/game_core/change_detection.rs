use crate::game_core::state::{Changed, DespawnedObjects, ResourceChangeTracking};
use crate::game_core::Game;
use crate::object::{Object, ObjectId};
use bevy::prelude::{
    Commands, Component, DespawnRecursiveExt, DetectChanges, Entity, Query, Res, ResMut, Resource,
    With,
};
use std::any::TypeId;

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
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Changed::default());
    }
}

/// Checks if the given resource has changed and if so inserts its ComponentId into the
/// ResourceChangeTracking resource
pub fn track_resource_changes<R: Resource>(
    resource: Res<R>,
    mut resources: ResMut<ResourceChangeTracking>,
    game: Res<Game>,
) {
    if resource.is_changed() {
        let component_id = game
            .game_world
            .components()
            .get_resource_id(TypeId::of::<R>())
            .unwrap_or_else(|| panic!("resource does not exist: {}", std::any::type_name::<R>()));

        if let Some(_) = resources.resources.get(&component_id) {
            resources.resources.insert(component_id, Changed::default());
        } else {
            resources.resources.insert(component_id, Changed::default());
        }
    }
}

// TODO: write tests for this
#[test]
fn test_component_change_tracking() {
    //let game = GameBuilder::<TestRunner>::new_game(TestRunner { schedule }),
}

#[test]
fn test_resource_change_tracking() {}
