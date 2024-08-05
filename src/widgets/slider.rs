use crate::prelude::*;
use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Drag, Pointer},
    focus::PickingInteraction,
    prelude::{Listener, ListenerMut, On, Pickable},
};

/// A slider change event.
#[derive(Reflect, Debug, Clone, PartialEq, Default)]

pub struct SliderChanged {
    /// The value of the slider
    pub value: f32,
}

/// Slider state
#[derive(Component, Reflect, Clone, Copy, PartialEq, Default)]
pub struct SliderState {
    /// The value of the slider
    pub value: f32,
}

/// A collection of slider styles
#[derive(Component, Reflect, Clone, PartialEq)]
pub struct SliderStyles {
    /// The "filled" background styles.
    fill: WoodpeckerStyle,
    /// The background styles
    bar: WoodpeckerStyle,
    /// The draggable button styles.
    button: ButtonStyles,
}

impl Default for SliderStyles {
    fn default() -> Self {
        let base_button_styles = WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            width: 20.0.into(),
            height: 20.0.into(),
            left: (-3.0).into(),
            top: (-7.0).into(),
            border_radius: Corner::all(10.0),
            ..Default::default()
        };
        Self {
            fill: WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                background_color: colors::PRIMARY,
                width: Units::Percentage(100.0),
                height: 7.0.into(),
                border_radius: Corner::all(12.0),
                ..Default::default()
            },
            bar: WoodpeckerStyle {
                background_color: colors::DARK_BACKGROUND,
                width: Units::Percentage(100.0),
                height: 5.0.into(),
                margin: Edge::all(16.0),
                border_radius: Corner::all(12.0),
                ..Default::default()
            },
            button: ButtonStyles {
                normal: WoodpeckerStyle {
                    background_color: colors::BACKGROUND,
                    ..base_button_styles
                },
                hovered: WoodpeckerStyle {
                    background_color: colors::BACKGROUND_LIGHT,
                    ..base_button_styles
                },
            },
        }
    }
}

/// A slider widget for numerical values.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[auto_update(render)]
#[props(Slider, SliderStyles)]
#[state(SliderState)]
pub struct Slider {
    /// Start value
    pub start: f32,
    /// End value
    pub end: f32,
    /// Initial Value
    pub value: f32,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
            value: 0.0,
        }
    }
}

/// A bundle for convince when creating the widget.
#[derive(Bundle, Clone)]
pub struct SliderBundle {
    /// The slider
    pub slider: Slider,
    /// The collection of styles used by the slider
    pub slider_styles: SliderStyles,
    /// The internal children used by the slider
    pub children: WidgetChildren,
    /// The styles of the slider
    pub styles: WoodpeckerStyle,
    /// The render mode of the slider. Default: Quad
    pub render: WidgetRender,
    /// Change detection event
    pub on_changed: On<OnChange<SliderChanged>>,
    /// Provides overrides for picking behavior.
    pub pickable: Pickable,
    /// Tracks entity interaction state.
    pub interaction: PickingInteraction,
}

impl Default for SliderBundle {
    fn default() -> Self {
        Self {
            slider: Default::default(),
            slider_styles: Default::default(),
            children: Default::default(),
            styles: Default::default(),
            render: WidgetRender::Quad,
            on_changed: On::<OnChange<SliderChanged>>::run(|| {}),
            pickable: Default::default(),
            interaction: Default::default(),
        }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &Slider,
        &mut WoodpeckerStyle,
        &SliderStyles,
        &mut WidgetChildren,
        &WidgetLayout,
    )>,
    state_query: Query<&SliderState>,
) {
    let Ok((slider, mut styles, slider_styles, mut children, widget_layout)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let default_state = SliderState {
        value: slider.value,
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, default_state);

    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let slider_left = (widget_layout.size.x * state.value) - 3.0;

    *styles = slider_styles.bar;

    let widget_layout = *widget_layout;
    let current_widget = *current_widget;
    commands
        .entity(*current_widget)
        .insert(On::<Pointer<Click>>::run(
            move |event: Listener<Pointer<Click>>,
                  mut state_query: Query<&mut SliderState>,
                  mut event_writer: EventWriter<OnChange<SliderChanged>>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                state.value = (event.pointer_location.position.x - widget_layout.location.x)
                    / widget_layout.size.x;
                state.value = state.value.clamp(0.0, 1.0);
                event_writer.send(OnChange {
                    target: *current_widget,
                    data: SliderChanged { value: state.value },
                });
            },
        ));

    children.add::<Element>((
        ElementBundle {
            styles: WoodpeckerStyle {
                width: (slider_left + 10.0).into(),
                ..slider_styles.fill
            },
            ..Default::default()
        },
        WidgetRender::Quad,
    ));

    children.add::<WButton>((
        WButtonBundle {
            button_styles: ButtonStyles {
                normal: WoodpeckerStyle {
                    left: slider_left.into(),
                    ..slider_styles.button.normal
                },
                hovered: WoodpeckerStyle {
                    left: slider_left.into(),
                    ..slider_styles.button.hovered
                },
            },
            ..default()
        },
        On::<Pointer<Drag>>::run(
            move |event: ListenerMut<Pointer<Drag>>,
                  mut state_query: Query<&mut SliderState>,
                  mut event_writer: EventWriter<OnChange<SliderChanged>>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                state.value = (event.pointer_location.position.x - widget_layout.location.x)
                    / widget_layout.size.x;
                state.value = state.value.clamp(0.0, 1.0);
                event_writer.send(OnChange {
                    target: *current_widget,
                    data: SliderChanged { value: state.value },
                });
            },
        ),
    ));

    children.apply(current_widget.as_parent());
}
