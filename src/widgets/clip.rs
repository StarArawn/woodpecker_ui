use crate::prelude::*;
use bevy::prelude::*;

/// A generic element widget used for layouts.
#[derive(Bundle, Clone)]
pub struct ClipBundle {
    /// The element component itself.
    pub clip: Clip,
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
            clip: Default::default(),
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
#[derive(Component, Widget, Reflect, PartialEq, Default, Clone)]
#[auto_update(render)]
#[props(Clip)]
pub struct Clip {}

pub fn render(entity: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };

    children.apply(entity.as_parent());
}
