use crate::prelude::*;
use bevy::prelude::*;
// use bevy_mod_picking::{
//     events::{Drag, Pointer},
//     prelude::{Listener, On},
//     PickableBundle,
// };

/// State to keep track of window data.
#[derive(Default, Debug, Component, Reflect, PartialEq, Clone)]
pub struct WindowState {
    /// The position of the window.
    position: Vec2,
}

/// Window widget
#[derive(Widget, Component, Reflect, PartialEq, Clone)]
#[auto_update(render)]
#[props(WoodpeckerWindow, PassedChildren)]
#[state(WindowState)]
#[require(WoodpeckerStyle, PassedChildren, WidgetRender = WidgetRender::Quad, WidgetChildren)]
pub struct WoodpeckerWindow {
    /// The title of the window.
    pub title: String,
    /// Initial Position
    pub initial_position: Vec2,
    /// Styles for the window widget
    pub window_styles: WoodpeckerStyle,
    /// Styles for the title
    pub title_styles: WoodpeckerStyle,
    /// Styles for the divider under the title
    pub divider_styles: WoodpeckerStyle,
    /// Styles for the children
    pub children_styles: WoodpeckerStyle,
}

impl Default for WoodpeckerWindow {
    fn default() -> Self {
        Self {
            title: Default::default(),
            initial_position: Default::default(),
            window_styles: WoodpeckerStyle {
                background_color: colors::BACKGROUND,
                border_color: colors::DARK_BACKGROUND,
                border: Edge::all(2.0),
                border_radius: Corner::all(5.0),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            title_styles: WoodpeckerStyle {
                background_color: colors::DARK_BACKGROUND,
                height: Units::Pixels(40.0),
                width: Units::Percentage(100.0),
                align_items: Some(WidgetAlignItems::Center),
                padding: Edge::all(0.0).left(10.0).right(10.0),
                margin: Edge::all(0.0).left(-2.0).right(-2.0).top(-2.0),
                border_radius: Corner::all(0.0).top_left(5.0).top_right(5.0),
                ..Default::default()
            },
            divider_styles: WoodpeckerStyle {
                ..Default::default()
            },
            children_styles: WoodpeckerStyle {
                ..Default::default()
            },
        }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &WoodpeckerWindow,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &WidgetLayout,
        &PassedChildren,
    )>,
    state_query: Query<&mut WindowState>,
) {
    let Ok((window, mut styles, mut children, layout, passed_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        WindowState {
            position: window.initial_position,
        },
    );

    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    *styles = WoodpeckerStyle {
        position: WidgetPosition::Fixed,
        left: state.position.x.into(),
        top: state.position.y.into(),
        ..window.window_styles
    };

    let current_widget = *current_widget;
    children
        // Title
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                width: layout.width().into(),
                ..window.title_styles
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: window.title.clone(),
                    word_wrap: false,
                },
            )),
            WidgetRender::Quad,
            Pickable::default(),
        ))
        .observe(
            current_widget,
            move |trigger: Trigger<Pointer<Drag>>,
                  mut state_query: Query<&mut WindowState>,
                  layout_query: Query<&WidgetLayout>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(layout) = layout_query.get(*current_widget) else {
                    return;
                };
                state.position =
                    layout.location + (trigger.pointer_location.position - layout.location);
            },
        )
        // Children
        .add::<Element>((Element, window.children_styles, passed_children.0.clone()));

    children.apply(current_widget.as_parent());
}
