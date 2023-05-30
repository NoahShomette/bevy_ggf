use crate::game_core::state::{Changed, DespawnedObjects};
use crate::object::{Object, ObjectId};
use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, Query, ResMut, With};

#[derive(Component)]
pub struct Despawn;

pub fn despawn(
    mut commands: Commands,
    query: Query<(Entity, &ObjectId), (With<Despawn>, With<Object>)>,
    mut despawn_objects: ResMut<DespawnedObjects>,
) {
    for (entity, object_id) in query.iter() {
        despawn_objects
            .despawned_objects
            .insert(*object_id, Changed::default());

        commands.entity(entity).despawn_recursive();
    }
}

pub fn track_component_changes<C: Component>(
    mut commands: Commands,
    query: Query<Entity, bevy::prelude::Changed<C>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Changed::default());
    }
}
