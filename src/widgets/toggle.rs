use crate::prelude::*;
use bevy::{
    ecs::change_detection::{MaybeLocation, Tick},
    prelude::*,
};

/// A toggle change event
#[derive(Debug, Reflect, Clone)]
pub struct ToggleChanged {
    /// Is the toggle "checked"?
    pub checked: bool,
}

/// The state of the toggle button
#[derive(Component, Debug, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ToggleState {
    /// Is hovering
    pub is_hovering: bool,
    /// Is checked
    pub is_checked: bool,
    /// Previous state (Used for animations)
    pub previous_checked: bool,
    /// Previous state (Used for animations)
    pub previous_hover: bool,
    /// The circle button transition
    pub circle_transition: Transition,
}

impl Default for ToggleState {
    fn default() -> Self {
        let checkbox_state_empty = ToggleState {
            is_hovering: false,
            is_checked: false,
            previous_checked: false,
            previous_hover: false,
            circle_transition: Transition::default(),
        };
        Self {
            is_hovering: Default::default(),
            is_checked: Default::default(),
            previous_checked: Default::default(),
            previous_hover: Default::default(),
            circle_transition: Transition {
                easing: TransitionEasing::QuadraticInOut,
                timeout: 250.0,
                style_a: ToggleWidgetStyles::default()
                    .check
                    .get_style(&checkbox_state_empty, false),
                style_b: ToggleWidgetStyles::default()
                    .check
                    .get_style(&checkbox_state_empty, false),
                ..Default::default()
            },
        }
    }
}

/// A collection of styles of the toggle state.
#[derive(PartialEq, Reflect, Clone)]
pub struct ToggleStyles {
    /// Normal
    pub normal: WoodpeckerStyle,
    /// Hovered
    pub hovered: WoodpeckerStyle,
    /// Checked
    pub checked: WoodpeckerStyle,
    /// Both hovered and checked
    pub hovered_checked: WoodpeckerStyle,
}

impl ToggleStyles {
    /// With a given toggle state it returns the correct styles.
    pub fn get_style(&self, state: &ToggleState, previous: bool) -> WoodpeckerStyle {
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

/// A collection of styles for the toggle widget
#[derive(Component, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ToggleWidgetStyles {
    /// Background styles
    pub background: ToggleStyles,
    /// Check styles
    pub check: ToggleStyles,
}

impl Default for ToggleWidgetStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ToggleWidgetStyles {
    fn from_theme(theme: &Theme) -> Self {
        let background_normal = WoodpeckerStyle {
            background_color: theme.background_light,
            width: 34.0.into(),
            height: 14.0.into(),
            border_radius: Corner::all(8.0),
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

        // `theme.text` (not `theme.background`/`.background_light`) for the knob in every
        // state -- like the slider thumb, it needs to read as a raised, solid knob against
        // its own track, not just a slightly-different shade of the same dark surface family.
        // `theme.text` self-adjusts (near-white in `Theme::dark()`, near-black in
        // `Theme::light()`), so it stays high-contrast in either theme. Only the *track*'s
        // color communicates hover/checked state, matching how a physical toggle switch's
        // knob never changes color -- only its position and the surface around it do.
        let check_base = WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            width: 20.0.into(),
            height: 20.0.into(),
            left: (-3.0).into(),
            top: (-3.0).into(),
            border_radius: Corner::all(10.0),
            background_color: theme.text,
            ..Default::default()
        };
        let check_normal = check_base;
        let check_hovered = check_base;
        let check_checked = WoodpeckerStyle {
            left: 20.0.into(),
            ..check_base
        };
        let check_hovered_checked = check_checked;
        Self {
            background: ToggleStyles {
                normal: background_normal,
                hovered: background_hovered,
                checked: background_checked,
                hovered_checked: background_hovered_checked,
            },
            check: ToggleStyles {
                normal: check_normal,
                hovered: check_hovered,
                checked: check_checked,
                hovered_checked: check_hovered_checked,
            },
        }
    }
}

/// A toggle button widget
#[derive(Widget, Component, Reflect, PartialEq, Clone, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(ToggleWidgetStyles, WidgetChildren, WoodpeckerStyle, WidgetRender = WidgetRender::Quad, Pickable, Transition = get_transition())]
pub struct Toggle;

fn get_transition() -> Transition {
    Transition {
        playing: false,
        ..default()
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &ToggleWidgetStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &mut Transition,
    )>,
    mut state_query: Query<&mut ToggleState>,
) {
    let Ok((toggle_styles, mut styles, mut children, mut transition)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, ToggleState::default());

    // TODO: See how we can remove this nonsense.. Maybe by just dereferencing early?
    let mut default_state = ToggleState::default();
    let mut tick_1 = Tick::default();
    let mut tick_2 = Tick::default();
    let tick_3 = Tick::default();
    let tick_4 = Tick::default();
    let mut caller = MaybeLocation::caller();

    if !state_query.contains(state_entity) {
        *styles = toggle_styles.background.get_style(&default_state, false);
    }

    let mut state: Mut<ToggleState> = state_query.get_mut(state_entity).unwrap_or(Mut::new(
        &mut default_state,
        &mut tick_1,
        &mut tick_2,
        tick_3,
        tick_4,
        caller.as_mut(),
    ));

    if !transition.is_playing() {
        *transition = Transition {
            easing: TransitionEasing::QuadraticInOut,
            reversing: false,
            timeout: 250.0,
            style_a: toggle_styles.background.get_style(&state, true),
            style_b: toggle_styles.background.get_style(&state, false),
            ..*transition
        };

        state.circle_transition = Transition {
            easing: TransitionEasing::QuadraticInOut,
            timeout: 250.0,
            style_a: toggle_styles.check.get_style(&state, true),
            style_b: toggle_styles.check.get_style(&state, false),
            ..Default::default()
        };
    }

    if state.previous_checked != state.is_checked {
        if transition.reversing {
            transition.start_reverse();
            state.circle_transition.start_reverse();
        } else {
            transition.start();
            state.circle_transition.start();
        }
        state.previous_checked = state.is_checked;
    } else if state.is_hovering != state.previous_hover || !transition.playing {
        state.previous_hover = state.is_hovering;
    }

    // Insert event listeners
    let current_widget = *current_widget;
    *children = WidgetChildren::default()
        .with_observe(
            current_widget,
            move |_: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut ToggleState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                state.is_checked = !state.is_checked;

                commands.trigger(Change {
                    target: *current_widget,
                    data: ToggleChanged {
                        checked: state.is_checked,
                    },
                });
            },
        )
        .with_self_hover_state(
            current_widget,
            state_entity,
            SystemCursorIcon::Pointer,
            |state: &mut ToggleState, hovering| state.is_hovering = hovering,
        );

    children.add::<Element>((Element, WidgetRender::Quad, state.circle_transition));

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn styles() -> ToggleStyles {
        ToggleStyles {
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
    ) -> ToggleState {
        ToggleState {
            is_hovering,
            is_checked,
            previous_checked,
            previous_hover,
            circle_transition: Transition::default(),
        }
    }

    #[test]
    fn current_unchecked_unhovered_uses_normal_style() {
        let styles = styles();
        // previous_* fields are deliberately the opposite of is_checked/is_hovering here, to
        // prove `previous: false` reads the current fields and not the previous ones.
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
        // Current is checked+hovering (which would select hovered_checked if read), but
        // previous_checked/previous_hover are both false -- previous=true must switch on the
        // previous_* fields, not is_checked/is_hovering, so the result must be `normal`.
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
