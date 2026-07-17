use bevy::{
    picking::{hover::PickingInteraction, pointer::PointerPress},
    prelude::*,
};

use crate::{children::WidgetChildren, CurrentWidget};

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

    pub(crate) fn click_focus(
        mut commands: Commands,
        mut current_focus: ResMut<CurrentFocus>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        query: Query<
            (Entity, Option<&PickingInteraction>),
            (With<Focusable>, Changed<PickingInteraction>),
        >,
        pointer_query: Query<&PointerPress>,
    ) {
        let mut none_selected = true;
        for (entity, picking_interaction) in query.iter() {
            if let Some(picking_interaction) = picking_interaction {
                // Check if pressed
                if mouse_input.just_pressed(MouseButton::Left)
                    && matches!(picking_interaction, PickingInteraction::Pressed)
                {
                    // Blur previously focused entity.
                    if current_focus.get() != entity {
                        commands.trigger(WidgetBlur {
                            target: current_focus.get(),
                        });
                    }
                    // Focus new entity
                    *current_focus = CurrentFocus::new(entity);
                    commands.trigger(WidgetFocus { target: entity });
                    none_selected = false;
                }
            }
        }

        if mouse_input.just_pressed(MouseButton::Left)
            && none_selected
            && pointer_query.iter().any(|press| press.is_primary_pressed())
        {
            // Blur if we have a focused entity because we had no "hits" this frame.
            if current_focus.get() != Entity::PLACEHOLDER {
                commands.trigger(WidgetBlur {
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
#[derive(Clone, PartialEq, Debug, Reflect, EntityEvent)]
pub struct WidgetFocus {
    /// The target of this event
    #[event_target]
    pub target: Entity,
}

/// A bevy_eventlistener Event that triggers when a widget has lost focus.
/// Note: The widget must have the Focusable component tag.
#[derive(Clone, PartialEq, Debug, Reflect, EntityEvent)]
pub struct WidgetBlur {
    /// The target of this event
    #[event_target]
    pub target: Entity,
}

/// A bevy_eventlistener Event that triggers when a focused widget is "confirmed" -- e.g. a
/// gamepad's south button while the `gamepad-nav` feature is enabled and a controller is
/// connected (see `gamepad_focus::activate`). Mirrors `WidgetFocus`/`WidgetBlur`'s shape.
/// Not itself feature-gated: the event type and [`WidgetChildren::on_click_or_activate`] stay
/// available unconditionally so a widget can be written "gamepad-nav ready" even in an app
/// that never enables the feature -- the observer simply never fires in that case.
#[derive(Clone, PartialEq, Debug, Reflect, EntityEvent)]
pub struct WidgetActivate {
    /// The target of this event
    #[event_target]
    pub target: Entity,
}

impl WidgetChildren {
    /// Adds observers for both `Pointer<Click>` and [`WidgetActivate`] that invoke the same
    /// `handler`, so a widget's click behavior also responds to a gamepad confirm press --
    /// one call instead of a hand-written parallel `WidgetActivate` observer per widget.
    ///
    /// `handler` only receives `Commands` (not the triggering event) since `Pointer<Click>`
    /// and `WidgetActivate` don't share a data shape -- for anything needing the click's own
    /// details (button, position, etc.), use [`WidgetChildren::observe`] directly instead.
    pub fn on_click_or_activate(
        &mut self,
        spawn_location: CurrentWidget,
        handler: impl Fn(&mut Commands) + Send + Sync + Clone + 'static,
    ) -> &mut Self {
        let click_handler = handler.clone();
        self.observe(
            spawn_location,
            move |_: On<Pointer<Click>>, mut commands: Commands| {
                click_handler(&mut commands);
            },
        );
        self.observe(
            spawn_location,
            move |_: On<WidgetActivate>, mut commands: Commands| {
                handler(&mut commands);
            },
        );
        self
    }

    /// Builder form of [`WidgetChildren::on_click_or_activate`].
    pub fn with_on_click_or_activate(
        mut self,
        spawn_location: CurrentWidget,
        handler: impl Fn(&mut Commands) + Send + Sync + Clone + 'static,
    ) -> Self {
        self.on_click_or_activate(spawn_location, handler);
        self
    }
}
