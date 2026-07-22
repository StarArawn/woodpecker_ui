use bevy::{platform::collections::HashMap, prelude::*, reflect::PartialReflect};
use core::any::TypeId;

/// Lives on the same entity as a widget's real prop/state/context components. Stores a
/// cloned snapshot of the last-diffed value for every [`crate::diffable_prop::DiffableProp`]
/// -tagged component type currently observed on that entity, keyed by concrete `TypeId`.
///
/// Being a plain, non-generic component (not a clone of the prop type itself) lets it live
/// directly alongside the real components without hitting Bevy's one-component-per-type
/// -per-entity limitation.
#[derive(Component, Default)]
pub(crate) struct PreviousSnapshot(HashMap<TypeId, Box<dyn PartialReflect>>);

impl PreviousSnapshot {
    pub(crate) fn get(&self, type_id: TypeId) -> Option<&dyn PartialReflect> {
        self.0.get(&type_id).map(|b| b.as_ref())
    }

    pub(crate) fn set(&mut self, type_id: TypeId, value: Box<dyn PartialReflect>) {
        self.0.insert(type_id, value);
    }
}
