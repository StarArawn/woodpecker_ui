use crate::prelude::*;
use bevy::prelude::*;
use std::cmp::Ordering;

/// [`Stepper`]'s two layouts.
#[derive(Reflect, Clone, Copy, PartialEq, Default, Debug)]
pub enum StepperOrientation {
    /// Steps flow left-to-right, connected by horizontal lines.
    #[default]
    Horizontal,
    /// Steps flow top-to-bottom, connected by vertical lines, label to the circle's right.
    Vertical,
}

/// Fired when a clickable [`Stepper`]'s step is clicked (only when `Stepper::clickable` is
/// `true` -- a non-clickable stepper is a pure progress indicator, matching MUI's own default).
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct StepperChanged {
    /// The clicked step's index.
    pub step: usize,
}

/// [`Stepper`]'s themed colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct StepperStyles {
    /// Circle background for a completed step.
    pub completed_color: Color,
    /// Circle background for the active step.
    pub active_color: Color,
    /// Circle background for an upcoming step.
    pub upcoming_color: Color,
    /// Circle number/checkmark color -- same for completed and active (both "on" states).
    pub circle_text_color: Color,
    /// Circle number color for an upcoming step.
    pub upcoming_circle_text_color: Color,
    /// Label text color for a completed or active step.
    pub label_color: Color,
    /// Label text color for an upcoming step.
    pub upcoming_label_color: Color,
    /// Connector line color after a completed step.
    pub connector_completed_color: Color,
    /// Connector line color after an active or upcoming step.
    pub connector_color: Color,
}

impl Default for StepperStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for StepperStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            completed_color: theme.primary,
            active_color: theme.primary,
            upcoming_color: theme.background_light,
            circle_text_color: Color::WHITE,
            upcoming_circle_text_color: theme.text.with_alpha(0.6),
            label_color: theme.text,
            upcoming_label_color: theme.text.with_alpha(0.6),
            connector_completed_color: theme.primary,
            connector_color: theme.border,
        }
    }
}

/// A step's derived visual state relative to [`Stepper::active`]. Pulled out as a pure,
/// unit-testable function rather than inlined `match` arms scattered through the render body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepState {
    Completed,
    Active,
    Upcoming,
}

fn step_state(index: usize, active: usize) -> StepState {
    match index.cmp(&active) {
        Ordering::Less => StepState::Completed,
        Ordering::Equal => StepState::Active,
        Ordering::Greater => StepState::Upcoming,
    }
}

/// A wizard/progress-through-steps indicator -- e.g. "Account → Payment → Review → Done".
/// Deliberately distinct from [`crate::widgets::NumberInput`]'s own "stepper" (its +/- value
/// buttons); this is MUI's `Stepper` concept. Circle-index (or checkmark, once completed) per
/// step with connector lines between; optionally clickable (`clickable: true`), firing
/// [`Change<StepperChanged>`] -- otherwise purely a progress display, matching MUI's own
/// non-clickable default.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, StepperStyles)]
pub struct Stepper {
    /// Step labels, in order.
    pub steps: Vec<String>,
    /// The current step, 0-indexed.
    pub active: usize,
    /// Horizontal (default) or vertical layout.
    pub orientation: StepperOrientation,
    /// Whether steps fire [`Change<StepperChanged>`] on click.
    pub clickable: bool,
}

fn circle(
    theme: &Theme,
    styles: &StepperStyles,
    icon_font: &IconFont,
    index: usize,
    state: StepState,
) -> impl Bundle + Clone {
    let (background, text_color, content, font) = match state {
        StepState::Completed => (
            styles.completed_color,
            styles.circle_text_color,
            icons::CHECK.to_string(),
            Some(icon_font.0.id()),
        ),
        StepState::Active => (
            styles.active_color,
            styles.circle_text_color,
            (index + 1).to_string(),
            None,
        ),
        StepState::Upcoming => (
            styles.upcoming_color,
            styles.upcoming_circle_text_color,
            (index + 1).to_string(),
            None,
        ),
    };
    (
        Element,
        WoodpeckerStyle {
            width: theme.control_height.into(),
            height: theme.control_height.into(),
            background_color: background,
            border_radius: Corner::all(theme.control_height / 2.0),
            align_items: Some(WidgetAlignItems::Center),
            justify_content: Some(WidgetAlignContent::Center),
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: theme.typography.body_small,
                color: text_color,
                font,
                ..Default::default()
            },
            WidgetRender::Text { content },
        )),
    )
}

fn label(
    theme: &Theme,
    styles: &StepperStyles,
    text: &str,
    state: StepState,
) -> impl Bundle + Clone {
    let color = if state == StepState::Upcoming {
        styles.upcoming_label_color
    } else {
        styles.label_color
    };
    (
        Element,
        WoodpeckerStyle {
            font_size: theme.typography.body_small,
            color,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: text.into(),
        },
    )
}

fn render(
    theme: Res<Theme>,
    current_widget: Res<CurrentWidget>,
    icon_font: Res<IconFont>,
    mut query: Query<(
        &Stepper,
        &StepperStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((stepper, stepper_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };
    let current_widget = *current_widget;

    styles.flex_direction = match stepper.orientation {
        StepperOrientation::Horizontal => WidgetFlexDirection::Row,
        StepperOrientation::Vertical => WidgetFlexDirection::Column,
    };
    styles.align_items = Some(WidgetAlignItems::Center);

    *children = WidgetChildren::default();
    let last = stepper.steps.len().saturating_sub(1);

    for (index, step_label) in stepper.steps.iter().enumerate() {
        let state = step_state(index, stepper.active);

        // Circle+label order/keys are the same regardless of orientation -- only `step_style`
        // below (row vs column) actually differs between them.
        let step_children = WidgetChildren::default()
            .with_child::<Element>(circle(&theme, stepper_styles, &icon_font, index, state))
            .with_key("circle")
            .with_child::<Element>(label(&theme, stepper_styles, step_label, state))
            .with_key("label");
        let step_style = match stepper.orientation {
            StepperOrientation::Horizontal => WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Column,
                align_items: Some(WidgetAlignItems::Center),
                gap: (0.0.into(), 6.0.into()),
                ..Default::default()
            },
            StepperOrientation::Vertical => WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::Center),
                gap: (theme.spacing.sm.into(), 0.0.into()),
                ..Default::default()
            },
        };

        if stepper.clickable {
            children
                .add::<Element>((Element, step_style, Pickable::default(), step_children))
                .self_hover_cursor(current_widget, SystemCursorIcon::Pointer)
                .observe(
                    current_widget,
                    move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                        commands.trigger(Change {
                            target: current_widget.entity(),
                            data: StepperChanged { step: index },
                        });
                    },
                );
        } else {
            children.add::<Element>((Element, step_style, step_children));
        }
        children.add_key(format!("step-{index}"));

        if index != last {
            let connector_color = if state == StepState::Completed {
                stepper_styles.connector_completed_color
            } else {
                stepper_styles.connector_color
            };
            let connector_style = match stepper.orientation {
                StepperOrientation::Horizontal => WoodpeckerStyle {
                    flex_grow: 1.0,
                    height: 1.0.into(),
                    background_color: connector_color,
                    margin: Edge::all(0.0).left(8.0).right(8.0),
                    ..Default::default()
                },
                StepperOrientation::Vertical => WoodpeckerStyle {
                    width: 1.0.into(),
                    height: 16.0.into(),
                    background_color: connector_color,
                    margin: Edge::all(0.0)
                        .left(theme.control_height / 2.0)
                        .top(4.0)
                        .bottom(4.0),
                    ..Default::default()
                },
            };
            children.add::<Element>((Element, connector_style, WidgetRender::Quad));
            children.add_key(format!("connector-{index}"));
        }
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = StepperStyles::from_theme(&theme);
        assert_eq!(styles.completed_color, theme.primary);
        assert_eq!(styles.upcoming_color, theme.background_light);

        let dark = StepperStyles::from_theme(&Theme::dark());
        let light = StepperStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.upcoming_color, light.upcoming_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn steps_before_active_are_completed() {
        assert_eq!(step_state(0, 2), StepState::Completed);
        assert_eq!(step_state(1, 2), StepState::Completed);
    }

    #[test]
    fn the_active_index_itself_is_active() {
        assert_eq!(step_state(2, 2), StepState::Active);
    }

    #[test]
    fn steps_after_active_are_upcoming() {
        assert_eq!(step_state(3, 2), StepState::Upcoming);
        assert_eq!(step_state(4, 2), StepState::Upcoming);
    }

    #[test]
    fn the_first_step_is_active_when_nothing_has_completed_yet() {
        assert_eq!(step_state(0, 0), StepState::Active);
        assert_eq!(step_state(1, 0), StepState::Upcoming);
    }
}
