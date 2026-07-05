use super::colors;
use crate::prelude::*;
use bevy::prelude::*;

/// A set of styles used to style a button.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub struct ButtonStyles {
    /// Normal styles(not hovered).
    pub normal: WoodpeckerStyle,
    /// Styles to apply when hovered.
    pub hovered: WoodpeckerStyle,
}

impl Default for ButtonStyles {
    fn default() -> Self {
        let normal = WoodpeckerStyle {
            background_color: colors::BACKGROUND_LIGHT,
            border_color: colors::BACKGROUND_LIGHT,
            border: Edge::all(2.0),
            border_radius: Corner::all(10.0),
            margin: Edge::new(20.0, 0.0, 0.0, 0.0),
            padding: Edge::all(0.0).left(5.0).right(5.0),
            font_size: 16.0,
            height: 28.0.into(),
            text_alignment: Some(TextAlign::Center),
            width: Units::Pixels(200.0),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        };
        Self {
            normal,
            hovered: WoodpeckerStyle {
                border_color: Srgba::new(0.592, 0.627, 0.749, 1.0).into(),
                ..normal
            },
        }
    }
}

#[derive(Component, Debug, Default, PartialEq, Clone)]
pub struct WButtonState {
    pub hovering: bool,
}

/// The Woodpecker UI Button
#[derive(Component, Widget, Default, Reflect, PartialEq, Clone)]
#[auto_update(render)]
#[props(WButton, ButtonStyles)]
#[state(WButtonState)]
#[require(WidgetRender = WidgetRender::Quad, WidgetChildren, WoodpeckerStyle = ButtonStyles::default().normal, Pickable, ButtonStyles)]
pub struct WButton;

pub fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(&mut WoodpeckerStyle, &ButtonStyles, &mut WidgetChildren)>,
    state_query: Query<&WButtonState>,
) {
    let Ok((mut styles, button_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, WButtonState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    if state.hovering {
        *styles = button_styles.hovered;
    } else {
        *styles = button_styles.normal;
    }

    commands.entity(**current_widget).observe(
        move |_: On<Pointer<Over>>, mut state_query: Query<&mut WButtonState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.hovering = true;
        },
    );
    commands.entity(**current_widget).observe(
        move |_: On<Pointer<Out>>, mut state_query: Query<&mut WButtonState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.hovering = false;
        },
    );

    children.apply(current_widget.as_parent());
}
