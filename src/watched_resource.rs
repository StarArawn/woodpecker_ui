use bevy::prelude::*;

use crate::diffable_prop::ReflectDiffableProp;

/// A component a widget adds to its own entity (usually via `#[require(...)]`) to mirror
/// resource `T`'s value onto it, so the generic reflection-based diffing (see
/// [`crate::diffing`]) sees it change like any other prop -- resources aren't entity
/// components and so aren't otherwise visible to that walk.
///
/// Being generic, callers must explicitly call `app.register_type::<WatchedResource<T>>()`
/// for each `T` watched, or diffing will silently never see it change.
#[derive(Component, Reflect, Clone, PartialEq, Deref, DerefMut)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct WatchedResource<T: Resource + Reflect + Clone + PartialEq>(pub T);

impl<T: Resource + Reflect + Clone + PartialEq + Default> Default for WatchedResource<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

/// Keeps `WatchedResource<T>` in sync with `Res<T>` on every entity that has one. Register
/// this to run before [`crate::runner::system`] (e.g. in `PreUpdate`) for each resource type
/// a widget watches.
///
/// Deliberately does NOT gate on `resource.is_changed()`: a widget can mount many frames
/// after `T` last changed and would otherwise be stuck at `T::default()` forever. The
/// per-entity `!=` check already keeps the steady-state cost cheap.
pub fn sync_watched_resource<T: Resource + Reflect + Clone + PartialEq>(
    resource: Res<T>,
    mut query: Query<&mut WatchedResource<T>>,
) {
    for mut watched in query.iter_mut() {
        if watched.0 != *resource {
            watched.0 = resource.clone();
        }
    }
}
