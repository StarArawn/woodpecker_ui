use crate::{picking_backend::MouseWheelScroll, prelude::*};
use bevy::prelude::*;

/// How [`ComboBox`] matches its typed filter text against each item, case-insensitively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum ComboBoxMatchMode {
    /// The item contains the filter text anywhere.
    #[default]
    Contains,
    /// The item starts with the filter text.
    StartsWith,
}

impl ComboBoxMatchMode {
    fn matches(self, item: &str, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let item = item.to_lowercase();
        let filter = filter.to_lowercase();
        match self {
            Self::Contains => item.contains(&filter),
            Self::StartsWith => item.starts_with(&filter),
        }
    }
}

/// Fired when an item is chosen -- a list click, or Enter with an item highlighted.
#[derive(Debug, Clone, Reflect)]
pub struct ComboBoxChanged {
    /// The chosen item's text.
    pub value: String,
}

/// A collection of styles for [`ComboBox`].
#[derive(Component, Clone, PartialEq, Reflect)]
pub struct ComboBoxStyles {
    /// The trigger row's own background/border (`ComboBox`'s own `WoodpeckerStyle`).
    pub background: WoodpeckerStyle,
    /// The embedded filter `TextBox`'s styles.
    pub trigger: TextboxStyles,
    /// The open/close disclosure icon's styles.
    pub icon: WoodpeckerStyle,
    /// The dropdown list panel's styles.
    pub list_area: WoodpeckerStyle,
    /// A single list item's styles.
    pub list_item: ButtonStyles,
    /// A list item's own text -- deliberately separate from `trigger.normal` (which carries a
    /// fixed `height: control_height` meant for the trigger `TextBox`'s own row). Reusing that
    /// height here made a highlighted item's fixed-height text child overflow its row the
    /// moment the highlighted row also grew a border (border-box sizing shrinks the row's own
    /// content box, but a child's *fixed* pixel height doesn't shrink to match).
    pub item_text: WoodpeckerStyle,
    /// Background applied to the keyboard-highlighted item, over `list_item`'s own.
    pub highlighted_background: Color,
    /// Border color drawn around the keyboard-highlighted item -- the clearest of the three
    /// item states (rest/hover/highlighted), since it's also exactly what Enter would select.
    pub highlighted_border_color: Color,
    /// Fixed height of a single list item, in logical pixels.
    pub item_extent: f32,
}

impl Default for ComboBoxStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ComboBoxStyles {
    fn from_theme(theme: &Theme) -> Self {
        let background = WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            align_items: Some(WidgetAlignItems::Center),
            background_color: theme.background,
            border: Edge::all(1.0),
            border_color: theme.border,
            border_radius: Corner::all(theme.control_radius),
            width: Units::Percentage(100.0),
            height: theme.control_height.into(),
            padding: Edge::all(0.0).left(12.0).right(8.0),
            ..Default::default()
        };
        let trigger_shared = WoodpeckerStyle {
            flex_grow: 1.0,
            height: theme.control_height.into(),
            background_color: Color::NONE,
            border: Edge::all(0.0),
            color: theme.text,
            font_size: theme.font_size,
            ..Default::default()
        };
        Self {
            background,
            trigger: TextboxStyles {
                normal: trigger_shared,
                hovered: trigger_shared,
                focused: trigger_shared,
                cursor: WoodpeckerStyle {
                    background_color: theme.primary,
                    position: WidgetPosition::Absolute,
                    width: 2.0.into(),
                    ..Default::default()
                },
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
            item_text: WoodpeckerStyle {
                color: theme.text,
                font_size: theme.font_size,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            highlighted_background: theme.primary.with_alpha(0.28),
            highlighted_border_color: theme.primary,
            item_extent: theme.control_height,
        }
    }
}

/// Self-managed state, mirroring [`crate::widgets::dropdown::DropdownState`]'s "initial prop,
/// then state takes over" shape.
#[derive(Component, Debug, Clone, PartialEq, Reflect, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ComboBoxState {
    is_open: bool,
    /// `is_open` as of the last render -- see `DropdownState::previous_open`'s doc comment for
    /// why this is needed (mirrors the identical pattern there).
    previous_open: bool,
    filter_text: String,
    highlighted_index: Option<usize>,
}

const MAX_VISIBLE_ITEMS: usize = 8;
const COMBO_BOX_OVERSCAN: usize = 4;
const COMBO_BOX_MAX_OVERSCAN: usize = 64;
const SCROLL_LINE: f32 = 64.0;
const SCROLLBAR_THICKNESS: f32 = 10.0;

#[derive(Component, Default, Clone, PartialEq)]
struct ComboBoxScrollState {
    prev_scroll_offset: f32,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle::default()
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

/// A searchable/autocomplete picker: `Dropdown`'s exact trigger/overlay/list-panel structure,
/// with the static-text trigger swapped for a live `TextBox` that filters the list as you
/// type. Fires `Change<ComboBoxChanged>` when an item is chosen (list click, or Enter with an
/// item keyboard-highlighted).
///
/// The one structurally novel thing here versus `Dropdown`: the trigger is a real, stateful
/// `TextBox` widget nested inside `ComboBox::render` (not a plain `Element`), so keyboard focus
/// naturally lands on the `TextBox` itself while typing -- meaning the Up/Down/Enter/Escape
/// list-navigation observers below are attached directly to that nested `TextBox` child (see
/// `WidgetChildren::observe`'s "last added child" scoping), not to `ComboBox`'s own root,
/// since `WidgetKeyboardButtonEvent` targets whatever is actually focused and does not bubble.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_style(), WidgetRender = WidgetRender::Quad, WidgetChildren, ComboBoxStyles, Transition = list_transition(), ComboBoxScrollState)]
pub struct ComboBox {
    /// The current value shown/typed in the trigger.
    pub current_value: String,
    /// The full, unfiltered list of choices.
    pub list: Vec<String>,
    /// How typed text filters `list`.
    pub match_mode: ComboBoxMatchMode,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    asset_server: Res<AssetServer>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut context_query: Query<&mut ScrollContext>,
    mut query: Query<(
        &ComboBox,
        &ComboBoxStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &WidgetLayout,
        &mut Transition,
        &mut ComboBoxScrollState,
    )>,
    mut state_query: Query<&mut ComboBoxState>,
) {
    let Ok((
        combo_box,
        combo_styles,
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
        ComboBoxState {
            is_open: false,
            previous_open: false,
            filter_text: combo_box.current_value.clone(),
            highlighted_index: None,
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

    *styles = combo_styles.background;
    transition.style_a = *styles;
    transition.style_b = *styles;

    let match_mode = combo_box.match_mode;
    let filtered: Vec<String> = combo_box
        .list
        .iter()
        .filter(|item| match_mode.matches(item, &state.filter_text))
        .cloned()
        .collect();

    let current_widget_val = *current_widget;
    let combo_entity = current_widget_val.0;

    *children = WidgetChildren::default();
    children.observe(
        current_widget_val,
        move |_: On<Pointer<Click>>, mut state_query: Query<&mut ComboBoxState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.is_open = true;
        },
    );

    children
        .add::<TextBox>((
            TextBox {
                initial_value: state.filter_text.clone(),
                ..Default::default()
            },
            combo_styles.trigger.clone(),
        ))
        .observe(
            current_widget_val,
            move |trigger: On<Change<TextChanged>>, mut state_query: Query<&mut ComboBoxState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.filter_text.clone_from(&trigger.data.value);
                state.is_open = true;
                state.highlighted_index = None;
            },
        )
        .observe(
            current_widget_val,
            move |trigger: On<WidgetKeyboardButtonEvent>,
                  mut commands: Commands,
                  combo_query: Query<&ComboBox>,
                  mut state_query: Query<&mut ComboBoxState>| {
                let Ok(combo_box) = combo_query.get(combo_entity) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let match_mode = combo_box.match_mode;
                let filtered: Vec<&String> = combo_box
                    .list
                    .iter()
                    .filter(|item| match_mode.matches(item, &state.filter_text))
                    .collect();

                match trigger.code {
                    KeyCode::ArrowDown => {
                        if filtered.is_empty() {
                            return;
                        }
                        state.is_open = true;
                        state.highlighted_index = Some(match state.highlighted_index {
                            Some(i) => (i + 1).min(filtered.len() - 1),
                            None => 0,
                        });
                    }
                    KeyCode::ArrowUp => {
                        if filtered.is_empty() {
                            return;
                        }
                        state.is_open = true;
                        state.highlighted_index = Some(match state.highlighted_index {
                            Some(i) => i.saturating_sub(1),
                            None => filtered.len() - 1,
                        });
                    }
                    KeyCode::Enter => {
                        let Some(index) = state.highlighted_index else {
                            return;
                        };
                        let Some(value) = filtered.get(index).map(|s| s.to_string()) else {
                            return;
                        };
                        state.filter_text.clone_from(&value);
                        state.is_open = false;
                        state.highlighted_index = None;
                        commands.trigger(Change {
                            target: combo_entity,
                            data: ComboBoxChanged { value },
                        });
                    }
                    KeyCode::Escape => {
                        state.is_open = false;
                        state.highlighted_index = None;
                    }
                    _ => {}
                }
            },
        );
    children.add_key("trigger");

    children
        .add::<Element>((
            Element,
            combo_styles.icon,
            WidgetRender::Svg {
                handle: if state.is_open {
                    asset_server.load("embedded://woodpecker_ui/embedded_assets/icons/arrow-up.svg")
                } else {
                    asset_server
                        .load("embedded://woodpecker_ui/embedded_assets/icons/arrow-down.svg")
                },
                color: Some(combo_styles.icon.color),
            },
            Pickable::default(),
        ))
        .observe(
            current_widget_val,
            move |mut trigger: On<Pointer<Click>>, mut state_query: Query<&mut ComboBoxState>| {
                trigger.propagate(false);
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_open = !state.is_open;
            },
        );
    children.add_key("icon");

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
                .observe(current_widget_val, |mut trigger: On<Pointer<Over>>| {
                    trigger.propagate(false);
                })
                .observe(current_widget_val, |mut trigger: On<Pointer<Out>>| {
                    trigger.propagate(false);
                })
                .observe(
                    current_widget_val,
                    move |mut trigger: On<Pointer<Click>>,
                          mut state_query: Query<&mut ComboBoxState>| {
                        trigger.propagate(false);
                        let Ok(mut state) = state_query.get_mut(state_entity) else {
                            return;
                        };
                        state.is_open = false;
                    },
                );
            overlay_children.add_key("overlay");

            let item_extent = combo_styles.item_extent;
            let viewport_height = if filtered.is_empty() {
                item_extent
            } else {
                filtered.len().min(MAX_VISIBLE_ITEMS) as f32 * item_extent
            };
            let list_area_padding = combo_styles.list_area.padding;
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
                    context.content_height = filtered.len() as f32 * item_extent;
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
                        COMBO_BOX_OVERSCAN,
                        COMBO_BOX_MAX_OVERSCAN,
                    );
                    scroll_state.prev_scroll_offset = scroll_offset;
                    let window = compute_virtual_window(
                        scroll_offset,
                        viewport_height,
                        item_extent,
                        filtered.len(),
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
                let item = &filtered[index];
                let item_value = item.clone();
                let highlighted = state.highlighted_index == Some(index);
                list_children.add::<WButton>((
                    WButton,
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        combo_styles.item_text,
                        WidgetRender::Text {
                            content: item.clone(),
                        },
                    )),
                    ButtonStyles {
                        normal: if highlighted {
                            WoodpeckerStyle {
                                background_color: combo_styles.highlighted_background,
                                border: Edge::all(1.0),
                                border_color: combo_styles.highlighted_border_color,
                                ..combo_styles.list_item.normal
                            }
                        } else {
                            combo_styles.list_item.normal
                        },
                        hovered: combo_styles.list_item.hovered,
                    },
                ));
                list_children.add_key(item.clone());
                list_children.observe(
                    current_widget_val,
                    move |mut trigger: On<Pointer<Click>>,
                          mut commands: Commands,
                          mut state_query: Query<&mut ComboBoxState>| {
                        trigger.propagate(false);
                        let Ok(mut state) = state_query.get_mut(state_entity) else {
                            return;
                        };
                        state.filter_text.clone_from(&item_value);
                        state.is_open = false;
                        state.highlighted_index = None;
                        commands.trigger(Change {
                            target: combo_entity,
                            data: ComboBoxChanged {
                                value: item_value.clone(),
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
                ..combo_styles.list_area
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
                                border_radius: combo_styles.list_area.border_radius,
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
                    current_widget_val,
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
            overlay_children.add_key("list");

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

    children.apply(current_widget_val.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtualized_window_is_much_smaller_than_a_large_filtered_list() {
        let item_extent = ComboBoxStyles::from_theme(&Theme::default()).item_extent;
        let filtered_len = 1_000;
        let viewport_height = filtered_len.min(MAX_VISIBLE_ITEMS) as f32 * item_extent;

        let window = compute_virtual_window(
            0.0,
            viewport_height,
            item_extent,
            filtered_len,
            COMBO_BOX_OVERSCAN,
        );

        assert!(window.end - window.start <= MAX_VISIBLE_ITEMS + COMBO_BOX_OVERSCAN);
    }

    #[test]
    fn empty_filter_matches_everything() {
        assert!(ComboBoxMatchMode::Contains.matches("Rust", ""));
        assert!(ComboBoxMatchMode::StartsWith.matches("Rust", ""));
    }

    #[test]
    fn contains_matches_anywhere_case_insensitively() {
        assert!(ComboBoxMatchMode::Contains.matches("TypeScript", "script"));
        assert!(ComboBoxMatchMode::Contains.matches("TypeScript", "TYPE"));
        assert!(!ComboBoxMatchMode::Contains.matches("Rust", "zig"));
    }

    #[test]
    fn starts_with_only_matches_the_prefix_case_insensitively() {
        assert!(ComboBoxMatchMode::StartsWith.matches("JavaScript", "java"));
        assert!(!ComboBoxMatchMode::StartsWith.matches("TypeScript", "script"));
    }
}
