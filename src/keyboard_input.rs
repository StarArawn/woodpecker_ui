use web_time::{Duration, Instant};

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

/// An event that fires when a keyboard button is pressed.
/// The event target is the currently focused entity.
/// Note: This does not continously fire unless a button is released.
#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetKeyboardButtonEvent {
    /// The target of this event
    #[target]
    pub target: Entity,

    /// The keyboard button pressed
    pub code: KeyCode,
}

/// An event that fires when a keyboard character is sent.
/// The event target is the currently focused entity.
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

/// An event that fires when the user pastes(ctrl + v).
/// The event target is the currently focused entity.
#[derive(Clone, PartialEq, Debug, Reflect, Event, EntityEvent)]
pub struct WidgetPasteEvent {
    /// The target of this event
    #[target]
    pub target: Entity,

    /// The char pressed
    /// Note this might be a series of chars such as a graphemes
    /// which is why we use SmolStr here.
    pub paste: smol_str::SmolStr,
}

#[cfg(target_arch = "wasm32")]
#[derive(Component)]
pub struct WidgetPasteEventWasm {
    /// The target of this event
    pub target: Entity,
    pub receiver: futures_channel::oneshot::Receiver<String>,
}

#[derive(Debug, Deref, DerefMut)]
pub(crate) struct TimeSinceLastPaste(Instant);
impl Default for TimeSinceLastPaste {
    fn default() -> Self {
        Self(Instant::now())
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn read_paste_events(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WidgetPasteEventWasm)>,
    mut paste_event_writer: EventWriter<WidgetPasteEvent>,
    mut time_since_last_paste: Local<TimeSinceLastPaste>,
) {
    for (entity, mut event) in &mut query {
        let Ok(Some(text)) = event.receiver.try_recv() else {
            continue;
        };
        *time_since_last_paste = TimeSinceLastPaste::default();
        paste_event_writer.send(WidgetPasteEvent {
            target: event.target,
            paste: smol_str::SmolStr::new(text.to_string()),
        });
        commands.entity(entity).despawn_recursive();
    }
}

pub(crate) fn runner(
    #[cfg(target_arch = "wasm32")] mut commands: Commands,
    mut time_since_last_paste: Local<TimeSinceLastPaste>,
    mut ctrl_pressed: Local<bool>,
    mut key_event: EventReader<KeyboardInput>,
    mut char_event_writer: EventWriter<WidgetKeyboardCharEvent>,
    mut button_event_writer: EventWriter<WidgetKeyboardButtonEvent>,
    #[cfg(not(target_arch = "wasm32"))] mut paste_event_writer: EventWriter<WidgetPasteEvent>,
    current_focus: Res<CurrentFocus>,
) {
    let mut v_pressed = false;

    for event in key_event.read() {
        if event.state == ButtonState::Released {
            match &event.key_code {
                KeyCode::ControlLeft => *ctrl_pressed = false,
                KeyCode::KeyV => {
                    *time_since_last_paste = TimeSinceLastPaste(
                        TimeSinceLastPaste::default()
                            .checked_sub(Duration::from_secs_f32(0.5))
                            .unwrap(),
                    );
                }
                _ => {}
            }
        }
        if current_focus.get() != Entity::PLACEHOLDER && event.state == ButtonState::Pressed {
            match &event.key_code {
                KeyCode::ControlLeft => *ctrl_pressed = true,
                KeyCode::KeyV => v_pressed = true,
                _ => {}
            }

            if *ctrl_pressed && v_pressed {
                if time_since_last_paste.elapsed().as_secs_f32() < 0.1 {
                    return;
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let Ok(mut clipboard) = arboard::Clipboard::new() else {
                        return;
                    };
                    let Ok(text) = clipboard.get_text() else {
                        return;
                    };
                    *time_since_last_paste = TimeSinceLastPaste::default();
                    paste_event_writer.send(WidgetPasteEvent {
                        target: current_focus.get(),
                        paste: smol_str::SmolStr::new(text),
                    });
                    return;
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let Some(clipboard) =
                        web_sys::window().and_then(|window| window.navigator().clipboard())
                    else {
                        warn!("no clipboard");
                        return;
                    };
                    let promise = clipboard.read_text();
                    let future = wasm_bindgen_futures::JsFuture::from(promise);

                    let (sender, receiver) = futures_channel::oneshot::channel::<String>();

                    let pool = bevy::tasks::TaskPool::new();
                    pool.spawn(async move {
                        let Ok(text) = future.await else {
                            return;
                        };
                        let Some(text) = text.as_string() else {
                            return;
                        };
                        let _ = sender.send(text);
                    });

                    commands.spawn(WidgetPasteEventWasm {
                        target: current_focus.get(),
                        receiver,
                    });

                    return;
                }
            }
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
