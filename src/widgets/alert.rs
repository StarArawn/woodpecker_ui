use crate::{
    prelude::*,
    widgets::badge::{BadgeStyles, BadgeVariant},
};
use bevy::prelude::*;

/// Fired when an [`Alert`]'s dismiss control is clicked. `Alert` doesn't remove itself --
/// same lifecycle model [`crate::widgets::ToastQueue`] uses (minus the auto-timeout) -- the
/// caller's own observer despawns/hides the alert entity in response.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct AlertDismissed;

/// [`Alert`]'s themed colors -- a separate sibling component so the message text color
/// live-resyncs on a [`Theme`] swap. The severity accent (left border) comes from
/// [`BadgeStyles`] instead of a second variant→color map, matching `Toast`'s own accent-border
/// treatment.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct AlertStyles {
    /// Message text color.
    pub text_color: Color,
    /// Dismiss glyph color -- a de-emphasized variant of `text_color`.
    pub dismiss_color: Color,
}

impl Default for AlertStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for AlertStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            text_color: theme.text,
            dismiss_color: theme.text.with_alpha(0.6),
        }
    }
}

/// An inline severity banner -- [`Paper`] with a colored left accent border (matching
/// `Toast`'s severity treatment), reusing [`BadgeVariant`] rather than a second, near-identical
/// enum. An optional dismiss glyph fires [`Change<AlertDismissed>`] on click; `Alert` doesn't
/// remove itself, matching `ToastQueue`'s own "caller owns the lifecycle" model.
///
/// The dismiss control is a plain clickable [`icons::X`] glyph (rendered via
/// [`IconFont`]), not a true `IconButton` -- `Toast`'s own dismiss control already established
/// this exact glyph-based pattern, so this matches it rather than introducing a new shape.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle,
    PassedChildren,
    WidgetChildren,
    AlertStyles,
    BadgeStyles
)]
pub struct Alert {
    /// The severity variant, driving the accent border color.
    pub variant: BadgeVariant,
    /// Whether to show a dismiss glyph.
    pub dismissible: bool,
}

fn render(
    current_widget: Res<CurrentWidget>,
    theme: Res<Theme>,
    icon_font: Res<IconFont>,
    mut query: Query<(
        &Alert,
        &AlertStyles,
        &BadgeStyles,
        &mut WidgetChildren,
        &PassedChildren,
    )>,
) {
    let Ok((alert, alert_styles, badge_styles, mut children, passed_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let current_widget = *current_widget;
    let (accent, _) = badge_styles.colors(alert.variant);

    let mut paper_children = WidgetChildren::default();
    paper_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            color: alert_styles.text_color,
            text_wrap: TextWrap::WordOrGlyph,
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        passed_children.0.clone(),
    ));
    paper_children.add_key("message");

    if alert.dismissible {
        paper_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: theme.typography.body,
                color: alert_styles.dismiss_color,
                margin: Edge::all(0.0).left(theme.spacing.sm),
                font: Some(icon_font.0.id()),
                ..Default::default()
            },
            WidgetRender::Text {
                content: icons::X.into(),
            },
            Pickable::default(),
        ));
        paper_children.add_key("dismiss");
        paper_children
            .hover_cursor(current_widget, SystemCursorIcon::Pointer)
            .observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                    commands.trigger(Change {
                        target: current_widget.entity(),
                        data: AlertDismissed,
                    });
                },
            );
    }

    *children = WidgetChildren::default().with_child::<Paper>((
        Paper { elevation: 0 },
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            padding: Edge::all(theme.spacing.md),
            border: Edge::all(0.0).left(3.0),
            border_color: accent,
            align_items: Some(WidgetAlignItems::Center),
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
        let styles = AlertStyles::from_theme(&theme);
        assert_eq!(styles.text_color, theme.text);
        assert_eq!(styles.dismiss_color, theme.text.with_alpha(0.6));

        let dark = AlertStyles::from_theme(&Theme::dark());
        let light = AlertStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.text_color, light.text_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
