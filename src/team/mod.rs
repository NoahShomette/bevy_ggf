use bevy::prelude::{Component, Reflect};
use bevy::reflect::FromReflect;
use serde::{Deserialize, Serialize};

/// Represents a team of players with a custom id
#[derive(Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect, Serialize, Deserialize)]
pub struct Team {
    id: usize,
    players: Vec<PlayerId>
}

/// A unique player
#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect, Serialize, Deserialize)]
pub struct PlayerId {
    id: usize,
}
