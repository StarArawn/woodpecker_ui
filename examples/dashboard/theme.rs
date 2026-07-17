use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// `bg_app`/`bg_sidebar`/`bg_topbar`/`bg_input` all resolve to the same token: `Theme`'s own
/// docs describe `dark_background` as "the darkest surface (text input fills, chrome bars)",
/// which is exactly this quartet's role in the dashboard.
pub fn bg_app(theme: &Theme) -> Color {
    theme.dark_background
}
pub fn bg_sidebar(theme: &Theme) -> Color {
    theme.dark_background
}
pub fn bg_topbar(theme: &Theme) -> Color {
    theme.dark_background
}
pub fn bg_input(theme: &Theme) -> Color {
    theme.dark_background
}
/// Deliberately far from `bg_app`/etc so a card reads as clearly "raised".
pub fn bg_card(theme: &Theme) -> Color {
    theme.background
}
pub fn bg_card_hover(theme: &Theme) -> Color {
    theme.background_mid
}
pub fn border(theme: &Theme) -> Color {
    theme.border
}

pub fn accent(theme: &Theme) -> Color {
    theme.primary
}
pub fn accent_hover(theme: &Theme) -> Color {
    theme.primary_light
}
pub fn success(theme: &Theme) -> Color {
    theme.success
}
pub fn danger(theme: &Theme) -> Color {
    theme.danger
}

pub fn text_primary(theme: &Theme) -> Color {
    theme.text
}
/// `Theme` has no dedicated secondary/muted text tokens -- alpha-fading `text` keeps both tiers
/// tied to the same base color (so a theme swap can't leave one recolored and the other stale)
/// while preserving roughly the same relative dimming the dashboard's old hardcoded shades had.
pub fn text_secondary(theme: &Theme) -> Color {
    theme.text.with_alpha(0.65)
}
pub fn text_muted(theme: &Theme) -> Color {
    theme.text.with_alpha(0.45)
}

pub fn radius_sm(theme: &Theme) -> f32 {
    theme.control_radius
}
pub fn radius_md(theme: &Theme) -> f32 {
    theme.panel_radius
}

pub fn space_xs(theme: &Theme) -> f32 {
    theme.spacing.xs
}
pub fn space_sm(theme: &Theme) -> f32 {
    theme.spacing.sm
}
pub fn space_md(theme: &Theme) -> f32 {
    theme.spacing.md
}
pub fn space_lg(theme: &Theme) -> f32 {
    theme.spacing.lg
}

/// A card-like container: rounded, bordered, dark surface. The common building block
/// behind stat tiles, table panels, and form panels throughout the dashboard.
pub fn card_style(theme: &Theme) -> WoodpeckerStyle {
    WoodpeckerStyle {
        background_color: bg_card(theme),
        border_color: border(theme),
        border: Edge::all(1.0),
        border_radius: Corner::all(radius_md(theme)),
        padding: Edge::all(space_lg(theme)),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

pub fn section_title_style(theme: &Theme) -> WoodpeckerStyle {
    WoodpeckerStyle {
        font_size: 18.0,
        color: text_primary(theme),
        margin: Edge::all(0.0).bottom(space_md(theme)),
        ..Default::default()
    }
}

pub fn label_style(theme: &Theme) -> WoodpeckerStyle {
    WoodpeckerStyle {
        font_size: 13.0,
        color: text_secondary(theme),
        width: Units::Pixels(140.0),
        ..Default::default()
    }
}

pub fn primary_button_styles(theme: &Theme) -> ButtonStyles {
    let base = WoodpeckerStyle {
        font_size: 14.0,
        color: Color::WHITE,
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        border_radius: Corner::all(radius_sm(theme)),
        padding: Edge::new(0.0, 18.0, 0.0, 18.0),
        height: 36.0.into(),
        ..Default::default()
    };
    ButtonStyles {
        normal: WoodpeckerStyle {
            background_color: accent(theme),
            ..base
        },
        hovered: WoodpeckerStyle {
            background_color: accent_hover(theme),
            ..base
        },
    }
}

/// Wraps a page's top-level content in a scroll context so it scrolls once its stacked
/// cards exceed the available viewport height, instead of just clipping/overflowing off the
/// bottom of the window with no way to reach the rest of it.
pub fn scrollable_page(content: WidgetChildren) -> WidgetChildren {
    WidgetChildren::default().with_child::<ScrollContextProvider>((
        ScrollContextProvider::default(),
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<ScrollBox>((ScrollBox::default(), PassedChildren(content))),
    ))
}

pub fn secondary_button_styles(theme: &Theme) -> ButtonStyles {
    let base = WoodpeckerStyle {
        font_size: 14.0,
        color: text_primary(theme),
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        border_radius: Corner::all(radius_sm(theme)),
        border: Edge::all(1.0),
        border_color: border(theme),
        padding: Edge::new(0.0, 18.0, 0.0, 18.0),
        height: 36.0.into(),
        ..Default::default()
    };
    ButtonStyles {
        normal: WoodpeckerStyle {
            background_color: bg_input(theme),
            ..base
        },
        hovered: WoodpeckerStyle {
            background_color: bg_card_hover(theme),
            ..base
        },
    }
}
