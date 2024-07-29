use crate::{
    children::WidgetChildren,
    prelude::{Units, Widget, WoodpeckerStyle},
    CurrentWidget,
};
use bevy::{prelude::*, window::PrimaryWindow};

/// The Woodpecker UI App widget bundle.
/// Typically this should be the root widget.
#[derive(Bundle, Default, Clone)]
pub struct WoodpeckerAppBundle {
    /// The app component itself.
    pub app: WoodpeckerApp,
    /// A widget children component
    pub children: WidgetChildren,
    /// The widget styles,
    pub styles: WoodpeckerStyle,
}

/// The Woodpecker UI App component
#[derive(Component, Widget, Reflect, Default, Clone)]
#[widget_systems(update, render)]
pub struct WoodpeckerApp {}

pub fn update(
    mut prev_size: Local<Vec2>,
    window_query: Query<(Entity, &Window), (Changed<Window>, With<PrimaryWindow>)>,
) -> bool {
    let should_update = window_query.iter().count() > 0;

    if !should_update {
        return false;
    }

    let Some((_, window)) = window_query.iter().next() else {
        return false;
    };

    if window.size() == *prev_size {
        return false;
    }
    *prev_size = window.size();

    true
}

// TODO: Add camera logical size when we add layouting.
pub fn render(
    entity: Res<CurrentWidget>,
    mut query: Query<(&mut WidgetChildren, &mut WoodpeckerStyle)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    let Ok((mut children, mut styles)) = query.get_mut(**entity) else {
        return;
    };

    *styles = WoodpeckerStyle {
        width: Units::Pixels(window.width()),
        height: Units::Pixels(window.height()),
        ..*styles
    };

    children.apply(entity.as_parent());
}
