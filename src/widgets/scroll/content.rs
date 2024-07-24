use crate::{
    layout::system::{WidgetLayout, WidgetPreviousLayout},
    prelude::*,
};
use bevy::prelude::*;

use super::ScrollContext;

#[derive(Component, Widget, Default, PartialEq, Eq, Clone)]
#[widget_systems(update, render)]
pub struct ScrollContent;

#[derive(Bundle, Default, Clone)]
pub struct ScrollContentBundle {
    pub scroll_content: ScrollContent,
    pub styles: WoodpeckerStyleProp,
    pub children: WidgetChildren,
    pub internal_styles: WoodpeckerStyle,
}

pub fn update(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut context_helper: ResMut<HookHelper>,
    query: Query<(
        Ref<ScrollContent>,
        Ref<WidgetChildren>,
        Ref<WidgetLayout>,
        Ref<WidgetPreviousLayout>,
    )>,
    context_query: Query<Entity, Changed<ScrollContext>>,
) -> bool {
    let Ok((sp, children, layout, prev_layout)) = query.get(**current_widget) else {
        return false;
    };

    let context_entity =
        context_helper.use_context::<ScrollContext>(&mut commands, *current_widget);

    sp.is_changed()
        || children.children_changed()
        || *prev_layout != *layout
        || context_query.contains(context_entity)
}

pub fn render(
    mut commands: Commands,
    mut context_helper: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &mut WidgetChildren,
        &mut WoodpeckerStyle,
        &WidgetLayout,
        &WidgetPreviousLayout,
    )>,
    mut context_query: Query<&mut ScrollContext>,
) {
    let Ok((mut children, mut styles, layout, prev_layout)) = query.get_mut(**current_widget)
    else {
        return;
    };
    let context_entity =
        context_helper.use_context::<ScrollContext>(&mut commands, *current_widget);

    let Ok(mut context) = context_query.get_mut(context_entity) else {
        return;
    };

    if *prev_layout != *layout {
        context.content_width = layout.width();
        context.content_height = layout.content_height();
    }

    *styles = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        flex_direction: WidgetFlexDirection::Column,
        min_width: Units::Pixels(context.scrollbox_width - context.pad_x - 10.0),
        min_height: Units::Pixels(context.scrollbox_height - context.pad_y),
        top: context.scroll_y().into(),
        left: context.scroll_x().into(),
        width: Units::Pixels(context.scrollable_width()),
        padding: Edge::all(10.0).right(10.0),
        height: Units::Pixels(context.content_height()),
        ..styles.clone()
    };

    children.apply(current_widget.as_parent());
}
