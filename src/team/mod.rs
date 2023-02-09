use bevy::prelude::Component;

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component)]
pub struct Team {
    id: usize,
}