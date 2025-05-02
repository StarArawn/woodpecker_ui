use bevy::{
    ecs::{query::QueryData, traversal::Traversal},
    prelude::*,
};
// use bevy_mod_picking::prelude::*;

/// This is a special event used by the TextBox widget and others
/// to let consumers know that their values have changed internally
#[derive(Clone, PartialEq, Debug, Reflect, Component)]
#[reflect(Component, Debug, Clone)]
pub struct Change<T: std::fmt::Debug + Clone + Reflect> {
    /// The target of this event
    pub target: Entity,
    /// The value of the change.
    pub data: T,
}

/// A traversal query (i.e. it implements [`Traversal`]) intended for use with [`Pointer`] events.
///
/// This will always traverse to the parent, if the entity being visited has one. Otherwise, it
/// propagates to the pointer's window and stops there.
#[derive(QueryData)]
pub struct ChangeTraversal {
    child_of: Option<&'static ChildOf>,
}

impl<E> Traversal<Change<E>> for ChangeTraversal
where
    E: std::fmt::Debug + Clone + Reflect,
{
    fn traverse(item: Self::Item<'_>, _change: &Change<E>) -> Option<Entity> {
        // Send event to parent, if it has one.
        if let Some(child_of) = item.child_of {
            return Some(child_of.parent());
        };

        None
    }
}

impl<E> Event for Change<E>
where
    E: std::fmt::Debug + Clone + Reflect,
{
    type Traversal = ChangeTraversal;

    const AUTO_PROPAGATE: bool = true;
}
