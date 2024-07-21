use std::time::Instant;

use crate::{
    children::WidgetChildren,
    focus::{Focusable, WidgetBlur, WidgetFocus},
    keyboard_input::{WidgetKeyboardButtonEvent, WidgetPasteEvent},
    prelude::{
        Edge, Units, Widget, WidgetKeyboardCharEvent, WidgetPosition, WidgetRender, WoodpeckerStyle,
    },
    CurrentWidget, DefaultFont,
};
use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Out, Over, Pointer},
    focus::PickingInteraction,
    picking_core::Pickable,
    prelude::{ListenerInput, On},
};
use bevy_vello::{text::VelloFont, vello::glyph::skrifa::FontRef};
use unicode_segmentation::UnicodeSegmentation;

use super::{Clip, ClipBundle, Element, ElementBundle};

#[derive(Component, Clone)]
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
            font_size: 14.0,
            line_height: 26.0,
            ..Default::default()
        };
        Self {
            normal: WoodpeckerStyle { ..shared.clone() },
            hovered: WoodpeckerStyle { ..shared.clone() },
            focused: WoodpeckerStyle {
                border_color: Srgba::new(0.933, 0.745, 0.745, 1.0).into(),
                ..shared.clone()
            },
        }
    }
}

/// A generic textbox widget!
#[derive(Bundle, Clone)]
pub struct TextBoxBundle {
    /// The textbox component itself.
    pub app: TextBox,
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
    /// On over event listener
    /// Note: If you override the default you will need to manually handle widget state.
    pub on_over: On<Pointer<Over>>,
    /// On out event listener
    /// Note: If you override the default you will need to manually handle widget state.
    pub on_out: On<Pointer<Out>>,
    /// Tells woodpecker we want this widget to get focus events.
    pub focuable: Focusable,
    /// On focus event listener
    pub on_focus: On<WidgetFocus>,
    /// On blur event listener
    pub on_blur: On<WidgetBlur>,
    /// On key char event listener
    pub on_key_char: On<WidgetKeyboardCharEvent>,
    /// On key char event listener
    pub on_key_button: On<WidgetKeyboardButtonEvent>,
    pub on_paste_event: On<WidgetPasteEvent>,
}

impl Default for TextBoxBundle {
    fn default() -> Self {
        Self {
            app: Default::default(),
            render: WidgetRender::Quad,
            children: Default::default(),
            styles: Default::default(),
            pickable: Default::default(),
            interaction: Default::default(),
            on_over: On::<Pointer<Over>>::listener_component_mut::<TextBox>(|_, text_box| {
                if !text_box.focused {
                    text_box.hovering = true;
                }
            }),
            on_out: On::<Pointer<Out>>::listener_component_mut::<TextBox>(|_, text_box| {
                text_box.hovering = false;
            }),
            textbox_styles: TextboxStyles::default(),
            focuable: Focusable,
            on_focus: On::<WidgetFocus>::listener_component_mut::<TextBox>(|_, text_box| {
                text_box.hovering = false;
                text_box.focused = true;
            }),
            on_blur: On::<WidgetBlur>::listener_component_mut::<TextBox>(|_, text_box| {
                text_box.focused = false;
            }),
            on_key_char: On::<WidgetKeyboardCharEvent>::run(
                |event: ResMut<ListenerInput<WidgetKeyboardCharEvent>>,
                 mut state_query: Query<(&mut TextBox, &WoodpeckerStyle)>,
                 default_font: Res<DefaultFont>,
                 font_assets: Res<Assets<VelloFont>>| {
                    if let Ok((mut state, styles)) = state_query.get_mut(event.target) {
                        let cursor_pos = state.cursor_position;
                        let font = get_font(&font_assets, styles, &default_font);

                        let char_pos: usize =
                            state.graphemes[0..cursor_pos].iter().map(|g| g.len()).sum();
                        state.current_value.insert_str(char_pos, &event.c);
                        state.cursor_position += 1;

                        // Update graphemes
                        set_graphemes(&mut state);

                        set_new_cursor_position(&mut state, &font, styles.font_size);

                        // TODO: Call changed event..
                    }
                },
            ),
            on_key_button: On::<WidgetKeyboardButtonEvent>::run(
                |event: ResMut<ListenerInput<WidgetKeyboardButtonEvent>>,
                 mut state_query: Query<(&mut TextBox, &WoodpeckerStyle)>,
                 default_font: Res<DefaultFont>,
                 font_assets: Res<Assets<VelloFont>>| {
                    if event.code == KeyCode::ArrowRight {
                        if let Ok((mut state, styles)) = state_query.get_mut(event.target) {
                            if state.cursor_position < state.graphemes.len() {
                                state.cursor_position += 1;
                            }
                            let font = get_font(&font_assets, styles, &default_font);
                            set_new_cursor_position(&mut state, &font, styles.font_size);
                        }
                    }
                    if event.code == KeyCode::ArrowLeft {
                        if let Ok((mut state, styles)) = state_query.get_mut(event.target) {
                            if state.cursor_position > 0 {
                                state.cursor_position -= 1;
                            }
                            let font = get_font(&font_assets, styles, &default_font);
                            set_new_cursor_position(&mut state, &font, styles.font_size);
                        }
                    }
                    if event.code == KeyCode::Backspace {
                        if let Ok((mut state, styles)) = state_query.get_mut(event.target) {
                            let cursor_pos = state.cursor_position;
                            let font = get_font(&font_assets, styles, &default_font);

                            if !state.current_value.is_empty() && cursor_pos != 0 {
                                let char_pos: usize = state.graphemes[0..cursor_pos - 1]
                                    .iter()
                                    .map(|g| g.len())
                                    .sum();
                                state.current_value.remove(char_pos);
                                state.cursor_position -= 1;

                                // Update graphemes
                                set_graphemes(&mut state);

                                set_new_cursor_position(&mut state, &font, styles.font_size);
                            }
                        }
                    }
                },
            ),
            on_paste_event: On::<WidgetPasteEvent>::run(
                |event: ResMut<ListenerInput<WidgetPasteEvent>>,
                 default_font: Res<DefaultFont>,
                 mut state_query: Query<(&mut TextBox, &WoodpeckerStyle)>,
                 font_assets: Res<Assets<VelloFont>>| {
                    let Ok((mut state, styles)) = state_query.get_mut(event.target) else {
                        return;
                    };
                    let char_pos: usize = state.graphemes[0..state.cursor_position]
                        .iter()
                        .map(|g| g.len())
                        .sum();
                    state.current_value.insert_str(char_pos, &event.paste);

                    state.cursor_position += get_graphemes(&event.paste).len();

                    // Update graphemes
                    set_graphemes(&mut state);
                    let font = get_font(&font_assets, styles, &default_font);
                    set_new_cursor_position(&mut state, &font, styles.font_size);
                },
            ),
        }
    }
}

/// The Woodpecker UI Button
#[derive(Component, Widget, Clone)]
#[widget_systems(update, render)]
pub struct TextBox {
    // Mouse input state
    hovering: bool,
    focused: bool,
    // Keyboard input state
    graphemes: Vec<String>,
    cursor_x: f32,
    cursor_position: usize,
    cursor_visible: bool,
    cursor_last_update: Instant,
    current_value: String,
}

impl Default for TextBox {
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

pub fn update(
    entity: Res<CurrentWidget>,
    query: Query<Entity, Or<(Changed<TextBox>, Changed<TextboxStyles>)>>,
    children_query: Query<&WidgetChildren>,
) -> bool {
    query.contains(**entity)
        || children_query
            .iter()
            .next()
            .map(|c| c.children_changed())
            .unwrap_or_default()
}

pub fn render(
    entity: Res<CurrentWidget>,
    mut query: Query<(
        &TextBox,
        &mut WoodpeckerStyle,
        &TextboxStyles,
        &mut WidgetChildren,
    )>,
) {
    let Ok((text_box, mut style, styles, mut children)) = query.get_mut(**entity) else {
        return;
    };

    if text_box.focused {
        *style = styles.focused.clone();
    } else if text_box.hovering {
        *style = styles.hovered.clone();
    } else {
        *style = styles.normal.clone();
    }

    let cursor_styles = WoodpeckerStyle {
        background_color: Srgba::new(0.933, 0.745, 0.745, 1.0).into(),
        position: WidgetPosition::Absolute,
        top: 5.0.into(),
        left: text_box.cursor_x.into(),
        width: 2.0.into(),
        height: (26.0 - 10.0).into(),
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
            alignment: bevy_vello::text::VelloTextAlignment::TopLeft,
            content: text_box.current_value.clone(),
            word_wrap: false,
        },
    ));

    if text_box.cursor_visible {
        clip_children.add::<Element>((
            ElementBundle {
                styles: cursor_styles,
                ..Default::default()
            },
            WidgetRender::Quad,
        ));
    }

    children.add::<Clip>(ClipBundle {
        children: clip_children,
        ..Default::default()
    });

    children.process(entity.as_parent());
}

fn get_font<'a>(
    font_assets: &'a Assets<VelloFont>,
    styles: &WoodpeckerStyle,
    default_font: &DefaultFont,
) -> FontRef<'a> {
    let font_handle = styles.font.as_ref().unwrap_or(&default_font.0);
    let font_asset = font_assets.get(font_handle).expect("Woodpecker UI: Expected to have a font or default font specified but found none. Either set the font on the style or use the DefaultFont resource.");
    let font = FontRef::<'a>::new(font_asset.font.data.data()).expect("Vello font creation error");
    font
}

fn get_graphemes(value: &str) -> Vec<&str> {
    UnicodeSegmentation::graphemes(value, true).collect::<Vec<_>>()
}

fn set_graphemes(state: &mut TextBox) {
    state.graphemes = get_graphemes(&state.current_value)
        .iter()
        .map(|g| g.to_string())
        .collect::<Vec<_>>();
}

fn set_new_cursor_position(state: &mut TextBox, font: &FontRef, font_size: f32) {
    let string_to_cursor = state.graphemes[0..state.cursor_position].join("");
    state.cursor_x = crate::font::measure_width(font, &string_to_cursor, font_size);
}

pub fn cursor_animation_system(
    mut state_query: ParamSet<(Query<(Entity, &TextBox)>, Query<&mut TextBox>)>,
) {
    let mut should_update = Vec::new();

    for (entity, state) in state_query.p0().iter() {
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
