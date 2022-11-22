use bevy::prelude::Bundle;
use crate::selection::SelectableEntity;





#[derive(Bundle)]
pub struct UnitBundle{
    selectable: SelectableEntity
    
}

