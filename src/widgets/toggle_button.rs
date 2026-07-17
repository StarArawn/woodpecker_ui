use crate::prelude::*;
use bevy::prelude::*;

/// Fired when a [`ToggleButton`] is clicked. `ToggleButton` has no internal selection state of
/// its own (same "caller owns it" model [`crate::widgets::Chip`] uses for `selected`) -- a
/// caller's own observer flips `ToggleButton::selected` in response.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct ToggleButtonChanged {
    /// What `selected` would become if the caller accepts this click.
    pub selected: bool,
}

/// [`ToggleButton`]'s themed styles -- forks [`ButtonStyles`]' own normal/hovered shape with
/// two more variants for the selected state.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ToggleButtonStyles {
    /// Unselected, not hovered.
    pub normal: WoodpeckerStyle,
    /// Unselected, hovered.
    pub hovered: WoodpeckerStyle,
    /// Selected, not hovered.
    pub selected: WoodpeckerStyle,
    /// Selected, hovered.
    pub selected_hovered: WoodpeckerStyle,
}

impl Default for ToggleButtonStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ToggleButtonStyles {
    fn from_theme(theme: &Theme) -> Self {
        let normal = WoodpeckerStyle {
            background_color: theme.background_mid,
            border_color: theme.border,
            border: Edge::all(1.0),
            border_radius: Corner::all(theme.control_radius),
            padding: Edge::all(0.0).left(16.0).right(16.0),
            font_size: theme.font_size,
            height: theme.control_height.into(),
            color: theme.text,
            text_alignment: Some(TextAlign::Center),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        };
        let selected = WoodpeckerStyle {
            background_color: theme.primary,
            border_color: theme.primary,
            color: Color::WHITE,
            ..normal
        };
        Self {
            normal,
            hovered: WoodpeckerStyle {
                background_color: theme.background_light,
                border_color: theme.primary,
                ..normal
            },
            selected,
            selected_hovered: WoodpeckerStyle {
                background_color: theme.primary.with_alpha(0.85),
                ..selected
            },
        }
    }
}

#[derive(Component, Default, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ToggleButtonState {
    hovering: bool,
}

/// A button with a persistent on/off visual state -- MUI's own `ToggleButton` (e.g. bold/
/// italic/underline formatting controls, a single "favorite" toggle). For a *set* of
/// mutually-exclusive segments sharing one border, see [`ButtonGroup`] instead of composing
/// several `ToggleButton`s by hand.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WidgetRender = WidgetRender::Quad,
    WidgetChildren,
    PassedChildren,
    WoodpeckerStyle = ToggleButtonStyles::default().normal,
    Pickable,
    ToggleButtonStyles
)]
pub struct ToggleButton {
    /// Whether this button is currently "on".
    pub selected: bool,
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &ToggleButton,
        &ToggleButtonStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &PassedChildren,
    )>,
    state_query: Query<&ToggleButtonState>,
) {
    let Ok((toggle_button, toggle_styles, mut styles, mut children, passed_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity =
        hooks.use_state(&mut commands, *current_widget, ToggleButtonState::default());
    let default_state = ToggleButtonState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);
    let current_widget = *current_widget;
    let selected = toggle_button.selected;
    let widget_entity = current_widget.entity();

    *styles = match (selected, state.hovering) {
        (true, true) => toggle_styles.selected_hovered,
        (true, false) => toggle_styles.selected,
        (false, true) => toggle_styles.hovered,
        (false, false) => toggle_styles.normal,
    };

    *children = passed_children.0.clone();
    children
        .self_hover_state(
            current_widget,
            state_entity,
            SystemCursorIcon::Pointer,
            |state: &mut ToggleButtonState, hovering| state.hovering = hovering,
        )
        .self_observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  toggle_button_query: Query<&ToggleButton>| {
                let Ok(toggle_button) = toggle_button_query.get(widget_entity) else {
                    return;
                };
                commands.trigger(Change {
                    target: widget_entity,
                    data: ToggleButtonChanged {
                        selected: !toggle_button.selected,
                    },
                });
            },
        );

    children.apply(current_widget.as_parent());
}

/// Fired when [`ButtonGroup`] selects a new segment.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct ButtonGroupChanged {
    /// The newly-selected segment's index.
    pub index: usize,
}

/// A row of mutually-exclusive segments sharing one visual border -- e.g. a "Day / Week /
/// Month" view switcher. A self-contained, closed option-set widget (its segments are built
/// from `options` directly) rather than a generic wrapper around arbitrary `ToggleButton`
/// children: this crate's children are opaque bundles once declared, so a parent can't reach
/// into a child's own style to strip its inner border/round only the outer corners the way
/// real MUI `ButtonGroup` does via CSS adjacency -- building the segments internally is what
/// actually makes the shared-border look possible. Fires [`Change<ButtonGroupChanged>`] on
/// selecting a new segment; the caller owns `selected`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_group)]
#[require(
    WoodpeckerStyle = WoodpeckerStyle { flex_direction: WidgetFlexDirection::Row, ..Default::default() },
    WidgetChildren,
    ToggleButtonStyles
)]
pub struct ButtonGroup {
    /// Segment labels, in order.
    pub options: Vec<String>,
    /// The currently-selected segment's index.
    pub selected: usize,
}

fn render_group(
    theme: Res<Theme>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&ButtonGroup, &ToggleButtonStyles, &mut WidgetChildren)>,
) {
    let Ok((group, toggle_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    let last = group.options.len().saturating_sub(1);

    for (index, label) in group.options.iter().enumerate() {
        let selected = index == group.selected;
        let base = if selected {
            toggle_styles.selected
        } else {
            toggle_styles.normal
        };
        let corner_radius = Corner::all(0.0)
            .top_left(if index == 0 {
                theme.control_radius
            } else {
                0.0
            })
            .bottom_left(if index == 0 {
                theme.control_radius
            } else {
                0.0
            })
            .top_right(if index == last {
                theme.control_radius
            } else {
                0.0
            })
            .bottom_right(if index == last {
                theme.control_radius
            } else {
                0.0
            });

        children
            .add::<Element>((
                Element,
                WoodpeckerStyle {
                    border_radius: corner_radius,
                    // Every segment but the first skips its own left border -- the previous
                    // segment's right border already draws that shared edge, avoiding a
                    // double-thickness seam between segments.
                    border: if index == 0 {
                        base.border
                    } else {
                        base.border.left(0.0)
                    },
                    ..base
                },
                WidgetRender::Quad,
                Pickable::default(),
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: base.font_size,
                        color: base.color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: label.clone(),
                    },
                )),
            ))
            .hover_cursor(current_widget, SystemCursorIcon::Pointer)
            .observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                    commands.trigger(Change {
                        target: current_widget.entity(),
                        data: ButtonGroupChanged { index },
                    });
                },
            );
        children.add_key(format!("segment-{index}"));
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = ToggleButtonStyles::from_theme(&theme);
        assert_eq!(styles.selected.background_color, theme.primary);
        assert_eq!(styles.normal.background_color, theme.background_mid);

        let dark = ToggleButtonStyles::from_theme(&Theme::dark());
        let light = ToggleButtonStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.normal.background_color, light.normal.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn default_toggle_button_is_unselected() {
        assert!(!ToggleButton::default().selected);
    }
}
