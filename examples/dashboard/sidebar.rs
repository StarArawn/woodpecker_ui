use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::theme;

pub const PAGE_OVERVIEW: usize = 0;
pub const PAGE_USERS: usize = 1;
pub const PAGE_SETTINGS: usize = 2;

struct NavItem {
    index: usize,
    label: &'static str,
}

const NAV_ITEMS: [NavItem; 3] = [
    NavItem {
        index: PAGE_OVERVIEW,
        label: "Overview",
    },
    NavItem {
        index: PAGE_USERS,
        label: "Users",
    },
    NavItem {
        index: PAGE_SETTINGS,
        label: "Settings",
    },
];

fn nav_button_styles(theme: &Theme) -> (ButtonStyles, ButtonStyles) {
    let base = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: 40.0.into(),
        align_items: Some(WidgetAlignItems::Center),
        padding: Edge::all(0.0)
            .left(theme::space_lg(theme))
            .right(theme::space_lg(theme)),
        font_size: 14.0,
        border: Edge::all(0.0).left(3.0),
        ..Default::default()
    };
    let inactive = ButtonStyles {
        normal: WoodpeckerStyle {
            background_color: theme::bg_sidebar(theme),
            border_color: theme::bg_sidebar(theme),
            color: theme::text_secondary(theme),
            ..base
        },
        hovered: WoodpeckerStyle {
            background_color: theme::bg_card(theme),
            border_color: theme::bg_sidebar(theme),
            color: theme::text_primary(theme),
            ..base
        },
    };
    let active = ButtonStyles {
        normal: WoodpeckerStyle {
            background_color: theme::bg_card(theme),
            border_color: theme::accent(theme),
            color: theme::text_primary(theme),
            ..base
        },
        hovered: WoodpeckerStyle {
            background_color: theme::bg_card(theme),
            border_color: theme::accent(theme),
            color: theme::text_primary(theme),
            ..base
        },
    };
    (inactive, active)
}

pub fn build_sidebar(theme: &Theme) -> WidgetChildren {
    let mut nav = WidgetChildren::default();
    for item in NAV_ITEMS {
        let (inactive, active) = nav_button_styles(theme);
        nav.add::<TabButton>((
            TabButtonBundle {
                tab_button: TabButton {
                    index: item.index,
                    title: item.label.into(),
                },
                internal_styles: WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            TabButtonStyles {
                inactive,
                active,
                underline_color: Color::NONE,
            },
        ));
        nav.add_key(item.index.to_string());
    }

    WidgetChildren::default()
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: 60.0.into(),
                align_items: Some(WidgetAlignItems::Center),
                padding: Edge::all(0.0).left(theme::space_lg(theme)),
                border: Edge::all(0.0).bottom(1.0),
                border_color: theme::border(theme),
                ..Default::default()
            },
            WidgetRender::Quad,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 18.0,
                    color: theme::text_primary(theme),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Woodpecker UI Dashboard".into(),
                },
            )),
        ))
        .with_key("brand")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                padding: Edge::all(theme::space_sm(theme)).left(0.0).right(0.0),
                ..Default::default()
            },
            nav,
        ))
        .with_key("nav")
}
