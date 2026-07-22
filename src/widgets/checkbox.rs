use crate::prelude::*;
use bevy::prelude::*;

/// A checkbox change event
#[derive(Clone, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct CheckboxChanged {
    /// Is the checkbox "checked"?
    pub checked: bool,
}

/// The state of the checkbox button
#[derive(Component, Debug, Default, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct CheckboxState {
    /// Is hovering
    pub is_hovering: bool,
    /// Is checked
    pub is_checked: bool,
    /// Previous state (Used for animations)
    pub previous_checked: bool,
    /// Previous state (Used for animations)
    pub previous_hover: bool,
    /// The checkmark transition
    pub check_transition: Transition,
}

/// A collection of styles of the checkbox state.
#[derive(PartialEq, Reflect, Clone)]
pub struct CheckboxStyles {
    /// Normal
    pub normal: WoodpeckerStyle,
    /// Hovered
    pub hovered: WoodpeckerStyle,
    /// Checked
    pub checked: WoodpeckerStyle,
    /// Both hovered and checked
    pub hovered_checked: WoodpeckerStyle,
}

impl CheckboxStyles {
    /// With a given checkbox state it returns the correct styles. `previous` reads
    /// `previous_checked`/`previous_hover` instead of the current fields.
    pub fn get_style(&self, state: &CheckboxState, previous: bool) -> WoodpeckerStyle {
        let (is_checked, is_hovering) = if previous {
            (state.previous_checked, state.previous_hover)
        } else {
            (state.is_checked, state.is_hovering)
        };
        match (is_checked, is_hovering) {
            (true, true) => self.hovered_checked,
            (true, false) => self.checked,
            (false, true) => self.hovered,
            (false, false) => self.normal,
        }
    }
}

/// A collection of styles for the checkbox widget
#[derive(Component, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct CheckboxWidgetStyles {
    /// Background styles
    pub background: CheckboxStyles,
    /// Check styles
    pub check: CheckboxStyles,
}

impl Default for CheckboxWidgetStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for CheckboxWidgetStyles {
    fn from_theme(theme: &Theme) -> Self {
        let background_normal = WoodpeckerStyle {
            background_color: theme.background_light,
            border: Edge::all(1.0),
            border_color: theme.border,
            width: 32.0.into(),
            height: 32.0.into(),
            border_radius: Corner::all(theme.control_radius),
            // Centers the checkmark glyph (below) within this box -- it's an `Auto`-flow
            // child with its own explicit size, not stretched/positioned any other way.
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        };
        let background_hovered = WoodpeckerStyle {
            background_color: theme.background,
            ..background_normal
        };
        let background_checked = WoodpeckerStyle {
            background_color: theme.primary_light,
            ..background_normal
        };
        let background_hovered_checked = WoodpeckerStyle {
            background_color: theme.primary,
            ..background_normal
        };

        // Explicit size is required -- `WidgetRender::Svg` has no intrinsic/aspect-based
        // fallback sizing the way text measurement does, so a fully `Auto`-sized SVG element
        // (the previous state of this style) collapses to zero size and never actually
        // renders anything, even once `is_checked` is true.
        let check_base = WoodpeckerStyle {
            color: Color::WHITE,
            width: 20.0.into(),
            height: 20.0.into(),
            ..Default::default()
        };
        let check_unchecked = WoodpeckerStyle {
            opacity: 0.0,
            ..check_base
        };
        let check_checked = WoodpeckerStyle {
            opacity: 1.0,
            ..check_base
        };
        Self {
            background: CheckboxStyles {
                normal: background_normal,
                hovered: background_hovered,
                checked: background_checked,
                hovered_checked: background_hovered_checked,
            },
            check: CheckboxStyles {
                normal: check_unchecked,
                hovered: check_unchecked,
                checked: check_checked,
                hovered_checked: check_checked,
            },
        }
    }
}

/// A checkbox button widget
#[derive(Widget, Component, Reflect, PartialEq, Clone, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(CheckboxWidgetStyles, WidgetChildren, WoodpeckerStyle,  WidgetRender = WidgetRender::Quad, Pickable, Transition = get_transition())]
pub struct Checkbox;

fn get_transition() -> Transition {
    Transition {
        playing: false,
        ..Default::default()
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    asset_server: Res<AssetServer>,
    mut query: Query<(
        &CheckboxWidgetStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &mut Transition,
    )>,
    mut state_query: Query<&mut CheckboxState>,
) {
    let Ok((checkbox_styles, mut styles, mut children, mut transition)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, CheckboxState::default());

    if !state_query.contains(state_entity) {
        *styles = checkbox_styles.background.get_style(&CheckboxState::default(), false);
    }

    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    if !transition.is_playing() {
        *transition = Transition {
            easing: TransitionEasing::QuadraticInOut,
            reversing: false,
            timeout: 150.0,
            style_a: checkbox_styles.background.get_style(&state, true),
            style_b: checkbox_styles.background.get_style(&state, false),
            ..*transition
        };
        state.check_transition = Transition {
            easing: TransitionEasing::QuadraticInOut,
            timeout: 150.0,
            style_a: checkbox_styles.check.get_style(&state, true),
            style_b: checkbox_styles.check.get_style(&state, false),
            ..Default::default()
        };
    }

    if state.previous_checked != state.is_checked {
        if transition.reversing {
            transition.start_reverse();
            state.check_transition.start_reverse();
        } else {
            transition.start();
            state.check_transition.start();
        }
        state.previous_checked = state.is_checked;
    } else if state.is_hovering != state.previous_hover {
        state.previous_hover = state.is_hovering;
    }

    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    // Insert event listeners
    children
        .self_hover_state(
            current_widget,
            state_entity,
            SystemCursorIcon::Pointer,
            |state: &mut CheckboxState, hovering| state.is_hovering = hovering,
        )
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut CheckboxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_checked = !state.is_checked;
                commands.trigger(Change {
                    target: *current_widget,
                    data: CheckboxChanged {
                        checked: state.is_checked,
                    },
                });
            },
        );

    let check_style = checkbox_styles.check.get_style(&state, false);
    children.add::<Element>((
        Element,
        check_style,
        WidgetRender::Svg {
            handle: asset_server
                .load("embedded://woodpecker_ui/embedded_assets/icons/checkmark.svg"),
            color: Some(check_style.color),
        },
        state.check_transition,
    ));

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn styles() -> CheckboxStyles {
        CheckboxStyles {
            normal: WoodpeckerStyle {
                width: 1.0.into(),
                ..Default::default()
            },
            hovered: WoodpeckerStyle {
                width: 2.0.into(),
                ..Default::default()
            },
            checked: WoodpeckerStyle {
                width: 3.0.into(),
                ..Default::default()
            },
            hovered_checked: WoodpeckerStyle {
                width: 4.0.into(),
                ..Default::default()
            },
        }
    }

    fn state(
        is_checked: bool,
        is_hovering: bool,
        previous_checked: bool,
        previous_hover: bool,
    ) -> CheckboxState {
        CheckboxState {
            is_hovering,
            is_checked,
            previous_checked,
            previous_hover,
            check_transition: Transition::default(),
        }
    }

    #[test]
    fn current_unchecked_unhovered_uses_normal_style() {
        let styles = styles();
        let state = state(false, false, true, true);
        assert_eq!(styles.get_style(&state, false), styles.normal);
    }

    #[test]
    fn current_hovering_not_checked_uses_hovered_style() {
        let styles = styles();
        let state = state(false, true, true, false);
        assert_eq!(styles.get_style(&state, false), styles.hovered);
    }

    #[test]
    fn current_checked_not_hovering_uses_checked_style() {
        let styles = styles();
        let state = state(true, false, false, true);
        assert_eq!(styles.get_style(&state, false), styles.checked);
    }

    #[test]
    fn current_checked_and_hovering_uses_hovered_checked_style() {
        let styles = styles();
        let state = state(true, true, false, false);
        assert_eq!(
            styles.get_style(&state, false),
            styles.hovered_checked,
            "both checked and hovering must select hovered_checked, not just one or the other"
        );
    }

    #[test]
    fn previous_true_reads_previous_fields_instead_of_current() {
        let styles = styles();
        let state = state(true, true, false, false);
        assert_eq!(
            styles.get_style(&state, true),
            styles.normal,
            "previous=true must select styles from previous_checked/previous_hover, not \
             is_checked/is_hovering"
        );
    }

    #[test]
    fn previous_unchecked_hovered_uses_hovered_style() {
        let styles = styles();
        let state = state(true, false, false, true);
        assert_eq!(styles.get_style(&state, true), styles.hovered);
    }

    #[test]
    fn previous_checked_not_hovered_uses_checked_style() {
        let styles = styles();
        let state = state(false, true, true, false);
        assert_eq!(styles.get_style(&state, true), styles.checked);
    }

    #[test]
    fn previous_checked_and_hovered_uses_hovered_checked_style() {
        let styles = styles();
        let state = state(false, false, true, true);
        assert_eq!(
            styles.get_style(&state, true),
            styles.hovered_checked,
            "both previous_checked and previous_hover set must select hovered_checked"
        );
    }
}
