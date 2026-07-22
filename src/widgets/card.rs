use crate::prelude::*;
use bevy::prelude::*;

/// [`Card`]'s themed header text styles.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct CardStyles {
    /// Title text color.
    pub title_color: Color,
    /// Title font size.
    pub title_font_size: f32,
    /// Subtitle text color.
    pub subtitle_color: Color,
    /// Subtitle font size.
    pub subtitle_font_size: f32,
}

impl Default for CardStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for CardStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            title_color: theme.text,
            title_font_size: theme.typography.h2,
            subtitle_color: theme.text.with_alpha(0.7),
            subtitle_font_size: theme.typography.body_small,
        }
    }
}

/// An optional row of action controls (e.g. buttons) shown at the bottom of a [`Card`].
/// Absent by default -- a card with no `CardActions` sibling simply has no action row.
#[derive(Component, Default, PartialEq, Clone)]
pub struct CardActions(pub WidgetChildren);

/// A [`Paper`]-backed surface with an optional title/subtitle header and an optional action
/// row, wrapping arbitrary body content. Generalizes the `card_style()`/`section()` pattern
/// several examples (`examples/dashboard/overview.rs`, `examples/dashboard/settings_page.rs`)
/// previously hand-rolled per file.
///
/// Deliberately doesn't have MUI's separate media slot or header action slot -- neither real
/// usage this generalizes needs one, and a caller wanting an image can simply put an
/// `Element`+`WidgetRender::Image` as the first item in the card's own children.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, PassedChildren, WidgetChildren, CardStyles)]
pub struct Card {
    /// Optional header title.
    pub title: Option<String>,
    /// Optional header subtitle, shown under the title.
    pub subtitle: Option<String>,
    /// Forwarded to the backing [`Paper`]'s own `elevation`.
    pub elevation: u8,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            title: None,
            subtitle: None,
            elevation: 1,
        }
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    theme: Res<Theme>,
    mut query: Query<(
        &Card,
        &CardStyles,
        &mut WidgetChildren,
        &PassedChildren,
        Option<&CardActions>,
    )>,
) {
    let Ok((card, card_styles, mut children, passed_children, actions)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let mut paper_children = WidgetChildren::default();

    if card.title.is_some() || card.subtitle.is_some() {
        let mut header = WidgetChildren::default();
        if let Some(title) = &card.title {
            header.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: card_styles.title_font_size,
                    color: card_styles.title_color,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: title.clone(),
                },
            ));
            header.add_key("title");
        }
        if let Some(subtitle) = &card.subtitle {
            header.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: card_styles.subtitle_font_size,
                    color: card_styles.subtitle_color,
                    margin: Edge::all(0.0).top(theme.spacing.xs),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: subtitle.clone(),
                },
            ));
            header.add_key("subtitle");
        }
        paper_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                margin: Edge::all(0.0).bottom(theme.spacing.md),
                ..Default::default()
            },
            header,
        ));
        paper_children.add_key("header");
    }

    paper_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            flex_grow: 1.0,
            ..Default::default()
        },
        passed_children.0.clone(),
    ));
    paper_children.add_key("body");

    if let Some(actions) = actions {
        paper_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Row,
                justify_content: Some(WidgetAlignContent::End),
                gap: (theme.spacing.sm.into(), 0.0.into()),
                margin: Edge::all(0.0).top(theme.spacing.md),
                ..Default::default()
            },
            actions.0.clone(),
        ));
        paper_children.add_key("actions");
    }

    *children = WidgetChildren::default().with_child::<Paper>((
        Paper {
            elevation: card.elevation,
        },
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            padding: Edge::all(theme.spacing.lg),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        paper_children,
    ));
    children.add_key("paper");

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = CardStyles::from_theme(&theme);
        assert_eq!(styles.title_color, theme.text);
        assert_eq!(styles.title_font_size, theme.typography.h2);
        assert_eq!(styles.subtitle_font_size, theme.typography.body_small);

        let dark = CardStyles::from_theme(&Theme::dark());
        let light = CardStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.title_color, light.title_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
