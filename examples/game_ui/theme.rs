use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::ItemRarity;

pub const BG_HUD_PANEL: Color = Color::srgb(0.086, 0.067, 0.051);
pub const BG_WINDOW: Color = Color::srgb(0.129, 0.102, 0.078);
pub const BG_WINDOW_HOVER: Color = Color::srgb(0.161, 0.129, 0.098);
pub const BG_INPUT: Color = Color::srgb(0.098, 0.078, 0.059);
pub const BORDER_GOLD: Color = Color::srgb(0.686, 0.541, 0.212);
pub const BORDER_DARK: Color = Color::srgb(0.302, 0.235, 0.157);
pub const PARCHMENT: Color = Color::srgb(0.882, 0.816, 0.686);
pub const PARCHMENT_TEXT: Color = Color::srgb(0.145, 0.106, 0.067);

pub const TEXT_PRIMARY: Color = Color::srgb(0.94, 0.91, 0.85);
pub const TEXT_MUTED: Color = Color::srgb(0.62, 0.56, 0.47);

pub const HEALTH: Color = Color::srgb(0.78, 0.14, 0.14);
pub const HEALTH_TRACK: Color = Color::srgb(0.22, 0.05, 0.05);
pub const MANA: Color = Color::srgb(0.16, 0.42, 0.86);
pub const MANA_TRACK: Color = Color::srgb(0.06, 0.10, 0.22);
pub const STAMINA: Color = Color::srgb(0.30, 0.62, 0.25);
pub const STAMINA_TRACK: Color = Color::srgb(0.09, 0.16, 0.07);
pub const XP: Color = Color::srgb(0.85, 0.65, 0.13);
pub const XP_TRACK: Color = Color::srgb(0.20, 0.16, 0.05);

pub const RARITY_COMMON: Color = Color::srgb(0.62, 0.62, 0.62);
pub const RARITY_UNCOMMON: Color = Color::srgb(0.24, 0.68, 0.29);
pub const RARITY_RARE: Color = Color::srgb(0.18, 0.45, 0.93);
pub const RARITY_EPIC: Color = Color::srgb(0.60, 0.28, 0.86);
pub const RARITY_LEGENDARY: Color = Color::srgb(0.93, 0.55, 0.11);

pub const RADIUS_SM: f32 = 5.0;
pub const RADIUS_MD: f32 = 9.0;

pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;

/// The true 5-color rarity palette, used directly on `ItemSlot`'s own border/glow.
///
/// Deliberately not routed through `Badge`: `BadgeVariant`'s 5 variants have different
/// semantics (Neutral/Success/Danger/Warning/Info) that don't map cleanly onto rarity tiers.
/// `ItemRarity::badge_variant()` provides a best-effort mapping for the rarity pill instead.
pub fn rarity_color(rarity: ItemRarity) -> Color {
    match rarity {
        ItemRarity::Common => RARITY_COMMON,
        ItemRarity::Uncommon => RARITY_UNCOMMON,
        ItemRarity::Rare => RARITY_RARE,
        ItemRarity::Epic => RARITY_EPIC,
        ItemRarity::Legendary => RARITY_LEGENDARY,
    }
}

/// Dark chrome background for HUD groupings (vitals cluster, hotbar tray, minimap).
pub fn hud_panel_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        background_color: BG_HUD_PANEL,
        border_color: BORDER_GOLD,
        border: Edge::all(2.0),
        border_radius: Corner::all(RADIUS_MD),
        padding: Edge::all(SPACE_SM),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

/// Styles fed into `WoodpeckerWindow::window_styles` for every floating menu panel.
pub fn window_card_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        background_color: BG_WINDOW,
        border_color: BORDER_GOLD,
        border: Edge::all(2.0),
        border_radius: Corner::all(RADIUS_MD),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

pub fn section_title_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        font_size: 16.0,
        color: TEXT_PRIMARY,
        margin: Edge::all(0.0).bottom(SPACE_SM),
        ..Default::default()
    }
}

pub fn label_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        font_size: 13.0,
        color: TEXT_MUTED,
        // Text overflows its declared width rather than clipping, so this must comfortably
        // fit every label used at this font size.
        width: Units::Pixels(140.0),
        ..Default::default()
    }
}

pub fn primary_button_styles() -> ButtonStyles {
    let base = WoodpeckerStyle {
        font_size: 13.0,
        color: Color::WHITE,
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        border_radius: Corner::all(RADIUS_SM),
        padding: Edge::new(0.0, 14.0, 0.0, 14.0),
        height: 30.0.into(),
        ..Default::default()
    };
    ButtonStyles {
        normal: WoodpeckerStyle {
            background_color: BORDER_GOLD,
            ..base
        },
        hovered: WoodpeckerStyle {
            background_color: Color::srgb(0.784, 0.639, 0.271),
            ..base
        },
    }
}

pub fn secondary_button_styles() -> ButtonStyles {
    let base = WoodpeckerStyle {
        font_size: 13.0,
        color: TEXT_PRIMARY,
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        border_radius: Corner::all(RADIUS_SM),
        border: Edge::all(1.0),
        border_color: BORDER_DARK,
        padding: Edge::new(0.0, 14.0, 0.0, 14.0),
        height: 30.0.into(),
        ..Default::default()
    };
    ButtonStyles {
        normal: WoodpeckerStyle {
            background_color: BG_INPUT,
            ..base
        },
        hovered: WoodpeckerStyle {
            background_color: BG_WINDOW_HOVER,
            ..base
        },
    }
}

/// Wraps panel content in a scroll context -- one per independently-scrolling region.
pub fn scrollable_panel(content: WidgetChildren) -> WidgetChildren {
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
