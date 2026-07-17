use crate::prelude::*;
use bevy::prelude::*;
use bevy_vello::vello::peniko::color::parse_color;
use palette::FromColor;
use vello::{
    kurbo::{self, RoundedRectRadii},
    peniko,
};

/// [`ColorPicker`]'s themed colors -- a separate sibling component (rather than the hardcoded
/// `colors::*` values this used before) so the panel live-resyncs on a [`Theme`] swap.
///
/// The hex-value text was previously a fixed `Color::WHITE`, correct only against
/// `Theme::dark()`'s dark panel -- once `panel_background` itself tracks the active theme, a
/// light theme would make that text illegible, so it's promoted to a themed `text_color`
/// (`theme.text`) here rather than staying a hardcoded white. The white selector-ring border
/// around each swatch dot and the copy-button's white background are left alone -- those are
/// deliberately theme-independent affordances (a white ring reads clearly against any
/// hue/saturation/value, unlike text sitting on the panel's own background).
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ColorPickerStyles {
    /// Panel background color.
    pub panel_background: Color,
    /// Panel corner radius.
    pub panel_radius: f32,
    /// Panel border color -- matches `Dropdown`/`Menu`'s own popup-panel border so the picker
    /// reads as a floating surface rather than blending into whatever it sits on.
    pub panel_border_color: Color,
    /// Panel border width.
    pub panel_border_width: f32,
    /// Panel drop shadow.
    pub panel_shadow: WidgetBoxShadow,
    /// Hex-value label text color.
    pub text_color: Color,
    /// Hex-value label font size.
    pub font_size: f32,
    /// Border color of the small inner ring on the saturation/value selector dot.
    pub selector_ring_color: Color,
}

impl Default for ColorPickerStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ColorPickerStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            panel_background: theme.background,
            panel_radius: theme.panel_radius,
            panel_border_color: theme.border,
            panel_border_width: 1.0,
            panel_shadow: theme.elevation.md,
            text_color: theme.text,
            font_size: theme.font_size,
            selector_ring_color: theme.background_light,
        }
    }
}

#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ColorPickerState {
    is_dragging: bool,
    current_color: Hsva,
}

/// Color picker changed event
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Default)]
pub struct ColorPickerChanged {
    /// Color picked
    pub color: Color,
}

/// Color picker widget
#[derive(Widget, Component, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, ColorPickerStyles)]
pub struct ColorPicker {
    /// Initial color to use
    pub initial_color: Color,
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    asset_server: Res<AssetServer>,
    mut query: Query<(
        &ColorPicker,
        &ColorPickerStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&ColorPickerState>,
) {
    let Ok((picker, picker_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let default_state = ColorPickerState {
        is_dragging: false,
        current_color: picker.initial_color.into(),
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, default_state);

    let state = state_query.get(state_entity).unwrap_or(&default_state);

    *styles = WoodpeckerStyle {
        background_color: picker_styles.panel_background,
        border_radius: Corner::all(picker_styles.panel_radius),
        border: Edge::all(picker_styles.panel_border_width),
        border_color: picker_styles.panel_border_color,
        box_shadow: Some(picker_styles.panel_shadow),
        width: 320.0.into(),
        ..Default::default()
    };

    let srgba_color: Color = state.current_color.into();
    let srgba_color = Color::Srgba(srgba_color.into());

    // So we can pass it in.
    let widget_entity = **current_widget;
    *children = WidgetChildren::default().with_child::<Clip>((
        Clip,
        WoodpeckerStyle {
            border_radius: Corner::all(picker_styles.panel_radius),
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        // Main color
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    background_color: srgba_color,
                    width: Units::Percentage(100.0),
                    height: 140.0.into(),
                    margin: Edge::all(0.0).bottom(20.0),
                    ..Default::default()
                },
                WidgetRender::Quad,
            ))
            .with_key("main_color")
            // Color hex value
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    margin: Edge::all(0.0).bottom(20.0).left(20.0).right(20.0),
                    ..Default::default()
                },
                WidgetChildren::default()
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: picker_styles.font_size,
                            color: picker_styles.text_color,
                            flex_grow: 1.0,
                            text_wrap: TextWrap::None,
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: srgba_color.to_srgba().to_hex(),
                        },
                    ))
                    .with_child::<IconButton>((
                        IconButton,
                        IconButtonStyles {
                            normal: WoodpeckerStyle {
                                background_color: Color::WHITE,
                                ..Default::default()
                            },
                            hovered: WoodpeckerStyle {
                                background_color: Color::WHITE.with_alpha(0.8),
                                ..Default::default()
                            },
                            width: 32.0.into(),
                            height: 32.0.into(),
                        },
                        WidgetRender::Svg {
                            handle: asset_server.load(
                                "embedded://woodpecker_ui/embedded_assets/icons/copy-outline.svg",
                            ),
                            color: None, // Set by IconButton
                        },
                    ))
                    .with_observe(
                        *current_widget,
                        move |_trigger: On<Pointer<Click>>,
                              state_query: Query<&ColorPickerState>| {
                            let Ok(state) = state_query.get(state_entity) else {
                                return;
                            };

                            let color: Color = state.current_color.into();
                            let hex = color.to_srgba().to_hex();

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let Ok(mut clipboard) = arboard::Clipboard::new() else {
                                    warn!("no clipboard");
                                    return;
                                };
                                let _ = clipboard.set_text(hex);
                            }

                            #[cfg(target_arch = "wasm32")]
                            {
                                let Some(clipboard) = web_sys::window()
                                    .and_then(|window| Some(window.navigator().clipboard()))
                                else {
                                    warn!("no clipboard");
                                    return;
                                };
                                let _ = clipboard.write_text(&hex);
                            }
                        },
                    ),
            ))
            .with_key("hex_value")
            // Hue
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    margin: Edge::all(0.0).left(20.0).right(20.0),
                    width: 280.0.into(),
                    height: 32.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        position: WidgetPosition::Absolute,
                        top: 7.0.into(),
                        left: (9.0 + ((state.current_color.hue / 365.0) * 245.0)).into(),
                        background_color: srgba_color,
                        width: 18.0.into(),
                        height: 18.0.into(),
                        border_radius: Corner::all(100.0),
                        border_color: Color::WHITE,
                        border: Edge::all(4.0),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                )),
                get_hue_gradient(state.current_color),
                Pickable::default(),
            ))
            .with_observe(
                *current_widget,
                move |trigger: On<Pointer<Drag>>,
                      mut commands: Commands,
                      mut query: Query<&mut ColorPickerState>,
                      layout_query: Query<&WidgetLayout>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    state.is_dragging = true;

                    let Ok(layout) = layout_query.get(widget_entity) else {
                        return;
                    };

                    let value =
                        (trigger.pointer_location.position.x - layout.location.x) / layout.size.x;
                    state.current_color.hue = value.clamp(0.0, 1.0) * 365.0;

                    let color: Color = state.current_color.into();
                    commands.trigger(Change {
                        target: widget_entity,
                        data: ColorPickerChanged {
                            color: color.to_srgba().into(),
                        },
                    });
                },
            )
            .with_observe(
                *current_widget,
                move |_trigger: On<Pointer<DragEnd>>, mut query: Query<&mut ColorPickerState>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    state.is_dragging = false;
                },
            )
            .with_observe(
                *current_widget,
                move |trigger: On<Pointer<Click>>,
                      mut commands: Commands,
                      mut query: Query<&mut ColorPickerState>,
                      layout_query: Query<&WidgetLayout>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    if state.is_dragging {
                        return;
                    }

                    let Ok(layout) = layout_query.get(widget_entity) else {
                        return;
                    };

                    let relative_x =
                        trigger.pointer_location.position.x - (layout.location.x + 40.0);
                    let value = relative_x / (layout.size.x - 80.0);
                    state.current_color.hue = value.clamp(0.0, 1.0) * 365.0;
                    let color: Color = state.current_color.into();
                    commands.trigger(Change {
                        target: widget_entity,
                        data: ColorPickerChanged {
                            color: color.to_srgba().into(),
                        },
                    });
                },
            )
            .with_hover_cursor(*current_widget, SystemCursorIcon::Pointer)
            .with_key("hue")
            // Saturation
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    margin: Edge::all(0.0).left(20.0).right(20.0).top(20.0),
                    width: 280.0.into(),
                    height: 32.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        position: WidgetPosition::Absolute,
                        top: 7.0.into(),
                        left: (9.0 + (state.current_color.saturation * 245.0)).into(),
                        background_color: srgba_color,
                        width: 18.0.into(),
                        height: 18.0.into(),
                        border_radius: Corner::all(100.0),
                        border_color: Color::WHITE,
                        border: Edge::all(4.0),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            position: WidgetPosition::Absolute,
                            left: (-3.0).into(),
                            top: (-3.0).into(),
                            background_color: srgba_color,
                            width: 16.0.into(),
                            height: 16.0.into(),
                            border_radius: Corner::all(100.0),
                            border_color: picker_styles.selector_ring_color,
                            border: Edge::all(3.0),
                            ..Default::default()
                        },
                        WidgetRender::Quad,
                    )),
                    WidgetRender::Quad,
                )),
                get_saturation_gradient(state.current_color),
                Pickable::default(),
            ))
            .with_observe(
                *current_widget,
                move |trigger: On<Pointer<Drag>>,
                      mut commands: Commands,
                      mut query: Query<&mut ColorPickerState>,
                      layout_query: Query<&WidgetLayout>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    state.is_dragging = true;

                    let Ok(layout) = layout_query.get(widget_entity) else {
                        return;
                    };

                    let value =
                        (trigger.pointer_location.position.x - layout.location.x) / layout.size.x;
                    state.current_color.saturation = value.clamp(0.0, 1.0);

                    let color: Color = state.current_color.into();
                    commands.trigger(Change {
                        target: widget_entity,
                        data: ColorPickerChanged {
                            color: color.to_srgba().into(),
                        },
                    });
                },
            )
            .with_observe(
                *current_widget,
                move |_trigger: On<Pointer<DragEnd>>, mut query: Query<&mut ColorPickerState>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    state.is_dragging = false;
                },
            )
            .with_observe(
                *current_widget,
                move |trigger: On<Pointer<Click>>,
                      mut commands: Commands,
                      mut query: Query<&mut ColorPickerState>,
                      layout_query: Query<&WidgetLayout>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    if state.is_dragging {
                        return;
                    }

                    let Ok(layout) = layout_query.get(widget_entity) else {
                        return;
                    };

                    let relative_x =
                        trigger.pointer_location.position.x - (layout.location.x + 40.0);
                    let value = relative_x / (layout.size.x - 80.0);
                    state.current_color.saturation = value.clamp(0.0, 1.0);
                    let color: Color = state.current_color.into();
                    commands.trigger(Change {
                        target: widget_entity,
                        data: ColorPickerChanged {
                            color: color.to_srgba().into(),
                        },
                    });
                },
            )
            .with_hover_cursor(*current_widget, SystemCursorIcon::Pointer)
            .with_key("saturation")
            // Value
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    margin: Edge::all(0.0).left(20.0).right(20.0).top(20.0).bottom(20.0),
                    width: 280.0.into(),
                    height: 32.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        position: WidgetPosition::Absolute,
                        top: 7.0.into(),
                        left: (9.0 + (state.current_color.value * 245.0)).into(),
                        background_color: srgba_color,
                        width: 18.0.into(),
                        height: 18.0.into(),
                        border_radius: Corner::all(100.0),
                        border_color: Color::WHITE,
                        border: Edge::all(4.0),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            position: WidgetPosition::Absolute,
                            left: (-3.0).into(),
                            top: (-3.0).into(),
                            background_color: srgba_color,
                            width: 16.0.into(),
                            height: 16.0.into(),
                            border_radius: Corner::all(100.0),
                            border_color: picker_styles.selector_ring_color,
                            border: Edge::all(3.0),
                            ..Default::default()
                        },
                        WidgetRender::Quad,
                    )),
                    WidgetRender::Quad,
                )),
                get_value_gradient(state.current_color),
                Pickable::default(),
            ))
            .with_observe(
                *current_widget,
                move |trigger: On<Pointer<Drag>>,
                      mut commands: Commands,
                      mut query: Query<&mut ColorPickerState>,
                      layout_query: Query<&WidgetLayout>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    state.is_dragging = true;

                    let Ok(layout) = layout_query.get(widget_entity) else {
                        return;
                    };

                    let value =
                        (trigger.pointer_location.position.x - layout.location.x) / layout.size.x;
                    state.current_color.value = value.clamp(0.0, 1.0);

                    let color: Color = state.current_color.into();
                    commands.trigger(Change {
                        target: widget_entity,
                        data: ColorPickerChanged {
                            color: color.to_srgba().into(),
                        },
                    });
                },
            )
            .with_observe(
                *current_widget,
                move |_trigger: On<Pointer<DragEnd>>, mut query: Query<&mut ColorPickerState>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    state.is_dragging = false;
                },
            )
            .with_observe(
                *current_widget,
                move |trigger: On<Pointer<Click>>,
                      mut commands: Commands,
                      mut query: Query<&mut ColorPickerState>,
                      layout_query: Query<&WidgetLayout>| {
                    let Ok(mut state) = query.get_mut(state_entity) else {
                        return;
                    };

                    if state.is_dragging {
                        return;
                    }

                    let Ok(layout) = layout_query.get(widget_entity) else {
                        return;
                    };

                    let relative_x =
                        trigger.pointer_location.position.x - (layout.location.x + 40.0);
                    let value = relative_x / (layout.size.x - 80.0);
                    state.current_color.value = value.clamp(0.0, 1.0);
                    let color: Color = state.current_color.into();
                    commands.trigger(Change {
                        target: widget_entity,
                        data: ColorPickerChanged {
                            color: color.to_srgba().into(),
                        },
                    });
                },
            )
            .with_hover_cursor(*current_widget, SystemCursorIcon::Pointer)
            .with_key("value"),
    ));

    children.apply(current_widget.as_parent());
}

/// Renders a color picker color gradient.
fn get_hue_gradient(color: Hsva) -> WidgetRender {
    WidgetRender::Custom {
        render: WidgetRenderCustom::new(move |vello_scene, layout, _styles, dpi| {
            let location_x = layout.location.x as f64 * dpi as f64;
            let location_y = layout.location.y as f64 * dpi as f64;
            let size_x = layout.size.x as f64 * dpi as f64;
            let size_y = layout.size.y as f64 * dpi as f64;

            let rect = kurbo::RoundedRect::new(
                location_x - layout.border.left.value_or(0.0) as f64,
                location_y - layout.border.top.value_or(0.0) as f64,
                location_x + (size_x + layout.border.right.value_or(0.0) as f64),
                location_y + (size_y + layout.border.bottom.value_or(0.0) as f64),
                RoundedRectRadii::new(50.0, 50.0, 50.0, 50.0),
            );

            let mut lch_color =
                palette::Hsva::new(0.0, color.saturation as f64, color.value as f64, 1.0);

            let mut stops = vec![];
            let step = 360.0 / size_x;
            for _ in 0..size_x as u32 {
                lch_color.hue += step;
                let srgba = palette::Srgba::from_color(lch_color);
                stops.push(peniko::Color::new([
                    srgba.red as f32,
                    srgba.green as f32,
                    srgba.blue as f32,
                    srgba.alpha as f32,
                ]));
            }

            let brush = peniko::Gradient::new_linear((location_x, 0.0), (location_x + size_x, 0.0))
                .with_stops(&*stops);

            vello_scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::default(),
                &brush,
                None,
                &rect,
            );

            vello_scene.push_layer(
                peniko::Fill::NonZero,
                peniko::Mix::Multiply,
                0.75,
                kurbo::Affine::default(),
                &rect,
            );

            let color = parse_color("#818181").unwrap();
            vello_scene.stroke(
                &kurbo::Stroke::new(5.0),
                kurbo::Affine::default(),
                color,
                None,
                &rect,
            );

            vello_scene.pop_layer();
        }),
    }
}

/// Renders a color picker color gradient.
fn get_saturation_gradient(color: Hsva) -> WidgetRender {
    WidgetRender::Custom {
        render: WidgetRenderCustom::new(move |vello_scene, layout, _styles, dpi| {
            let location_x = layout.location.x as f64 * dpi as f64;
            let location_y = layout.location.y as f64 * dpi as f64;
            let size_x = layout.size.x as f64 * dpi as f64;
            let size_y = layout.size.y as f64 * dpi as f64;

            let rect = kurbo::RoundedRect::new(
                location_x - layout.border.left.value_or(0.0) as f64,
                location_y - layout.border.top.value_or(0.0) as f64,
                location_x + (size_x + layout.border.right.value_or(0.0) as f64),
                location_y + (size_y + layout.border.bottom.value_or(0.0) as f64),
                RoundedRectRadii::new(50.0, 50.0, 50.0, 50.0),
            );

            let mut color = palette::Hsva::new(color.hue as f64, 0.0, color.value as f64, 1.0);

            let mut stops = vec![];
            let step = 1.0 / size_x;
            for _ in 0..size_x as u32 {
                color.saturation += step;
                let srgba: palette::Alpha<palette::rgb::Rgb<palette::encoding::Srgb, f64>, f64> =
                    palette::Srgba::from_color(color);
                stops.push(peniko::Color::new([
                    srgba.red as f32,
                    srgba.green as f32,
                    srgba.blue as f32,
                    srgba.alpha as f32,
                ]));
            }

            let brush = peniko::Gradient::new_linear((location_x, 0.0), (location_x + size_x, 0.0))
                .with_stops(&*stops);

            vello_scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::default(),
                &brush,
                None,
                &rect,
            );

            vello_scene.push_layer(
                peniko::Fill::NonZero,
                peniko::Mix::Multiply,
                0.75,
                kurbo::Affine::default(),
                &rect,
            );

            let color = parse_color("#818181").unwrap();
            vello_scene.stroke(
                &kurbo::Stroke::new(5.0),
                kurbo::Affine::default(),
                color,
                None,
                &rect,
            );

            vello_scene.pop_layer();
        }),
    }
}

/// Renders a color picker color gradient.
fn get_value_gradient(color: Hsva) -> WidgetRender {
    WidgetRender::Custom {
        render: WidgetRenderCustom::new(move |vello_scene, layout, _styles, dpi| {
            let location_x = layout.location.x as f64 * dpi as f64;
            let location_y = layout.location.y as f64 * dpi as f64;
            let size_x = layout.size.x as f64 * dpi as f64;
            let size_y = layout.size.y as f64 * dpi as f64;

            let rect = kurbo::RoundedRect::new(
                location_x - layout.border.left.value_or(0.0) as f64,
                location_y - layout.border.top.value_or(0.0) as f64,
                location_x + (size_x + layout.border.right.value_or(0.0) as f64),
                location_y + (size_y + layout.border.bottom.value_or(0.0) as f64),
                RoundedRectRadii::new(50.0, 50.0, 50.0, 50.0),
            );
            let mut color = palette::Hsva::new(color.hue as f64, color.saturation as f64, 0.0, 1.0);

            let mut stops = vec![];
            let step = 1.0 / size_x;
            for _ in 0..size_x as u32 {
                color.value += step;
                let srgba: palette::Alpha<palette::rgb::Rgb<palette::encoding::Srgb, f64>, f64> =
                    palette::Srgba::from_color(color);
                stops.push(peniko::Color::new([
                    srgba.red as f32,
                    srgba.green as f32,
                    srgba.blue as f32,
                    srgba.alpha as f32,
                ]));
            }

            let brush = peniko::Gradient::new_linear((location_x, 0.0), (location_x + size_x, 0.0))
                .with_stops(&*stops);

            vello_scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::default(),
                &brush,
                None,
                &rect,
            );

            vello_scene.push_layer(
                peniko::Fill::NonZero,
                peniko::Mix::Multiply,
                0.75,
                kurbo::Affine::default(),
                &rect,
            );

            let color = parse_color("#818181").unwrap();
            vello_scene.stroke(
                &kurbo::Stroke::new(5.0),
                kurbo::Affine::default(),
                color,
                None,
                &rect,
            );

            vello_scene.pop_layer();
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = ColorPickerStyles::from_theme(&theme);
        assert_eq!(styles.panel_background, theme.background);
        assert_eq!(styles.panel_radius, theme.panel_radius);
        assert_eq!(styles.text_color, theme.text);
        assert_eq!(styles.font_size, theme.font_size);
        assert_eq!(styles.selector_ring_color, theme.background_light);

        let dark = ColorPickerStyles::from_theme(&Theme::dark());
        let light = ColorPickerStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.panel_background, light.panel_background,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
