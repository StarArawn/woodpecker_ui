use crate::prelude::*;
use bevy::prelude::*;

/// [`Paper`]'s themed base surface -- a separate sibling component (matching every other
/// widget's `*Styles` convention) so it live-resyncs on a [`Theme`] swap.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct PaperStyles {
    /// Base (elevation 0) background color.
    pub background_color: Color,
    /// Base (elevation 0) border color.
    pub border_color: Color,
    /// Border thickness.
    pub border_width: f32,
    /// Corner radius.
    pub corner_radius: f32,
    /// Drop shadow used at elevation 1.
    pub shadow_sm: WidgetBoxShadow,
    /// Drop shadow used at elevation 2.
    pub shadow_md: WidgetBoxShadow,
    /// Drop shadow used at elevation 3.
    pub shadow_lg: WidgetBoxShadow,
    /// Drop shadow used at elevation 4.
    pub shadow_xl: WidgetBoxShadow,
}

impl Default for PaperStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for PaperStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.background,
            border_color: theme.border,
            border_width: 1.0,
            corner_radius: theme.panel_radius,
            shadow_sm: theme.elevation.sm,
            shadow_md: theme.elevation.md,
            shadow_lg: theme.elevation.lg,
            shadow_xl: theme.elevation.xl,
        }
    }
}

/// A themed surface -- the base building block [`Card`](super::Card)/`Alert`/`Drawer`/
/// `Accordion` all compose rather than each re-deriving their own background/border/radius.
/// Purely presentational: no interactivity, no state.
#[derive(Widget, Component, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, PaperStyles)]
pub struct Paper {
    /// A coarse 0..=4 elevation tier. Drives both a progressively lighter background/border
    /// step (`lighten`, below) and a real drop shadow sized from `Theme::elevation`'s matching
    /// tier (1 -> `sm`, 2 -> `md`, 3 -> `lg`, 4 -> `xl`). `0` renders exactly `PaperStyles`'
    /// base colors, with no shadow.
    pub elevation: u8,
}

/// How much lighter (toward white) one elevation step nudges a color, per RGB channel.
const ELEVATION_STEP: f32 = 0.02;

fn lighten(color: Color, steps: u8) -> Color {
    if steps == 0 {
        return color;
    }
    let srgba = color.to_srgba();
    let amount = (ELEVATION_STEP * steps as f32).min(1.0);
    Color::srgba(
        (srgba.red + amount).min(1.0),
        (srgba.green + amount).min(1.0),
        (srgba.blue + amount).min(1.0),
        srgba.alpha,
    )
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &Paper,
        &PaperStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((paper, paper_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let elevation = paper.elevation.min(4);
    // Mutate in place -- preserves any caller-provided style (e.g. a margin) instead of
    // discarding it.
    styles.background_color = lighten(paper_styles.background_color, elevation);
    styles.border_color = lighten(paper_styles.border_color, elevation);
    styles.box_shadow = match elevation {
        0 => None,
        1 => Some(paper_styles.shadow_sm),
        2 => Some(paper_styles.shadow_md),
        3 => Some(paper_styles.shadow_lg),
        _ => Some(paper_styles.shadow_xl),
    };
    if styles.border == Edge::all(0.0) {
        styles.border = Edge::all(paper_styles.border_width);
    }
    if styles.border_radius == Corner::all(0.0) {
        styles.border_radius = Corner::all(paper_styles.corner_radius);
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = PaperStyles::from_theme(&theme);
        assert_eq!(styles.background_color, theme.background);
        assert_eq!(styles.border_color, theme.border);
        assert_eq!(styles.corner_radius, theme.panel_radius);

        let dark = PaperStyles::from_theme(&Theme::dark());
        let light = PaperStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn lighten_is_a_no_op_at_elevation_zero() {
        let base = Color::srgba(0.1, 0.1, 0.1, 1.0);
        assert_eq!(lighten(base, 0), base);
    }

    #[test]
    fn paper_styles_shadow_tiers_track_the_themes_elevation_scale() {
        let theme = Theme::dark();
        let styles = PaperStyles::from_theme(&theme);
        assert_eq!(styles.shadow_sm, theme.elevation.sm);
        assert_eq!(styles.shadow_md, theme.elevation.md);
        assert_eq!(styles.shadow_lg, theme.elevation.lg);
        assert_eq!(styles.shadow_xl, theme.elevation.xl);
    }

    #[test]
    fn lighten_increases_with_elevation_and_clamps_at_one() {
        let base = Color::srgba(0.1, 0.1, 0.1, 1.0);
        let e1 = lighten(base, 1).to_srgba();
        let e2 = lighten(base, 2).to_srgba();
        let e4 = lighten(base, 4).to_srgba();
        assert!(e1.red < e2.red);
        assert!(e2.red < e4.red);
        assert!(e4.red <= 1.0);

        let near_white = Color::srgba(0.99, 0.99, 0.99, 1.0);
        let clamped = lighten(near_white, 4).to_srgba();
        assert_eq!(clamped.red, 1.0);
    }
}
