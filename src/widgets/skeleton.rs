use crate::prelude::*;
use bevy::prelude::*;

/// [`Skeleton`]'s three shapes.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SkeletonVariant {
    /// A rounded rectangle -- a card, an image, a generic block.
    #[default]
    Rect,
    /// A full circle -- an avatar placeholder.
    Circle,
    /// A thin pill -- one line of text.
    Text,
}

/// [`Skeleton`]'s themed shimmer colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct SkeletonStyles {
    /// The shimmer's low point.
    pub base_color: Color,
    /// The shimmer's high point.
    pub highlight_color: Color,
}

impl Default for SkeletonStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for SkeletonStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            base_color: theme.background_light,
            highlight_color: theme.background_mid,
        }
    }
}

/// Milliseconds for one shimmer pulse (base -> highlight; `looping` then reverses back).
const SHIMMER_TIMEOUT: f32 = 900.0;

/// A loading placeholder -- a shape (see [`SkeletonVariant`]) that pulses between
/// [`SkeletonStyles::base_color`] and `highlight_color` indefinitely, via a looping
/// [`Transition`] on its own entity rather than a `WidgetRender::Custom` closure: `Transition`
/// already interpolates `background_color` every frame on its own (see
/// `crate::widgets::transition::update_transitions`), so `Skeleton`'s `render` only needs to
/// set the `Transition` up once -- no per-frame Rust-side work of its own at all.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, SkeletonStyles)]
pub struct Skeleton {
    /// Which shape to render.
    pub variant: SkeletonVariant,
    /// Width, in logical pixels.
    pub width: f32,
    /// Height, in logical pixels.
    pub height: f32,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self {
            variant: SkeletonVariant::default(),
            width: 120.0,
            height: 16.0,
        }
    }
}

/// The shimmer's `(style_a, style_b)` pair for a given shape/size/color combination --
/// identical on every field except `background_color`, so a *settled* (non-playing) instance
/// still renders the right shape/size regardless of which end it lands on (see
/// `WoodpeckerStyle::lerp`'s "starts as a full copy of `style_a`" behavior). Pulled out of
/// `render` as a pure, unit-testable function.
fn shimmer_styles(
    variant: SkeletonVariant,
    width: f32,
    height: f32,
    control_radius: f32,
    base_color: Color,
    highlight_color: Color,
) -> (WoodpeckerStyle, WoodpeckerStyle) {
    let radius = match variant {
        SkeletonVariant::Circle | SkeletonVariant::Text => height / 2.0,
        SkeletonVariant::Rect => control_radius,
    };
    let constant = WoodpeckerStyle {
        width: width.into(),
        height: height.into(),
        border_radius: Corner::all(radius),
        ..Default::default()
    };
    (
        WoodpeckerStyle {
            background_color: base_color,
            ..constant
        },
        WoodpeckerStyle {
            background_color: highlight_color,
            ..constant
        },
    )
}

fn render(
    theme: Res<Theme>,
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &Skeleton,
        &SkeletonStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        Option<&Transition>,
    )>,
) {
    let Ok((skeleton, skeleton_styles, mut styles, mut children, existing)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let (style_a, style_b) = shimmer_styles(
        skeleton.variant,
        skeleton.width,
        skeleton.height,
        theme.control_radius,
        skeleton_styles.base_color,
        skeleton_styles.highlight_color,
    );

    // Reuse whatever's already playing if the shape/colors haven't actually changed -- a
    // fresh `Transition` (even with `.start()` called) resets the shimmer's phase, which
    // would look like a stutter on every unrelated re-render.
    let transition = match existing {
        Some(existing) if existing.style_a == style_a && existing.style_b == style_b => *existing,
        _ => {
            let mut transition = Transition {
                easing: TransitionEasing::SineInOut,
                timeout: SHIMMER_TIMEOUT,
                looping: true,
                style_a,
                style_b,
                ..Default::default()
            };
            transition.start();
            transition
        }
    };

    // Only the initial style matters here -- `update_transitions` (a separate system that
    // ticks every frame) takes over interpolating `background_color` from here on, since
    // `transition` is `looping: true` and never stops playing.
    *styles = style_a;
    commands.entity(current_widget.entity()).insert(transition);

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = SkeletonStyles::from_theme(&theme);
        assert_eq!(styles.base_color, theme.background_light);
        assert_eq!(styles.highlight_color, theme.background_mid);

        let dark = SkeletonStyles::from_theme(&Theme::dark());
        let light = SkeletonStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.base_color, light.base_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn rect_uses_the_theme_control_radius() {
        let (style_a, _) = shimmer_styles(
            SkeletonVariant::Rect,
            120.0,
            16.0,
            6.0,
            Color::BLACK,
            Color::WHITE,
        );
        assert_eq!(style_a.border_radius, Corner::all(6.0));
    }

    #[test]
    fn circle_and_text_are_fully_rounded_by_half_their_height() {
        let (circle, _) = shimmer_styles(
            SkeletonVariant::Circle,
            40.0,
            40.0,
            6.0,
            Color::BLACK,
            Color::WHITE,
        );
        assert_eq!(circle.border_radius, Corner::all(20.0));

        let (text, _) = shimmer_styles(
            SkeletonVariant::Text,
            120.0,
            16.0,
            6.0,
            Color::BLACK,
            Color::WHITE,
        );
        assert_eq!(text.border_radius, Corner::all(8.0));
    }

    #[test]
    fn style_a_and_style_b_differ_only_in_background_color() {
        let (style_a, style_b) = shimmer_styles(
            SkeletonVariant::Rect,
            120.0,
            16.0,
            6.0,
            Color::BLACK,
            Color::WHITE,
        );
        assert_eq!(style_a.width, style_b.width);
        assert_eq!(style_a.height, style_b.height);
        assert_eq!(style_a.border_radius, style_b.border_radius);
        assert_ne!(style_a.background_color, style_b.background_color);
        assert_eq!(style_a.background_color, Color::BLACK);
        assert_eq!(style_b.background_color, Color::WHITE);
    }
}
