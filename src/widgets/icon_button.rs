use crate::prelude::*;
use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Out, Over, Pointer},
    focus::PickingInteraction,
    picking_core::Pickable,
    prelude::On,
};

use super::colors;

#[derive(Component, Clone, PartialEq)]
pub struct IconButtonStyles {
    pub normal: WoodpeckerStyle,
    pub hovered: WoodpeckerStyle,
    pub width: Units,
    pub height: Units,
}

impl Default for IconButtonStyles {
    fn default() -> Self {
        let normal = WoodpeckerStyle {
            background_color: colors::BACKGROUND_LIGHT,
            ..Default::default()
        };
        Self {
            normal,
            hovered: WoodpeckerStyle {
                background_color: colors::PRIMARY,
                ..normal
            },
            width: 32.0.into(),
            height: 32.0.into(),
        }
    }
}

/// A generic button widget used for easy buttons!
#[derive(Bundle, Clone)]
pub struct IconButtonBundle {
    /// The button component itself.
    pub button: IconButton,
    /// The rendering of the button widget.
    pub render: WidgetRender,
    /// A widget children component
    pub children: WidgetChildren,
    /// The widget styles,
    pub styles: WoodpeckerStyle,
    /// The button styles
    pub button_styles: IconButtonStyles,
    /// Provides overrides for picking behavior.
    pub pickable: Pickable,
    /// Tracks entity interaction state.
    pub interaction: PickingInteraction,
}

impl Default for IconButtonBundle {
    fn default() -> Self {
        Self {
            button: Default::default(),
            render: WidgetRender::Quad,
            children: Default::default(),
            styles: ButtonStyles::default().normal,
            pickable: Default::default(),
            interaction: Default::default(),
            button_styles: IconButtonStyles::default(),
        }
    }
}

#[derive(Component, Default, PartialEq, Clone)]
pub struct IconButtonState {
    pub hovering: bool,
}

/// The Woodpecker UI Button
#[derive(Component, Widget, Default, Reflect, PartialEq, Clone)]
#[auto_update(render)]
#[props(IconButton, IconButtonStyles)]
#[state(IconButtonState)]
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
        .insert(On::<Pointer<Over>>::run(
            move |mut state_query: Query<&mut IconButtonState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = true;
            },
        ))
        .insert(On::<Pointer<Out>>::run(
            move |mut state_query: Query<&mut IconButtonState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
            },
        ));

    children.apply(current_widget.as_parent());
}
