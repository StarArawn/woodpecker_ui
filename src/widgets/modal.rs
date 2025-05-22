use bevy::prelude::*;
// use bevy_mod_picking::{
//     events::{Click, Out, Over, Pointer},
//     prelude::{ListenerMut, On},
//     PickableBundle,
// };

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
#[require(WoodpeckerStyle = get_styles(), PassedChildren, WidgetChildren, Transition = get_transition(), WidgetRender = WidgetRender::Layer)]
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

fn get_styles() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        position: WidgetPosition::Fixed,
        ..Default::default()
    }
}

fn get_transition() -> Transition {
    let styles = get_styles();
    Transition {
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
    }
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
            Element,
            WoodpeckerStyle {
                background_color: modal.overlay_color,
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                position: WidgetPosition::Absolute,
                ..Default::default()
            },
            Pickable::default(),
            WidgetRender::Quad,
        ))
        .observe(
            *current_widget,
            move |mut trigger: Trigger<Pointer<Over>>| {
                trigger.propagate(false);
            },
        )
        .observe(
            *current_widget,
            move |mut trigger: Trigger<Pointer<Out>>| {
                trigger.propagate(false);
            },
        )
        .observe(
            *current_widget,
            move |mut trigger: Trigger<Pointer<Click>>| {
                trigger.propagate(false);
            },
        )
        // Window
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                background_color: colors::BACKGROUND,
                border_color: colors::PRIMARY,
                border: Edge::all(2.0),
                border_radius: Corner::all(10.0),
                min_width: modal.min_size.x.into(),
                min_height: modal.min_size.y.into(),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            WidgetChildren::default()
                // Title Bar
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        height: Units::Pixels(24.0),
                        width: Units::Percentage(100.0),
                        padding: Edge::new(0.0, 0.0, 0.0, 5.0),
                        justify_items: Some(WidgetJustifyItems::Center),
                        ..Default::default()
                    },
                    // Title text
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 14.0,
                            text_wrap: TextWrap::None,
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: modal.title.clone(),
                        },
                    )),
                ))
                // Border
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        background_color: Srgba::new(0.239, 0.258, 0.337, 1.0).into(),
                        width: Units::Percentage(100.0),
                        height: 2.0.into(),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                ))
                // Content
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: Units::Percentage(100.0),
                        ..Default::default()
                    },
                    passed_children.0.clone(),
                )),
            WidgetRender::Quad,
        ));

    internal_children.apply(current_widget.as_parent());
}
