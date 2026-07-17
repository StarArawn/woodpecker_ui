use crate::prelude::*;
use bevy::prelude::*;

/// Which role of [`crate::theming::Typography`]'s scale a [`Typography`] widget resolves its
/// `font_size` from.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TypographyVariant {
    /// Large page/section heading.
    H1,
    /// Sub-heading / card title.
    H2,
    /// Small heading / emphasized label.
    H3,
    /// Default body text.
    #[default]
    Body,
    /// Secondary/de-emphasized body text.
    BodySmall,
    /// Smallest legible text.
    Caption,
}

impl TypographyVariant {
    fn font_size(self, typography: &crate::theming::Typography) -> f32 {
        match self {
            TypographyVariant::H1 => typography.h1,
            TypographyVariant::H2 => typography.h2,
            TypographyVariant::H3 => typography.h3,
            TypographyVariant::Body => typography.body,
            TypographyVariant::BodySmall => typography.body_small,
            TypographyVariant::Caption => typography.caption,
        }
    }
}

/// A thin text wrapper resolving `font_size` from [`Theme::typography`] by [`TypographyVariant`]
/// instead of every call site hand-picking a raw `f32` -- directly replaces the repeated
/// `font_size: 18.0`/`font_size: 12.0`/etc. boilerplate scattered across `examples/dashboard/`.
/// A leaf widget: no children of its own, `WidgetRender::Text` lives on its own entity rather
/// than a nested `Element`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Text { content: String::new() })]
pub struct Typography {
    /// The text to render.
    pub text: String,
    /// Which role of the theme's type scale to use.
    pub variant: TypographyVariant,
    /// Overrides the variant's default color ([`Theme::text`]) when set.
    pub color: Option<Color>,
}

fn render(
    theme: Res<Theme>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &Typography,
        &mut WoodpeckerStyle,
        &mut WidgetRender,
        &mut WidgetChildren,
    )>,
) {
    let Ok((typography, mut styles, mut render, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    styles.font_size = typography.variant.font_size(&theme.typography);
    styles.color = typography.color.unwrap_or(theme.text);
    *render = WidgetRender::Text {
        content: typography.text.clone(),
    };

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_variant_maps_to_its_own_theme_scale_entry() {
        let typography = crate::theming::Typography::default_scale();
        assert_eq!(TypographyVariant::H1.font_size(&typography), typography.h1);
        assert_eq!(TypographyVariant::H2.font_size(&typography), typography.h2);
        assert_eq!(TypographyVariant::H3.font_size(&typography), typography.h3);
        assert_eq!(
            TypographyVariant::Body.font_size(&typography),
            typography.body
        );
        assert_eq!(
            TypographyVariant::BodySmall.font_size(&typography),
            typography.body_small
        );
        assert_eq!(
            TypographyVariant::Caption.font_size(&typography),
            typography.caption
        );
    }

    #[test]
    fn default_variant_is_body() {
        assert_eq!(TypographyVariant::default(), TypographyVariant::Body);
    }
}
