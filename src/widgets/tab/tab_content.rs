use crate::prelude::*;
use bevy::prelude::*;

use super::TabContext;

/// A tab with content
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[auto_update(render)]
#[props(TabContent)]
#[context(TabContext)]
pub struct TabContent {
    /// Tab index(should match tab button index)
    pub index: usize,
}

/// A bundle for creating a tab with content.
#[derive(Bundle, Clone, Default)]
pub struct TabContentBundle {
    /// A tab
    pub tab_content: TabContent,
    /// Children
    pub children: PassedChildren,
    /// internal styles
    pub internal_styles: WoodpeckerStyle,
    /// internal children
    pub internal_children: WidgetChildren,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &TabContent,
        &mut WidgetChildren,
        &mut WoodpeckerStyle,
        &PassedChildren,
    )>,
    context_query: Query<&TabContext>,
) {
    let Ok((tab, mut children, mut styles, passed_children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let context_entity = hooks.use_context(&mut commands, *current_widget, TabContext::default());

    let Ok(context) = context_query.get(context_entity) else {
        return;
    };

    *styles = WoodpeckerStyle {
        display: if context.current_index == tab.index {
            WidgetDisplay::Flex
        } else {
            WidgetDisplay::None
        },
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        ..Default::default()
    };

    *children = WidgetChildren::default().with_child::<Element>((
        ElementBundle {
            styles: WoodpeckerStyle {
                background_color: colors::BACKGROUND,
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            children: passed_children.0.clone(),
            ..Default::default()
        },
        WidgetRender::Quad,
    ));

    children.apply(current_widget.as_parent());
}
