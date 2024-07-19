use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
    reflect::Reflect,
};
use bevy_mod_picking::prelude::EntityEvent;

use crate::focus::CurrentFocus;

#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetKeyboardEvent {
    /// The target of this event
    #[target]
    pub target: Entity,

    pub c: smol_str::SmolStr,
}

pub(crate) fn runner(
    mut key_event: EventReader<KeyboardInput>,
    mut event_writer: EventWriter<WidgetKeyboardEvent>,
    current_focus: Res<CurrentFocus>,
) {
    for event in key_event.read() {
        if current_focus.get() != Entity::PLACEHOLDER {
            match &event.logical_key {
                Key::Character(c) => {
                    event_writer.send(WidgetKeyboardEvent {
                        target: current_focus.get(),
                        c: c.clone(),
                    });
                }
                Key::Space => {
                    event_writer.send(WidgetKeyboardEvent {
                        target: current_focus.get(),
                        c: smol_str::SmolStr::new(" "),
                    });
                }
                _ => {}
            }
        }
    }
}
