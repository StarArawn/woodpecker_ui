use crate::prelude::*;
use bevy::prelude::*;

/// One destination in a [`NavigationRail`].
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct NavigationRailItem {
    /// A [`crate::icons`] glyph (rendered via [`IconFont`]).
    pub icon: String,
    /// The destination's label, shown as a small caption under the icon.
    pub label: String,
}

/// Fired when a different destination is selected -- mirrors
/// [`crate::widgets::BottomNavigationChanged`]'s shape (the selection semantics are identical,
/// just vertical instead of horizontal).
#[derive(Debug, Clone, Reflect)]
pub struct NavigationRailChanged {
    /// The index of the newly-selected item.
    pub index: usize,
    /// The newly-selected item's label.
    pub label: String,
}

/// [`NavigationRail`]'s themed colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct NavigationRailStyles {
    /// Rail background color.
    pub background_color: Color,
    /// Dividing border color (right edge).
    pub border_color: Color,
    /// Icon/label color for the selected item.
    pub selected_color: Color,
    /// Icon/label color for unselected items.
    pub unselected_color: Color,
    /// Background highlight behind the selected item's icon.
    pub selected_background: Color,
}

impl Default for NavigationRailStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for NavigationRailStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.dark_background,
            border_color: theme.border,
            selected_color: theme.primary,
            unselected_color: theme.text.with_alpha(0.6),
            selected_background: theme.background_light,
        }
    }
}

/// Per-widget state -- the currently-selected index, seeded from `NavigationRail::selected` on
/// mount and then owned by clicks from then on (same "initial prop, then state takes over"
/// pattern `BottomNavigation`/`RadioGroup` use).
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct NavigationRailState {
    selected: usize,
}

/// A narrow, icon-only, vertical single-select navigation column -- e.g. a compact app-switcher
/// rail alongside a wider content area. Distinct from [`crate::widgets::Drawer`]'s own
/// `Persistent` variant: a `Drawer` is a wide (200px+) text-labeled nav list meant to be the
/// *primary* navigation, where a rail is a narrow (72px), icon-only, often-secondary strip
/// (matching MUI's own `NavigationRail` vs. a persistent `Drawer` distinction) -- reuses
/// `BottomNavigation`'s selection semantics rotated 90 degrees rather than composing `Drawer`.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, NavigationRailStyles)]
pub struct NavigationRail {
    /// The destinations, in display order.
    pub items: Vec<NavigationRailItem>,
    /// Which item is selected initially.
    pub selected: usize,
    /// Rail width, in pixels.
    pub width: f32,
}

impl Default for NavigationRail {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            width: 72.0,
        }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    theme: Res<Theme>,
    icon_font: Res<IconFont>,
    mut query: Query<(
        &NavigationRail,
        &NavigationRailStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&NavigationRailState>,
) {
    let Ok((rail, rail_styles, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        NavigationRailState {
            selected: rail.selected,
        },
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    styles.width = rail.width.into();
    styles.height = Units::Percentage(100.0);
    styles.flex_direction = WidgetFlexDirection::Column;
    styles.align_items = Some(WidgetAlignItems::Center);
    styles.padding = Edge::all(theme.spacing.sm).top(theme.spacing.md);
    styles.background_color = rail_styles.background_color;
    styles.border_color = rail_styles.border_color;
    styles.border = Edge::all(0.0).right(1.0);

    *children = WidgetChildren::default();
    let current_widget = *current_widget;

    for (index, item) in rail.items.iter().enumerate() {
        let selected = index == state.selected;
        let color = if selected {
            rail_styles.selected_color
        } else {
            rail_styles.unselected_color
        };
        let item_label = item.label.clone();

        children
            .add::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Column,
                    align_items: Some(WidgetAlignItems::Center),
                    padding: Edge::all(theme.spacing.xs),
                    margin: Edge::all(0.0).bottom(theme.spacing.xs),
                    background_color: if selected {
                        rail_styles.selected_background
                    } else {
                        Color::NONE
                    },
                    border_radius: Corner::all(theme.control_radius),
                    ..Default::default()
                },
                Pickable::default(),
                WidgetChildren::default()
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: theme.typography.h3,
                            color,
                            font: Some(icon_font.0.id()),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: item.icon.clone(),
                        },
                    ))
                    .with_key("icon")
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: theme.typography.caption,
                            color,
                            margin: Edge::all(0.0).top(theme.spacing.xs),
                            text_wrap: TextWrap::None,
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: item.label.clone(),
                        },
                    ))
                    .with_key("label"),
            ))
            .observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>,
                      mut commands: Commands,
                      mut state_query: Query<&mut NavigationRailState>| {
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    if state.selected == index {
                        return;
                    }
                    state.selected = index;
                    commands.trigger(Change {
                        target: current_widget.entity(),
                        data: NavigationRailChanged {
                            index,
                            label: item_label.clone(),
                        },
                    });
                },
            )
            .hover_cursor(current_widget, SystemCursorIcon::Pointer);
        children.add_key(item.label.clone());
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = NavigationRailStyles::from_theme(&theme);
        assert_eq!(styles.background_color, theme.dark_background);
        assert_eq!(styles.selected_color, theme.primary);

        let dark = NavigationRailStyles::from_theme(&Theme::dark());
        let light = NavigationRailStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn default_navigation_rail_is_72px_wide() {
        assert_eq!(NavigationRail::default().width, 72.0);
    }
}
