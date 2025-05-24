use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// The windowing context, this manages window z-ordering.
#[derive(Component, Reflect, Default, PartialEq, Clone)]
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
        if self.order.iter().any(|e| *e == entity) {
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
#[auto_update(render)]
#[props(WindowingContextProvider)]
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
