use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Out, Over, Pointer},
    prelude::{ListenerMut, On},
    PickableBundle,
};

use crate::prelude::*;

use super::colors;

#[derive(Component, Reflect, PartialEq, Clone, Debug)]
pub struct ModalState {
    previous_visibility: bool,
}

/// A widget that displays a modal
#[derive(Component, Widget, Reflect, PartialEq, Clone, Debug)]
#[auto_update(render)]
#[props(Modal)]
#[state(ModalState)]
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
    pub overlay_color: Color,
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
            overlay_color: Srgba::new(0.0, 0.0, 0.0, 0.95).into(),
            transition_play: false,
            min_size: Vec2::new(400.0, 250.0),
        }
    }
}

/// Default modal widget
/// A simple widget that renders a modal.
#[derive(Bundle, Clone)]
pub struct ModalBundle {
    /// The modal widget
    pub modal: Modal,
    /// The styles of the modal
    pub styles: WoodpeckerStyle,
    /// The children of the modal
    pub children: PassedChildren,
    /// The internal children of the modal
    pub internal_children: WidgetChildren,
    /// A transition used to fade in/out the modal.
    pub transition: Transition,
    /// Used to tell woodpecker that the modal should create its
    /// own render layer context so our fade in/out works as a
    /// group.
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

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &Modal,
        &mut WidgetChildren,
        &PassedChildren,
        &mut WoodpeckerStyle,
        &mut Transition,
    )>,
    mut modal_state: Query<&mut ModalState>,
) {
    let Ok((modal, mut internal_children, passed_children, mut styles, mut transition)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        ModalState {
            previous_visibility: modal.visible,
        },
    );

    let Ok(mut state) = modal_state.get_mut(state_entity) else {
        return;
    };

    *transition = Transition {
        reversing: !modal.visible,
        timeout: modal.timeout,
        ..*transition
    };

    if state.previous_visibility != modal.visible {
        if transition.reversing {
            transition.start_reverse()
        } else {
            transition.start();
        }
        // Make sure initial state is correct.
        *styles = transition.update();
        state.previous_visibility = modal.visible;
    }

    // *internal_children = WidgetChildren::default();

    let should_render = transition.is_playing() || modal.visible;
    if !should_render {
        return;
    }

    internal_children
        // Overlay
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    background_color: modal.overlay_color,
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    position: WidgetPosition::Absolute,
                    ..Default::default()
                },
                ..Default::default()
            },
            PickableBundle::default(),
            On::<Pointer<Over>>::run(|mut event: ListenerMut<Pointer<Over>>| {
                event.stop_propagation();
            }),
            On::<Pointer<Out>>::run(|mut event: ListenerMut<Pointer<Out>>| {
                event.stop_propagation();
            }),
            On::<Pointer<Click>>::run(|mut event: ListenerMut<Pointer<Click>>| {
                event.stop_propagation();
            }),
            WidgetRender::Quad,
        ))
        // Window
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    background_color: colors::BACKGROUND,
                    border_color: colors::PRIMARY,
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
                    .with_child::<Element>(ElementBundle {
                        styles: WoodpeckerStyle {
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

    internal_children.apply(current_widget.as_parent());
}
