use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
    reflect::Reflect,
};
use bevy_mod_picking::prelude::EntityEvent;

use crate::focus::CurrentFocus;

#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetKeyboardButtonEvent {
    /// The target of this event
    #[target]
    pub target: Entity,

    /// The keyboard button pressedy
    pub code: KeyCode,
}

#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetKeyboardCharEvent {
    /// The target of this event
    #[target]
    pub target: Entity,

    /// The char pressed
    /// Note this might be a series of chars such as a graphemes
    /// which is why we use SmolStr here.
    pub c: smol_str::SmolStr,
}

pub(crate) fn runner(
    mut key_event: EventReader<KeyboardInput>,
    mut char_event_writer: EventWriter<WidgetKeyboardCharEvent>,
    mut button_event_writer: EventWriter<WidgetKeyboardButtonEvent>,
    current_focus: Res<CurrentFocus>,
) {
    for event in key_event.read() {
        if current_focus.get() != Entity::PLACEHOLDER && event.state == ButtonState::Pressed {
            match &event.logical_key {
                Key::Character(c) => {
                    char_event_writer.send(WidgetKeyboardCharEvent {
                        target: current_focus.get(),
                        c: c.clone(),
                    });
                }
                Key::Space => {
                    char_event_writer.send(WidgetKeyboardCharEvent {
                        target: current_focus.get(),
                        c: smol_str::SmolStr::new(" "),
                    });
                }
                _ => {}
            }

            // Also send a button event.
            button_event_writer.send(WidgetKeyboardButtonEvent {
                target: current_focus.get(),
                code: event.key_code,
            });
        }
    }
}
