use bevy::prelude::*;
use bevy_mod_picking::{focus::PickingInteraction, prelude::EntityEvent};

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Focusable;

#[derive(Resource, Debug, Clone, Copy)]
pub struct CurrentFocus(Entity);

impl CurrentFocus {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn get(&self) -> Entity {
        self.0
    }

    pub fn set(&mut self, entity: Entity) {
        self.0 = entity;
    }

    pub(crate) fn find_next_focus() {
        // TODO: Write system to find next focused object
        todo!();
    }

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
        mouse_input: Res<ButtonInput<MouseButton>>,
    ) {
        // TODO: This probably wont work well for our UI hierarchy. Enough for now..
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
        // TODO: Allow users to define "primary" button.
        if none_selected && mouse_input.pressed(MouseButton::Left) {
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

#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetFocus {
    /// The target of this event
    #[target]
    pub target: Entity,
}

#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetBlur {
    /// The target of this event
    #[target]
    pub target: Entity,
}
