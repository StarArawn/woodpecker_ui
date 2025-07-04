use crate::prelude::*;
use bevy::prelude::*;

use super::colors;

/// A collection of styles for icon buttons.
#[derive(Component, Clone, Copy, PartialEq)]
pub struct IconButtonStyles {
    /// Normal Styles
    pub normal: WoodpeckerStyle,
    /// Hovered Styles
    pub hovered: WoodpeckerStyle,
    /// Width of the icon
    pub width: Units,
    /// Height of the icon
    pub height: Units,
}

impl Default for IconButtonStyles {
    fn default() -> Self {
        let normal = WoodpeckerStyle {
            background_color: colors::BACKGROUND_MID,
            ..Default::default()
        };
        Self {
            normal,
            hovered: WoodpeckerStyle {
                background_color: colors::BACKGROUND_LIGHT,
                ..normal
            },
            width: 32.0.into(),
            height: 32.0.into(),
        }
    }
}

#[derive(Component, Debug, Default, PartialEq, Clone)]
pub struct IconButtonState {
    pub hovering: bool,
}

/// A generic button widget used for easy buttons!
#[derive(Component, Widget, Default, Reflect, PartialEq, Clone)]
#[auto_update(render)]
#[props(IconButton, IconButtonStyles)]
#[state(IconButtonState)]
#[require(WidgetRender = WidgetRender::Quad, WidgetChildren, WoodpeckerStyle = ButtonStyles::default().normal, IconButtonStyles, Pickable)]
pub struct IconButton;

pub fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &mut WoodpeckerStyle,
        &IconButtonStyles,
        &mut WidgetChildren,
        &mut WidgetRender,
    )>,
    state_query: Query<&IconButtonState>,
) {
    let Ok((mut styles, button_styles, mut children, mut render)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, IconButtonState::default());

    let default_state = IconButtonState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    if state.hovering {
        render.set_color(button_styles.hovered.background_color);
        *styles = WoodpeckerStyle {
            width: button_styles.width,
            height: button_styles.height,
            ..button_styles.hovered
        };
    } else {
        render.set_color(button_styles.normal.background_color);
        *styles = WoodpeckerStyle {
            width: button_styles.width,
            height: button_styles.height,
            ..button_styles.normal
        };
    }

    commands
        .entity(**current_widget)
        .observe(
            move |_: Trigger<Pointer<Over>>, mut state_query: Query<&mut IconButtonState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = true;
            },
        )
        .observe(
            move |_: Trigger<Pointer<Out>>, mut state_query: Query<&mut IconButtonState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
            },
        );

    children.apply(current_widget.as_parent());
}
