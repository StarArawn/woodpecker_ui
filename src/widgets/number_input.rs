use crate::prelude::*;
use bevy::{
    prelude::*,
    window::{CursorIcon, PrimaryWindow},
};

/// A number input change event.
#[derive(Reflect, Debug, Clone, PartialEq, Default)]
pub struct NumberInputChanged {
    /// The new value, already clamped to `NumberInput::min..=NumberInput::max`.
    pub value: f32,
}

/// Number input state -- `value` is the committed numeric value; `displayed` is exactly what
/// the embedded `TextBox` should show right now. They're tracked separately because `TextBox`
/// resets its internal editor (losing cursor position) whenever the `initial_value` it's
/// given differs from what it last saw -- if `displayed` were always freshly recomputed via
/// `format_value(value)`, any keystroke that reformats differently from what was actually
/// typed (e.g. `"15."` -> `"15"`) would trigger that reset mid-edit, desyncing the cursor from
/// the text (and, if the cursor's stale byte offset falls outside the new text, panicking).
/// So typing echoes `displayed` back verbatim -- matching what `TextBox`'s own editor already
/// has -- and only a step-button click or a blur re-derives `displayed` from `value`.
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct NumberInputState {
    /// The current committed value.
    pub value: f32,
    /// Exactly what the embedded textbox should display.
    pub displayed: String,
}

/// A collection of styles for the number input.
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct NumberInputStyles {
    /// The outer rounded-pill container styles -- the sole border/background for the whole
    /// control; the buttons and value textbox below are transparent regions layered on top
    /// of it.
    pub container: WoodpeckerStyle,
    /// The value textbox's styles: transparent/borderless (the container already draws the
    /// pill), sized to fill the space between the two buttons (`flex_grow`/`min_width`), and
    /// center-aligned via `text_alignment` -- correct here specifically because the textbox's
    /// own internal `Clip` wrapper is the sole full-width parent of its rendered text, unlike
    /// a plain `Element` sitting directly in a row next to sibling buttons (this crate's
    /// renderer aligns text against its *parent's* width, not its own box).
    pub label: TextboxStyles,
    /// The `-` button styles, rounded only on its left corners to match the container.
    pub decrement_button: ButtonStyles,
    /// The `+` button styles, rounded only on its right corners to match the container.
    pub increment_button: ButtonStyles,
}

impl Default for NumberInputStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for NumberInputStyles {
    fn from_theme(theme: &Theme) -> Self {
        let step_button_base = WoodpeckerStyle {
            width: theme.control_height.into(),
            height: theme.control_height.into(),
            background_color: Color::NONE,
            border_color: theme.border,
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        };
        let left_radius = Corner::all(0.0)
            .top_left(theme.control_radius)
            .bottom_left(theme.control_radius);
        let right_radius = Corner::all(0.0)
            .top_right(theme.control_radius)
            .bottom_right(theme.control_radius);
        // A hairline border on the label-facing edge only, separating each button from the
        // number the way a physical stepper controls' seams would.
        let left_border = Edge::all(0.0).right(1.0);
        let right_border = Edge::all(0.0).left(1.0);
        Self {
            container: WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::Center),
                background_color: theme.dark_background,
                border: Edge::all(1.0),
                border_color: theme.border,
                border_radius: Corner::all(theme.control_radius),
                height: theme.control_height.into(),
                ..Default::default()
            },
            label: {
                let shared = WoodpeckerStyle {
                    flex_grow: 1.0,
                    min_width: 50.0.into(),
                    // A fixed pixel height, not `Percentage(100.0)` -- `TextBox`'s own
                    // vertical cursor-centering math in `text_box.rs` resolves `height` via
                    // `Units::value_or`, which treats `Percentage(100.0)` as the literal
                    // fraction `1.0`, not "100% of the parent" (that resolution only happens
                    // during taffy layout, which this style-level helper has no access to).
                    // The container is already exactly `control_height` tall, so this is
                    // visually identical to 100% here while keeping the cursor math correct.
                    height: theme.control_height.into(),
                    background_color: Color::NONE,
                    border: Edge::all(0.0),
                    color: theme.text,
                    font_size: theme.font_size,
                    text_alignment: Some(TextAlign::Center),
                    ..Default::default()
                };
                TextboxStyles {
                    normal: shared,
                    hovered: shared,
                    // The only visual affordance that the value is editable -- the container
                    // already supplies the pill's border, so the textbox itself stays
                    // borderless even while focused, just with a faint fill.
                    focused: WoodpeckerStyle {
                        background_color: theme.background_light,
                        ..shared
                    },
                    cursor: WoodpeckerStyle {
                        background_color: theme.primary,
                        position: WidgetPosition::Absolute,
                        width: 2.0.into(),
                        ..Default::default()
                    },
                }
            },
            decrement_button: ButtonStyles {
                normal: WoodpeckerStyle {
                    border: left_border,
                    border_radius: left_radius,
                    ..step_button_base
                },
                hovered: WoodpeckerStyle {
                    background_color: theme.background_light,
                    border: left_border,
                    border_radius: left_radius,
                    ..step_button_base
                },
            },
            increment_button: ButtonStyles {
                normal: WoodpeckerStyle {
                    border: right_border,
                    border_radius: right_radius,
                    ..step_button_base
                },
                hovered: WoodpeckerStyle {
                    background_color: theme.background_light,
                    border: right_border,
                    border_radius: right_radius,
                    ..step_button_base
                },
            },
        }
    }
}

/// A bounded numeric stepper: `-`/`+` buttons step by `step`; the value between them is an
/// editable textbox (click it to type an exact value) that also supports Alt+drag to scrub
/// it continuously, without either interaction interfering with the other -- see the `Alt`
/// guard in `TextBox`'s own `Pointer<Press>`/`DragStart`/`Drag` handlers. Fires
/// `Change<NumberInputChanged>` on any value change (step click, scrub, or a valid typed
/// value), matching `Slider`/`SliderChanged`'s shape.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(NumberInputStyles, WidgetChildren, WoodpeckerStyle, WidgetRender = WidgetRender::Quad, Pickable)]
pub struct NumberInput {
    /// The initial value -- only read on first mount, like `Slider::value`.
    pub value: f32,
    /// The minimum allowed value.
    pub min: f32,
    /// The maximum allowed value.
    pub max: f32,
    /// The amount each step-button click changes the value by.
    pub step: f32,
    /// How many pixels of Alt+drag on the value correspond to one `step`. Lower is more
    /// sensitive.
    pub drag_pixels_per_step: f32,
}

impl Default for NumberInput {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            drag_pixels_per_step: 4.0,
        }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    mut query: Query<(
        &NumberInput,
        &NumberInputStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&NumberInputState>,
) {
    let Ok((number_input, number_input_styles, mut styles, mut children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let initial_value = number_input.value.clamp(number_input.min, number_input.max);
    let default_state = NumberInputState {
        value: initial_value,
        displayed: format_value(initial_value),
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, default_state.clone());
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    *styles = number_input_styles.container;

    let current_widget = *current_widget;
    let (min, max, step, drag_pixels_per_step) = (
        number_input.min,
        number_input.max,
        number_input.step,
        number_input.drag_pixels_per_step,
    );

    // Commits a definitive value -- from a step click or a blur-time normalization -- by
    // clamping it and reformatting `displayed` to match, notifying via
    // `Change<NumberInputChanged>` if the value actually changed. Deliberately not used by the
    // `Change<TextChanged>` handler below, which needs `displayed` to stay exactly what was
    // typed, not reformatted.
    let commit_value =
        move |value: f32,
              commands: &mut Commands,
              state_query: &mut Query<&mut NumberInputState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            let value = value.clamp(min, max);
            state.displayed = format_value(value);
            if value == state.value {
                return;
            }
            state.value = value;
            commands.trigger(Change {
                target: *current_widget,
                data: NumberInputChanged { value },
            });
        };

    let apply_delta = move |delta: f32,
                            commands: &mut Commands,
                            state_query: &mut Query<&mut NumberInputState>| {
        let Ok(current) = state_query.get(state_entity).map(|s| s.value) else {
            return;
        };
        commit_value(current + delta, commands, state_query);
    };

    *children = WidgetChildren::default();

    let apply_delta_minus = apply_delta;
    children
        .add::<WButton>((
            WButton,
            number_input_styles.decrement_button,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: number_input_styles.label.normal.color,
                    text_wrap: TextWrap::None,
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: icons::MINUS.into(),
                },
            )),
        ))
        .observe(
            current_widget,
            move |_: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut NumberInputState>| {
                apply_delta_minus(-step, &mut commands, &mut state_query);
            },
        );
    children.add_key("decrement");

    // `state.displayed` (not a freshly-`format_value`'d string) is what's passed every render
    // -- while typing, it's exactly what `Change<TextChanged>` last reported, matching what
    // `TextBox`'s own editor already has, so this never forces a mid-edit reset. It only
    // actually changes here following a step-button click or a blur-time reformat.
    children
        .add::<TextBox>((
            TextBox {
                initial_value: state.displayed.clone(),
                ..Default::default()
            },
            number_input_styles.label.clone(),
        ))
        .observe(
            current_widget,
            move |trigger: On<Change<TextChanged>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut NumberInputState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.displayed.clone_from(&trigger.data.value);

                // Ignore anything that doesn't parse yet (e.g. a bare "-" while typing a
                // negative number, or an emptied field) -- only a fully valid number commits.
                // Deliberately not `commit_value`, which would reformat `displayed` and fight
                // the in-progress edit.
                let Ok(value) = trigger.data.value.parse::<f32>() else {
                    return;
                };
                let value = value.clamp(min, max);
                if value == state.value {
                    return;
                }
                state.value = value;
                commands.trigger(Change {
                    target: *current_widget,
                    data: NumberInputChanged { value },
                });
            },
        )
        .observe(
            current_widget,
            move |_: On<WidgetBlur>,
                  mut commands: Commands,
                  mut state_query: Query<&mut NumberInputState>| {
                // Normalize whatever's currently shown (e.g. a trailing "15." left over from
                // typing) back to the canonical `format_value` rendering once editing ends.
                let Ok(current) = state_query.get(state_entity).map(|s| s.value) else {
                    return;
                };
                commit_value(current, &mut commands, &mut state_query);
            },
        )
        // Alt+drag scrubs the value continuously; a plain click still edits/selects text as
        // normal (see the matching `Alt` guard in `TextBox`'s own Press/Over/DragStart/Drag
        // handlers, which step aside so the two interactions don't fight over the same drag).
        // These `Over`/`Out` handlers give the scrub cursor immediately on hover, not just
        // once a drag actually starts -- if Alt is pressed *after* the pointer is already
        // hovering (no fresh `Over` event to react to), the cursor won't update until the
        // next `Over`/`Out`, but the drag itself is still correctly gated either way.
        .observe(
            current_widget,
            move |_: On<Pointer<Over>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  mut commands: Commands,
                  window: Single<Entity, With<PrimaryWindow>>| {
                if !(keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight))
                {
                    return;
                }
                commands
                    .entity(*window)
                    .insert(CursorIcon::from(SystemCursorIcon::EwResize));
            },
        )
        .observe(
            current_widget,
            move |_: On<Pointer<Out>>,
                  mut commands: Commands,
                  window: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*window)
                    .insert(CursorIcon::from(SystemCursorIcon::Default));
            },
        )
        .observe(
            current_widget,
            move |_: On<Pointer<DragStart>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  mut commands: Commands,
                  window: Single<Entity, With<PrimaryWindow>>| {
                if !(keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight))
                {
                    return;
                }
                commands
                    .entity(*window)
                    .insert(CursorIcon::from(SystemCursorIcon::EwResize));
            },
        )
        .observe(
            current_widget,
            move |trigger: On<Pointer<Drag>>,
                  keyboard_input: Res<ButtonInput<KeyCode>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut NumberInputState>| {
                if !(keyboard_input.pressed(KeyCode::AltLeft)
                    || keyboard_input.pressed(KeyCode::AltRight))
                {
                    return;
                }
                let Ok(current) = state_query.get(state_entity).map(|s| s.value) else {
                    return;
                };
                let delta = (trigger.delta.x / drag_pixels_per_step) * step;
                commit_value(current + delta, &mut commands, &mut state_query);
            },
        )
        .observe(
            current_widget,
            move |_: On<Pointer<DragEnd>>,
                  mut commands: Commands,
                  window: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*window)
                    .insert(CursorIcon::from(SystemCursorIcon::Default));
            },
        );
    children.add_key("value");

    children
        .add::<WButton>((
            WButton,
            number_input_styles.increment_button,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: number_input_styles.label.normal.color,
                    text_wrap: TextWrap::None,
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: icons::PLUS.into(),
                },
            )),
        ))
        .observe(
            current_widget,
            move |_: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut NumberInputState>| {
                apply_delta(step, &mut commands, &mut state_query);
            },
        );
    children.add_key("increment");

    children.apply(current_widget.as_parent());
}

/// Trims a trailing `.0` off whole numbers (`"5"` not `"5.0"`) while keeping one decimal place
/// for fractional values (`"5.3"`), matching how a stepper's value is conventionally shown.
fn format_value(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_positive_value_has_no_decimal_point() {
        assert_eq!(format_value(5.0), "5");
    }

    #[test]
    fn whole_negative_value_has_no_decimal_point() {
        assert_eq!(format_value(-5.0), "-5");
    }

    #[test]
    fn zero_formats_without_a_decimal_point() {
        assert_eq!(format_value(0.0), "0");
    }

    #[test]
    fn fractional_positive_value_keeps_one_decimal_place() {
        assert_eq!(format_value(5.3), "5.3");
    }

    #[test]
    fn fractional_negative_value_keeps_one_decimal_place() {
        assert_eq!(format_value(-5.3), "-5.3");
    }

    #[test]
    fn large_whole_value_has_no_decimal_point() {
        assert_eq!(
            format_value(123_456.0),
            "123456",
            "the `fract() == 0.0` branch must still be taken at large magnitudes, not just \
             near zero"
        );
    }

    #[test]
    fn large_fractional_value_keeps_one_decimal_place() {
        assert_eq!(format_value(123_456.7), "123456.7");
    }

    #[test]
    fn tiny_positive_fraction_rounds_to_zero_point_zero() {
        // `0.001` is not itself `0.0`, so it takes the fractional branch (`fract() != 0.0`),
        // but rounding to one decimal place collapses it to "0.0" -- distinct from the whole
        // value `0` formatting as plain "0". This is a real (if surprising) consequence of the
        // one-decimal-place formatting, not a bug in the test.
        assert_eq!(format_value(0.001), "0.0");
    }

    #[test]
    fn tiny_negative_fraction_rounds_to_negative_zero_point_zero() {
        // Mirrors `tiny_positive_fraction_rounds_to_zero_point_zero`, but on the negative side
        // the sign survives rounding, producing "-0.0" rather than "0.0".
        assert_eq!(format_value(-0.001), "-0.0");
    }

    #[test]
    fn fractional_value_rounds_to_nearest_tenth() {
        assert_eq!(
            format_value(0.05),
            "0.1",
            "0.05 isn't exactly representable in f32 (it's stored slightly above the true \
             value), so it rounds up to the next tenth rather than down"
        );
    }

    #[test]
    fn fractional_value_uses_round_half_to_even_on_exact_ties() {
        // -0.25 is exactly representable in f32, so rounding to one decimal place hits a true
        // tie between -0.2 and -0.3. Rust's float formatting breaks ties to the even digit
        // (-0.2, since 2 is even), not away from zero (-0.3).
        assert_eq!(format_value(-0.25), "-0.2");
    }

    #[test]
    fn negative_zero_input_preserves_its_sign() {
        assert_eq!(
            format_value(-0.0),
            "-0",
            "negative zero takes the whole-number branch (`fract() == 0.0`) but the `.0` \
             precision formatting of f32 still prints the sign"
        );
    }
}
