use crate::prelude::*;
use bevy::{ecs::component::Tick, prelude::*};
use bevy_mod_picking::{
    events::{Click, Out, Over, Pointer},
    focus::PickingInteraction,
    prelude::{On, Pickable},
};

use super::colors;

#[derive(Debug, Reflect, Clone)]
pub struct ToggleChanged {
    pub checked: bool,
}

#[derive(Component, Reflect, PartialEq, Clone)]
pub struct ToggleState {
    pub is_hovering: bool,
    pub is_checked: bool,
    pub previous_checked: bool,
    pub previous_hover: bool,
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
                style_a: ToggleWidgetStyles::default().check.get_style(&checkbox_state_empty, false),
                style_b: ToggleWidgetStyles::default().check.get_style(&checkbox_state_empty, false),
                ..Default::default()
            },
        }
    }
}

#[derive(PartialEq, Reflect, Clone)]
pub struct ToggleStyles {
    pub normal: WoodpeckerStyle,
    pub hovered: WoodpeckerStyle,
    pub checked: WoodpeckerStyle,
    pub hovered_checked: WoodpeckerStyle,
}

impl ToggleStyles {
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

#[derive(Component, Reflect, PartialEq, Clone)]
pub struct ToggleWidgetStyles {
    pub background: ToggleStyles,
    pub check: ToggleStyles,
}

impl Default for ToggleWidgetStyles {
    fn default() -> Self {
        let background_normal = WoodpeckerStyle {
            background_color: colors::BACKGROUND_LIGHT,
            width: 34.0.into(),
            height: 14.0.into(),
            border_radius: Corner::all(8.0),
            ..Default::default()
        };
        let background_hovered = WoodpeckerStyle {
            background_color: colors::BACKGROUND,
            ..background_normal
        };
        let background_checked = WoodpeckerStyle {
            background_color: colors::PRIMARY_LIGHT,
            ..background_normal
        };
        let background_hovered_checked = WoodpeckerStyle {
            background_color: colors::PRIMARY,
            ..background_normal
        };

        let check_base = WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            width: 20.0.into(),
            height: 20.0.into(),
            left: (-3.0).into(),
            top: (-3.0).into(),
            border_radius: Corner::all(10.0),
            ..Default::default()
        };
        let check_normal = WoodpeckerStyle {
            background_color: colors::BACKGROUND,
            ..check_base
        };
        let check_hovered = WoodpeckerStyle {
            background_color: colors::BACKGROUND_LIGHT,
            ..check_base
        };
        let check_checked = WoodpeckerStyle {
            left: 20.0.into(),
            background_color: colors::PRIMARY,
            ..check_base
        };
        let check_hovered_checked = WoodpeckerStyle {
            left: 20.0.into(),
            background_color: colors::PRIMARY_LIGHT,
            ..check_base
        };
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

#[derive(Widget, Component, Reflect, PartialEq, Clone, Default)]
#[auto_update(render)]
#[props(Toggle, ToggleWidgetStyles)]
#[state(ToggleState)]
pub struct Toggle;

#[derive(Bundle, Clone)]
pub struct ToggleBundle {
    pub toggle: Toggle,
    pub toggle_styles: ToggleWidgetStyles,
    pub children: WidgetChildren,
    pub styles: WoodpeckerStyle,
    pub render: WidgetRender,
    /// Provides overrides for picking behavior.
    pub pickable: Pickable,
    /// Tracks entity interaction state.
    pub interaction: PickingInteraction,
    /// Used to animate..
    pub transition: Transition,
    /// Change detection event
    pub on_changed: On<OnChange<ToggleChanged>>,
}

impl Default for ToggleBundle {
    fn default() -> Self {
        Self {
            toggle: Default::default(),
            toggle_styles: Default::default(),
            children: Default::default(),
            styles: Default::default(),
            render: WidgetRender::Quad,
            pickable: Default::default(),
            interaction: Default::default(),
            transition: Transition {
                playing: false,
                ..default()
            },
            on_changed: On::<OnChange<ToggleChanged>>::run(|| {}),
        }
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

    if !state_query.contains(state_entity) {
        *styles = toggle_styles.background.get_style(&default_state, false);
    }

    let mut state: Mut<ToggleState> = state_query.get_mut(state_entity).unwrap_or(Mut::new(
        &mut default_state,
        &mut tick_1,
        &mut tick_2,
        tick_3,
        tick_4,
    ));

    if !transition.is_playing() {
        *transition = Transition {
            easing: TransitionEasing::QuadraticInOut,
            reversing: false,
            timeout: 250.0,
            style_a: toggle_styles.background.get_style(&*state, true),
            style_b: toggle_styles.background.get_style(&*state, false),
            ..transition.clone()
        };

        state.circle_transition = Transition {
            easing: TransitionEasing::QuadraticInOut,
            timeout: 250.0,
            style_a: toggle_styles.check.get_style(&*state, true),
            style_b: toggle_styles.check.get_style(&*state, false),
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
    commands
        .entity(*current_widget)
        .insert(On::<Pointer<Over>>::run(
            move |mut state_query: Query<&mut ToggleState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_hovering = true;
            },
        ))
        .insert(On::<Pointer<Out>>::run(
            move |mut state_query: Query<&mut ToggleState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_hovering = false;
            },
        ))
        .insert(On::<Pointer<Click>>::run(
            move |mut state_query: Query<&mut ToggleState>,
            mut event_writer: EventWriter<OnChange<ToggleChanged>>,
            | {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_checked = !state.is_checked;
                event_writer.send(OnChange {
                    target: *current_widget,
                    data: ToggleChanged {
                        checked: state.is_checked,
                    },
                });
            },
        ));

    children.add::<Element>((
        ElementBundle::default(),
        WidgetRender::Quad,
        state.circle_transition,
    ));

    children.apply(current_widget.as_parent());
}
