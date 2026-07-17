use crate::prelude::*;
use bevy::prelude::*;

/// One destination in a [`BottomNavigation`].
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct BottomNavItem {
    /// A [`crate::icons`] glyph shown above the label (rendered via [`IconFont`]).
    pub icon: String,
    /// The destination's label.
    pub label: String,
}

/// Fired when a different destination is selected -- mirrors [`crate::widgets::RadioChanged`]'s
/// shape, since the selection semantics are the same (mutually-exclusive, index-based).
#[derive(Debug, Clone, Reflect)]
pub struct BottomNavigationChanged {
    /// The index of the newly-selected item.
    pub index: usize,
    /// The newly-selected item's label.
    pub label: String,
}

/// [`BottomNavigation`]'s themed colors -- a separate sibling component so it live-resyncs on
/// a [`Theme`] swap.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct BottomNavigationStyles {
    /// Bar background color.
    pub background_color: Color,
    /// Dividing border color (top edge, matching `AppBarPosition::Bottom`).
    pub border_color: Color,
    /// Icon/label color for the selected item.
    pub selected_color: Color,
    /// Icon/label color for unselected items.
    pub unselected_color: Color,
}

impl Default for BottomNavigationStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for BottomNavigationStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.dark_background,
            border_color: theme.border,
            selected_color: theme.primary,
            unselected_color: theme.text.with_alpha(0.6),
        }
    }
}

/// Per-widget state -- the currently-selected index, seeded from `BottomNavigation::selected`
/// on mount and then owned by clicks from then on (the same "initial prop, then state takes
/// over" pattern `RadioGroup`/`Dropdown` use).
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct BottomNavigationState {
    selected: usize,
}

/// A fixed-height row of mutually-exclusive, evenly-spaced icon+label destinations, pinned to
/// the bottom of its container.
///
/// Doesn't nest an actual [`AppBar`] child, despite the visual similarity to
/// `AppBar { position: Bottom }`: `AppBar`'s leading/title/trailing slots are built for a
/// "content, flexible gap, actions" layout, not N evenly-distributed full-width items --
/// forcing that shape through `AppBar`'s slots (e.g. stuffing every item into `trailing`)
/// doesn't actually span the full bar width. Instead this reuses `AppBarPosition::Bottom`'s
/// exact chrome recipe (fixed height, dark background, top border) directly.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, BottomNavigationStyles)]
pub struct BottomNavigation {
    /// The destinations, in display order.
    pub items: Vec<BottomNavItem>,
    /// Which item is selected initially.
    pub selected: usize,
    /// Bar height, in pixels.
    pub height: f32,
}

impl Default for BottomNavigation {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            height: 56.0,
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
        &BottomNavigation,
        &BottomNavigationStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&BottomNavigationState>,
) {
    let Ok((nav, nav_styles, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        BottomNavigationState {
            selected: nav.selected,
        },
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    styles.width = Units::Percentage(100.0);
    styles.height = nav.height.into();
    styles.flex_direction = WidgetFlexDirection::Row;
    styles.background_color = nav_styles.background_color;
    styles.border_color = nav_styles.border_color;
    styles.border = Edge::all(0.0).top(1.0);

    *children = WidgetChildren::default();
    let current_widget = *current_widget;

    for (index, item) in nav.items.iter().enumerate() {
        let selected = index == state.selected;
        let color = if selected {
            nav_styles.selected_color
        } else {
            nav_styles.unselected_color
        };
        let item_label = item.label.clone();

        children
            .add::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_grow: 1.0,
                    height: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Column,
                    align_items: Some(WidgetAlignItems::Center),
                    justify_content: Some(WidgetAlignContent::Center),
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
                      mut state_query: Query<&mut BottomNavigationState>| {
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    if state.selected == index {
                        return;
                    }
                    state.selected = index;
                    commands.trigger(Change {
                        target: current_widget.entity(),
                        data: BottomNavigationChanged {
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
        let styles = BottomNavigationStyles::from_theme(&theme);
        assert_eq!(styles.background_color, theme.dark_background);
        assert_eq!(styles.selected_color, theme.primary);

        let dark = BottomNavigationStyles::from_theme(&Theme::dark());
        let light = BottomNavigationStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
