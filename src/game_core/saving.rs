use bevy::{
    ecs::{
        component::Component,
        system::Resource,
        world::{EntityMut, World},
    },
    utils::HashMap,
};
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    mapping::{
        terrain::TileTerrainInfo,
        tiles::{ObjectStackingClass, Tile, TileObjects, TilePosition},
    },
    movement::TileMovementCosts,
    object::{Object, ObjectGridPosition, ObjectId},
};

/// An id hand assigned to components using the [`SaveId`] trait that identifies each component
///
/// Is simply a u8 under the type
pub type ComponentId = u8;

/// An id hand assigned to resources using the [`SaveId`] trait that identifies each component
///
/// Is simply a u8 under the type
pub type ResourceId = u8;

#[derive(Debug)]
pub struct ComponentBinaryState {
    pub id: ComponentId,
    pub component: Vec<u8>,
}

/// A registry that contains deserialization functions for game components
#[derive(Resource, Clone, Default)]
pub struct GameSerDeRegistry {
    pub component_de_map: HashMap<ComponentId, ComponentDeserializeFn>,
    pub resource_de_map: HashMap<ResourceId, ResourceDeserializeFn>,
}

impl GameSerDeRegistry {
    pub fn new() -> GameSerDeRegistry {
        GameSerDeRegistry::default()
    }

    /// Registers a component into the [`GameSerDeRegistry`] for automatic serialization and deserialization
    pub fn register_component<C>(&mut self)
    where
        C: Component + Serialize + DeserializeOwned + SaveId,
    {
        if self.component_de_map.contains_key(&C::save_id_const()) {
            panic!(
                "SavingMap component_de_map already contains key {}",
                C::save_id_const(),
            )
        }
        self.component_de_map
            .insert(C::save_id_const(), component_deserialize_onto::<C>);
    }

    /// Registers a component into the [`GameSerDeRegistry`] for automatic serialization and deserialization
    pub fn register_resource<R>(&mut self)
    where
        R: Resource + Serialize + DeserializeOwned + SaveId,
    {
        if self.resource_de_map.contains_key(&R::save_id_const()) {
            panic!(
                "SavingMap component_de_map already contains key {}",
                R::save_id_const(),
            )
        }
        self.resource_de_map
            .insert(R::save_id_const(), resource_deserialize_into_world::<R>);
    }

    pub fn deserialize_component_onto(&self, data: &ComponentBinaryState, entity: &mut EntityMut) {
        if let Some(deserialize_fn) = self.component_de_map.get(&data.id) {
            deserialize_fn(&data.component, entity);
        }
    }

    /// Adds the default registry which has all the basic Bevy_GGF components and resources
    pub fn default_registry() -> GameSerDeRegistry {
        let mut game_registry = GameSerDeRegistry::new();

        game_registry.register_component::<TilePosition>();
        game_registry.register_component::<Tile>();
        game_registry.register_component::<TileTerrainInfo>();
        game_registry.register_component::<TileObjects>();
        game_registry.register_component::<TileMovementCosts>();
        game_registry.register_component::<ObjectId>();
        game_registry.register_component::<ObjectGridPosition>();
        game_registry.register_component::<Object>();
        game_registry.register_component::<ObjectStackingClass>();

        game_registry
    }
}

pub type ComponentDeserializeFn = fn(data: &Vec<u8>, entity: &mut EntityMut);

/// Deserializes a binary component onto the given entity.
pub fn component_deserialize_onto<T>(data: &Vec<u8>, entity: &mut EntityMut)
where
    T: Serialize + DeserializeOwned + Component + SaveId,
{
    let Some(keyframe) = bincode::deserialize::<T>(data).ok() else {
        return;
    };
    entity.insert(keyframe);
}

pub type ResourceDeserializeFn = fn(data: &Vec<u8>, world: &mut World);

/// Deserializes a binary component onto the given entity.
pub fn resource_deserialize_into_world<T>(data: &Vec<u8>, world: &mut World)
where
    T: Serialize + DeserializeOwned + Resource + SaveId,
{
    let Some(resource) = bincode::deserialize::<T>(data).ok() else {
        return;
    };
    world.insert_resource(resource);
}

/// Must be implemented on any components for objects that are expected to be saved
///
/// You must ensure that both this traits [save_id] function and [save_id_const] functions match
#[bevy_trait_query::queryable]
pub trait SaveId {
    fn save_id(&self) -> ComponentId;
    fn save_id_const() -> ComponentId
    where
        Self: Sized;

    /// Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself
    fn to_binary(&self) -> Option<Vec<u8>>;

    /// Saves self according to the implementation given in to_binary. For curves it saves the keyframe and not the entire component
    fn save(&self) -> Option<(ComponentId, Vec<u8>)> {
        let Some(data) = self.to_binary() else {
            return None;
        };
        Some((self.save_id(), data))
    }
}
