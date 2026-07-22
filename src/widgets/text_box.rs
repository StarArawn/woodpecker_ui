use std::sync::Arc;

use bevy_vello::vello::{
    kurbo::{Affine, Rect, Vec2},
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
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};

use super::{Clip, Element};

/// A textbox change event.
#[derive(Debug, Clone, Reflect)]
pub struct TextChanged {
    /// The current text value
    pub value: String,
}

/// A collection of textbox styles.
#[derive(Component, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TextboxStyles {
    /// Normal styles
    pub normal: WoodpeckerStyle,
    /// Hovered styles
    pub hovered: WoodpeckerStyle,
    /// Focused styles
    pub focused: WoodpeckerStyle,
    /// Cursor styles -- `top`/`height` are ignored (`render()` computes both dynamically
    /// every frame from the live cursor/layout geometry, matching the actual text and
    /// textbox size); set `background_color`/`width`/`position` here.
    pub cursor: WoodpeckerStyle,
}

impl Default for TextboxStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TextboxStyles {
    fn from_theme(theme: &Theme) -> Self {
        let shared = WoodpeckerStyle {
            background_color: theme.dark_background,
            color: theme.text,
            width: Units::Percentage(100.0),
            height: theme.control_height.into(),
            border_color: theme.border,
            border: Edge::all(1.0),
            border_radius: Corner::all(theme.control_radius),
            padding: Edge::new(0.0, 10.0, 0.0, 10.0),
            font_size: theme.font_size,
            ..Default::default()
        };
        Self {
            normal: WoodpeckerStyle { ..shared },
            hovered: WoodpeckerStyle { ..shared },
            focused: WoodpeckerStyle {
                border_color: theme.primary,
                ..shared
            },
            cursor: WoodpeckerStyle {
                background_color: theme.primary,
                position: WidgetPosition::Absolute,
                width: 2.0.into(),
                ..Default::default()
            },
        }
    }
}

/// The tab behavior. Defaults to 4 spaces.
#[derive(Reflect, PartialEq, Clone, Copy)]
pub enum TabMode {
    /// Tab characters :(
    Tab,
    /// Space characters :D
    /// u8 value is how many spaces.
    /// defaults to 4.
    Space(u8),
}

impl Default for TabMode {
    fn default() -> Self {
        Self::Space(4)
    }
}

/// The Woodpecker UI Button
#[derive(Component, Reflect, Default, PartialEq, Widget, Clone)]
#[reflect(Component, DiffableProp, PartialEq, Clone)]
#[auto_update(render)]
#[require(WidgetRender = WidgetRender::Quad, WidgetChildren, WoodpeckerStyle, TextboxStyles, Pickable, Focusable, WatchLayout)]
pub struct TextBox {
    /// An initial value
    pub initial_value: String,
    /// Indicates this is a multi-line text editor.
    pub multi_line: bool,
    /// Optional text highlighting used for syntax highlighting or other
    /// text coloring.
    #[reflect(ignore)]
    pub text_highlighting: ApplyHighlighting,
    /// The tab behavior. Defaults to 4 spaces.
    pub tab_mode: TabMode,
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
    pub cursor: parley::BoundingBox,
    /// Selections
    pub selections: Vec<(parley::BoundingBox, usize)>,
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

// TODO: Remove once Parley is updated.
unsafe impl Send for TextBoxState {}
unsafe impl Sync for TextBoxState {}

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
            cursor: parley::BoundingBox::default(),
            cursor_visible: Default::default(),
            cursor_last_update: Instant::now(),
            current_value: String::new(),
            initial_value: String::new(),
            engine: parley::PlainEditor::new(0.0),
            multi_line: false,
        }
    }
}

/// `TextBoxState` holds a `parley::PlainEditor`, which can't derive `Reflect`, so it can
/// never be a generic [`crate::diffable_prop::DiffableProp`] -- this mirrors the fields its
/// hand-rolled `PartialEq` compares, snapshotted separately so `crate::diffing` can still
/// detect state changes.
#[derive(Component, Default, PartialEq, Clone)]
pub(crate) struct PreviousTextBoxStateSnapshot {
    hovering: bool,
    focused: bool,
    cursor: parley::BoundingBox,
    selections: Vec<(parley::BoundingBox, usize)>,
    cursor_visible: bool,
    current_value: String,
}

impl From<&TextBoxState> for PreviousTextBoxStateSnapshot {
    fn from(state: &TextBoxState) -> Self {
        Self {
            hovering: state.hovering,
            focused: state.focused,
            cursor: state.cursor,
            selections: state.selections.clone(),
            cursor_visible: state.cursor_visible,
            current_value: state.current_value.clone(),
        }
    }
}

pub(crate) fn diff_text_box_state(world: &mut World, entity: Entity) -> bool {
    let Some(hook_helper) = world.get_resource::<HookHelper>() else {
        return false;
    };
    let Some(state_entity) = hook_helper.get_state::<TextBoxState>(CurrentWidget(entity)) else {
        return false;
    };
    let Some(state) = world.get::<TextBoxState>(state_entity) else {
        return false;
    };
    let snapshot = PreviousTextBoxStateSnapshot::from(state);

    let changed = match world.get::<PreviousTextBoxStateSnapshot>(entity) {
        Some(previous) => *previous != snapshot,
        None => true,
    };

    if changed {
        world.entity_mut(entity).insert(snapshot);
    }

    changed
}

pub fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hook_helper: ResMut<HookHelper>,
    font_manager: Res<FontManager>,
    default_font: Res<DefaultFont>,
    mut query: Query<(
        Ref<TextBox>,
        &mut WoodpeckerStyle,
        &TextboxStyles,
        &mut WidgetChildren,
    )>,
    widget_layout: Query<&WidgetLayout>,
    mut state_query: Query<&mut TextBoxState>,
) {
    let Ok((text_box, mut style, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let tab_mode = text_box.tab_mode;

    let alignment = match styles.normal.text_alignment.unwrap_or(TextAlign::Left) {
        TextAlign::Left => parley::Alignment::Left,
        TextAlign::Right => parley::Alignment::Right,
        TextAlign::Center => parley::Alignment::Center,
        TextAlign::Justified => parley::Alignment::Justify,
        TextAlign::End => parley::Alignment::End,
    };

    let mut default_engine = parley::PlainEditor::new(styles.normal.font_size);
    default_engine.set_text(&text_box.initial_value);
    default_engine.set_alignment(alignment);
    let text_styles = default_engine.edit_styles();
    text_styles.insert(StyleProperty::LineHeight(
        parley::LineHeight::FontSizeRelative(
            styles
                .normal
                .line_height
                .map(|lh| styles.normal.font_size / lh)
                .unwrap_or(1.2),
        ),
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

    // Falls back to `font_size` before the first layout pass -- irrelevant in practice, since
    // the cursor this seeds `cursor_styles` for is only ever rendered while focused, by which
    // point at least one layout has already run.
    let mut widget_height = styles.normal.font_size;
    if let Ok(layout) = widget_layout.get(current_widget.entity()) {
        state.engine.set_width(Some(layout.size.x));
        widget_height = layout.size.y;
    }
    state.engine.set_alignment(alignment);

    if text_box.initial_value != state.initial_value {
        state.initial_value = text_box.initial_value.clone();
        state.current_value.clone_from(&text_box.initial_value);
        state.engine.set_text(&text_box.initial_value);

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
                styles.focused.height
            },
            ..styles.focused
        };
    } else if state.hovering {
        *style = WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: if text_box.multi_line {
                Units::Percentage(100.0)
            } else {
                styles.hovered.height
            },
            ..styles.hovered
        };
    } else {
        *style = WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: if text_box.multi_line {
                Units::Percentage(100.0)
            } else {
                styles.normal.height
            },
            ..styles.normal
        };
    }

    // The cursor's own height comes from parley's cursor geometry (derived from the actual
    // font/line-height), not a caller-provided style value -- so it's always right for
    // whatever font size a `TextboxStyles` sets, not just whatever value a caller happened to
    // pre-compute `styles.cursor.height` from. The vertical-centering offset for single-line
    // boxes is measured against `widget_height`, the box's *actual resolved* layout height
    // (works for any `height` unit -- `Pixels`, `Percentage`, `Auto`), not a static guess.
    let cursor_height = state.cursor.height() as f32;
    let cursor_styles = WoodpeckerStyle {
        top: (state.cursor.y0 as f32
            + if text_box.multi_line {
                2.0
            } else {
                (widget_height - cursor_height) / 2.0
            })
        .into(),
        left: (state.cursor.x0 as f32).into(),
        height: cursor_height.into(),
        ..styles.cursor
    };

    let current_widget = *current_widget;
    *children = WidgetChildren::default()
        .with_observe(
            current_widget,
            move |trigger: On<WidgetKeyboardCharEvent>,
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

                // Keyboard events are routed to whatever entity the *global* `CurrentFocus`
                // resource points at, independent of this box's own `state.focused` and of
                // the Alt guards on Press/Over/DragStart/Drag below -- a `Focusable` widget
                // can still be focused by an Alt-held click (the global focus system doesn't
                // check Alt), so blocking edits has to happen here too, not just at the
                // mouse-driven entry points.
                if keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight)
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

                commands.trigger(Change {
                    target: *current_widget,
                    data: TextChanged {
                        value: state.current_value.clone(),
                    },
                });
            },
        )
        .with_observe(
            current_widget,
            move |trigger: On<Pointer<Press>>,
                  mouse_input: Res<ButtonInput<MouseButton>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut font_manager: ResMut<FontManager>,
                  widget_layout: Query<&WidgetLayout>,
                  pointer_world: PointerWorldPosition,
                  mut state_query: Query<&mut TextBoxState>| {
                let Ok(styles) = style_query.get(trigger.entity) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(widget_layout) = widget_layout.get(trigger.entity) else {
                    return;
                };

                if !mouse_input.just_pressed(MouseButton::Left) {
                    return;
                }

                // Lets a caller layer its own Alt-drag interaction (e.g. `NumberInput`'s
                // scrub-to-adjust) directly onto a `TextBox` without it also moving the
                // cursor/selection underneath -- both would otherwise fire for the same
                // gesture, and a value change mid-drag forces `TextBox`'s own `set_text`
                // reset, which doesn't know to carry forward a selection endpoint this
                // handler had just set from mouse coordinates that may be far outside the
                // reformatted text's new bounds.
                if keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight)
                {
                    return;
                }

                let mut driver = font_manager.driver(&mut state.engine);

                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };

                if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    driver.extend_selection_to_point(
                        cursor_pos_world.x
                            - widget_layout.location.x
                            - widget_layout.padding.left.value_or(0.0),
                        cursor_pos_world.y
                            - widget_layout.location.y
                            - widget_layout.padding.top.value_or(0.0),
                    );
                } else {
                    driver.move_to_point(
                        cursor_pos_world.x
                            - widget_layout.location.x
                            - widget_layout.padding.left.value_or(0.0),
                        cursor_pos_world.y
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
            move |trigger: On<Pointer<DragStart>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut font_manager: ResMut<FontManager>,
                  widget_layout: Query<&WidgetLayout>,
                  pointer_world: PointerWorldPosition,
                  mut state_query: Query<&mut TextBoxState>| {
                // See the matching check in the `Pointer<Press>` handler above.
                if keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight)
                {
                    return;
                }

                let Ok(styles) = style_query.get(trigger.entity) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(widget_layout) = widget_layout.get(trigger.entity) else {
                    return;
                };

                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };
                let mut driver = font_manager.driver(&mut state.engine);

                let start_point = bevy::prelude::Vec2::new(
                    cursor_pos_world.x
                        - widget_layout.location.x
                        - widget_layout.padding.left.value_or(0.0),
                    cursor_pos_world.y
                        - widget_layout.location.y
                        - widget_layout.padding.top.value_or(0.0),
                );
                driver.move_to_point(start_point.x, start_point.y);
                state.cursor = state
                    .engine
                    .cursor_geometry(styles.font_size)
                    .unwrap_or_default();
                state.selections = state.engine.selection_geometry();
            },
        )
        .with_observe(
            current_widget,
            move |trigger: On<Pointer<Drag>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut font_manager: ResMut<FontManager>,
                  widget_layout: Query<&WidgetLayout>,
                  pointer_world: PointerWorldPosition,
                  mut state_query: Query<&mut TextBoxState>| {
                // See the matching check in the `Pointer<Press>` handler above.
                if keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight)
                {
                    return;
                }

                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(widget_layout) = widget_layout.get(trigger.entity) else {
                    return;
                };
                let Ok(styles) = style_query.get(trigger.entity) else {
                    return;
                };

                let mut driver = font_manager.driver(&mut state.engine);

                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };

                let final_point = bevy::prelude::Vec2::new(
                    cursor_pos_world.x
                        - widget_layout.location.x
                        - widget_layout.padding.left.value_or(0.0),
                    cursor_pos_world.y
                        - widget_layout.location.y
                        - widget_layout.padding.top.value_or(0.0),
                );

                driver.extend_selection_to_point(final_point.x, final_point.y);
                state.cursor = state
                    .engine
                    .cursor_geometry(styles.font_size)
                    .unwrap_or_default();
                state.selections = state.engine.selection_geometry();
            },
        )
        .with_observe(
            current_widget,
            move |_trigger: On<Pointer<Over>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut TextBoxState>,
                  camera_query: Query<Entity, With<PrimaryWindow>>| {
                // While Alt is held, stay visually "not editable" -- no hover highlight, no
                // text-cursor icon -- rather than implying a click here would let you type.
                // Matches the same guard on Press/DragStart/Drag, and lets a caller layering
                // its own Alt interaction (e.g. `NumberInput`'s scrub-to-adjust) own the
                // cursor icon instead via its own `Pointer<Over>` on this same entity.
                if keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight)
                {
                    return;
                }

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
            move |_trigger: On<Pointer<Out>>,
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
            move |_trigger: On<WidgetFocus>, mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
                state.focused = true;
            },
        )
        .with_observe(
            current_widget,
            move |trigger: On<WidgetBlur>,
                  style_query: Query<&WoodpeckerStyle>,
                  mut font_manager: ResMut<FontManager>,
                  mut state_query: Query<&mut TextBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(styles) = style_query.get(trigger.target) else {
                    return;
                };

                state.hovering = false;
                state.focused = false;

                let mut driver = font_manager.driver(&mut state.engine);
                driver.move_to_text_start();
                state.cursor = state
                    .engine
                    .cursor_geometry(styles.font_size)
                    .unwrap_or_default();
                state.selections = state.engine.selection_geometry();
            },
        )
        .with_observe(
            current_widget,
            move |trigger: On<WidgetPasteEvent>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  mut commands: Commands,
                  style_query: Query<&WoodpeckerStyle>,
                  mut state_query: Query<&mut TextBoxState>,
                  mut font_manager: ResMut<FontManager>| {
                // See the matching check in the `WidgetKeyboardCharEvent` handler above.
                if keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight)
                {
                    return;
                }

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

                commands.trigger(Change {
                    target: *current_widget,
                    data: TextChanged {
                        value: state.current_value.clone(),
                    },
                });
            },
        )
        .with_observe(
            current_widget,
            move |trigger: On<WidgetKeyboardButtonEvent>,
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
                    tab_mode,
                );
            },
        );

    let mut clip_children = WidgetChildren::default();

    clip_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: style.font_size,
            color: style.color,
            text_alignment: style.text_alignment,
            text_wrap: if text_box.multi_line {
                TextWrap::WordOrGlyph
            } else {
                TextWrap::None
            },
            // Keeps text above selection/cursor without reordering children, which would
            // force an expensive text layout recompute.
            z_index: Some(WidgetZ::Relative(2)),
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
    clip_children.add_key("text");

    if !state.selections.is_empty() {
        let selections = state.selections.clone();
        let Ok(layout) = widget_layout.get(current_widget.entity()) else {
            return;
        };
        let pos = layout.location;
        clip_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                height: Units::Pixels(selections.iter().map(|s| s.0.height() as f32).sum()),
                ..styles.cursor
            },
            WidgetRender::Custom {
                render: WidgetRenderCustom::new(move |scene, _widget_layout, styles, scale| {
                    let transform = Affine::default().with_translation(Vec2::new(
                        (pos.x * scale) as f64,
                        (pos.y * scale) as f64,
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
                            &Rect::new(
                                selection.0.x0,
                                selection.0.y0,
                                selection.0.x1,
                                selection.0.y1,
                            ),
                        );
                    }
                }),
            },
        ));
        clip_children.add_key("selection");
    }

    if state.cursor_visible && state.focused {
        clip_children.add::<Element>((Element, cursor_styles, WidgetRender::Quad));
        clip_children.add_key("cursor");
    }

    let mut clip_styles = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        ..Default::default()
    };

    if !text_box.multi_line {
        // Stretch to the full box height so `align_items: Center` below has room to center in.
        clip_styles.height = Units::Percentage(100.0);
        clip_styles.align_items = Some(WidgetAlignItems::Center);
    }

    children.add::<Clip>((Clip, clip_styles, clip_children));

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
    trigger: On<WidgetKeyboardButtonEvent>,
    mut commands: Commands,
    style_query: Query<&WoodpeckerStyle>,
    mut state_query: Query<&mut TextBoxState>,
    mut font_manager: ResMut<FontManager>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state_entity: Entity,
    tab_mode: TabMode,
) {
    // See the matching check in the `WidgetKeyboardCharEvent` handler above -- covers every
    // branch below (Tab, Enter, arrows, backspace, copy, delete) in one place.
    if keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight) {
        return;
    }

    if trigger.code == KeyCode::Tab {
        // Shift+Tab is reserved for moving focus back out, even from a multi-line box --
        // see `tab_focus::tab_navigate`'s matching exemption. Without this, both systems
        // would react to the same Shift+Tab keypress: this would insert a tab/spaces *and*
        // `tab_navigate` would also move focus away.
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            return;
        }

        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };

        // Tabs are normally for focus unless you have focus on a multi-line textbox.
        if !state.multi_line {
            return;
        }
        let mut driver = font_manager.driver(&mut state.engine);
        match tab_mode {
            TabMode::Tab => {
                driver.insert_or_replace_selection("\t");
            }
            TabMode::Space(spaces) => {
                driver.insert_or_replace_selection(
                    &std::iter::repeat_n(' ', spaces as usize).collect::<String>(),
                );
            }
        }
        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
        state.current_value = state.engine.text().to_string();
        commands.trigger(Change {
            target: trigger.target,
            data: TextChanged {
                value: state.current_value.clone(),
            },
        });
    }

    if trigger.code == KeyCode::Enter {
        let Ok(styles) = style_query.get(trigger.target) else {
            return;
        };
        let Ok(mut state) = state_query.get_mut(state_entity) else {
            return;
        };
        if !state.multi_line {
            return;
        }
        let mut driver = font_manager.driver(&mut state.engine);
        driver.insert_or_replace_selection("\n");
        state.selections = state.engine.selection_geometry();
        state.cursor = state
            .engine
            .cursor_geometry(styles.font_size)
            .unwrap_or_default();
        state.current_value = state.engine.text().to_string();
        commands.trigger(Change {
            target: trigger.target,
            data: TextChanged {
                value: state.current_value.clone(),
            },
        });
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
        commands.trigger(Change {
            target: trigger.target,
            data: TextChanged {
                value: state.current_value.clone(),
            },
        });
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
            commands.trigger(Change {
                target: trigger.target,
                data: TextChanged {
                    value: state.current_value.clone(),
                },
            });
        }
    }
}
