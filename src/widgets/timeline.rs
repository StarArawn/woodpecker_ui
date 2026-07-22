use crate::prelude::*;
use bevy::prelude::*;

/// One entry in a [`Timeline`].
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct TimelineItem {
    /// The entry's main line.
    pub title: String,
    /// An optional smaller line under `title` (typically a timestamp).
    pub subtitle: Option<String>,
    /// Overrides [`TimelineStyles::dot_color`] for this entry's dot -- e.g. a red dot for an
    /// error event in an otherwise neutral activity feed.
    pub dot_color: Option<Color>,
}

/// [`Timeline`]'s themed colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TimelineStyles {
    /// Default dot color, when an item doesn't override it.
    pub dot_color: Color,
    /// The connector line color between dots.
    pub connector_color: Color,
    /// Title text color.
    pub title_color: Color,
    /// Subtitle text color.
    pub subtitle_color: Color,
}

impl Default for TimelineStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TimelineStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            dot_color: theme.primary,
            connector_color: theme.border,
            title_color: theme.text,
            subtitle_color: theme.text.with_alpha(0.6),
        }
    }
}

/// A vertical sequence of dated/ordered events -- e.g. an order's status history, an audit
/// log. Purely presentational, no state of its own (unlike most of this crate's other
/// composite widgets, a timeline has nothing to click or toggle).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = WoodpeckerStyle { flex_direction: WidgetFlexDirection::Column, ..Default::default() },
    WidgetChildren,
    TimelineStyles
)]
pub struct Timeline {
    /// The entries, in order (typically newest-first or oldest-first, caller's choice).
    pub items: Vec<TimelineItem>,
}

fn render(
    theme: Res<Theme>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&Timeline, &TimelineStyles, &mut WidgetChildren)>,
) {
    let Ok((timeline, timeline_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    let last = timeline.items.len().saturating_sub(1);
    const DOT_SIZE: f32 = 10.0;

    for (index, item) in timeline.items.iter().enumerate() {
        let dot_color = item.dot_color.unwrap_or(timeline_styles.dot_color);

        let mut rail = WidgetChildren::default();
        rail.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: DOT_SIZE.into(),
                height: DOT_SIZE.into(),
                background_color: dot_color,
                border_radius: Corner::all(DOT_SIZE / 2.0),
                margin: Edge::all(0.0).top(4.0),
                ..Default::default()
            },
            WidgetRender::Quad,
        ));
        rail.add_key("dot");
        if index != last {
            rail.add::<Divider>((
                Divider { vertical: true },
                DividerStyles {
                    color: timeline_styles.connector_color,
                },
                WoodpeckerStyle {
                    flex_grow: 1.0,
                    margin: Edge::all(0.0).top(4.0),
                    ..Default::default()
                },
            ));
            rail.add_key("connector");
        }

        let mut content = WidgetChildren::default();
        content.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: theme.typography.body,
                color: timeline_styles.title_color,
                text_wrap: TextWrap::WordOrGlyph,
                ..Default::default()
            },
            WidgetRender::Text {
                content: item.title.clone(),
            },
        ));
        content.add_key("title");
        if let Some(subtitle) = &item.subtitle {
            content.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: theme.typography.caption,
                    color: timeline_styles.subtitle_color,
                    margin: Edge::all(0.0).top(2.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: subtitle.clone(),
                },
            ));
            content.add_key("subtitle");
        }

        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Row,
                gap: (theme.spacing.sm.into(), 0.0.into()),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_direction: WidgetFlexDirection::Column,
                        align_items: Some(WidgetAlignItems::Center),
                        ..Default::default()
                    },
                    rail,
                ))
                .with_key("rail")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_direction: WidgetFlexDirection::Column,
                        flex_grow: 1.0,
                        padding: Edge::all(0.0).bottom(theme.spacing.md),
                        ..Default::default()
                    },
                    content,
                ))
                .with_key("content"),
        ));
        children.add_key(format!("item-{index}"));
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = TimelineStyles::from_theme(&theme);
        assert_eq!(styles.dot_color, theme.primary);
        assert_eq!(styles.connector_color, theme.border);

        let dark = TimelineStyles::from_theme(&Theme::dark());
        let light = TimelineStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.connector_color, light.connector_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
