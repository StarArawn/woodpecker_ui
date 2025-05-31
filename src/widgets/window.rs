use crate::{picking_backend::compute_letterboxed_transform, prelude::*};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};
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
    drag_offset: Vec2,
}

/// Window widget
#[derive(Widget, Component, Reflect, PartialEq, Clone)]
#[auto_update(render)]
#[props(WoodpeckerWindow, PassedChildren)]
#[state(WindowState)]
#[context(WindowingContext)]
#[require(WoodpeckerStyle, PassedChildren, WidgetRender = WidgetRender::Quad, WidgetChildren, Pickable, Focusable)]
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
        Option<&TitleChildren>,
    )>,
    state_query: Query<&mut WindowState>,
    mut context_query: Query<&mut WindowingContext>,
) {
    let Ok((window, mut styles, mut children, layout, passed_children, title_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        WindowState {
            position: window.initial_position,
            drag_offset: Vec2::new(0.0, 0.0),
        },
    );

    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    // Setup windowing context.
    let context_entity =
        hooks.use_context(&mut commands, *current_widget, WindowingContext::default());

    let Ok(mut context) = context_query.get_mut(context_entity) else {
        return;
    };

    let z_index = context.get_or_add(current_widget.entity());

    *styles = WoodpeckerStyle {
        position: WidgetPosition::Fixed,
        left: state.position.x.into(),
        top: state.position.y.into(),
        z_index: Some(WidgetZ::Global(z_index)),
        ..window.window_styles
    };

    *children = WidgetChildren::default();
    let current_widget = *current_widget;
    children
        .observe(
            current_widget,
            move |trigger: Trigger<WidgetFocus>,
                  mut context_query: Query<&mut WindowingContext>| {
                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };

                context.shift_to_top(trigger.target);
            },
        )
        // Title
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                width: layout.width().into(),
                ..window.title_styles
            },
            if let Some(title_children) = title_children.as_ref() {
                title_children.0.clone()
            } else {
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 14.0,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: window.title.clone(),
                    },
                ))
            },
            WidgetRender::Quad,
            Pickable::default(),
        ))
        .observe(
            current_widget,
            move |_trigger: Trigger<Pointer<Pressed>>,
                  mut context_query: Query<&mut WindowingContext>| {
                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };

                context.shift_to_top(current_widget.entity());
            },
        )
        .observe(
            current_widget,
            move |_trigger: Trigger<Pointer<Over>>,
                  mut commands: Commands,
                  entity: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*entity)
                    .insert(CursorIcon::from(SystemCursorIcon::Grab));
            },
        )
        .observe(
            current_widget,
            move |_trigger: Trigger<Pointer<Out>>,
                  mut commands: Commands,
                  entity: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*entity)
                    .insert(CursorIcon::from(SystemCursorIcon::Default));
            },
        )
        .observe(
            current_widget,
            move |_trigger: Trigger<Pointer<DragEnd>>,
                  mut commands: Commands,
                  entity: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*entity)
                    .insert(CursorIcon::from(SystemCursorIcon::Grab));
            },
        )
        .observe(
            current_widget,
            move |trigger: Trigger<Pointer<DragStart>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut WindowState>,
                  window: Single<(Entity, &Window), With<PrimaryWindow>>,
                  camera: Query<&Camera, With<WoodpeckerView>>,
                  mut context_query: Query<&mut WindowingContext>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                let Some(camera) = camera.iter().next() else {
                    return;
                };

                let (offset, size, _scale) = compute_letterboxed_transform(
                    window.1.size(),
                    camera.logical_target_size().unwrap(),
                );

                let cursor_pos_world = ((trigger.pointer_location.position - offset) / size)
                    * camera.logical_target_size().unwrap();

                state.drag_offset = state.position - cursor_pos_world;

                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };

                commands
                    .entity(window.0)
                    .insert(CursorIcon::from(SystemCursorIcon::Grabbing));

                context.shift_to_top(current_widget.entity());
            },
        )
        .observe(
            current_widget,
            move |trigger: Trigger<Pointer<Drag>>,
                  mut state_query: Query<&mut WindowState>,
                  window: Single<(Entity, &Window), With<PrimaryWindow>>,
                  camera: Query<&Camera, With<WoodpeckerView>>,
                  mut context_query: Query<&mut WindowingContext>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                let Some(camera) = camera.iter().next() else {
                    return;
                };

                let (offset, size, _scale) = compute_letterboxed_transform(
                    window.1.size(),
                    camera.logical_target_size().unwrap(),
                );

                let cursor_pos_world = ((trigger.pointer_location.position - offset) / size)
                    * camera.logical_target_size().unwrap();

                state.position = cursor_pos_world + state.drag_offset;

                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };

                context.shift_to_top(current_widget.entity());
            },
        )
        // Children
        .add::<Element>((Element, window.children_styles, passed_children.0.clone()));

    children.apply(current_widget.as_parent());
}
