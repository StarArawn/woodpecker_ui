mod tab_button;
mod tab_content;

use crate::prelude::*;
use bevy::prelude::*;

pub use tab_button::*;
pub use tab_content::*;

/// Tab context
#[derive(Component, Default, Clone, PartialEq, Reflect)]
pub struct TabContext {
    /// The current tab selected
    current_index: usize,
}

/// Tab context provider
/// Provides the context for the tab widgets.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[auto_update(render)]
#[props(TabContextProvider)]
pub struct TabContextProvider {
    /// The initial tab that is selected.
    initial_tab_index: usize,
}

/// A tab context provider bundle
#[derive(Bundle, Clone, Default)]
pub struct TabContextProviderBundle {
    /// Tab Context Provider
    pub provider: TabContextProvider,
    /// Internal styles
    pub styles: WoodpeckerStyle,
    /// Children passed in
    pub children: PassedChildren,
    /// Internal Children
    pub internal_children: WidgetChildren,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&TabContextProvider, &mut WidgetChildren, &PassedChildren)>,
) {
    let Ok((provider, mut children, passed_children)) = query.get_mut(**current_widget) else {
        return;
    };

    let _context_entity = hooks.use_context(
        &mut commands,
        *current_widget,
        TabContext {
            current_index: provider.initial_tab_index,
        },
    );

    children.add::<Element>((
        ElementBundle {
            styles: WoodpeckerStyle {
                background_color: colors::BACKGROUND_LIGHT,
                border_color: colors::BACKGROUND_LIGHT,
                border: Edge::all(2.0),
                border_radius: Corner::all(8.0),
                ..Default::default()
            },
            children: WidgetChildren::default().with_child::<Clip>(ClipBundle {
                styles: WoodpeckerStyle {
                    border_radius: Corner::all(8.0),
                    flex_direction: WidgetFlexDirection::Column,
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    ..Default::default()
                },
                children: passed_children.0.clone(),
                ..Default::default()
            }),
            ..Default::default()
        },
        WidgetRender::Quad,
    ));

    children.apply(current_widget.as_parent());
}
