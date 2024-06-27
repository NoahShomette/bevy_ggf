use crate::{
    mapping::{
        terrain::TileTerrainInfo,
        tiles::{ObjectStackingClass, Tile, TileObjects, TilePosition},
    },
    movement::TileMovementCosts,
    object::{Object, ObjectGridPosition, ObjectId, ObjectInfo}, player::{Player, PlayerMarker},
};

use super::saving::{BinaryComponentId, SaveId};

impl SaveId for TilePosition {
    fn save_id(&self) -> BinaryComponentId {
        Self::save_id_const()
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        0
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for Tile {
    fn save_id(&self) -> BinaryComponentId {
        1
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        1
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for TileTerrainInfo {
    fn save_id(&self) -> BinaryComponentId {
        2
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        2
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for TileObjects {
    fn save_id(&self) -> BinaryComponentId {
        3
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        3
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for TileMovementCosts {
    fn save_id(&self) -> BinaryComponentId {
        4
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        4
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for ObjectId {
    fn save_id(&self) -> BinaryComponentId {
        5
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        5
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for ObjectGridPosition {
    fn save_id(&self) -> BinaryComponentId {
        6
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        6
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for Object {
    fn save_id(&self) -> BinaryComponentId {
        7
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        7
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for ObjectStackingClass {
    fn save_id(&self) -> BinaryComponentId {
        8
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        8
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for PlayerMarker {
    fn save_id(&self) -> BinaryComponentId {
        9
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        9
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for Player {
    fn save_id(&self) -> BinaryComponentId {
        10
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        10
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl SaveId for ObjectInfo {
    fn save_id(&self) -> BinaryComponentId {
        11
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        11
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}
