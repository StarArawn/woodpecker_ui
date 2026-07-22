use crate::prelude::*;
use bevy::prelude::*;

/// [`Avatar`]'s themed defaults -- a separate sibling component (rather than baking
/// `Theme::default().primary`/`Color::WHITE` into `Avatar`'s own fields) so a newly-spawned
/// avatar's *default* background and its initials text color both track the active [`Theme`]
/// live, instead of freezing at whatever `Theme::default()` happened to resolve to the moment
/// `Avatar::default()` was constructed (Rust's `Default` trait has no access to `Resource`s, so
/// it can't read the *actual* active theme at construction time -- only a `ThemedStyle`
/// component, resynced by `register_themed_style`, can).
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct AvatarStyles {
    /// Background color used when `Avatar::color` is `None`.
    pub default_color: Color,
    /// The initials text color.
    pub text_color: Color,
}

impl Default for AvatarStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for AvatarStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            default_color: theme.primary,
            text_color: Color::WHITE,
        }
    }
}

/// A circular avatar showing a person/entity's initials on a colored background.
///
/// Deliberately initials-only (no image loading); build a photo avatar directly from an
/// `Element` with `WidgetRender::Image` instead.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, AvatarStyles)]
pub struct Avatar {
    /// The initials to display (typically 1-2 characters -- longer strings aren't truncated,
    /// so keep it short).
    pub initials: String,
    /// The circle's background color. `None` (the default) follows the active theme's
    /// [`AvatarStyles::default_color`], live-updating on a `Theme` swap; `Some(color)` pins an
    /// explicit per-instance color (e.g. a per-user avatar color), same as before this field
    /// became optional.
    pub color: Option<Color>,
    /// The diameter of the circle, in logical pixels.
    pub size: f32,
}

impl Default for Avatar {
    fn default() -> Self {
        Self {
            initials: String::new(),
            color: None,
            size: 32.0,
        }
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &Avatar,
        &AvatarStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((avatar, avatar_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    // Mutate in place -- preserves any caller-provided style (e.g. a margin) instead of
    // discarding it.
    styles.width = avatar.size.into();
    styles.height = avatar.size.into();
    styles.background_color = avatar.color.unwrap_or(avatar_styles.default_color);
    styles.border_radius = Corner::all(avatar.size / 2.0);
    styles
        .justify_content
        .get_or_insert(WidgetAlignContent::Center);
    styles.align_items.get_or_insert(WidgetAlignItems::Center);

    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: (avatar.size * 0.4).max(10.0),
            color: avatar_styles.text_color,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: avatar.initials.clone(),
        },
    ));
    children.add_key("initials");

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_color_tracks_the_passed_in_theme() {
        let dark = AvatarStyles::from_theme(&Theme::dark());
        let light = AvatarStyles::from_theme(&Theme::light());
        assert_eq!(dark.default_color, Theme::dark().primary);
        assert_eq!(light.default_color, Theme::light().primary);
    }
}
