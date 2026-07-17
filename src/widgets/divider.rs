use crate::prelude::*;
use bevy::prelude::*;

/// [`Divider`]'s themed color, a separate sibling component (rather than a plain field on
/// `Divider` itself) so it can live-resync on a [`Theme`] swap -- see
/// [`ThemeRegisterExt::register_themed_style`]. A caller wanting a custom color provides its
/// own `DividerStyles` (plus [`ThemeOverride`] to opt out of future resyncs).
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct DividerStyles {
    /// Line color.
    pub color: Color,
}

impl Default for DividerStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for DividerStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            color: theme.border,
        }
    }
}

/// A thin line separating content -- horizontal by default (fills its parent's width),
/// or vertical when `vertical: true` (fills its parent's height instead).
#[derive(Widget, Component, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, DividerStyles)]
pub struct Divider {
    /// Renders as a vertical line (fills height, fixed 1px width) instead of the default
    /// horizontal line (fills width, fixed 1px height).
    pub vertical: bool,
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &Divider,
        &DividerStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((divider, divider_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    // Mutate in place -- a caller may have set `height`/`margin`/etc via their own bundle
    // (e.g. a shorter vertical divider inset from its row's edges); only fill in the
    // thickness/length/color a divider actually needs, don't discard the rest.
    styles.background_color = divider_styles.color;
    if divider.vertical {
        styles.width = 1.0.into();
        if styles.height == Units::Auto {
            styles.height = Units::Percentage(100.0);
        }
    } else {
        styles.height = 1.0.into();
        if styles.width == Units::Auto {
            styles.width = Units::Percentage(100.0);
        }
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_theme_border_color() {
        let dark = DividerStyles::from_theme(&Theme::dark());
        let light = DividerStyles::from_theme(&Theme::light());
        assert_eq!(dark.color, Theme::dark().border);
        assert_eq!(light.color, Theme::light().border);
        assert_ne!(
            dark.color, light.color,
            "dark/light themes have different border colors -- from_theme must track the \
             passed-in theme, not a fixed default"
        );
    }
}
