use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// The windowing context, this manages window z-ordering.
#[derive(Component, Reflect, Default, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct WindowingContext {
    /// Initial order of the window entities
    pub order: Vec<Entity>,
    /// The z-index map of entities.
    pub z_indices: HashMap<Entity, u32>,
}

impl WindowingContext {
    /// Adds a new entity to the window stack at a specific z-index.
    pub fn add(&mut self, entity: Entity, index: u32) {
        self.order.push(entity);
        self.z_indices.insert(entity, index);
    }

    /// Shifts a window entity to the top of the stack.
    pub fn shift_to_top(&mut self, entity: Entity) {
        if let Some(index) = self.order.iter().position(|e| *e == entity) {
            self.order.remove(index);
            self.order.push(entity);
        }

        self.z_indices.clear();
        for (index, entity) in self.order.iter().enumerate() {
            self.z_indices.insert(*entity, index as u32);
        }
    }

    /// Get a window entity's z-index.
    pub fn get(&self, entity: Entity) -> u32 {
        *self.z_indices.get(&entity).unwrap()
    }

    /// Get or add a new window entity's z-index
    pub fn get_or_add(&mut self, entity: Entity) -> u32 {
        if self.order.contains(&entity) {
            self.get(entity)
        } else {
            self.add(entity, 0);
            self.shift_to_top(entity);
            self.get(entity)
        }
    }
}

/// A widget that creates and provides the windowing context context.
/// This is used by the other scrolling widgets so they can understand how to
/// behave.
#[derive(Component, Widget, Reflect, Default, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq, Clone)]
#[auto_update(render)]
#[require(WidgetChildren, WoodpeckerStyle)]
pub struct WindowingContextProvider {
    /// The initial windowing context
    pub initial_value: WindowingContext,
    #[reflect(ignore)]
    /// An optional TaggedContext that allows you to tag the context
    /// for smarter querying later.
    pub tag: Option<TaggedContext>,
}

fn render(
    mut commands: Commands,
    mut context: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&mut WidgetChildren, &WindowingContextProvider)>,
) {
    let Ok((mut children, provider)) = query.get_mut(**current_widget) else {
        return;
    };

    // Setup windowing context.
    let entity = context.use_context(
        &mut commands,
        *current_widget,
        provider.initial_value.clone(),
    );

    if let Some(tag) = provider.tag.as_ref() {
        (tag.f)(commands.entity(entity));
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real `Entity` ids (not `Entity::from_raw`) so equality/hashing behave exactly as they
    /// would for entities spawned by the widget systems this struct actually serves.
    fn spawn_entities(count: usize) -> Vec<Entity> {
        let mut world = World::new();
        (0..count).map(|_| world.spawn_empty().id()).collect()
    }

    #[test]
    fn add_assigns_the_given_z_index_and_appends_to_order() {
        let entities = spawn_entities(1);
        let mut context = WindowingContext::default();
        context.add(entities[0], 7);
        assert_eq!(context.order, vec![entities[0]]);
        assert_eq!(context.get(entities[0]), 7);
    }

    #[test]
    fn shift_to_top_moves_the_entity_to_the_end_of_order() {
        let entities = spawn_entities(3);
        let mut context = WindowingContext::default();
        for (i, e) in entities.iter().enumerate() {
            context.add(*e, i as u32);
        }
        context.shift_to_top(entities[0]);
        assert_eq!(
            context.order,
            vec![entities[1], entities[2], entities[0]],
            "bringing entities[0] to the top must move it to the end of `order` -- the end of \
             `order` is what shift_to_top treats as topmost"
        );
    }

    #[test]
    fn shift_to_top_renumbers_the_rest_of_the_stack_to_match_the_new_order() {
        let entities = spawn_entities(3);
        let mut context = WindowingContext::default();
        for (i, e) in entities.iter().enumerate() {
            // Deliberately non-sequential starting indices so this test can't pass by
            // accident just because the initial values already happened to be 0..n.
            context.add(*e, (10 + i) as u32);
        }
        context.shift_to_top(entities[0]);
        assert_eq!(
            context.get(entities[1]),
            0,
            "the other windows must shift down to fill the gap left by the one that moved to \
             the top"
        );
        assert_eq!(
            context.get(entities[2]),
            1,
            "the other windows must shift down to fill the gap left by the one that moved to \
             the top"
        );
        assert_eq!(
            context.get(entities[0]),
            2,
            "the brought-to-front entity must land on the highest (topmost) z-index"
        );
    }

    #[test]
    fn shift_to_top_on_an_untracked_entity_still_renumbers_the_existing_order() {
        let entities = spawn_entities(3);
        let mut context = WindowingContext::default();
        context.add(entities[0], 5);
        context.add(entities[1], 6);

        // entities[2] was never added, so shift_to_top can't find it in `order` and must fall
        // through without inserting it -- but it should still renumber the tracked entities,
        // since the renumbering loop runs unconditionally.
        context.shift_to_top(entities[2]);

        assert_eq!(
            context.order,
            vec![entities[0], entities[1]],
            "an untracked entity must never be inserted into the stack by shift_to_top"
        );
        assert_eq!(context.get(entities[0]), 0);
        assert_eq!(context.get(entities[1]), 1);
    }

    #[test]
    fn get_or_add_returns_the_existing_index_without_reordering() {
        let entities = spawn_entities(2);
        let mut context = WindowingContext::default();
        context.add(entities[0], 3);
        context.add(entities[1], 9);

        let index = context.get_or_add(entities[0]);

        assert_eq!(
            index, 3,
            "get_or_add must not renumber an entity that's already tracked"
        );
        assert_eq!(
            context.order,
            vec![entities[0], entities[1]],
            "get_or_add on an already-tracked entity must leave stacking order untouched"
        );
    }

    #[test]
    fn get_or_add_gives_a_brand_new_entity_the_top_z_order() {
        let entities = spawn_entities(3);
        let mut context = WindowingContext::default();
        context.add(entities[0], 0);
        context.add(entities[1], 1);

        let index = context.get_or_add(entities[2]);

        assert_eq!(
            index, 2,
            "a freshly-inserted window must start out on top of the existing stack, not \
             buried underneath it"
        );
        assert_eq!(
            context.order.last(),
            Some(&entities[2]),
            "the newly-added entity must end up last in `order`, which is what this struct \
             treats as the topmost slot"
        );
    }

    #[test]
    fn repeated_bring_to_front_calls_preserve_relative_order_of_everything_else() {
        let entities = spawn_entities(4);
        let mut context = WindowingContext::default();
        for (i, e) in entities.iter().enumerate() {
            context.add(*e, i as u32);
        }

        context.shift_to_top(entities[1]);
        context.shift_to_top(entities[0]);
        context.shift_to_top(entities[1]);

        assert_eq!(
            context.order,
            vec![entities[2], entities[3], entities[0], entities[1]],
            "each bring-to-front call must move only its target entity, leaving the relative \
             order of every other entity untouched"
        );
        // z_indices must always be exactly the 0..n permutation implied by `order` -- this is
        // the invariant every other WindowingContext method relies on.
        for (i, e) in context.order.iter().enumerate() {
            assert_eq!(context.get(*e), i as u32);
        }
    }
}
