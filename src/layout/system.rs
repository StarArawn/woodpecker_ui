use bevy::prelude::*;
use bevy_trait_query::One;
use taffy::Layout;

use crate::context::{Widget, WoodpeckerContext};

use super::{UiLayout, WoodpeckerStyle};

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct WidgetLayout(Layout);

pub(crate) fn run(
    mut commands: Commands,
    mut layout: ResMut<UiLayout>,
    mut query: Query<(
        Entity,
        One<&dyn Widget>,
        &WoodpeckerStyle,
        &mut Transform,
        &mut Sprite,
        Option<&Parent>,
    )>,
    context: Res<WoodpeckerContext>,
) {
    for (entity, _, styles, _, _, parent) in query.iter() {
        trace!("Adding entity: {} to the layout!", entity);
        layout.upsert_node(parent.map(|p| p.get()), entity, styles, None);
    }

    layout.compute(context.get_root_widget(), Vec2::new(1280.0, 720.0));

    for (i, (entity, _, _, mut transform, mut sprite, _)) in query.iter_mut().enumerate() {
        let Some(layout) = layout.get_layout(entity) else {
            continue;
        };
        commands.entity(entity).insert(WidgetLayout(*layout));
        transform.translation = Vec2::new(layout.location.x, layout.location.y).extend(i as f32);
        sprite.custom_size = Some(Vec2::new(layout.size.width, layout.size.height));
    }
}
