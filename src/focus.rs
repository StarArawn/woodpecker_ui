use bevy::prelude::*;
use bevy_mod_picking::{
    focus::PickingInteraction,
    prelude::{EntityEvent, PointerPress},
};

/// Marks an entity as focusable
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Focusable;

/// A resource used to keep track of the currently focused entity.
#[derive(Resource, Debug, Clone, Copy)]
pub struct CurrentFocus(Entity);

impl CurrentFocus {
    /// Create a new CurrentFocus.
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// Gets the entity that has focus.
    pub fn get(&self) -> Entity {
        self.0
    }

    /// Sets the entity that has focus.
    pub fn set(&mut self, entity: Entity) {
        self.0 = entity;
    }

    #[allow(dead_code)]
    pub(crate) fn find_next_focus() {
        // TODO: Write system to find next focused object
        todo!();
    }

    #[allow(dead_code)]
    pub(crate) fn find_prev_focus() {
        // TODO: Write system to find prev focused object
        todo!();
    }

    pub(crate) fn click_focus(
        mut current_focus: ResMut<CurrentFocus>,
        query: Query<
            (Entity, Option<&PickingInteraction>),
            (With<Focusable>, Changed<PickingInteraction>),
        >,
        mut focus_writer: EventWriter<WidgetFocus>,
        mut blur_writer: EventWriter<WidgetBlur>,
        pointer_query: Query<&PointerPress>,
    ) {
        let mut none_selected = true;
        for (entity, picking_interaction) in query.iter() {
            if let Some(picking_interaction) = picking_interaction {
                // Check if pressed
                if matches!(picking_interaction, PickingInteraction::Pressed) {
                    // Blur previously focused entity.
                    if current_focus.get() != entity {
                        blur_writer.send(WidgetBlur {
                            target: current_focus.get(),
                        });
                    }
                    // Focus new entity
                    *current_focus = CurrentFocus::new(entity);
                    focus_writer.send(WidgetFocus { target: entity });
                    none_selected = false;
                }
            }
        }

        if none_selected && pointer_query.iter().any(|press| press.is_primary_pressed()) {
            // Blur if we have a focused entity because we had no "hits" this frame.
            if current_focus.get() != Entity::PLACEHOLDER {
                blur_writer.send(WidgetBlur {
                    target: current_focus.get(),
                });
            }
            // Remove current focus.
            *current_focus = CurrentFocus::new(Entity::PLACEHOLDER);
        }
    }
}

/// A bevy_eventlistener Event that triggers when a widget has focus.
/// Note: The widget must have the Focusable component tag.
#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetFocus {
    /// The target of this event
    #[target]
    pub target: Entity,
}

/// A bevy_eventlistener Event that triggers when a widget has lost focus.
/// Note: The widget must have the Focusable component tag.
#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetBlur {
    /// The target of this event
    #[target]
    pub target: Entity,
}
