use crate::prelude::*;
use bevy::prelude::*;

/// The Woodpecker UI Element
#[derive(Component, Widget, Reflect, PartialEq, Default, Clone)]
#[auto_update(render)]
#[props(Clip)]
#[require(WidgetChildren, WoodpeckerStyle = WoodpeckerStyle {
    width: Units::Percentage(100.0),
    height: Units::Percentage(100.0),
    ..Default::default()
}, WidgetRender::Layer)]
pub struct Clip;

pub fn render(entity: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };

    children.apply(entity.as_parent());
}
