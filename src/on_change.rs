use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct OnChange<T: std::fmt::Debug + Clone + Reflect> {
    /// The target of this event
    #[target]
    pub target: Entity,
    /// The value of the change.
    pub data: T,
}
