use bevy_vello::prelude::VelloFont;
use web_time::Instant;

use crate::{
    keyboard_input::{WidgetKeyboardButtonEvent, WidgetPasteEvent},
    prelude::*,
    DefaultFont,
};
use bevy::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

use super::{colors, Clip, Element};

/// A textbox change event.
#[derive(Debug, Clone, Reflect)]
pub struct TextChanged {
    /// The current text value
    pub value: String,
}

/// A collection of textbox styles.
#[derive(Component, Clone, PartialEq)]
pub struct TextboxStyles {
    /// Normal styles
    pub normal: WoodpeckerStyle,
    /// Hovered styles
    pub hovered: WoodpeckerStyle,
    /// Focused styles
    pub focused: WoodpeckerStyle,
    /// Cursor styles
    pub cursor: WoodpeckerStyle,
}

impl Default for TextboxStyles {
    fn default() -> Self {
        let shared = WoodpeckerStyle {
            background_color: colors::DARK_BACKGROUND,
            width: Units::Percentage(100.0),
            height: 26.0.into(),
            border_color: colors::BACKGROUND_LIGHT,
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
                border_color: colors::PRIMARY,
                ..shared
            },
            cursor: WoodpeckerStyle {
                background_color: colors::PRIMARY,
                position: WidgetPosition::Absolute,
                top: 5.0.into(),
                width: 2.0.into(),
                height: (shared.height.value_or(26.0) - 10.0).into(),
                ..Default::default()
            },
        }
    }
}

/// The Woodpecker UI Button
#[derive(Component, Reflect, Default, PartialEq, Widget, Clone)]
#[auto_update(render)]
#[props(TextBox, TextboxStyles)]
#[state(TextBoxState)]
#[require(WidgetRender = WidgetRender::Quad, WidgetChildren, WoodpeckerStyle, TextboxStyles, Pickable, Focusable)]
pub struct TextBox {
    /// An initial value
    pub initial_value: String,
}

/// The textbox state
#[derive(Component, Debug, PartialEq, Clone)]
pub struct TextBoxState {
    // Mouse state
    /// Is hovering?
    pub hovering: bool,
    /// Is Focused
    pub focused: bool,
    // Keyboard input state
    /// A list of current graphemes.
    pub graphemes: Vec<String>,
    /// The position of the cursor in pixels
    pub cursor_x: f32,
    /// The position of the cursor as a grapheme index.
    pub cursor_position: usize,
    /// Visibility state
    pub cursor_visible: bool,
    /// A last updated timer, used to blink the cursor
    pub cursor_last_update: Instant,
    /// The current text value of the textbox.
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

        set_new_cursor_position(
            &mut state,
            &mut font_manager,
            &styles
                .normal
                .font
                .map(Handle::Weak)
                .unwrap_or(default_font.0.clone_weak()),
            styles.normal.font_size,
        );
    }

    if state.focused {
        *style = styles.focused
    } else if state.hovering {
        *style = styles.hovered
    } else {
        *style = styles.normal
    }

    let cursor_styles = WoodpeckerStyle {
        left: state.cursor_x.into(),
        ..styles.cursor
    };

    let current_widget = *current_widget;
    *children = WidgetChildren::default()
        .with_observe(
            current_widget,
            move |trigger: Trigger<WidgetKeyboardCharEvent>,
                  mut commands: Commands,
                  style_query: Query<&WoodpeckerStyle>,
                  default_font: Res<DefaultFont>,
                  mut font_manager: ResMut<FontManager>,
                  mut state_query: Query<&mut TextBoxState>| {
                let Ok(styles) = style_query.get(trigger.target) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let cursor_pos = state.cursor_position;

                let char_pos: usize = state.graphemes[0..cursor_pos].iter().map(|g| g.len()).sum();
                state.current_value.insert_str(char_pos, &trigger.c);
                state.cursor_position += 1;

                commands.trigger_targets(
                    Change {
                        target: *current_widget,
                        data: TextChanged {
                            value: state.current_value.clone(),
                        },
                    },
                    *current_widget,
                );

                // Update graphemes
                set_graphemes(&mut state);

                set_new_cursor_position(
                    &mut state,
                    &mut font_manager,
                    &styles
                        .font
                        .map(Handle::Weak)
                        .unwrap_or(default_font.0.clone_weak()),
                    styles.font_size,
                );
            },
        )
        .with_observe(
            current_widget,
            move |_trigger: Trigger<Pointer<Over>>, mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if !state.focused {
                    state.hovering = true;
                }
            },
        )
        .with_observe(
            current_widget,
            move |_trigger: Trigger<Pointer<Out>>, mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if !state.focused {
                    state.hovering = false;
                }
            },
        )
        .with_observe(
            current_widget,
            move |_trigger: Trigger<WidgetFocus>, mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
                state.focused = true;
            },
        )
        .with_observe(
            current_widget,
            move |_trigger: Trigger<WidgetBlur>, mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
                state.focused = false;
            },
        )
        .with_observe(
            current_widget,
            move |trigger: Trigger<WidgetPasteEvent>,
                  mut commands: Commands,
                  style_query: Query<&WoodpeckerStyle>,
                  mut state_query: Query<&mut TextBoxState>,
                  default_font: Res<DefaultFont>,
                  mut font_manager: ResMut<FontManager>| {
                let Ok(styles) = style_query.get(trigger.target) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                let cursor_pos = state.cursor_position;

                let char_pos: usize = state.graphemes[0..cursor_pos].iter().map(|g| g.len()).sum();
                state.current_value.insert_str(char_pos, &trigger.paste);
                state.cursor_position += trigger.paste.len();

                commands.trigger_targets(
                    Change {
                        target: *current_widget,
                        data: TextChanged {
                            value: state.current_value.clone(),
                        },
                    },
                    *current_widget,
                );

                // Update graphemes
                set_graphemes(&mut state);

                set_new_cursor_position(
                    &mut state,
                    &mut font_manager,
                    &styles
                        .font
                        .map(Handle::Weak)
                        .unwrap_or(default_font.0.clone_weak()),
                    styles.font_size,
                );
            },
        )
        .with_observe(
            current_widget,
            move |trigger: Trigger<WidgetKeyboardButtonEvent>,
                  mut commands: Commands,
                  style_query: Query<&WoodpeckerStyle>,
                  mut state_query: Query<&mut TextBoxState>,
                  default_font: Res<DefaultFont>,
                  mut font_manager: ResMut<FontManager>,
                  keyboard_input: Res<ButtonInput<KeyCode>>| {
                if trigger.code == KeyCode::ArrowRight {
                    let Ok(styles) = style_query.get(trigger.target) else {
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
                    set_new_cursor_position(
                        &mut state,
                        &mut font_manager,
                        &styles
                            .font
                            .map(Handle::Weak)
                            .unwrap_or(default_font.0.clone_weak()),
                        styles.font_size,
                    );
                }
                if trigger.code == KeyCode::ArrowLeft {
                    let Ok(styles) = style_query.get(trigger.target) else {
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

                    set_new_cursor_position(
                        &mut state,
                        &mut font_manager,
                        &styles
                            .font
                            .map(Handle::Weak)
                            .unwrap_or(default_font.0.clone_weak()),
                        styles.font_size,
                    );
                }
                if trigger.code == KeyCode::Backspace {
                    let Ok(styles) = style_query.get(trigger.target) else {
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

                        commands.trigger_targets(
                            Change {
                                target: *current_widget,
                                data: TextChanged {
                                    value: state.current_value.clone(),
                                },
                            },
                            *current_widget,
                        );

                        // Update graphemes
                        set_graphemes(&mut state);

                        set_new_cursor_position(
                            &mut state,
                            &mut font_manager,
                            &styles
                                .font
                                .map(Handle::Weak)
                                .unwrap_or(default_font.0.clone_weak()),
                            styles.font_size,
                        );
                    }
                }
                if trigger.code == KeyCode::Delete {
                    let Ok(styles) = style_query.get(trigger.target) else {
                        return;
                    };
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    let cursor_pos = state.cursor_position;

                    if !state.current_value.is_empty() && cursor_pos != 0 {
                        let char_pos: usize =
                            state.graphemes[0..cursor_pos].iter().map(|g| g.len()).sum();
                        state.current_value.remove(char_pos);

                        commands.trigger_targets(
                            Change {
                                target: *current_widget,
                                data: TextChanged {
                                    value: state.current_value.clone(),
                                },
                            },
                            *current_widget,
                        );

                        // Update graphemes
                        set_graphemes(&mut state);

                        set_new_cursor_position(
                            &mut state,
                            &mut font_manager,
                            &styles
                                .font
                                .map(Handle::Weak)
                                .unwrap_or(default_font.0.clone_weak()),
                            styles.font_size,
                        );
                    }
                }
            },
        );

    let mut clip_children = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: style.font_size,
            color: style.color,
            ..Default::default()
        },
        WidgetRender::Text {
            content: state.current_value.clone(),
            word_wrap: false,
        },
    ));

    if state.cursor_visible {
        clip_children.add::<Element>((Element, cursor_styles, WidgetRender::Quad));
    }

    children.add::<Clip>((
        Clip,
        WoodpeckerStyle {
            width: style.width,
            height: style.height,
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        clip_children,
    ));

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
            Vec2::new(1.0, 1.0),
        )
        .unwrap();

    let mut max_x: f32 = 0.0;

    for run in buffer.layout_runs() {
        max_x = max_x.max(run.line_w);
    }

    state.cursor_x = max_x;
}

// IMPORTANT: When modifying widget entities we need to verify we aren't modifying previous widget values.
pub fn cursor_animation_system(
    mut state_query: ParamSet<(
        Query<(Entity, &TextBoxState), Without<PreviousWidget>>,
        Query<&mut TextBoxState, Without<PreviousWidget>>,
    )>,
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
