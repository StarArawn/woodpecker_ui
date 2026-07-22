use crate::prelude::*;
use bevy::prelude::*;

/// A thin column-flex wrapper for [`ListItem`]s. Generalizes the row-list pattern
/// `examples/dashboard/overview.rs`'s `activity_list` previously hand-built (a plain `Element`
/// with `flex_direction: Column`).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, PassedChildren)]
pub struct List;

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&mut WoodpeckerStyle, &mut WidgetChildren, &PassedChildren)>,
) {
    let Ok((mut styles, mut children, passed_children)) = query.get_mut(**current_widget) else {
        return;
    };

    styles.flex_direction = WidgetFlexDirection::Column;
    if styles.width == Units::Auto {
        styles.width = Units::Percentage(100.0);
    }

    *children = passed_children.0.clone();
    children.apply(current_widget.as_parent());
}

/// [`ListItem`]'s themed colors -- a separate sibling component so it live-resyncs on a
/// [`Theme`] swap.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ListItemStyles {
    /// Background when `ListItem::selected` is `true`.
    pub background_selected: Color,
    /// Background while hovered (and not selected).
    pub background_hover: Color,
    /// Primary text color.
    pub primary_text_color: Color,
    /// Secondary text color.
    pub secondary_text_color: Color,
    /// Bottom divider color between rows.
    pub divider_color: Color,
}

impl Default for ListItemStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ListItemStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_selected: theme.background_light,
            background_hover: theme.background_mid,
            primary_text_color: theme.text,
            secondary_text_color: theme.text.with_alpha(0.6),
            divider_color: theme.border,
        }
    }
}

#[derive(Component, Default, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ListItemState {
    hovering: bool,
}

/// Fired when a [`ListItem`] is clicked.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct ListItemClicked;

/// Optional leading content (e.g. an [`Avatar`](super::Avatar) or icon) shown before a
/// [`ListItem`]'s text. Absent by default.
#[derive(Component, Default, PartialEq, Clone)]
pub struct ListItemLeading(pub WidgetChildren);

/// Optional trailing content (e.g. a timestamp, [`Badge`](super::Badge), or action button)
/// shown after a [`ListItem`]'s text. Absent by default.
#[derive(Component, Default, PartialEq, Clone)]
pub struct ListItemTrailing(pub WidgetChildren);

/// One row within a [`List`] -- optional leading/trailing content around a primary/secondary
/// text pair, with hover/selected highlighting. Fires [`Change<ListItemClicked>`] on click.
/// Generalizes `examples/dashboard/overview.rs`'s `activity_row()`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(item_render)]
#[require(WoodpeckerStyle, WidgetChildren, Pickable, WidgetRender = WidgetRender::Quad, ListItemStyles)]
pub struct ListItem {
    /// The row's main text.
    pub primary_text: String,
    /// An optional smaller line under `primary_text`.
    pub secondary_text: Option<String>,
    /// Whether this row is the current selection.
    pub selected: bool,
    /// Uses tighter vertical padding when `true`.
    pub dense: bool,
}

fn item_render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    theme: Res<Theme>,
    mut query: Query<(
        &ListItem,
        &ListItemStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        Option<&ListItemLeading>,
        Option<&ListItemTrailing>,
    )>,
    state_query: Query<&ListItemState>,
) {
    let Ok((item, item_styles, mut styles, mut children, leading, trailing)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, ListItemState::default());
    let default_state = ListItemState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let current_widget = *current_widget;

    styles.width = Units::Percentage(100.0);
    styles.flex_direction = WidgetFlexDirection::Row;
    styles.align_items = Some(WidgetAlignItems::Center);
    styles.justify_content = Some(WidgetAlignContent::SpaceBetween);
    let vertical_padding = if item.dense {
        theme.spacing.xs
    } else {
        theme.spacing.sm
    };
    styles.padding = Edge::all(vertical_padding).left(0.0).right(0.0);
    styles.border = Edge::all(0.0).bottom(1.0);
    styles.border_color = item_styles.divider_color;
    styles.background_color = if item.selected {
        item_styles.background_selected
    } else if state.hovering {
        item_styles.background_hover
    } else {
        Color::NONE
    };

    *children = WidgetChildren::default();

    if let Some(leading) = leading {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(0.0).right(theme.spacing.sm),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            leading.0.clone(),
        ));
        children.add_key("leading");
    }

    let mut text_column = WidgetChildren::default();
    text_column.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: theme.typography.body,
            color: item_styles.primary_text_color,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: item.primary_text.clone(),
        },
    ));
    text_column.add_key("primary");
    if let Some(secondary_text) = &item.secondary_text {
        text_column.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: theme.typography.body_small,
                color: item_styles.secondary_text_color,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: secondary_text.clone(),
            },
        ));
        text_column.add_key("secondary");
    }
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Column,
            flex_grow: 1.0,
            ..Default::default()
        },
        text_column,
    ));
    children.add_key("text");

    if let Some(trailing) = trailing {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(0.0).left(theme.spacing.sm),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            trailing.0.clone(),
        ));
        children.add_key("trailing");
    }

    children.self_hover_state(
        current_widget,
        state_entity,
        SystemCursorIcon::Pointer,
        |state: &mut ListItemState, hovering| state.hovering = hovering,
    );
    children.self_observe(
        current_widget,
        move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
            commands.trigger(Change {
                target: current_widget.entity(),
                data: ListItemClicked,
            });
        },
    );

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = ListItemStyles::from_theme(&theme);
        assert_eq!(styles.background_selected, theme.background_light);
        assert_eq!(styles.primary_text_color, theme.text);
        assert_eq!(styles.secondary_text_color, theme.text.with_alpha(0.6));

        let dark = ListItemStyles::from_theme(&Theme::dark());
        let light = ListItemStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.primary_text_color, light.primary_text_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
