use crate::{
    layout::system::{WidgetLayout, WidgetPreviousLayout},
    prelude::*,
};
use bevy::prelude::*;

use super::ScrollContext;

/// A widget that is used to keep track of the layout of the content that
/// the scroll box wraps so that we can correctly calculate the amount of
/// scroll necessary.
#[derive(Component, Widget, Reflect, Default, PartialEq, Eq, Clone)]
#[auto_update(render)]
#[props(ScrollContent, WidgetLayout)]
#[context(ScrollContext)]
pub struct ScrollContent;

/// A bundle for convince when creating the widget.
#[derive(Bundle, Default, Clone)]
pub struct ScrollContentBundle {
    /// The scroll content
    pub scroll_content: ScrollContent,
    /// The styles
    pub styles: WoodpeckerStyleProp,
    /// The children
    pub children: WidgetChildren,
    /// The styles that are actually used after render.
    // TODO: Now that we have proper diffing we can remove this.
    pub internal_styles: WoodpeckerStyle,
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
        context_helper.use_context(&mut commands, *current_widget, ScrollContext::default());

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
        height: context.content_height.into(),
        ..*styles
    };

    children.apply(current_widget.as_parent());
}
