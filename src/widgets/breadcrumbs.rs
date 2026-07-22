use crate::prelude::*;
use bevy::prelude::*;

/// Fired when a [`Breadcrumbs`] segment is clicked -- always one of the *non-current* segments
/// (`items[..items.len() - 1]`); the trail's last segment is the current page and isn't a
/// [`Link`], so it never fires this.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct BreadcrumbClicked {
    /// The clicked segment's index into [`Breadcrumbs::items`].
    pub index: usize,
}

/// [`Breadcrumbs`]' themed colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct BreadcrumbsStyles {
    /// Separator glyph color between segments.
    pub separator_color: Color,
    /// Text color for the trail's last (current-page, non-clickable) segment.
    pub current_color: Color,
}

impl Default for BreadcrumbsStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for BreadcrumbsStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            separator_color: theme.text.with_alpha(0.4),
            current_color: theme.text,
        }
    }
}

/// A row of trail segments (e.g. "Settings / Billing / Invoices") -- every segment but the
/// last composes [`Link`] directly (rather than a parallel clickable-text implementation) and
/// fires [`Change<BreadcrumbClicked>`] on click; the last segment is the current page, drawn as
/// plain (non-clickable) text.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    BreadcrumbsStyles
)]
pub struct Breadcrumbs {
    /// The trail, root-first -- the last entry is the current page.
    pub items: Vec<String>,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        flex_direction: WidgetFlexDirection::Row,
        align_items: Some(WidgetAlignItems::Center),
        ..Default::default()
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&Breadcrumbs, &BreadcrumbsStyles, &mut WidgetChildren)>,
) {
    let Ok((breadcrumbs, breadcrumbs_styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    let last = breadcrumbs.items.len().saturating_sub(1);
    for (index, label) in breadcrumbs.items.iter().enumerate() {
        if index == last {
            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: breadcrumbs_styles.current_color,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.clone(),
                },
            ));
            children.add_key(format!("segment-{index}"));
        } else {
            children
                .add::<Link>((Link {
                    label: label.clone(),
                },))
                .observe(
                    current_widget,
                    move |_trigger: On<Change<LinkClicked>>, mut commands: Commands| {
                        commands.trigger(Change {
                            target: current_widget.entity(),
                            data: BreadcrumbClicked { index },
                        });
                    },
                );
            children.add_key(format!("segment-{index}"));

            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: breadcrumbs_styles.separator_color,
                    margin: Edge::all(0.0).left(6.0).right(6.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "/".into(),
                },
            ));
            children.add_key(format!("separator-{index}"));
        }
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = BreadcrumbsStyles::from_theme(&theme);
        assert_eq!(styles.current_color, theme.text);

        let dark = BreadcrumbsStyles::from_theme(&Theme::dark());
        let light = BreadcrumbsStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.current_color, light.current_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
