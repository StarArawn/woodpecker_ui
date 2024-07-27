use std::time::Instant;

use crate::{keyboard_input::{WidgetKeyboardButtonEvent, WidgetPasteEvent}, prelude::*, DefaultFont};
use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Out, Over, Pointer},
    focus::PickingInteraction,
    picking_core::Pickable,
    prelude::{ListenerInput, On},
};
use bevy_vello::text::VelloFont;
use unicode_segmentation::UnicodeSegmentation;

use super::{Clip, ClipBundle, Element, ElementBundle};

#[derive(Debug, Clone, Reflect)]
pub struct ChangedText {
    pub value: String,
}

#[derive(Component, Clone, PartialEq)]
pub struct TextboxStyles {
    pub normal: WoodpeckerStyle,
    pub hovered: WoodpeckerStyle,
    pub focused: WoodpeckerStyle,
}

impl Default for TextboxStyles {
    fn default() -> Self {
        let shared = WoodpeckerStyle {
            background_color: Srgba::new(0.160, 0.172, 0.235, 1.0).into(),
            width: Units::Percentage(100.0),
            height: 26.0.into(),
            border_color: Srgba::new(0.360, 0.380, 0.474, 1.0).into(),
            border: Edge::new(0.0, 0.0, 0.0, 2.0),
            padding: Edge::new(0.0, 5.0, 0.0, 5.0),
            margin: Edge::new(0.0, 0.0, 0.0, 2.0),
            font_size: 14.0,
            ..Default::default()
        };
        Self {
            normal: WoodpeckerStyle { ..shared },
            hovered: WoodpeckerStyle { ..shared },
            focused: WoodpeckerStyle {
                border_color: Srgba::new(0.933, 0.745, 0.745, 1.0).into(),
                ..shared
            },
        }
    }
}

/// A generic textbox widget!
#[derive(Bundle, Clone)]
pub struct TextBoxBundle {
    /// The textbox component itself.
    pub text_box: TextBox,
    /// The rendering of the button widget.
    pub render: WidgetRender,
    /// A widget children component
    pub children: WidgetChildren,
    /// The widget styles,
    pub styles: WoodpeckerStyle,
    /// The textbox styles
    pub textbox_styles: TextboxStyles,
    /// Provides overrides for picking behavior.
    pub pickable: Pickable,
    /// Tracks entity interaction state.
    pub interaction: PickingInteraction,
    /// Tells woodpecker we want this widget to get focus events.
    pub focuable: Focusable,
}

impl Default for TextBoxBundle {
    fn default() -> Self {
        Self {
            text_box: Default::default(),
            render: WidgetRender::Quad,
            children: Default::default(),
            styles: Default::default(),
            pickable: Default::default(),
            interaction: Default::default(),
            textbox_styles: TextboxStyles::default(),
            focuable: Focusable,
        }
    }
}

/// The Woodpecker UI Button
#[derive(Component, Reflect, Default, PartialEq, Widget, Clone)]
#[auto_update(render)]
#[props(TextBox, TextboxStyles)]
#[state(TextBoxState)]
pub struct TextBox {
    pub initial_value: String,
}

#[derive(Component, PartialEq, Clone)]
pub struct TextBoxState {
    // Mouse input state
    pub hovering: bool,
    pub focused: bool,
    // Keyboard input state
    pub graphemes: Vec<String>,
    pub cursor_x: f32,
    pub cursor_position: usize,
    pub cursor_visible: bool,
    pub cursor_last_update: Instant,
    pub current_value: String,
}

impl Default for TextBoxState {
    fn default() -> Self {
        Self {
            hovering: Default::default(),
            focused: Default::default(),
            graphemes: Default::default(),
            cursor_x: 0.0,
            cursor_position: Default::default(),
            cursor_visible: Default::default(),
            cursor_last_update: Instant::now(),
            current_value: String::new(),
        }
    }
}

pub fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hook_helper: ResMut<HookHelper>,
    mut font_manager: ResMut<FontManager>,
    default_font: Res<DefaultFont>,
    mut query: Query<(
        Ref<TextBox>,
        &mut WoodpeckerStyle,
        &TextboxStyles,
        &mut WidgetChildren,
    )>,
    mut state_query: Query<&mut TextBoxState>,
) {
    let Ok((text_box, mut style, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hook_helper.use_state(
        &mut commands,
        *current_widget,
        TextBoxState {
            current_value: text_box.initial_value.clone(),
            ..Default::default()
        },
    );

    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    if text_box.is_changed() {
        state.current_value.clone_from(&text_box.initial_value);

        // Update graphemes
        set_graphemes(&mut state);

        state.cursor_position = state.cursor_position.min(state.graphemes.len());

        // TODO: Use style font?
        set_new_cursor_position(
            &mut state,
            &mut font_manager,
            &default_font.0,
            styles.normal.font_size,
        );
    }

    // Hook up events
    let widget_entity = **current_widget;
    commands
        .entity(widget_entity)
        // Char event
        .insert((
            On::<WidgetKeyboardCharEvent>::run(
                move |event: ResMut<ListenerInput<WidgetKeyboardCharEvent>>,
                      style_query: Query<&WoodpeckerStyle>,
                      mut state_query: Query<&mut TextBoxState>,
                      default_font: Res<DefaultFont>,
                      mut font_manager: ResMut<FontManager>,
                      mut event_writer: EventWriter<OnChange<ChangedText>>| {
                    let Ok(styles) = style_query.get(event.target) else {
                        return;
                    };
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };

                    let cursor_pos = state.cursor_position;

                    let char_pos: usize =
                        state.graphemes[0..cursor_pos].iter().map(|g| g.len()).sum();
                    state.current_value.insert_str(char_pos, &event.c);
                    state.cursor_position += 1;

                    event_writer.send(OnChange {
                        target: widget_entity,
                        data: ChangedText {
                            value: state.current_value.clone(),
                        },
                    });

                    // Update graphemes
                    set_graphemes(&mut state);

                    // TODO: Use style font?
                    set_new_cursor_position(
                        &mut state,
                        &mut font_manager,
                        &default_font.0,
                        styles.font_size,
                    );
                },
            ),
            On::<Pointer<Over>>::run(move |mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if !state.focused {
                    state.hovering = true;
                }
            }),
            On::<Pointer<Out>>::run(move |mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
            }),
            On::<WidgetFocus>::run(move |mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
                state.focused = true;
            }),
            On::<WidgetBlur>::run(move |mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.focused = false;
            }),
        ))
        // Paste event
        .insert(On::<WidgetPasteEvent>::run(
            move |event: ResMut<ListenerInput<WidgetPasteEvent>>,
                  default_font: Res<DefaultFont>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut state_query: Query<&mut TextBoxState>,
                  mut font_manager: ResMut<FontManager>,
                  mut event_writer: EventWriter<OnChange<ChangedText>>| {
                let Ok(styles) = style_query.get(event.target) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let char_pos: usize = state.graphemes[0..state.cursor_position]
                    .iter()
                    .map(|g| g.len())
                    .sum();
                state.current_value.insert_str(char_pos, &event.paste);

                event_writer.send(OnChange {
                    target: widget_entity,
                    data: ChangedText {
                        value: state.current_value.clone(),
                    },
                });

                state.cursor_position += get_graphemes(&event.paste).len();

                // Update graphemes
                set_graphemes(&mut state);
                // TODO: Use style font?
                set_new_cursor_position(
                    &mut state,
                    &mut font_manager,
                    &default_font.0,
                    styles.font_size,
                );
            },
        ))
        .insert(On::<WidgetKeyboardButtonEvent>::run(
            move |event: ResMut<ListenerInput<WidgetKeyboardButtonEvent>>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut state_query: Query<&mut TextBoxState>,
                  default_font: Res<DefaultFont>,
                  mut font_manager: ResMut<FontManager>,
                  mut event_writer: EventWriter<OnChange<ChangedText>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>| {
                if event.code == KeyCode::ArrowRight {
                    let Ok(styles) = style_query.get(event.target) else {
                        return;
                    };
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    if state.cursor_position < state.graphemes.len() {
                        if keyboard_input.pressed(KeyCode::ControlLeft) {
                            if state
                                .graphemes
                                .get(state.cursor_position)
                                .map(|g| g.contains(' '))
                                .unwrap_or_default()
                            {
                                state.cursor_position += 1;
                            } else {
                                while !state
                                    .graphemes
                                    .get(state.cursor_position)
                                    .map(|g| g.contains(' '))
                                    .unwrap_or(true)
                                {
                                    state.cursor_position += 1;
                                }
                            }
                        } else {
                            state.cursor_position += 1;
                        }
                    }
                    // TODO: Use style font?
                    set_new_cursor_position(
                        &mut state,
                        &mut font_manager,
                        &default_font.0,
                        styles.font_size,
                    );
                }
                if event.code == KeyCode::ArrowLeft {
                    let Ok(styles) = style_query.get(event.target) else {
                        return;
                    };
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };

                    if keyboard_input.pressed(KeyCode::ControlLeft) {
                        if state.cursor_position > 0 {
                            if state
                                .graphemes
                                .get(state.cursor_position - 1)
                                .map(|g| g.contains(' '))
                                .unwrap_or_default()
                            {
                                state.cursor_position -= 1;
                            } else {
                                while !state
                                    .graphemes
                                    .get(state.cursor_position - 1)
                                    .map(|g| g.contains(' '))
                                    .unwrap_or(true)
                                {
                                    state.cursor_position -= 1;
                                }
                            }
                        }
                    } else if state.cursor_position > 0 {
                        state.cursor_position -= 1;
                    }

                    // TODO: Use style font?
                    set_new_cursor_position(
                        &mut state,
                        &mut font_manager,
                        &default_font.0,
                        styles.font_size,
                    );
                }
                if event.code == KeyCode::Backspace {
                    let Ok(styles) = style_query.get(event.target) else {
                        return;
                    };
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    let cursor_pos = state.cursor_position;

                    if !state.current_value.is_empty() && cursor_pos != 0 {
                        let char_pos: usize = state.graphemes[0..cursor_pos - 1]
                            .iter()
                            .map(|g| g.len())
                            .sum();
                        state.current_value.remove(char_pos);
                        state.cursor_position -= 1;

                        event_writer.send(OnChange {
                            target: widget_entity,
                            data: ChangedText {
                                value: state.current_value.clone(),
                            },
                        });

                        // Update graphemes
                        set_graphemes(&mut state);

                        // TODO: Use style font?
                        set_new_cursor_position(
                            &mut state,
                            &mut font_manager,
                            &default_font.0,
                            styles.font_size,
                        );
                    }
                }
            },
        ));

    if state.focused {
        *style = styles.focused
    } else if state.hovering {
        *style = styles.hovered
    } else {
        *style = styles.normal
    }

    let cursor_styles = WoodpeckerStyle {
        background_color: styles.focused.border_color,
        position: WidgetPosition::Absolute,
        top: 5.0.into(),
        left: state.cursor_x.into(),
        width: 2.0.into(),
        height: (style.height.value_or(26.0) - 10.0).into(),
        ..Default::default()
    };

    let mut clip_children = WidgetChildren::default().with_child::<Element>((
        ElementBundle {
            styles: WoodpeckerStyle {
                font_size: style.font_size,
                ..Default::default()
            },
            ..Default::default()
        },
        WidgetRender::Text {
            content: state.current_value.clone(),
            word_wrap: false,
        },
    ));

    if state.cursor_visible {
        clip_children.add::<Element>((
            ElementBundle {
                styles: cursor_styles,
                ..Default::default()
            },
            WidgetRender::Quad,
        ));
    }

    children.add::<Clip>(ClipBundle {
        styles: WoodpeckerStyle {
            width: style.width,
            height: style.height,
            align_items: Some(WidgetAlignItems::Center),
            ..ClipBundle::default().styles // Take styles from clip bundle too!
        },
        children: clip_children,
        ..Default::default()
    });

    children.apply(current_widget.as_parent());
}

fn get_graphemes(value: &str) -> Vec<&str> {
    UnicodeSegmentation::graphemes(value, true).collect::<Vec<_>>()
}

fn set_graphemes(state: &mut TextBoxState) {
    state.graphemes = get_graphemes(&state.current_value)
        .iter()
        .map(|g| g.to_string())
        .collect::<Vec<_>>();
}

fn set_new_cursor_position(
    state: &mut TextBoxState,
    font_manager: &mut FontManager,
    font_handle: &Handle<VelloFont>,
    font_size: f32,
) {
    let string_to_cursor = if state.graphemes.is_empty() {
        "".into()
    } else {
        state.graphemes[0..state.cursor_position].join("")
    };

    let buffer = font_manager
        .layout(
            Vec2::new(1000000.0, font_size * 1.2),
            &WoodpeckerStyle {
                font_size,
                ..Default::default()
            },
            font_handle,
            &string_to_cursor,
            false,
        )
        .unwrap();

    let mut max_x: f32 = 0.0;

    for run in buffer.layout_runs() {
        max_x = max_x.max(run.line_w);
    }

    state.cursor_x = max_x;
}

pub fn cursor_animation_system(
    mut state_query: ParamSet<(Query<(Entity, &TextBoxState)>, Query<&mut TextBoxState>)>,
) {
    let mut should_update = Vec::new();

    for (entity, state) in state_query.p0().iter() {
        // Avoid mutating state if we can avoid it.
        if state.cursor_last_update.elapsed().as_secs_f32() > 0.5 && state.focused {
            should_update.push(entity);
        }
    }

    for state_entity in should_update.drain(..) {
        if let Ok(mut state) = state_query.p1().get_mut(state_entity) {
            state.cursor_last_update = Instant::now();
            state.cursor_visible = !state.cursor_visible;
        }
    }
}
