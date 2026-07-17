use bevy::prelude::*;

/// Marker trait used to opt a prop/state/context component type into the generic
/// reflection-based re-render diffing system (see [`crate::diffing`]).
///
/// Opt in once at the type's definition site:
///
/// ```ignore
/// #[derive(Component, Reflect, Clone, PartialEq)]
/// #[reflect(Component, DiffableProp, PartialEq)]
/// pub struct MyProp {
///     pub value: i32,
/// }
/// ```
///
/// Opted-in components are compared each frame against their last-diffed snapshot; any
/// difference (or a new widget entity) triggers a re-render. Types that don't opt in are
/// simply invisible to the diff.
#[bevy::reflect::reflect_trait]
pub trait DiffableProp: Reflect {}

impl<T: Reflect> DiffableProp for T {}
