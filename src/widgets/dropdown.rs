use crate::{picking_backend::MouseWheelScroll, prelude::*};
use bevy::prelude::*;

/// A textbox change event.
#[derive(Debug, Clone, Reflect)]
pub struct DropdownChanged {
    /// The current text value
    pub value: String,
}

/// Dropdown Styles
#[derive(Component, Clone, PartialEq, Reflect)]
pub struct DropdownStyles {
    /// Dropdown Background Styles
    pub background: WoodpeckerStyle,
    /// Dropdown Text Styles
    pub text: WoodpeckerStyle,
    /// Dropdown Icon Styles
    pub icon: WoodpeckerStyle,
    /// Dropdown List Area Styles
    pub list_area: WoodpeckerStyle,
    /// Dropdown List Item Styles
    pub list_item: ButtonStyles,
    /// Border color drawn around the trigger while it has keyboard focus (e.g. via Tab) --
    /// the only visible indication that focus landed here at all, since nothing else about
    /// the trigger's appearance otherwise changes.
    pub focused_border_color: Color,
    /// Background applied to the keyboard-highlighted list item (`DropdownState`'s roving
    /// cursor, moved with Up/Down while open), over `list_item`'s own.
    pub highlighted_background: Color,
    /// Border color drawn around the keyboard-highlighted item -- the clearest of the three
    /// item states (rest/hover/highlighted), since it's also exactly what Enter/Space selects.
    pub highlighted_border_color: Color,
    /// Fixed height of a single list item, in logical pixels.
    pub item_extent: f32,
}

impl Default for DropdownStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for DropdownStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                background_color: theme.background,
                border: Edge::all(1.0),
                border_color: theme.border,
                border_radius: Corner::all(theme.control_radius),
                width: Units::Percentage(100.0),
                height: theme.control_height.into(),
                padding: Edge::all(0.0).left(12.0).right(8.0),
                ..Default::default()
            },
            text: WoodpeckerStyle {
                color: theme.text,
                font_size: theme.font_size,
                flex_grow: 1.0,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            icon: WoodpeckerStyle {
                color: theme.text,
                width: 16.0.into(),
                height: 16.0.into(),
                ..Default::default()
            },
            list_area: WoodpeckerStyle {
                background_color: theme.background,
                border: Edge::all(1.0),
                border_color: theme.border,
                border_radius: Corner::all(theme.control_radius),
                position: WidgetPosition::Absolute,
                left: 0.0.into(),
                top: (theme.control_height + 4.0).into(),
                width: Units::Percentage(100.0),
                min_height: theme.control_height.into(),
                flex_direction: WidgetFlexDirection::Column,
                padding: Edge::all(4.0),
                gap: (0.0.into(), 2.0.into()),
                box_shadow: Some(theme.elevation.md),
                ..Default::default()
            },
            list_item: {
                let normal = WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    background_color: Color::NONE,
                    width: Units::Percentage(100.0),
                    height: theme.control_height.into(),
                    font_size: theme.font_size,
                    padding: Edge::all(0.0).left(12.0).right(8.0),
                    border_radius: Corner::all(theme.control_radius * 0.7),
                    ..Default::default()
                };
                ButtonStyles {
                    normal,
                    hovered: WoodpeckerStyle {
                        background_color: theme.primary.with_alpha(0.12),
                        ..normal
                    },
                }
            },
            focused_border_color: theme.primary,
            highlighted_background: theme.primary.with_alpha(0.28),
            highlighted_border_color: theme.primary,
            item_extent: theme.control_height,
        }
    }
}

/// Dropdown state
#[derive(Default, Debug, Component, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct DropdownState {
    /// Is open?
    is_open: bool,
    /// `is_open` as of the last render -- lets `render` notice an open/close flip (`is_open`
    /// itself is mutated by observer closures, not by `render`) and kick off the list panel's
    /// `Transition` in the matching direction.
    previous_open: bool,
    /// The current value
    current_value: String,
    /// Whether the trigger currently has keyboard focus -- driven by `WidgetFocus`/
    /// `WidgetBlur` observers, the same pattern `TextBoxState::focused` uses, since reading
    /// `Res<CurrentFocus>` directly in `render` wouldn't itself trigger a re-render when focus
    /// changes (the generic diffing system only reacts to prop/state/context changes, not
    /// arbitrary resources).
    focused: bool,
    /// The keyboard-roving-cursor position within `Dropdown::list`, moved by Up/Down while
    /// open and confirmed by Enter/Space. `None` until the first Up/Down/open.
    highlighted_index: Option<usize>,
}

/// The list panel's open/close `Transition` preset -- a quick fade, not yet started.
fn list_transition() -> Transition {
    Transition {
        easing: TransitionEasing::CubicOut,
        timeout: 120.0,
        looping: false,
        playing: false,
        ..Default::default()
    }
}

const MAX_VISIBLE_ITEMS: usize = 8;
const DROPDOWN_OVERSCAN: usize = 4;
const DROPDOWN_MAX_OVERSCAN: usize = 64;
const SCROLL_LINE: f32 = 64.0;
const SCROLLBAR_THICKNESS: f32 = 10.0;

#[derive(Component, Default, Clone, PartialEq)]
struct DropdownScrollState {
    prev_scroll_offset: f32,
}

/// A dropdown widget
#[derive(Widget, Default, Component, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle,
    WidgetRender = WidgetRender::Quad,
    WidgetChildren,
    Pickable,
    Focusable,
    DropdownStyles,
    Transition = list_transition(),
    DropdownScrollState
)]
pub struct Dropdown {
    /// The current value
    pub current_value: String,
    /// A list of items in the dropdown
    pub list: Vec<String>,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    asset_server: Res<AssetServer>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut context_query: Query<&mut ScrollContext>,
    mut query: Query<(
        &Dropdown,
        &DropdownStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &WidgetLayout,
        &mut Transition,
        &mut DropdownScrollState,
    )>,
    mut state_query: Query<&mut DropdownState>,
) {
    let Ok((
        dropdown,
        dropdown_styles,
        mut styles,
        mut children,
        layout,
        mut transition,
        mut scroll_state,
    )) = query.get_mut(**current_widget)
    else {
        return;
    };
    let (trigger_loc, trigger_size) = (layout.location, layout.size);

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        DropdownState {
            is_open: false,
            current_value: dropdown.current_value.clone(),
            ..Default::default()
        },
    );

    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    *transition = Transition {
        reversing: !state.is_open,
        timeout: 120.0,
        ..*transition
    };
    if state.previous_open != state.is_open {
        if transition.reversing {
            transition.start_reverse();
        } else {
            transition.start();
        }
        state.previous_open = state.is_open;
    }

    let dropdown_entity = **current_widget;

    *styles = if state.focused {
        WoodpeckerStyle {
            border_color: dropdown_styles.focused_border_color,
            ..dropdown_styles.background
        }
    } else {
        dropdown_styles.background
    };
    transition.style_a = *styles;
    transition.style_b = *styles;

    *children = WidgetChildren::default()
        .with_observe(
            *current_widget,
            move |_trigger: On<Pointer<Click>>, mut state_query: Query<&mut DropdownState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                state.is_open = !state.is_open;
            },
        )
        .with_observe(
            *current_widget,
            move |_trigger: On<WidgetFocus>, mut state_query: Query<&mut DropdownState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.focused = true;
            },
        )
        .with_observe(
            *current_widget,
            move |_trigger: On<WidgetBlur>, mut state_query: Query<&mut DropdownState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.focused = false;
            },
        )
        .with_observe(
            *current_widget,
            move |trigger: On<WidgetKeyboardButtonEvent>,
                  mut commands: Commands,
                  dropdown_query: Query<&Dropdown>,
                  mut state_query: Query<&mut DropdownState>| {
                let Ok(dropdown) = dropdown_query.get(dropdown_entity) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if dropdown.list.is_empty() {
                    return;
                }
                match trigger.code {
                    KeyCode::ArrowDown => {
                        state.is_open = true;
                        state.highlighted_index = Some(match state.highlighted_index {
                            Some(i) => (i + 1).min(dropdown.list.len() - 1),
                            None => 0,
                        });
                    }
                    KeyCode::ArrowUp => {
                        state.is_open = true;
                        state.highlighted_index = Some(match state.highlighted_index {
                            Some(i) => i.saturating_sub(1),
                            None => dropdown.list.len() - 1,
                        });
                    }
                    KeyCode::Enter | KeyCode::Space => {
                        if state.is_open {
                            if let Some(value) = state
                                .highlighted_index
                                .and_then(|index| dropdown.list.get(index).cloned())
                            {
                                state.current_value.clone_from(&value);
                                state.is_open = false;
                                commands.trigger(Change {
                                    target: dropdown_entity,
                                    data: DropdownChanged { value },
                                });
                            } else {
                                state.is_open = false;
                            }
                        } else {
                            state.is_open = true;
                        }
                    }
                    KeyCode::Escape => {
                        state.is_open = false;
                    }
                    _ => {}
                }
            },
        )
        .with_self_hover_cursor(*current_widget, SystemCursorIcon::Pointer)
        // Text
        .with_child::<Element>((
            Element,
            dropdown_styles.text,
            WidgetRender::Text {
                content: state.current_value.clone(),
            },
        ))
        .with_key("text")
        // Icon
        .with_child::<Element>((
            Element,
            dropdown_styles.icon,
            WidgetRender::Svg {
                handle: if state.is_open {
                    asset_server.load("embedded://woodpecker_ui/embedded_assets/icons/arrow-up.svg")
                } else {
                    asset_server
                        .load("embedded://woodpecker_ui/embedded_assets/icons/arrow-down.svg")
                },
                color: Some(dropdown_styles.icon.color),
            },
        ))
        .with_key("icon");

    if transition.is_playing() || state.is_open {
        if let Some(overlay_root) = overlay_root.as_ref() {
            let mut overlay_children = WidgetChildren::default();

            overlay_children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    position: WidgetPosition::Fixed,
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    ..Default::default()
                },
                Pickable::default(),
            ));
            overlay_children
                .observe(*current_widget, |mut trigger: On<Pointer<Over>>| {
                    trigger.propagate(false);
                })
                .observe(*current_widget, |mut trigger: On<Pointer<Out>>| {
                    trigger.propagate(false);
                })
                .observe(
                    *current_widget,
                    move |mut trigger: On<Pointer<Click>>,
                          mut state_query: Query<&mut DropdownState>| {
                        info!("Clicked outside of the area!");
                        trigger.propagate(false);
                        let Ok(mut state) = state_query.get_mut(state_entity) else {
                            return;
                        };
                        state.is_open = false;
                    },
                );
            overlay_children.add_key("overlay");

            let item_extent = dropdown_styles.item_extent;
            let viewport_height = if dropdown.list.is_empty() {
                item_extent
            } else {
                dropdown.list.len().min(MAX_VISIBLE_ITEMS) as f32 * item_extent
            };
            let list_area_padding = dropdown_styles.list_area.padding;
            let list_area_height = viewport_height
                + list_area_padding.top.value_or(0.0)
                + list_area_padding.bottom.value_or(0.0);
            let context_entity =
                hooks.use_own_context(&mut commands, *current_widget, ScrollContext::default());
            let scroll_offset = context_query
                .get_mut(context_entity)
                .ok()
                .map(|mut context| {
                    context.scrollbox_width = trigger_size.x;
                    context.scrollbox_height = viewport_height;
                    context.content_width = trigger_size.x;
                    context.content_height = dropdown.list.len() as f32 * item_extent;
                    let current_scroll_y = context.scroll_y();
                    context.set_scroll_y(current_scroll_y);
                    (-context.scroll_y()).max(0.0)
                });
            let (row_range, lead_spacer, trail_spacer, scroll_offset) =
                if let Some(scroll_offset) = scroll_offset {
                    let overscan = dynamic_overscan(
                        scroll_state.prev_scroll_offset,
                        scroll_offset,
                        item_extent,
                        DROPDOWN_OVERSCAN,
                        DROPDOWN_MAX_OVERSCAN,
                    );
                    scroll_state.prev_scroll_offset = scroll_offset;
                    let window = compute_virtual_window(
                        scroll_offset,
                        viewport_height,
                        item_extent,
                        dropdown.list.len(),
                        overscan,
                    );
                    (
                        window.start..window.end,
                        window.lead_spacer,
                        window.trail_spacer,
                        scroll_offset,
                    )
                } else {
                    (0..0, 0.0, 0.0, 0.0)
                };

            let mut list_children = WidgetChildren::default();
            if lead_spacer > 0.0 {
                list_children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: lead_spacer.into(),
                        ..Default::default()
                    },
                ));
                list_children.add_key("lead_spacer");
            }
            for index in row_range {
                let item = &dropdown.list[index];
                let item_value = item.clone();
                let is_highlighted = state.highlighted_index == Some(index);
                let list_item_styles = if is_highlighted {
                    ButtonStyles {
                        normal: WoodpeckerStyle {
                            background_color: dropdown_styles.highlighted_background,
                            border: Edge::all(1.0),
                            border_color: dropdown_styles.highlighted_border_color,
                            ..dropdown_styles.list_item.normal
                        },
                        ..dropdown_styles.list_item
                    }
                } else {
                    dropdown_styles.list_item
                };
                list_children.add::<WButton>((
                    WButton,
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        dropdown_styles.text,
                        WidgetRender::Text {
                            content: item.clone(),
                        },
                    )),
                    list_item_styles,
                ));
                list_children.add_key(item.clone());
                list_children.observe(
                    *current_widget,
                    move |mut trigger: On<Pointer<Click>>,
                          mut commands: Commands,
                          mut state_query: Query<&mut DropdownState>| {
                        trigger.propagate(false);
                        let Ok(mut state) = state_query.get_mut(state_entity) else {
                            return;
                        };
                        state.current_value.clone_from(&item_value);
                        state.is_open = false;
                        state.highlighted_index = Some(index);
                        commands.trigger(Change {
                            target: dropdown_entity,
                            data: DropdownChanged {
                                value: state.current_value.clone(),
                            },
                        });
                    },
                );
            }
            if trail_spacer > 0.0 {
                list_children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: trail_spacer.into(),
                        ..Default::default()
                    },
                ));
                list_children.add_key("trail_spacer");
            }

            let list_area_style = WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                left: trigger_loc.x.into(),
                top: (trigger_loc.y + trigger_size.y + 4.0).into(),
                width: trigger_size.x.into(),
                height: list_area_height.into(),
                opacity: 1.0,
                flex_direction: WidgetFlexDirection::Row,
                ..dropdown_styles.list_area
            };
            const SLIDE_OFFSET: f32 = 6.0;
            let list_area_transition = Transition {
                style_a: WoodpeckerStyle {
                    opacity: 0.0,
                    top: (trigger_loc.y + trigger_size.y + 4.0 - SLIDE_OFFSET).into(),
                    ..list_area_style
                },
                style_b: list_area_style,
                ..*transition
            };
            overlay_children
                .add::<Element>((
                    Element,
                    list_area_style,
                    WidgetChildren::default()
                        .with_child::<Clip>((
                            Clip,
                            WoodpeckerStyle {
                                width: Units::Percentage(100.0),
                                height: Units::Percentage(100.0),
                                overflow: WidgetOverflow::Clip,
                                border_radius: dropdown_styles.list_area.border_radius,
                                ..Default::default()
                            },
                            WidgetChildren::default().with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    position: WidgetPosition::Absolute,
                                    width: Units::Percentage(100.0),
                                    top: (-scroll_offset).into(),
                                    flex_direction: WidgetFlexDirection::Column,
                                    ..Default::default()
                                },
                                list_children,
                            )),
                        ))
                        .with_child::<ScrollBar>(ScrollBar {
                            thickness: SCROLLBAR_THICKNESS,
                            ..Default::default()
                        }),
                    WidgetRender::Quad,
                    list_area_transition,
                    StackingContext,
                ))
                .observe(
                    *current_widget,
                    move |mut trigger: On<Pointer<MouseWheelScroll>>,
                          mut context_query: Query<&mut ScrollContext>| {
                        trigger.propagate(false);
                        let Ok(mut context) = context_query.get_mut(context_entity) else {
                            return;
                        };
                        let scroll_y = context.scroll_y();
                        context.set_scroll_y(scroll_y + trigger.scroll.y * SCROLL_LINE);
                    },
                );
            overlay_children.add_key("list_area");

            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    z_index: Some(WidgetZ::Global(StackingTier::Dropdown as u32)),
                    ..Default::default()
                },
                overlay_children,
            ));
            children.add_key("overlay_wrapper");
            children.portal_to(overlay_root.0);
        }
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtualized_window_is_much_smaller_than_a_large_list() {
        let item_extent = DropdownStyles::from_theme(&Theme::default()).item_extent;
        let list_len = 1_000;
        let viewport_height = list_len.min(MAX_VISIBLE_ITEMS) as f32 * item_extent;

        let window = compute_virtual_window(
            0.0,
            viewport_height,
            item_extent,
            list_len,
            DROPDOWN_OVERSCAN,
        );

        assert!(window.end - window.start <= MAX_VISIBLE_ITEMS + DROPDOWN_OVERSCAN);
    }
}
