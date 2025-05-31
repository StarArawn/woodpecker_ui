use crate::prelude::*;
use bevy::prelude::*;

/// The Woodpecker UI Element
#[derive(Component, Widget, PartialEq, Reflect, Default, Clone)]
#[auto_update(render)]
#[props(Element, WoodpeckerStyle)]
#[require(WidgetChildren, WoodpeckerStyle)]
pub struct Element;

pub fn render(entity: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };

    children.apply(entity.as_parent());
}
