use crate::prelude::*;
use bevy::prelude::*;

/// Fired when a different option is selected in a [`RadioGroup`].
#[derive(Debug, Clone, Reflect)]
pub struct RadioChanged {
    /// The index of the newly-selected option.
    pub index: usize,
    /// The newly-selected option's label.
    pub value: String,
}

/// A collection of styles for [`RadioGroup`].
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct RadioGroupStyles {
    /// Gap between options.
    pub gap: f32,
    /// Diameter of each radio dot, in logical pixels.
    pub dot_size: f32,
    /// Label text color.
    pub label_color: Color,
    /// Ring/fill color when an option is selected.
    pub selected_color: Color,
    /// Ring color when an option is not selected.
    pub unselected_color: Color,
}

impl Default for RadioGroupStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for RadioGroupStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            gap: 10.0,
            dot_size: 18.0,
            label_color: theme.text,
            selected_color: theme.primary,
            unselected_color: theme.border,
        }
    }
}

/// Per-widget state -- the currently-selected index, seeded from `RadioGroup::selected` on
/// mount and then owned by clicks from then on (the same "initial prop, then state takes
/// over" pattern used by `Dropdown`/`Checkbox`).
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct RadioGroupState {
    selected: usize,
    /// Previous state (Used for animations)
    previous_selected: usize,
    /// The dot-fill transition
    dot_transition: Transition,
}

fn get_transition() -> Transition {
    Transition {
        easing: TransitionEasing::QuadraticInOut,
        timeout: 150.0,
        playing: false,
        ..Default::default()
    }
}

/// A set of mutually-exclusive labeled options, rendered inline (not in a popup -- see
/// [`crate::widgets::Dropdown`] for a collapsed picker instead).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_style(), WidgetChildren, RadioGroupStyles)]
pub struct RadioGroup {
    /// The option labels.
    pub options: Vec<String>,
    /// Which option is selected initially.
    pub selected: usize,
    /// Lays out options in a row instead of the default column.
    pub horizontal: bool,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    theme: Res<Theme>,
    mut query: Query<(
        &RadioGroup,
        &RadioGroupStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    mut state_query: Query<&mut RadioGroupState>,
) {
    let Ok((radio_group, radio_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        RadioGroupState {
            selected: radio_group.selected,
            previous_selected: radio_group.selected,
            dot_transition: get_transition(),
        },
    );
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    if !state.dot_transition.is_playing() {
        state.dot_transition = Transition {
            playing: false,
            ..get_transition()
        };
    }
    let old_selected = state.previous_selected;
    if old_selected != state.selected {
        state.dot_transition.start();
        state.previous_selected = state.selected;
    }

    styles.flex_direction = if radio_group.horizontal {
        WidgetFlexDirection::Row
    } else {
        WidgetFlexDirection::Column
    };
    styles.gap = if radio_group.horizontal {
        (radio_styles.gap.into(), 0.0.into())
    } else {
        (0.0.into(), radio_styles.gap.into())
    };

    *children = WidgetChildren::default();
    let current_widget = *current_widget;

    for (index, option) in radio_group.options.iter().enumerate() {
        let selected = index == state.selected;
        let ring_color = if selected {
            radio_styles.selected_color
        } else {
            radio_styles.unselected_color
        };
        let dot_size = radio_styles.dot_size;
        let label_color = radio_styles.label_color;
        let option_value = option.clone();

        let dot_fill_visible = WoodpeckerStyle {
            width: (dot_size * 0.5).into(),
            height: (dot_size * 0.5).into(),
            background_color: radio_styles.selected_color,
            border_radius: Corner::all(dot_size),
            opacity: 1.0,
            ..Default::default()
        };
        let dot_fill_hidden = WoodpeckerStyle {
            opacity: 0.0,
            ..dot_fill_visible
        };
        let was_selected = index == old_selected;
        let dot_fill_transition = Transition {
            style_a: if was_selected {
                dot_fill_visible
            } else {
                dot_fill_hidden
            },
            style_b: if selected {
                dot_fill_visible
            } else {
                dot_fill_hidden
            },
            ..state.dot_transition
        };

        let mut dot_children = WidgetChildren::default();
        dot_children.add::<Element>((
            Element,
            if selected {
                dot_fill_visible
            } else {
                dot_fill_hidden
            },
            WidgetRender::Quad,
            dot_fill_transition,
        ));
        dot_children.add_key("fill");

        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            Pickable::default(),
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: dot_size.into(),
                        height: dot_size.into(),
                        border: Edge::all(2.0),
                        border_color: ring_color,
                        border_radius: Corner::all(dot_size),
                        margin: Edge::all(0.0).right(8.0),
                        justify_content: Some(WidgetAlignContent::Center),
                        align_items: Some(WidgetAlignItems::Center),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                    dot_children,
                ))
                .with_key("dot")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: theme.font_size,
                        color: label_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: option.clone(),
                    },
                ))
                .with_key("label"),
        ));
        children.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut RadioGroupState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if state.selected == index {
                    return;
                }
                state.selected = index;
                commands.trigger(Change {
                    target: *current_widget,
                    data: RadioChanged {
                        index,
                        value: option_value.clone(),
                    },
                });
            },
        );
        children.hover_cursor(current_widget, SystemCursorIcon::Pointer);
        children.add_key(option.clone());
    }

    children.apply(current_widget.as_parent());
}
