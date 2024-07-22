use crate::{
    children::WidgetChildren,
    prelude::{Widget, WidgetRender, WoodpeckerStyle},
    CurrentWidget,
};
use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Out, Over, Pointer},
    focus::PickingInteraction,
    picking_core::Pickable,
    prelude::On,
};

#[derive(Component, Default, Clone)]
pub struct ButtonStyles {
    pub normal: WoodpeckerStyle,
    pub hovered: WoodpeckerStyle,
}

/// A generic button widget used for easy buttons!
#[derive(Bundle, Clone)]
pub struct WButtonBundle {
    /// The button component itself.
    pub app: WButton,
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
    /// On over event listener
    /// Note: If you override the default you will need to manually handle widget state.
    pub on_over: On<Pointer<Over>>,
    /// On out event listener
    /// Note: If you override the default you will need to manually handle widget state.
    pub on_out: On<Pointer<Out>>,
}

impl Default for WButtonBundle {
    fn default() -> Self {
        Self {
            app: Default::default(),
            render: WidgetRender::Quad,
            children: Default::default(),
            styles: Default::default(),
            pickable: Default::default(),
            interaction: Default::default(),
            on_over: On::<Pointer<Over>>::listener_component_mut::<WButton>(|_, button| {
                button.hovering = true;
            }),
            on_out: On::<Pointer<Out>>::listener_component_mut::<WButton>(|_, button| {
                button.hovering = false;
            }),
            button_styles: ButtonStyles::default(), // TODO: Add default button styles..
        }
    }
}

/// The Woodpecker UI Button
#[derive(Component, Widget, Default, Clone)]
#[widget_systems(update, render)]
pub struct WButton {
    pub hovering: bool,
}

pub fn update(
    entity: Res<CurrentWidget>,
    query: Query<Entity, Or<(Changed<WButton>, Changed<ButtonStyles>)>>,
    children_query: Query<&WidgetChildren>,
) -> bool {
    query.contains(**entity)
        || children_query
            .iter()
            .next()
            .map(|c| c.children_changed())
            .unwrap_or_default()
}

pub fn render(
    entity: Res<CurrentWidget>,
    mut query: Query<(
        &WButton,
        &mut WoodpeckerStyle,
        &ButtonStyles,
        &mut WidgetChildren,
    )>,
) {
    let Ok((button, mut styles, button_styles, mut children)) = query.get_mut(**entity) else {
        return;
    };

    if button.hovering {
        *styles = button_styles.hovered.clone();
    } else {
        *styles = button_styles.normal.clone();
    }

    children.apply(entity.as_parent());
}
