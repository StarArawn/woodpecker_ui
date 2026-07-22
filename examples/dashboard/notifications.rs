use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::theme;

struct Notification {
    key: &'static str,
    title: &'static str,
    detail: &'static str,
}

const NOTIFICATIONS: [Notification; 4] = [
    Notification {
        key: "deploy",
        title: "Deployment succeeded",
        detail: "billing-service v2.4.1 is live",
    },
    Notification {
        key: "alert",
        title: "Error rate spike",
        detail: "worker-pool error rate crossed 1%",
    },
    Notification {
        key: "invite",
        title: "Marcus Chen joined",
        detail: "Accepted their team invite",
    },
    Notification {
        key: "billing",
        title: "Invoice paid",
        detail: "October invoice was paid automatically",
    },
];

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct NotificationsMenuState {
    open: bool,
}

/// A self-contained bell-button + [`Drawer`] pairing -- demonstrates `Drawer`'s `Temporary`
/// variant, nested as one ordinary child among the topbar's other widgets (it owns its own
/// open/close state internally, the same way `Dropdown`/`Tooltip` do, so nothing about the
/// topbar or dashboard shell needs to know it's there). Watches `Theme` so a theme swap
/// recolors it.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WatchedResource<Theme>)]
pub struct NotificationsMenu;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    mut query: Query<(&WatchedResource<Theme>, &mut WidgetChildren), With<NotificationsMenu>>,
    state_query: Query<&NotificationsMenuState>,
) {
    let Ok((watched_theme, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let theme = watched_theme.0;

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        NotificationsMenuState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let open = state.open;

    *children = WidgetChildren::default();
    children
        .add::<IconButton>((
            IconButton,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 16.0,
                    color: theme::text_primary(&theme),
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: icons::BELL.into(),
                },
            )),
        ))
        .observe(
            *current_widget,
            move |_trigger: On<Pointer<Click>>, mut query: Query<&mut NotificationsMenuState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.open = true;
                }
            },
        );
    children.add_key("bell");

    let mut rows = WidgetChildren::default();
    for notification in NOTIFICATIONS {
        rows.add::<ListItem>((
            ListItem {
                primary_text: notification.title.into(),
                secondary_text: Some(notification.detail.into()),
                ..Default::default()
            },
            ListItemStyles {
                divider_color: theme::border(&theme),
                ..ListItemStyles::from_theme(&theme)
            },
        ));
        rows.add_key(notification.key);
    }

    children.add::<Drawer>((
        Drawer {
            open,
            position: DrawerPosition::Right,
            size: 320.0,
            ..Default::default()
        },
        DrawerStyles {
            background_color: theme::bg_topbar(&theme),
            border_color: theme::border(&theme),
            ..DrawerStyles::from_theme(&theme)
        },
        PassedChildren(
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        padding: Edge::all(theme::space_lg(&theme)),
                        border: Edge::all(0.0).bottom(1.0),
                        border_color: theme::border(&theme),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        theme::section_title_style(&theme),
                        WidgetRender::Text {
                            content: "Notifications".into(),
                        },
                    )),
                ))
                .with_key("header")
                .with_child::<List>((
                    List,
                    WoodpeckerStyle {
                        padding: Edge::all(theme::space_lg(&theme)),
                        ..Default::default()
                    },
                    PassedChildren(rows),
                ))
                .with_key("rows"),
        ),
    ));
    children.add_key("drawer");
    children.observe(
        *current_widget,
        move |_trigger: On<Change<DrawerCloseRequested>>,
              mut query: Query<&mut NotificationsMenuState>| {
            if let Ok(mut state) = query.get_mut(state_entity) {
                state.open = false;
            }
        },
    );

    children.apply(current_widget.as_parent());
}
