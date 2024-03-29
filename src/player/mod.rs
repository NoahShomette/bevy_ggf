use crate::game_core::state::Changed;
use bevy::prelude::{Component, FromReflect, Reflect, Resource};
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

/// A list of all players in the game. This is copied into the game world to allow accessing it
#[derive(
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Resource,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
pub struct PlayerList {
    pub players: Vec<Player>,
}

impl PlayerList {
    pub fn new_changed_component(&self) -> Changed {
        let mut players_seen = HashMap::new();
        for player in self.players.iter() {
            players_seen.insert(player.id, false);
        }
        Changed { players_seen }
    }
}

/// Represents a team of players with a custom id
#[derive(
    Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect, Serialize, Deserialize,
)]
pub struct Team {
    id: usize,
    player_ids: Vec<usize>,
}

/// A unique player with unique information used to drive game systems
#[derive(
    Clone, Copy, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect, Serialize, Deserialize,
)]
pub struct Player {
    id: usize,
    pub needs_state: bool,
}

impl Player {
    pub fn new(id: usize, needs_state: bool) -> Player {
        Player { id, needs_state }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

/// A component that marks something as related to the given player - used to mark objects as player 
/// owned chiefly
#[derive(
    Clone, Copy, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect, Serialize, Deserialize,
)]
pub struct PlayerMarker {
    id: usize,
}

impl PlayerMarker {
    pub fn new(id: usize) -> PlayerMarker {
        PlayerMarker { id }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}
