use crate::prelude::*;
use bevy::prelude::*;

/// A slider change event.
#[derive(Reflect, Debug, Clone, PartialEq, Default)]

pub struct SliderChanged {
    /// The value of the slider
    pub value: f32,
}

/// Slider state.
///
/// `value` is always a normalized 0..1 fraction along the track, independent of
/// `Slider::start`/`Slider::end`.
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct SliderState {
    /// The value of the slider, as a 0..1 fraction of the `start..end` range.
    pub value: f32,
}

/// Normalizes a caller-facing domain value into a 0..1 fraction of `start..end`.
fn normalize(value: f32, start: f32, end: f32) -> f32 {
    if end == start {
        return 0.0;
    }
    ((value - start) / (end - start)).clamp(0.0, 1.0)
}

/// Denormalizes a 0..1 fraction back into the caller-facing `start..end` domain.
fn denormalize(fraction: f32, start: f32, end: f32) -> f32 {
    start + fraction * (end - start)
}

/// A collection of slider styles
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct SliderStyles {
    /// The "filled" background styles.
    fill: WoodpeckerStyle,
    /// The background styles
    bar: WoodpeckerStyle,
    /// The draggable button styles.
    button: ButtonStyles,
}

impl Default for SliderStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for SliderStyles {
    fn from_theme(theme: &Theme) -> Self {
        let base_button_styles = WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            width: 20.0.into(),
            height: 20.0.into(),
            left: (-3.0).into(),
            top: (-7.0).into(),
            border_radius: Corner::all(10.0),
            ..Default::default()
        };
        Self {
            fill: WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                background_color: theme.primary,
                width: Units::Percentage(100.0),
                height: 7.0.into(),
                border_radius: Corner::all(12.0),
                ..Default::default()
            },
            bar: WoodpeckerStyle {
                background_color: theme.dark_background,
                width: Units::Percentage(100.0),
                height: 5.0.into(),
                margin: Edge::all(16.0),
                border_radius: Corner::all(12.0),
                ..Default::default()
            },
            // `theme.text` (not `theme.background`/`.background_light`) -- the thumb needs to
            // read clearly as a raised, grabbable handle against *both* the track and
            // whatever's behind the whole slider, not just have a *plausible* one-step
            // lighter shade. `theme.text` self-adjusts (near-white in `Theme::dark()`,
            // near-black in `Theme::light()`), so it stays high-contrast in either theme.
            button: ButtonStyles {
                normal: WoodpeckerStyle {
                    background_color: theme.text,
                    ..base_button_styles
                },
                hovered: WoodpeckerStyle {
                    background_color: theme.primary_light,
                    ..base_button_styles
                },
            },
        }
    }
}

/// A slider widget for numerical values.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(SliderStyles, WidgetChildren, WoodpeckerStyle, WidgetRender = WidgetRender::Quad, Pickable)]
pub struct Slider {
    /// Start value
    pub start: f32,
    /// End value
    pub end: f32,
    /// Initial Value
    pub value: f32,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
            value: 0.0,
        }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &Slider,
        &mut WoodpeckerStyle,
        &SliderStyles,
        &mut WidgetChildren,
        &WidgetLayout,
    )>,
    state_query: Query<&SliderState>,
) {
    let Ok((slider, mut styles, slider_styles, mut children, widget_layout)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let default_state = SliderState {
        value: normalize(slider.value, slider.start, slider.end),
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, default_state);

    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let slider_left = (widget_layout.size.x * state.value) - 3.0;

    *styles = slider_styles.bar;

    let current_widget = *current_widget;
    let (start, end) = (slider.start, slider.end);
    *children = WidgetChildren::default().with_observe(
        current_widget,
        move |trigger: On<Pointer<Click>>,
              mut commands: Commands,
              layout_query: Query<&WidgetLayout>,
              mut state_query: Query<&mut SliderState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            let Ok(widget_layout) = layout_query.get(*current_widget) else {
                return;
            };

            state.value = (trigger.pointer_location.position.x - widget_layout.location.x)
                / widget_layout.size.x;
            state.value = state.value.clamp(0.0, 1.0);
            commands.trigger(Change {
                target: *current_widget,
                data: SliderChanged {
                    value: denormalize(state.value, start, end),
                },
            });
        },
    );
    children.self_hover_cursor(current_widget, SystemCursorIcon::Pointer);

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: (slider_left + 10.0).into(),
            ..slider_styles.fill
        },
        WidgetRender::Quad,
    ));

    children
        .add::<WButton>((
            WButton,
            ButtonStyles {
                normal: WoodpeckerStyle {
                    left: slider_left.into(),
                    ..slider_styles.button.normal
                },
                hovered: WoodpeckerStyle {
                    left: slider_left.into(),
                    ..slider_styles.button.hovered
                },
            },
        ))
        .observe(
            current_widget,
            move |trigger: On<Pointer<Drag>>,
                  mut commands: Commands,
                  layout_query: Query<&WidgetLayout>,
                  mut state_query: Query<&mut SliderState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(widget_layout) = layout_query.get(*current_widget) else {
                    return;
                };
                state.value = (trigger.pointer_location.position.x - widget_layout.location.x)
                    / widget_layout.size.x;
                state.value = state.value.clamp(0.0, 1.0);
                commands.trigger(Change {
                    target: *current_widget,
                    data: SliderChanged {
                        value: denormalize(state.value, start, end),
                    },
                });
            },
        );

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_maps_start_and_end_to_zero_and_one() {
        assert_eq!(normalize(10.0, 10.0, 20.0), 0.0);
        assert_eq!(normalize(20.0, 10.0, 20.0), 1.0);
    }

    #[test]
    fn normalize_clamps_values_below_start_to_zero() {
        assert_eq!(
            normalize(-5.0, 10.0, 20.0),
            0.0,
            "a value below the domain start must clamp to the low end of the fraction, not go negative"
        );
    }

    #[test]
    fn normalize_clamps_values_above_end_to_one() {
        assert_eq!(
            normalize(1000.0, 10.0, 20.0),
            1.0,
            "a value above the domain end must clamp to the high end of the fraction, not exceed 1.0"
        );
    }

    #[test]
    fn normalize_handles_degenerate_range_without_dividing_by_zero() {
        // start == end means (end - start) == 0.0 -- dividing by that would yield NaN/inf,
        // so normalize must special-case it instead of panicking or propagating garbage.
        assert_eq!(normalize(10.0, 10.0, 10.0), 0.0);
        assert_eq!(normalize(0.0, 5.0, 5.0), 0.0);
        assert_eq!(normalize(1000.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn denormalize_maps_zero_and_one_to_start_and_end() {
        assert_eq!(denormalize(0.0, 10.0, 20.0), 10.0);
        assert_eq!(denormalize(1.0, 10.0, 20.0), 20.0);
    }

    #[test]
    fn denormalize_is_unclamped_outside_zero_one() {
        // Unlike normalize, denormalize has no clamp -- callers (e.g. drag handlers) rely on
        // normalize doing the clamping on the way in, so denormalize just extrapolates linearly.
        assert_eq!(denormalize(-1.0, 10.0, 20.0), 0.0);
        assert_eq!(denormalize(2.0, 10.0, 20.0), 30.0);
    }

    #[test]
    fn denormalize_handles_degenerate_range() {
        // start == end collapses the whole domain to a single point regardless of fraction,
        // and must not panic.
        assert_eq!(denormalize(0.0, 10.0, 10.0), 10.0);
        assert_eq!(denormalize(0.5, 10.0, 10.0), 10.0);
        assert_eq!(denormalize(1.0, 10.0, 10.0), 10.0);
    }

    #[test]
    fn normalize_then_denormalize_round_trips_interior_values() {
        for value in [10.0_f32, 12.5, 15.0, 17.5, 20.0] {
            let fraction = normalize(value, 10.0, 20.0);
            assert_eq!(
                denormalize(fraction, 10.0, 20.0),
                value,
                "converting a value to a fraction and back must reproduce the original value"
            );
        }
    }

    #[test]
    fn denormalize_then_normalize_round_trips_interior_fractions() {
        for fraction in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let value = denormalize(fraction, 10.0, 20.0);
            assert_eq!(
                normalize(value, 10.0, 20.0),
                fraction,
                "converting a fraction to a value and back must reproduce the original fraction"
            );
        }
    }
}
