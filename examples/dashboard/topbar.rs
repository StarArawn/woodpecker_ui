use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::{UserPage, UserSearchQuery};
use crate::notifications::NotificationsMenu;
use crate::theme;

/// Builds the dashboard's top [`AppBar`] -- generalizes what used to be a hand-built `Element`
/// wrapper (breadcrumb, spacer, search, divider, notifications, avatar all laid out by hand).
/// `AppBar` supplies the fixed height/border/background and the leading/trailing split (its
/// own internal spacer replaces the hand-built one), so this only needs to build the two slots.
pub fn build_topbar(
    theme: &Theme,
    icon_font: &IconFont,
    current_widget: CurrentWidget,
) -> impl Bundle + Clone {
    let trailing = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            // `TextBox::render` always forces its own `width` to `Percentage(100.0)`
            // regardless of what `TextboxStyles::normal.width` says (that field only affects
            // color/border/padding/etc, never the box's own width/height) -- so a `TextBox`
            // declared directly, with no explicitly-sized wrapper around it, just inherits
            // whatever width its immediate parent slot happens to resolve to. This wrapper is
            // what actually pins the search box to 240px (matching the working pattern
            // `users_page.rs::form_text_input` already uses inside a sized `labeled_control`).
            WoodpeckerStyle {
                width: 240.0.into(),
                height: 34.0.into(),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<TextBox>((
                    TextBox {
                        initial_value: "".into(),
                        ..Default::default()
                    },
                    TextboxStyles {
                        // Layered on top of `TextboxStyles::from_theme(theme)` (not
                        // `..Default::default()`) -- `Default` bakes in `Theme::default()`
                        // (always dark) for every field this doesn't explicitly touch, so
                        // `hovered`/`focused`/`cursor` would stay frozen dark regardless of the
                        // live theme (visible as a jarring near-black fill when this box is
                        // focused under a light theme). Only `normal` needs dashboard-specific
                        // tweaks (padding/font_size); the rest of the theme-derived base is
                        // already correct as-is.
                        normal: WoodpeckerStyle {
                            padding: Edge::all(0.0)
                                .left(theme::space_sm(theme))
                                .right(theme::space_sm(theme)),
                            font_size: 13.0,
                            ..TextboxStyles::from_theme(theme).normal
                        },
                        ..TextboxStyles::from_theme(theme)
                    },
                ))
                .with_observe(
                    current_widget,
                    move |trigger: On<Change<TextChanged>>,
                          mut search: ResMut<UserSearchQuery>,
                          mut page: ResMut<UserPage>| {
                        search.0 = trigger.data.value.clone();
                        // A new search should always start back on page 1.
                        page.0 = 0;
                    },
                ),
        ))
        .with_key("search")
        .with_child::<Divider>((
            Divider { vertical: true },
            DividerStyles {
                color: theme::border(theme),
            },
            WoodpeckerStyle {
                height: 24.0.into(),
                ..Default::default()
            },
        ))
        .with_key("divider")
        .with_child::<NotificationsMenu>(NotificationsMenu)
        .with_key("notifications")
        .with_child::<Tooltip>((
            Tooltip {
                text: "Ava Thompson".into(),
                placement: PopoverPlacement::Bottom,
            },
            PassedChildren(WidgetChildren::default().with_child::<Avatar>((Avatar {
                initials: "AT".into(),
                color: Some(theme::accent(theme)),
                size: 34.0,
            },))),
        ))
        .with_key("avatar")
        .with_child::<IconButton>((
            IconButton,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 16.0,
                    color: theme::text_primary(theme),
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    // Shows the icon for what a click switches *to* (matches the convention
                    // most apps use -- e.g. GitHub's own theme toggle) rather than the
                    // currently-active mode.
                    content: if *theme == Theme::dark() {
                        icons::SUN.into()
                    } else {
                        icons::MOON.into()
                    },
                },
            )),
        ))
        .with_observe(
            current_widget,
            // Reads the *live* `Theme` fresh at click time rather than closing over the
            // `theme` this build call happened to see -- `.observe()` only ever attaches its
            // observer once per reused entity (see `WidgetChildren::observe`'s doc comment),
            // so a captured-by-value snapshot would permanently freeze at whichever theme was
            // active on first mount, making every click after the first compare against that
            // stale value instead of the current one (same reasoning as `examples/theme.rs`'s
            // own toggle button).
            |_trigger: On<Pointer<Click>>, theme: Res<Theme>, mut commands: Commands| {
                let next = if *theme == Theme::dark() {
                    Theme::light()
                } else {
                    Theme::dark()
                };
                commands.insert_resource(next);
            },
        )
        .with_key("theme-toggle");

    (
        AppBar {
            position: AppBarPosition::Top,
            height: 60.0,
            ..Default::default()
        },
        AppBarStyles {
            background_color: theme::bg_topbar(theme),
            border_color: theme::border(theme),
            ..AppBarStyles::default()
        },
        AppBarLeading(
            WidgetChildren::default()
                .with_child::<Breadcrumbs>((Breadcrumbs {
                    items: vec!["Acme Corp".into(), "Production".into(), "Overview".into()],
                },))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<BreadcrumbClicked>>, mut queue: ResMut<ToastQueue>| {
                        queue.push(
                            format!(
                                "Switch workspace: segment {} coming soon",
                                trigger.data.index
                            ),
                            BadgeVariant::Info,
                        );
                    },
                )
                .with_key("breadcrumbs"),
        ),
        AppBarTrailing(trailing),
    )
}
