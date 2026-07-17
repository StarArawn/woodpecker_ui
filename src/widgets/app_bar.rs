use crate::prelude::*;
use bevy::prelude::*;

/// Which edge of its container an [`AppBar`] sits against -- purely affects which side gets
/// the dividing border (`Top` borders its bottom edge, `Bottom` borders its top edge), not
/// actual screen positioning: an `AppBar` renders inline wherever its caller places it (the
/// same normal-flow placement `examples/dashboard/topbar.rs`'s hand-built topbar `Element`
/// already used), rather than `position: Fixed`.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AppBarPosition {
    /// Sits at the top of its container; border on the bottom edge.
    #[default]
    Top,
    /// Sits at the bottom of its container; border on the top edge.
    Bottom,
}

/// [`AppBar`]'s themed colors -- a separate sibling component so it live-resyncs on a
/// [`Theme`] swap.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct AppBarStyles {
    /// Bar background color.
    pub background_color: Color,
    /// Dividing border color.
    pub border_color: Color,
    /// Title text color.
    pub title_color: Color,
}

impl Default for AppBarStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for AppBarStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.dark_background,
            border_color: theme.border,
            title_color: theme.text,
        }
    }
}

/// Optional leading content (e.g. a menu button) shown before an [`AppBar`]'s title. Absent
/// by default.
#[derive(Component, Default, PartialEq, Clone)]
pub struct AppBarLeading(pub WidgetChildren);

/// Optional trailing content (e.g. actions, an avatar) shown after an [`AppBar`]'s title.
/// Absent by default.
#[derive(Component, Default, PartialEq, Clone)]
pub struct AppBarTrailing(pub WidgetChildren);

/// A fixed-height chrome bar with leading/title/trailing slots -- generalizes the row-flex
/// `Element` pattern `examples/dashboard/topbar.rs` previously hand-built, and the shell
/// [`crate::widgets::BottomNavigation`] composes for its own bottom-edge bar.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, AppBarStyles)]
pub struct AppBar {
    /// An optional title shown after the leading slot.
    pub title: Option<String>,
    /// Which edge this bar sits against -- see [`AppBarPosition`].
    pub position: AppBarPosition,
    /// Bar height, in pixels.
    pub height: f32,
}

impl Default for AppBar {
    fn default() -> Self {
        Self {
            title: None,
            position: AppBarPosition::default(),
            height: 60.0,
        }
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    theme: Res<Theme>,
    mut query: Query<(
        &AppBar,
        &AppBarStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        Option<&AppBarLeading>,
        Option<&AppBarTrailing>,
    )>,
) {
    let Ok((app_bar, app_bar_styles, mut styles, mut children, leading, trailing)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    styles.width = Units::Percentage(100.0);
    styles.height = app_bar.height.into();
    styles.flex_direction = WidgetFlexDirection::Row;
    styles.align_items = Some(WidgetAlignItems::Center);
    styles.padding = Edge::all(0.0)
        .left(theme.spacing.lg)
        .right(theme.spacing.lg);
    styles.background_color = app_bar_styles.background_color;
    styles.border_color = app_bar_styles.border_color;
    styles.border = match app_bar.position {
        AppBarPosition::Top => Edge::all(0.0).bottom(1.0),
        AppBarPosition::Bottom => Edge::all(0.0).top(1.0),
    };
    styles.box_shadow = Some(WidgetBoxShadow {
        y_offset: match app_bar.position {
            AppBarPosition::Top => theme.elevation.sm.y_offset,
            AppBarPosition::Bottom => -theme.elevation.sm.y_offset,
        },
        ..theme.elevation.sm
    });

    *children = WidgetChildren::default();

    if let Some(leading) = leading {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(0.0).right(theme.spacing.md),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            leading.0.clone(),
        ));
        children.add_key("leading");
    }

    if let Some(title) = &app_bar.title {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: theme.typography.h2,
                color: app_bar_styles.title_color,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: title.clone(),
            },
        ));
        children.add_key("title");
    }

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        WidgetChildren::default(),
    ));
    children.add_key("spacer");

    if let Some(trailing) = trailing {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                gap: (theme.spacing.sm.into(), 0.0.into()),
                ..Default::default()
            },
            trailing.0.clone(),
        ));
        children.add_key("trailing");
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = AppBarStyles::from_theme(&theme);
        assert_eq!(styles.background_color, theme.dark_background);
        assert_eq!(styles.border_color, theme.border);
        assert_eq!(styles.title_color, theme.text);

        let dark = AppBarStyles::from_theme(&Theme::dark());
        let light = AppBarStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
