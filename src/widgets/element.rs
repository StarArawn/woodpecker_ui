use crate::{
    children::WidgetChildren,
    prelude::{Widget, WoodpeckerStyle},
    CurrentWidget,
};
use bevy::prelude::*;

/// A generic element widget used for layouts.
#[derive(Bundle, Default, Clone)]
pub struct ElementBundle {
    /// The element component itself.
    pub element: Element,
    /// A widget children component
    pub children: WidgetChildren,
    /// The widget styles,
    pub styles: WoodpeckerStyle,
}

/// The Woodpecker UI Element
#[derive(Component, Widget, Default, Clone)]
#[widget_systems(update, render)]
pub struct Element {}

pub fn update(
    entity: Res<CurrentWidget>,
    query: Query<Entity, Or<(Added<Element>, Changed<WoodpeckerStyle>)>>,
    children_query: Query<&WidgetChildren>,
) -> bool {
    query.contains(**entity)
        || children_query
            .iter()
            .next()
            .map(|c| c.children_changed())
            .unwrap_or_default()
}

pub fn render(entity: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };

    children.apply(entity.as_parent());
}
