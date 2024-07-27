use crate::prelude::*;
use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Out, Over, Pointer},
    focus::PickingInteraction,
    picking_core::Pickable,
    prelude::On,
};

#[derive(Component, Clone, PartialEq)]
pub struct ButtonStyles {
    pub normal: WoodpeckerStyle,
    pub hovered: WoodpeckerStyle,
}

impl Default for ButtonStyles {
    fn default() -> Self {
        let normal = WoodpeckerStyle {
            background_color: Srgba::new(0.254, 0.270, 0.349, 1.0).into(),
            border_color: Srgba::new(0.254, 0.270, 0.349, 1.0).into(),
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

/// A generic button widget used for easy buttons!
#[derive(Bundle, Clone)]
pub struct WButtonBundle {
    /// The button component itself.
    pub button: WButton,
    /// The rendering of the button widget.
    pub render: WidgetRender,
    /// A widget children component
    pub children: WidgetChildren,
    /// The widget styles,
    pub styles: WoodpeckerStyle,
    /// The button styles
    pub button_styles: ButtonStyles,
    /// Provides overrides for picking behavior.
    pub pickable: Pickable,
    /// Tracks entity interaction state.
    pub interaction: PickingInteraction,
}

impl Default for WButtonBundle {
    fn default() -> Self {
        Self {
            button: Default::default(),
            render: WidgetRender::Quad,
            children: Default::default(),
            styles: ButtonStyles::default().normal,
            pickable: Default::default(),
            interaction: Default::default(),
            button_styles: ButtonStyles::default(), // TODO: Add default button styles..
        }
    }
}

#[derive(Component, Default, PartialEq, Clone)]
pub struct WButtonState {
    pub hovering: bool,
}

/// The Woodpecker UI Button
#[derive(Component, Widget, Default, Reflect, PartialEq, Clone)]
#[auto_update(render)]
#[props(WButton, ButtonStyles)]
#[state(WButtonState)]
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

    commands
        .entity(**current_widget)
        .insert(On::<Pointer<Over>>::run(
            move |mut state_query: Query<&mut WButtonState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = true;
            },
        ))
        .insert(On::<Pointer<Out>>::run(
            move |mut state_query: Query<&mut WButtonState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.hovering = false;
            },
        ));

    children.apply(current_widget.as_parent());
}
