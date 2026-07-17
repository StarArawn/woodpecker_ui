use bevy::prelude::*;
use woodpecker_ui::prelude::*;

mod mock_data;
mod notifications;
mod overview;
mod settings_page;
mod sidebar;
mod theme;
mod topbar;
mod users_page;

use mock_data::{
    seed_activity, seed_revenue_trend, seed_users, CompactRows, EditingUser, PermissionAssignment,
    UserPage, UserSearchQuery, Users,
};
use notifications::NotificationsMenu;
use overview::MrrInfoPopover;
use settings_page::{PermissionsPanel, PreviewSplitWidth, PreviewSplitter};
use users_page::{RoleFilterChips, UserEditorModal, UsersTable};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<DashboardShell>()
        .register_widget::<UsersTable>()
        .register_widget::<UserEditorModal>()
        .register_widget::<RoleFilterChips>()
        .register_widget::<NotificationsMenu>()
        .register_widget::<PermissionsPanel>()
        .register_widget::<MrrInfoPopover>()
        .register_widget::<PreviewSplitter>()
        .register_watched_resource::<Theme>()
        .register_watched_resource::<Users>()
        .register_watched_resource::<EditingUser>()
        .register_watched_resource::<UserSearchQuery>()
        .register_watched_resource::<UserPage>()
        .register_watched_resource::<PermissionAssignment>()
        .register_watched_resource::<CompactRows>()
        .register_watched_resource::<PreviewSplitWidth>()
        .insert_resource(Theme::dark())
        .insert_resource(seed_users())
        .insert_resource(EditingUser::default())
        .insert_resource(UserSearchQuery::default())
        .insert_resource(UserPage::default())
        .insert_resource(PermissionAssignment::default())
        .insert_resource(CompactRows::default())
        .insert_resource(PreviewSplitWidth::default())
        .insert_resource(settings_page::PreviewSplitWidthAtDragStart::default())
        .register_widget::<WhatsNewWindow>()
        .add_systems(Startup, startup)
        .run();
}

/// Per-widget state for [`WhatsNewWindow`] -- owns its own open/closed state (seeded open),
/// the same pattern [`notifications::NotificationsMenu`] uses, since [`WoodpeckerWindow`]'s own
/// `visible` field is purely caller-controlled (no internal state, no built-in close button in
/// its title bar) -- something has to own "is it open" and drive `visible` from it.
#[derive(Component, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct WhatsNewState {
    open: bool,
}

impl Default for WhatsNewState {
    fn default() -> Self {
        Self { open: true }
    }
}

/// A dismissible "What's New" floating window -- see [`WhatsNewState`] for why this wraps
/// [`WoodpeckerWindow`] instead of declaring one directly.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_whats_new)]
#[require(WoodpeckerStyle, WidgetChildren, WatchedResource<Theme>)]
struct WhatsNewWindow;

fn render_whats_new(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    mut query: Query<(&WatchedResource<Theme>, &mut WidgetChildren), With<WhatsNewWindow>>,
    state_query: Query<&WhatsNewState>,
) {
    let Ok((watched_theme, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let theme = watched_theme.0;

    let state_entity = hooks.use_state(&mut commands, *current_widget, WhatsNewState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let open = state.open;
    let current_widget = *current_widget;

    // `WoodpeckerWindow::default()`'s own `window_styles`/`title_styles` are baked from
    // `WindowStyles::default()`, i.e. `Theme::default()` (always dark, since `Default` can't
    // read the live `Theme` resource) -- not the crate's own `register_themed_style::<WindowStyles>()`
    // resync, which only ever touches the separate `WindowStyles` *component*, never this
    // widget's own `window_styles`/`title_styles` *fields* (see `WoodpeckerWindow::render`,
    // which paints straight from `window.window_styles`, not the sibling component). Since
    // this widget re-renders on every `Theme` swap (`WatchedResource<Theme>` above), computing
    // both from the *live* `theme` here -- mirroring `WoodpeckerWindow::default()`'s own
    // construction -- is what actually keeps this window in sync with the rest of the
    // dashboard, instead of freezing at whatever theme was active on first mount.
    let window_theme_styles = WindowStyles::from_theme(&theme);

    *children = WidgetChildren::default();
    children
        .add::<WoodpeckerWindow>((
            WoodpeckerWindow {
                title: "What's New".into(),
                initial_position: Vec2::new(10.0, 10_000.0),
                visible: open,
                window_styles: WoodpeckerStyle {
                    background_color: window_theme_styles.body_background,
                    border_color: window_theme_styles.border_color,
                    border: Edge::all(2.0),
                    border_radius: Corner::all(window_theme_styles.corner_radius),
                    flex_direction: WidgetFlexDirection::Column,
                    // A fixed `width` (not just `min_width`) so the wrapper's size is known
                    // on the very first render -- an `Auto`-width window whose only child is
                    // wrapping text has a circular sizing dependency (text needs a width to
                    // know its height, width needs the text's measured size) that only
                    // resolves after a few extra re-renders; a genuinely static window like
                    // this one (no other reactive prop driving repeated renders after mount)
                    // doesn't get enough of those and can stay stuck at a wrong pre-settle
                    // size (see `diff_window_wrapper_layout`'s doc comment for the same
                    // failure mode on `examples/viewport.rs`).
                    width: 280.0.into(),
                    ..Default::default()
                },
                title_styles: WoodpeckerStyle {
                    background_color: window_theme_styles.title_bar_background,
                    height: Units::Pixels(40.0),
                    width: Units::Percentage(100.0),
                    align_items: Some(WidgetAlignItems::Center),
                    padding: Edge::all(0.0).left(10.0).right(10.0),
                    // Separates the title bar from the body below it -- without this, the two
                    // fills (title_bar_background/body_background) blend together whenever
                    // they're close in tone, most noticeably in light mode.
                    border: Edge::all(2.0).bottom(0.0),
                    border_color: window_theme_styles.border_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            // Replaces the title bar's default (plain text) content with a title-text-plus-
            // close-button row -- `WoodpeckerWindow` itself has no built-in close affordance
            // (its `visible` field is purely caller-controlled), and the title bar is the
            // conventional place for one, not the body content.
            TitleChildren(
                WidgetChildren::default()
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            flex_grow: 1.0,
                            font_size: theme.font_size,
                            color: WindowStyles::from_theme(&theme).title_text_color,
                            text_wrap: TextWrap::None,
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: "What's New".into(),
                        },
                    ))
                    .with_key("title-text")
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 13.0,
                            color: WindowStyles::from_theme(&theme)
                                .title_text_color
                                .with_alpha(0.6),
                            font: Some(icon_font.0.id()),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: icons::X.into(),
                        },
                        Pickable::default(),
                    ))
                    .with_observe(
                        current_widget,
                        move |mut trigger: On<Pointer<Click>>,
                              mut state_query: Query<&mut WhatsNewState>| {
                            // Stops the click from also bubbling into the title bar's own
                            // `Pointer<Press>`-based drag/focus handling -- closing shouldn't
                            // also shift the (about to disappear) window to the front.
                            trigger.propagate(false);
                            if let Ok(mut state) = state_query.get_mut(state_entity) {
                                state.open = false;
                            }
                        },
                    )
                    // Not `.with_hover_cursor(...)` -- that doesn't stop `Pointer<Over>`/
                    // `Pointer<Out>` from bubbling, so the title bar's own drag-cursor
                    // observers (which set `Grab` on the whole window whenever the pointer is
                    // anywhere over the title bar, including this button) fire right after and
                    // clobber the `Pointer` cursor this sets. Same `propagate(false)` reasoning
                    // as the click observer above, just for hover instead of click.
                    .with_observe(
                        current_widget,
                        move |mut trigger: On<Pointer<Over>>, mut cursor: CursorSetter| {
                            trigger.propagate(false);
                            cursor.set(SystemCursorIcon::Pointer);
                        },
                    )
                    .with_observe(
                        current_widget,
                        move |mut trigger: On<Pointer<Out>>, mut cursor: CursorSetter| {
                            trigger.propagate(false);
                            cursor.set(SystemCursorIcon::Default);
                        },
                    )
                    .with_key("close"),
            ),
            PassedChildren(
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        padding: Edge::all(theme::space_md(&theme)),
                        flex_direction: WidgetFlexDirection::Column,
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: 13.0,
                                color: theme::text_secondary(&theme),
                                text_wrap: TextWrap::WordOrGlyph,
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: "Right-click a row in Users for quick actions. Drag \
                                          this window's title bar to move it out of the way."
                                    .into(),
                            },
                        ))
                        .with_key("body"),
                )),
            ),
        ))
        .add_key("window");

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<DashboardShell>(DashboardShell)
            .with_key("shell-root"),
    );
}

/// Builds the entire dashboard tree (topbar, sidebar, pages, toasts, speed dial, floating
/// window, overlay root) -- previously a one-shot `Startup` system, now a real, reactive
/// widget so a `Theme` swap (see the topbar's own theme-toggle button) can rebuild the whole
/// tree with the new theme's colors, the same way any other watched-resource-driven widget in
/// this crate re-renders on change. `WatchedResource<Theme>` (not a bare `Res<Theme>` query
/// param) is what actually triggers that re-render -- see `runner::system`'s diffing, which
/// only re-invokes a widget's `render()` when one of its own components changes.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_dashboard_shell)]
#[require(
    WidgetChildren,
    WatchedResource<Theme>,
    WoodpeckerStyle = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        ..Default::default()
    }
)]
struct DashboardShell;

fn render_dashboard_shell(
    asset_server: Res<AssetServer>,
    icon_font: Res<IconFont>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<Theme>, &mut WidgetChildren), With<DashboardShell>>,
) {
    let Ok((watched_theme, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let theme = watched_theme.0;
    let current_widget = *current_widget;

    let activity = seed_activity();
    let (revenue_this_year, revenue_last_year) = seed_revenue_trend();

    let main_area = WidgetChildren::default()
        .with_child::<AppBar>(topbar::build_topbar(&theme, &icon_font, current_widget))
        .with_key("topbar")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                // No explicit `height`: `flex_grow: 1.0` fills the remaining height after
                // the topbar's fixed 60px without fighting an explicit `height: 100%`.
                width: Units::Percentage(100.0),
                flex_grow: 1.0,
                padding: Edge::all(theme::space_lg(&theme)),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<TabContent>(TabContentBundle {
                    tab_content: TabContent {
                        index: sidebar::PAGE_OVERVIEW,
                    },
                    children: PassedChildren(overview::build_overview(
                        &theme,
                        current_widget,
                        &activity,
                        &revenue_this_year,
                        &revenue_last_year,
                    )),
                    ..Default::default()
                })
                .with_key("page-overview")
                .with_child::<TabContent>(TabContentBundle {
                    tab_content: TabContent {
                        index: sidebar::PAGE_USERS,
                    },
                    children: PassedChildren(users_page::build_users_page(&theme, current_widget)),
                    ..Default::default()
                })
                .with_key("page-users")
                .with_child::<TabContent>(TabContentBundle {
                    tab_content: TabContent {
                        index: sidebar::PAGE_SETTINGS,
                    },
                    children: PassedChildren(settings_page::build_settings_page(
                        &theme,
                        current_widget,
                        asset_server.load("woodpecker.jpg"),
                    )),
                    ..Default::default()
                })
                .with_key("page-settings"),
        ))
        .with_key("page")
        .with_child::<BottomNavigation>(BottomNavigation {
            items: vec![
                BottomNavItem {
                    icon: icons::FILE_PLUS.into(),
                    label: "New".into(),
                },
                BottomNavItem {
                    icon: icons::MAGNIFYING_GLASS.into(),
                    label: "Search".into(),
                },
                BottomNavItem {
                    icon: icons::DOWNLOAD_SIMPLE.into(),
                    label: "Export".into(),
                },
                BottomNavItem {
                    icon: icons::QUESTION.into(),
                    label: "Help".into(),
                },
            ],
            height: 52.0,
            ..Default::default()
        })
        .with_key("quick-actions")
        .with_observe(
            current_widget,
            |trigger: On<Change<BottomNavigationChanged>>, mut queue: ResMut<ToastQueue>| {
                queue.push(
                    format!("{} quick action selected", trigger.data.label),
                    BadgeVariant::Info,
                );
            },
        );

    let app_shell = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Row,
            background_color: theme::bg_app(&theme),
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default()
            .with_child::<NavigationRail>(NavigationRail {
                items: vec![
                    NavigationRailItem {
                        icon: icons::BUILDINGS.into(),
                        label: "Acme".into(),
                    },
                    NavigationRailItem {
                        icon: icons::CHART_LINE_UP.into(),
                        label: "Reports".into(),
                    },
                    NavigationRailItem {
                        icon: icons::CREDIT_CARD.into(),
                        label: "Billing".into(),
                    },
                    NavigationRailItem {
                        icon: icons::BOOKS.into(),
                        label: "Docs".into(),
                    },
                ],
                ..Default::default()
            })
            .with_key("workspace-rail")
            .with_observe(
                current_widget,
                |trigger: On<Change<NavigationRailChanged>>, mut queue: ResMut<ToastQueue>| {
                    queue.push(
                        format!("Switched to {} workspace", trigger.data.label),
                        BadgeVariant::Info,
                    );
                },
            )
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: 240.0.into(),
                    height: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Column,
                    background_color: theme::bg_sidebar(&theme),
                    border: Edge::all(0.0).right(1.0),
                    border_color: theme::border(&theme),
                    ..Default::default()
                },
                sidebar::build_sidebar(&theme),
                WidgetRender::Quad,
            ))
            .with_key("sidebar")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    // No explicit `width`: `flex_grow: 1.0` alone fills the remaining
                    // space after the sidebar's fixed 240px, in this Row-direction parent.
                    height: Units::Percentage(100.0),
                    flex_grow: 1.0,
                    flex_direction: WidgetFlexDirection::Column,
                    ..Default::default()
                },
                main_area,
            ))
            .with_key("main"),
    ));

    *children = WidgetChildren::default()
        .with_child::<TabContextProvider>((TabContextProviderBundle {
            children: PassedChildren(app_shell),
            ..Default::default()
        },))
        .with_key("shell")
        // Sits alongside (not inside) the app shell -- it's `position: Fixed`, pinned to
        // the bottom-right of the whole screen regardless of where it's declared.
        .with_child::<ToastViewport>(ToastViewport)
        .with_key("toasts")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                right: 24.0.into(),
                // Clears the bottom quick-actions bar (52px tall) with some margin.
                bottom: 76.0.into(),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<SpeedDial>((SpeedDial {
                icon: icons::PLUS.into(),
                actions: vec![
                    SpeedDialAction {
                        icon: icons::USER_PLUS.into(),
                    },
                    SpeedDialAction {
                        icon: icons::CHART_BAR.into(),
                    },
                    SpeedDialAction {
                        icon: icons::ENVELOPE_SIMPLE.into(),
                    },
                ],
                placement: PopoverPlacement::Top,
            },)),
        ))
        .with_observe(
            current_widget,
            |trigger: On<Change<SpeedDialActionClicked>>, mut queue: ResMut<ToastQueue>| {
                let label = ["New user", "New report", "Invite teammate"][trigger.data.index];
                queue.push(format!("{label} coming soon"), BadgeVariant::Info);
            },
        )
        .with_key("speed-dial")
        .with_child::<WindowingContextProvider>(
            WidgetChildren::default()
                .with_child::<WhatsNewWindow>(WhatsNewWindow)
                .with_key("whats-new"),
        )
        .with_key("floating-window")
        .with_child::<OverlayRootWidget>(OverlayRootWidget);

    children.apply(current_widget.as_parent());
}
