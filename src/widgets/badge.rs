use crate::prelude::*;
use bevy::prelude::*;

/// A semantic color variant for [`Badge`].
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BadgeVariant {
    /// Neutral gray -- the default, for status that isn't good/bad/risky.
    #[default]
    Neutral,
    /// Green -- positive/active/success status.
    Success,
    /// Red -- negative/error/inactive status.
    Danger,
    /// Amber -- caution/pending status.
    Warning,
    /// The library's accent blue -- informational status.
    Info,
}

/// A (background, text) color pair for one [`BadgeVariant`].
#[derive(Reflect, Clone, Copy, PartialEq)]
pub struct BadgeVariantColors {
    /// Background/fill color.
    pub background: Color,
    /// Label text color.
    pub text: Color,
}

/// [`Badge`]'s themed colors, one pair per [`BadgeVariant`] -- a separate sibling component
/// (rather than a method on `BadgeVariant` reaching into the static `colors` module) so it can
/// live-resync on a [`Theme`] swap. Also reused directly by [`crate::widgets::Toast`] (same
/// severity palette) rather than duplicating this mapping.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct BadgeStyles {
    /// Colors for [`BadgeVariant::Neutral`].
    pub neutral: BadgeVariantColors,
    /// Colors for [`BadgeVariant::Success`].
    pub success: BadgeVariantColors,
    /// Colors for [`BadgeVariant::Danger`].
    pub danger: BadgeVariantColors,
    /// Colors for [`BadgeVariant::Warning`].
    pub warning: BadgeVariantColors,
    /// Colors for [`BadgeVariant::Info`].
    pub info: BadgeVariantColors,
}

impl BadgeStyles {
    /// Looks up the (background, text) color pair for a given variant.
    pub fn colors(&self, variant: BadgeVariant) -> (Color, Color) {
        let c = match variant {
            BadgeVariant::Neutral => self.neutral,
            BadgeVariant::Success => self.success,
            BadgeVariant::Danger => self.danger,
            BadgeVariant::Warning => self.warning,
            BadgeVariant::Info => self.info,
        };
        (c.background, c.text)
    }
}

impl Default for BadgeStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for BadgeStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            neutral: BadgeVariantColors {
                background: theme.background_light,
                text: theme.text,
            },
            success: BadgeVariantColors {
                background: theme.success,
                text: Color::WHITE,
            },
            danger: BadgeVariantColors {
                background: theme.danger,
                text: Color::WHITE,
            },
            warning: BadgeVariantColors {
                background: theme.warning,
                text: Color::WHITE,
            },
            info: BadgeVariantColors {
                background: theme.primary,
                text: Color::WHITE,
            },
        }
    }
}

/// A small pill-shaped label used to show a short status (e.g. "Active", "Pending",
/// "Beta"). Purely presentational -- no interactivity or state.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, BadgeStyles)]
pub struct Badge {
    /// The text shown inside the badge.
    pub label: String,
    /// The semantic color variant.
    pub variant: BadgeVariant,
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &Badge,
        &BadgeStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((badge, badge_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let (background_color, text_color) = badge_styles.colors(badge.variant);

    // Mutate in place to preserve any caller-provided style. Compared against
    // `Corner::all(0.0)`/`Edge::all(0.0)`, not `::default()` (which is `Units::Auto` and
    // would never match).
    styles.background_color = background_color;
    if styles.border_radius == Corner::all(0.0) {
        styles.border_radius = Corner::all(100.0);
    }
    if styles.padding == Edge::all(0.0) {
        styles.padding = Edge::all(0.0).left(10.0).right(10.0).top(3.0).bottom(3.0);
    }
    styles
        .justify_content
        .get_or_insert(WidgetAlignContent::Center);
    styles.align_items.get_or_insert(WidgetAlignItems::Center);

    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 12.0,
            color: text_color,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: badge.label.clone(),
        },
    ));
    children.add_key("label");

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_maps_each_variant_to_its_theme_token() {
        let theme = Theme::dark();
        let styles = BadgeStyles::from_theme(&theme);
        assert_eq!(
            styles.colors(BadgeVariant::Neutral),
            (theme.background_light, theme.text)
        );
        assert_eq!(
            styles.colors(BadgeVariant::Success),
            (theme.success, Color::WHITE)
        );
        assert_eq!(
            styles.colors(BadgeVariant::Danger),
            (theme.danger, Color::WHITE)
        );
        assert_eq!(
            styles.colors(BadgeVariant::Warning),
            (theme.warning, Color::WHITE)
        );
        assert_eq!(
            styles.colors(BadgeVariant::Info),
            (theme.primary, Color::WHITE)
        );
    }

    #[test]
    fn from_theme_tracks_the_passed_in_theme_not_a_fixed_default() {
        let dark = BadgeStyles::from_theme(&Theme::dark());
        let light = BadgeStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.colors(BadgeVariant::Neutral),
            light.colors(BadgeVariant::Neutral)
        );
    }
}
