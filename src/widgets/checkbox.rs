use crate::prelude::*;
use bevy::prelude::*;

use super::colors;

/// A checkbox change event
#[derive(Clone, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct CheckboxChanged {
    /// Is the checkbox "checked"?
    pub checked: bool,
}

/// The state of the checkbox button
#[derive(Component, Debug, Default, Reflect, PartialEq, Clone)]
pub struct CheckboxState {
    /// Is hovering
    pub is_hovering: bool,
    /// Is checked
    pub is_checked: bool,
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
    /// With a given checkbox state it returns the correct styles.
    pub fn get_style(&self, state: &CheckboxState) -> WoodpeckerStyle {
        match (state.is_checked, state.is_hovering) {
            (true, true) => self.hovered_checked,
            (true, false) => self.checked,
            (false, true) => self.hovered,
            (false, false) => self.normal,
        }
    }
}

/// A collection of styles for the checkbox widget
#[derive(Component, Reflect, PartialEq, Clone)]
pub struct CheckboxWidgetStyles {
    /// Background styles
    pub background: CheckboxStyles,
    /// Check styles
    pub check: CheckboxStyles,
}

impl Default for CheckboxWidgetStyles {
    fn default() -> Self {
        let background_normal = WoodpeckerStyle {
            background_color: colors::BACKGROUND_LIGHT,
            width: 32.0.into(),
            height: 32.0.into(),
            border_radius: Corner::all(2.0),
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
            color: Color::WHITE,
            ..Default::default()
        };
        Self {
            background: CheckboxStyles {
                normal: background_normal,
                hovered: background_hovered,
                checked: background_checked,
                hovered_checked: background_hovered_checked,
            },
            check: CheckboxStyles {
                normal: check_base,
                hovered: check_base,
                checked: check_base,
                hovered_checked: check_base,
            },
        }
    }
}

/// A checkbox button widget
#[derive(Widget, Component, Reflect, PartialEq, Clone, Default)]
#[auto_update(render)]
#[props(Checkbox, CheckboxWidgetStyles)]
#[state(CheckboxState)]
#[require(CheckboxWidgetStyles, WidgetChildren, WoodpeckerStyle,  WidgetRender = WidgetRender::Quad, Pickable)]
pub struct Checkbox;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    asset_server: Res<AssetServer>,
    mut query: Query<(
        &CheckboxWidgetStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&CheckboxState>,
) {
    let Ok((checkbox_styles, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, CheckboxState::default());

    let default_state = CheckboxState::default();

    if !state_query.contains(state_entity) {
        *styles = checkbox_styles.background.get_style(&default_state);
    }

    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    // Insert event listeners
    children
        .observe(
            current_widget,
            move |_: Trigger<Pointer<Over>>, mut state_query: Query<&mut CheckboxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_hovering = true;
            },
        )
        .observe(
            current_widget,
            move |_: Trigger<Pointer<Out>>, mut state_query: Query<&mut CheckboxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_hovering = false;
            },
        )
        .observe(
            current_widget,
            move |trigger: Trigger<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut CheckboxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_checked = !state.is_checked;
                commands.trigger_targets(
                    Change {
                        target: *current_widget,
                        data: CheckboxChanged {
                            checked: state.is_checked,
                        },
                    },
                    trigger.target,
                );
            },
        );

    if state.is_checked {
        let check_styles = checkbox_styles.check.get_style(&default_state);
        children.add::<Element>((
            Element,
            check_styles,
            WidgetRender::Svg {
                handle: asset_server
                    .load("embedded://woodpecker_ui/embedded_assets/icons/checkmark.svg"),
                color: Some(check_styles.color),
            },
        ));
    }

    children.apply(current_widget.as_parent());
}
