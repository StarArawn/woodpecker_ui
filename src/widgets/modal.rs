use bevy::prelude::*;

use crate::prelude::*;

#[derive(Component, Widget, PartialEq, Clone, Debug)]
#[widget_systems(update, render)]
pub struct Modal {
    /// The text to display in the modal's title bar
    pub title: String,
    /// A set of styles to apply to the children element wrapper.
    pub children_styles: WoodpeckerStyle,
    /// Is the modal open?
    pub visible: bool,
    /// Animation timeout in milliseconds.
    pub timeout: f32,
    /// The overlay background alpha value
    pub overlay_alpha: f32,
    /// State for animation play
    pub transition_play: bool,
    /// The min size of the modal,
    pub min_size: Vec2,
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            title: Default::default(),
            children_styles: Default::default(),
            visible: false,
            timeout: 250.0,
            overlay_alpha: 0.95,
            transition_play: false,
            min_size: Vec2::new(400.0, 250.0),
        }
    }
}

/// Default modal widget
/// A simple widget that renders a modal.
#[derive(Bundle, Clone)]
pub struct ModalBundle {
    pub modal: Modal,
    pub styles: WoodpeckerStyle,
    pub children: PassedChildren,
    pub internal_children: WidgetChildren,
    pub transition: Transition,
    pub widget_render: WidgetRender,
}

impl Default for ModalBundle {
    fn default() -> Self {
        let styles = WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            position: WidgetPosition::Fixed,
            ..Default::default()
        };
        Self {
            modal: Default::default(),
            styles,
            children: Default::default(),
            internal_children: Default::default(),
            transition: Transition {
                easing: TransitionEasing::Linear,
                looping: false,
                playing: false,
                style_a: WoodpeckerStyle {
                    opacity: 0.0,
                    ..styles
                },
                style_b: WoodpeckerStyle {
                    opacity: 1.0,
                    ..styles
                },
                ..Default::default()
            },
            widget_render: WidgetRender::Layer,
        }
    }
}

fn update(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hook_helper: ResMut<HookHelper>,
    query: Query<&'static Modal, Without<PreviousWidget>>,
    prev_query: Query<&'static Modal, With<PreviousWidget>>,
    children_query: Query<&WidgetChildren>,
) -> bool {
    hook_helper.compare(*current_widget, &mut commands, &query, &prev_query)
        || children_query
            .get(**current_widget)
            .map(|c| c.children_changed())
            .unwrap_or_default()
}

fn render(
    entity: Res<CurrentWidget>,
    mut visible_changed: Local<bool>,
    mut query: Query<(
        &Modal,
        &mut WidgetChildren,
        &PassedChildren,
        &mut WoodpeckerStyle,
        &mut Transition,
    )>,
) {
    let Ok((modal, mut internal_children, passed_children, mut styles, mut transition)) =
        query.get_mut(**entity)
    else {
        return;
    };

    *transition = Transition {
        reversing: !modal.visible,
        timeout: modal.timeout,
        ..transition.clone()
    };

    if *visible_changed != modal.visible {
        if transition.reversing {
            transition.start_reverse()
        } else {
            transition.start();
        }
        // Make sure initial state is correct.
        *styles = transition.update();
        *visible_changed = modal.visible;
    }

    if !transition.is_playing() && !modal.visible {
        return;
    }

    *internal_children = WidgetChildren::default()
        // Overlay
        .with_child::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    background_color: Srgba::new(0.0, 0.0, 0.0, modal.overlay_alpha).into(),
                    // width: Units::Percentage(100.0),
                    // height: Units::Percentage(100.0),
                    position: WidgetPosition::Absolute,
                    ..Default::default()
                },
                ..Default::default()
            },
            WidgetRender::Quad,
        ))
        // Window
        .with_child::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    background_color: Srgba::new(0.188, 0.203, 0.274, 1.0).into(),
                    border_color: Srgba::new(0.933, 0.745, 0.745, 1.0).into(),
                    border: Edge::all(2.0),
                    border_radius: Corner::all(10.0),
                    min_width: modal.min_size.x.into(),
                    min_height: modal.min_size.y.into(),
                    flex_direction: WidgetFlexDirection::Column,
                    ..Default::default()
                },
                children: WidgetChildren::default()
                    // Title Bar
                    .with_child::<Element>(ElementBundle {
                        styles: WoodpeckerStyle {
                            height: Units::Pixels(24.0),
                            width: Units::Percentage(100.0),
                            padding: Edge::new(0.0, 0.0, 0.0, 5.0),
                            ..Default::default()
                        },
                        // Title text
                        children: WidgetChildren::default().with_child::<Element>((
                            ElementBundle {
                                styles: WoodpeckerStyle {
                                    font_size: 14.0,
                                    line_height: Some(18.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: modal.title.clone(),
                                word_wrap: false,
                            },
                        )),
                        ..Default::default()
                    })
                    // Border
                    .with_child::<Element>((
                        ElementBundle {
                            styles: WoodpeckerStyle {
                                background_color: Srgba::new(0.239, 0.258, 0.337, 1.0).into(),
                                width: Units::Percentage(100.0),
                                height: 2.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        WidgetRender::Quad,
                    ))
                    // Content
                    .with_child::<Clip>(ClipBundle {
                        children: passed_children.0.clone(),
                        ..Default::default()
                    }),
                ..Default::default()
            },
            WidgetRender::Quad,
        ));

    internal_children.apply(entity.as_parent());
}
