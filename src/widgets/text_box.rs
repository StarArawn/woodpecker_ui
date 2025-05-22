use std::sync::Arc;

use bevy_vello::vello::{
    kurbo::{Affine, Vec2},
    peniko::Brush,
};
use parley::{FontFamily, StyleProperty};
use web_time::Instant;

use crate::{
    keyboard_input::{WidgetKeyboardButtonEvent, WidgetPasteEvent},
    prelude::*,
    DefaultFont,
};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};

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
    /// Indicates this is a multi-line text editor.
    pub multi_line: bool,
    /// Optional text highlighting used for syntax highlighting or other
    /// text coloring.
    #[reflect(ignore)]
    pub text_highlighting: ApplyHighlighting,
}

/// Applies color highlighting to the text.
#[derive(Clone)]
pub struct ApplyHighlighting {
    inner: Arc<dyn Fn(&str) -> Option<Highlighted> + Send + Sync + 'static>,
}

impl ApplyHighlighting {
    /// Creates a new color highlighting applier.
    pub fn new(f: impl Fn(&str) -> Option<Highlighted> + Send + Sync + 'static) -> Self {
        Self { inner: Arc::new(f) }
    }
}

impl PartialEq for ApplyHighlighting {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Default for ApplyHighlighting {
    fn default() -> Self {
        Self {
            inner: Arc::new(|_| None),
        }
    }
}

/// The textbox state
#[derive(Component, Clone)]
pub struct TextBoxState {
    // Mouse state
    /// Is hovering?
    pub hovering: bool,
    /// Is Focused
    pub focused: bool,
    // Keyboard input state
    /// Cursor position.
    pub cursor: parley::Rect,
    /// Selections
    pub selections: Vec<(parley::Rect, usize)>,
    /// Visibility state
    pub cursor_visible: bool,
    /// A last updated timer, used to blink the cursor
    pub cursor_last_update: Instant,
    /// The current text value of the textbox.
    pub current_value: String,
    /// The initial text value of the textbox.
    pub initial_value: String,
    /// Parley text editing engine.
    pub engine: parley::PlainEditor<Brush>,
    /// Indicates this is a multi-line text editor.
    pub multi_line: bool,
}

impl PartialEq for TextBoxState {
    fn eq(&self, other: &Self) -> bool {
        self.hovering == other.hovering
            && self.focused == other.focused
            && self.cursor == other.cursor
            && self.selections == other.selections
            && self.cursor_visible == other.cursor_visible
            && self.current_value == other.current_value
    }
}

impl Default for TextBoxState {
    fn default() -> Self {
        Self {
            hovering: Default::default(),
            focused: Default::default(),
            selections: vec![],
            cursor: parley::Rect::default(),
            cursor_visible: Default::default(),
            cursor_last_update: Instant::now(),
            current_value: String::new(),
            initial_value: String::new(),
            engine: parley::PlainEditor::new(0.0),
            multi_line: false,
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

    let mut default_engine = parley::PlainEditor::new(styles.normal.font_size);
    default_engine.set_text(&text_box.initial_value);
    let text_styles = default_engine.edit_styles();
    text_styles.insert(StyleProperty::LineHeight(
        styles
            .normal
            .line_height
            .map(|lh| styles.normal.font_size / lh)
            .unwrap_or(1.2),
    ));
    text_styles.insert(StyleProperty::FontStack(parley::FontStack::Single(
        FontFamily::Named(
            font_manager
                .get_family(styles.normal.font.as_ref().unwrap_or(&default_font.0.id()))
                .into(),
        ),
    )));

    let state_entity = hook_helper.use_state(
        &mut commands,
        *current_widget,
        TextBoxState {
            initial_value: text_box.initial_value.clone(),
            current_value: text_box.initial_value.clone(),
            engine: default_engine,
            multi_line: text_box.multi_line,
            ..Default::default()
        },
    );

    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    if text_box.initial_value != state.initial_value {
        state.initial_value = text_box.initial_value.clone();
        state.current_value.clone_from(&text_box.initial_value);
        state.engine.set_text(&text_box.initial_value);
        let mut driver = font_manager.driver(&mut state.engine);
        driver.move_to_line_end();
        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.normal.font_size)
            .unwrap_or_default();
    }

    if state.focused {
        *style = WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: if text_box.multi_line {
                Units::Percentage(100.0)
            } else {
                style.height
            },
            ..styles.focused
        };
    } else if state.hovering {
        *style = WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: if text_box.multi_line {
                Units::Percentage(100.0)
            } else {
                style.height
            },
            ..styles.hovered
        };
    } else {
        *style = WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: if text_box.multi_line {
                Units::Percentage(100.0)
            } else {
                style.height
            },
            ..styles.normal
        };
    }

    let cursor_styles = WoodpeckerStyle {
        top: ((state.cursor.min_y() + 2.0) as f32).into(),
        left: (state.cursor.min_x() as f32).into(),
        ..styles.cursor
    };

    let current_widget = *current_widget;
    *children = WidgetChildren::default()
        .with_observe(
            current_widget,
            move |trigger: Trigger<WidgetKeyboardCharEvent>,
                  mut commands: Commands,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  mut font_manager: ResMut<FontManager>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut state_query: Query<&mut TextBoxState>| {
                let Ok(styles) = style_query.get(trigger.target) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                // Ignore for copy/paste.
                if keyboard_input.pressed(KeyCode::SuperLeft)
                    || keyboard_input.pressed(KeyCode::ControlLeft)
                {
                    return;
                }

                let mut driver = font_manager.driver(&mut state.engine);
                driver.insert_or_replace_selection(&trigger.c);

                state.cursor = state
                    .engine
                    .cursor_geometry(styles.font_size)
                    .unwrap_or_default();
                state.selections = state.engine.selection_geometry();
                state.current_value = state.engine.text().to_string();

                commands.trigger_targets(
                    Change {
                        target: *current_widget,
                        data: TextChanged {
                            value: state.current_value.clone(),
                        },
                    },
                    *current_widget,
                );
            },
        )
        .with_observe(
            current_widget,
            move |trigger: Trigger<Pointer<Pressed>>,
                  mouse_input: Res<ButtonInput<MouseButton>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut font_manager: ResMut<FontManager>,
                  widget_layout: Query<&WidgetLayout>,
                  mut state_query: Query<&mut TextBoxState>| {
                let Ok(styles) = style_query.get(trigger.target) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(widget_layout) = widget_layout.get(trigger.target) else {
                    return;
                };

                if !state.focused {
                    return;
                }

                if !mouse_input.just_pressed(MouseButton::Left) {
                    return;
                }

                let mut driver = font_manager.driver(&mut state.engine);

                if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    driver.extend_selection_to_point(
                        trigger.pointer_location.position.x
                            - widget_layout.location.x
                            - widget_layout.padding.left.value_or(0.0),
                        trigger.pointer_location.position.y
                            - widget_layout.location.y
                            - widget_layout.padding.top.value_or(0.0),
                    );
                } else {
                    driver.move_to_point(
                        trigger.pointer_location.position.x
                            - widget_layout.location.x
                            - widget_layout.padding.left.value_or(0.0),
                        trigger.pointer_location.position.y
                            - widget_layout.location.y
                            - widget_layout.padding.top.value_or(0.0),
                    );
                }

                state.selections = state.engine.selection_geometry();

                state.cursor = state
                    .engine
                    .cursor_geometry(styles.font_size)
                    .unwrap_or_default();
            },
        )
        .with_observe(
            current_widget,
            move |trigger: Trigger<Pointer<DragStart>>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut font_manager: ResMut<FontManager>,
                  widget_layout: Query<&WidgetLayout>,
                  mut state_query: Query<&mut TextBoxState>| {
                let Ok(styles) = style_query.get(trigger.target) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(widget_layout) = widget_layout.get(trigger.target) else {
                    return;
                };

                if !state.focused {
                    return;
                }
                let mut driver = font_manager.driver(&mut state.engine);
                driver.move_to_point(
                    trigger.pointer_location.position.x
                        - widget_layout.location.x
                        - widget_layout.padding.left.value_or(0.0),
                    trigger.pointer_location.position.y
                        - widget_layout.location.y
                        - widget_layout.padding.top.value_or(0.0),
                );
                state.cursor = state
                    .engine
                    .cursor_geometry(styles.font_size)
                    .unwrap_or_default();
                state.selections = state.engine.selection_geometry();
            },
        )
        .with_observe(
            current_widget,
            move |trigger: Trigger<Pointer<Drag>>,
                  _style_query: Query<&WoodpeckerStyle>,
                  mut font_manager: ResMut<FontManager>,
                  widget_layout: Query<&WidgetLayout>,
                  mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(widget_layout) = widget_layout.get(trigger.target) else {
                    return;
                };

                if !state.focused {
                    return;
                }
                let mut driver = font_manager.driver(&mut state.engine);
                driver.extend_selection_to_point(
                    trigger.pointer_location.position.x
                        - widget_layout.location.x
                        - widget_layout.padding.left.value_or(0.0),
                    trigger.pointer_location.position.y
                        - widget_layout.location.y
                        - widget_layout.padding.top.value_or(0.0),
                );

                state.selections = state.engine.selection_geometry();
            },
        )
        .with_observe(
            current_widget,
            move |_trigger: Trigger<Pointer<Over>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut TextBoxState>,
                  camera_query: Query<Entity, With<PrimaryWindow>>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if !state.focused {
                    state.hovering = true;
                }

                commands
                    .entity(camera_query.single().unwrap())
                    .insert(CursorIcon::from(SystemCursorIcon::Text));
            },
        )
        .with_observe(
            current_widget,
            move |_trigger: Trigger<Pointer<Out>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut TextBoxState>,
                  camera_query: Query<Entity, With<PrimaryWindow>>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if !state.focused {
                    state.hovering = false;
                }

                commands
                    .entity(camera_query.single().unwrap())
                    .insert(CursorIcon::from(SystemCursorIcon::Default));
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
                  mut font_manager: ResMut<FontManager>| {
                let Ok(styles) = style_query.get(trigger.target) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                let mut driver = font_manager.driver(&mut state.engine);
                driver.insert_or_replace_selection(&trigger.paste.to_string());

                state.cursor = state
                    .engine
                    .cursor_geometry(styles.font_size)
                    .unwrap_or_default();

                state.current_value = state.engine.text().to_string();

                commands.trigger_targets(
                    Change {
                        target: *current_widget,
                        data: TextChanged {
                            value: state.current_value.clone(),
                        },
                    },
                    *current_widget,
                );
            },
        )
        .with_observe(
            current_widget,
            move |trigger: Trigger<WidgetKeyboardButtonEvent>,
                  commands: Commands,
                  style_query: Query<&WoodpeckerStyle>,
                  state_query: Query<&mut TextBoxState>,
                  font_manager: ResMut<FontManager>,
                  keyboard_input: Res<ButtonInput<KeyCode>>| {
                textbox_handle_keyboard_events(
                    trigger,
                    commands,
                    style_query,
                    state_query,
                    font_manager,
                    keyboard_input,
                    state_entity,
                );
            },
        );

    let mut clip_children = WidgetChildren::default();

    if !state.selections.is_empty() {
        let selections = state.selections.clone();
        clip_children.add::<Element>((
            Element,
            styles.cursor,
            WidgetRender::Custom {
                render: WidgetRenderCustom::new(move |scene, widget_layout, styles, scale| {
                    let transform = Affine::default().with_translation(Vec2::new(
                        (widget_layout.location.x * scale) as f64,
                        ((widget_layout.location.y - 5.0) * scale) as f64,
                    ));
                    let color = styles.background_color.to_srgba();
                    for selection in selections.iter() {
                        scene.fill(
                            vello::peniko::Fill::NonZero,
                            transform,
                            &Brush::Solid(vello::peniko::Color::new([
                                color.red,
                                color.green,
                                color.blue,
                                color.alpha,
                            ])),
                            None,
                            &selection.0,
                        );
                    }
                }),
            },
        ));
    }

    clip_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: style.font_size,
            color: style.color,
            text_wrap: if text_box.multi_line {
                TextWrap::WordOrGlyph
            } else {
                TextWrap::None
            },
            ..Default::default()
        },
        if let Some(text_highlight) = (text_box.text_highlighting.inner)(&state.current_value) {
            WidgetRender::RichText {
                content: RichText::from_hightlighted(&state.current_value, text_highlight),
            }
        } else {
            WidgetRender::Text {
                content: state.current_value.clone(),
            }
        },
    ));

    if state.cursor_visible {
        clip_children.add::<Element>((Element, cursor_styles, WidgetRender::Quad));
    }

    children.add::<Clip>((
        Clip,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            ..Default::default()
        },
        clip_children,
    ));

    children.apply(current_widget.as_parent());
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

pub fn textbox_handle_keyboard_events(
    trigger: Trigger<WidgetKeyboardButtonEvent>,
    mut commands: Commands,
    style_query: Query<&WoodpeckerStyle>,
    mut state_query: Query<&mut TextBoxState>,
    mut font_manager: ResMut<FontManager>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state_entity: Entity,
) {
    if trigger.code == KeyCode::Enter {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };
        let mut driver = font_manager.driver(&mut state.engine);
        driver.insert_or_replace_selection("\n");
        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
        state.current_value = state.engine.text().to_string();
        commands.trigger_targets(
            Change {
                target: trigger.target,
                data: TextChanged {
                    value: state.current_value.clone(),
                },
            },
            trigger.target,
        );
    }

    if trigger.code == KeyCode::ArrowDown {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };
        let mut driver = font_manager.driver(&mut state.engine);
        let shift = keyboard_input.pressed(KeyCode::ShiftLeft);

        if shift {
            driver.select_down();
        } else {
            driver.move_down();
        }

        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
    }
    if trigger.code == KeyCode::ArrowUp {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };

        let shift = keyboard_input.pressed(KeyCode::ShiftLeft);

        let mut driver = font_manager.driver(&mut state.engine);
        if shift {
            driver.select_up();
        } else {
            driver.move_up();
        }
        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
    }

    if trigger.code == KeyCode::ArrowRight {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };
        let mut driver = font_manager.driver(&mut state.engine);
        let shift = keyboard_input.pressed(KeyCode::ShiftLeft);

        if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::AltLeft)
        {
            if shift {
                driver.select_word_right();
            } else {
                driver.move_word_right();
            }
        } else if keyboard_input.pressed(KeyCode::SuperLeft) {
            if shift {
                driver.select_to_line_end();
            } else {
                driver.move_to_line_end();
            }
        } else if shift {
            driver.select_left();
        } else {
            driver.move_right();
        }
        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
    }
    if trigger.code == KeyCode::ArrowLeft {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };

        let shift = keyboard_input.pressed(KeyCode::ShiftLeft);

        let mut driver = font_manager.driver(&mut state.engine);
        if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::AltLeft)
        {
            if shift {
                driver.select_word_left();
            } else {
                driver.move_word_left();
            }
        } else if keyboard_input.pressed(KeyCode::SuperLeft) {
            if shift {
                driver.select_to_line_start();
            } else {
                driver.move_to_line_start();
            }
        } else if shift {
            driver.select_left();
        } else {
            driver.move_left();
        }
        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
    }
    if trigger.code == KeyCode::Backspace {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };
        let mut driver = font_manager.driver(&mut state.engine);
        driver.backdelete();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
        state.selections = state.engine.selection_geometry();
        state.current_value = state.engine.text().to_string();
        commands.trigger_targets(
            Change {
                target: trigger.target,
                data: TextChanged {
                    value: state.current_value.clone(),
                },
            },
            trigger.target,
        );
    }
    if (keyboard_input.pressed(KeyCode::SuperLeft) || keyboard_input.pressed(KeyCode::ControlLeft))
        && keyboard_input.just_pressed(KeyCode::KeyC)
    {
        let Ok(state) = state_query.get_mut(state_entity) else {
            return;
        };
        if let Some(text) = state.engine.selected_text() {
            #[cfg(not(target_arch = "wasm32"))]
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                match clipboard.set_text(text) {
                    Ok(_) => {}
                    Err(err) => error!("{err}"),
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let Some(clipboard) =
                    web_sys::window().and_then(|window| Some(window.navigator().clipboard()))
                else {
                    warn!("no clipboard");
                    return;
                };
                let promise = clipboard.write_text(text);
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
            }
        }
    }
    if trigger.code == KeyCode::Delete {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };

        if !state.current_value.is_empty() {
            let mut driver = font_manager.driver(&mut state.engine);
            driver.delete();
            state.cursor = state
                .engine
                .cursor_geometry(styles.font_size)
                .unwrap_or_default();
            state.selections = state.engine.selection_geometry();
            state.current_value = state.engine.text().to_string();
            commands.trigger_targets(
                Change {
                    target: trigger.target,
                    data: TextChanged {
                        value: state.current_value.clone(),
                    },
                },
                trigger.target,
            );
        }
    }
}
