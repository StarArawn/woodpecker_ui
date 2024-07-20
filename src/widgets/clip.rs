use crate::{
    children::WidgetChildren,
    prelude::{Units, Widget, WidgetRender, WoodpeckerStyle},
    CurrentWidget,
};
use bevy::prelude::*;

/// A generic element widget used for layouts.
#[derive(Bundle, Clone)]
pub struct ClipBundle {
    /// The element component itself.
    pub app: Clip,
    /// A widget children component
    pub children: WidgetChildren,
    /// The widget styles,
    pub styles: WoodpeckerStyle,
    /// Widget render
    pub widget_render: WidgetRender,
}

impl Default for ClipBundle {
    fn default() -> Self {
        Self {
            app: Default::default(),
            children: Default::default(),
            styles: WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            widget_render: WidgetRender::Layer,
        }
    }
}

/// The Woodpecker UI Element
#[derive(Component, Default, Clone)]
pub struct Clip {}
impl Widget for Clip {}

pub fn update(
    entity: Res<CurrentWidget>,
    query: Query<Entity, Or<(Added<Clip>, Changed<WoodpeckerStyle>)>>,
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

    children.process(entity.as_parent());
}
