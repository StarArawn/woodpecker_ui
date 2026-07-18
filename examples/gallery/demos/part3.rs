use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use super::part2::alert_text;

pub(super) fn demo_alert() -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Alert>((
            Alert {
                variant: BadgeVariant::Info,
                ..Default::default()
            },
            PassedChildren(
                WidgetChildren::default()
                    .with_child::<Element>(alert_text("A new version is available.")),
            ),
        ))
        .with_key("info")
        .with_child::<Alert>((
            Alert {
                variant: BadgeVariant::Danger,
                dismissible: true,
            },
            PassedChildren(
                WidgetChildren::default()
                    .with_child::<Element>(alert_text("Failed to connect to the server.")),
            ),
        ))
        .with_key("danger")
}

pub(super) fn demo_progress_bar() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 260.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<ProgressBar>(ProgressBar { value: 0.6 }),
    ))
}

pub(super) fn demo_skeleton() -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Skeleton>((Skeleton {
            variant: SkeletonVariant::Circle,
            width: 40.0,
            height: 40.0,
        },))
        .with_key("avatar")
        .with_child::<Skeleton>((Skeleton {
            variant: SkeletonVariant::Text,
            width: 140.0,
            height: 14.0,
        },))
        .with_key("line")
        .with_child::<Skeleton>((Skeleton {
            variant: SkeletonVariant::Rect,
            width: 220.0,
            height: 100.0,
        },))
        .with_key("card")
}

pub(super) fn demo_spinner() -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Spinner>((Spinner {
            size: 40.0,
            mode: SpinnerMode::Indeterminate,
        },))
        .with_key("indeterminate")
        .with_child::<Spinner>((Spinner {
            size: 40.0,
            mode: SpinnerMode::Determinate(0.6),
        },))
        .with_key("determinate")
}

/// Just the trigger button -- unlike most overlay widgets here, `Toast` needs no per-instance
/// open/closed state of its own, so pushing to the global [`ToastQueue`] resource from the
/// click observer is enough. `ToastViewport` itself is NOT spawned here: its own doc comment
/// says to place exactly one near the app's root, so it lives once in `startup()` alongside
/// `OverlayRootWidget` -- spawning a fresh one every time this story is selected (and
/// despawning it on navigating away) would portal a new, independent viewport to the same
/// overlay root each time, all reading the same queue, which looked like toasts never
/// dismissing when really it was stale/duplicate viewports piling up.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_toast_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct ToastDemo;

fn render_toast_demo(
    current_widget: Res<CurrentWidget>,
    mut query: Query<&mut WidgetChildren, With<ToastDemo>>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<WButton>(WButton::text("Show toast"))
        .observe(
            current_widget,
            |_trigger: On<Pointer<Click>>, mut queue: ResMut<ToastQueue>| {
                queue.push("Changes saved successfully.", BadgeVariant::Success);
            },
        );
    children.add_key("trigger");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_toast() -> WidgetChildren {
    WidgetChildren::default().with_child::<ToastDemo>(ToastDemo)
}

fn card_stat_row(label: &str, value: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            justify_content: Some(WidgetAlignContent::SpaceBetween),
            margin: Edge::all(0.0).bottom(8.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::new(0.6, 0.63, 0.7, 1.0).into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.into(),
                },
            ))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::WHITE.into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: value.into(),
                },
            ))
            .with_key("value"),
    )
}

pub(super) fn demo_card() -> WidgetChildren {
    WidgetChildren::default().with_child::<Card>((
        Card {
            title: Some("Server Status".into()),
            ..Default::default()
        },
        WoodpeckerStyle {
            width: 260.0.into(),
            ..Default::default()
        },
        PassedChildren(
            WidgetChildren::default()
                .with_child::<Element>(card_stat_row("Uptime", "14d 6h"))
                .with_key("uptime")
                .with_child::<Element>(card_stat_row("Requests/sec", "1,204"))
                .with_key("rps"),
        ),
    ))
}

pub(super) fn demo_paper() -> WidgetChildren {
    let mut tiles = WidgetChildren::default();
    for elevation in 0..=2u8 {
        tiles.add::<Paper>((
            Paper { elevation },
            WoodpeckerStyle {
                width: 120.0.into(),
                height: 90.0.into(),
                align_items: Some(WidgetAlignItems::Center),
                justify_content: Some(WidgetAlignContent::Center),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    color: Srgba::WHITE.into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("elevation {elevation}"),
                },
            )),
        ));
        tiles.add_key(format!("elevation-{elevation}"));
    }
    tiles
}

fn accordion_body_text(content: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            text_wrap: TextWrap::WordOrGlyph,
            ..Default::default()
        },
        WidgetRender::Text {
            content: content.into(),
        },
    )
}

pub(super) fn demo_accordion() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 380.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Accordion>((
            Accordion {
                mode: AccordionMode::Single,
            },
            PassedChildren(
                WidgetChildren::default()
                    .with_child::<AccordionItem>((
                        AccordionItem {
                            key: "shipping".into(),
                            label: "Shipping".into(),
                        },
                        PassedChildren(WidgetChildren::default().with_child::<Element>(
                            accordion_body_text("Orders ship within 2 business days."),
                        )),
                    ))
                    .with_key("shipping")
                    .with_child::<AccordionItem>((
                        AccordionItem {
                            key: "returns".into(),
                            label: "Returns".into(),
                        },
                        PassedChildren(WidgetChildren::default().with_child::<Element>(
                            accordion_body_text("Returns are accepted within 30 days."),
                        )),
                    ))
                    .with_key("returns"),
            ),
        )),
    ))
}

pub(super) fn demo_app_bar() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<AppBar>((
            AppBar {
                title: Some("Woodpecker UI".into()),
                position: AppBarPosition::Top,
                ..Default::default()
            },
            AppBarTrailing(
                WidgetChildren::default()
                    .with_child::<WButton>(WButton::text("Sign out"))
                    .with_key("sign-out"),
            ),
        )),
    ))
}

/// Mirrors `examples/drawer.rs`'s own `DrawerDemo` shape -- `Drawer::open` is caller-owned (no
/// internal fallback), so a trigger + `Drawer` needs a dedicated state-holding mini-widget.
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct DrawerDemoState {
    open: bool,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_drawer_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct DrawerDemo;

fn drawer_nav_item(label: &str) -> ListItem {
    ListItem {
        primary_text: label.into(),
        ..Default::default()
    }
}

fn render_drawer_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<DrawerDemo>>,
    state_query: Query<&DrawerDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity =
        hooks.use_state(&mut commands, *current_widget, DrawerDemoState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let open = state.open;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children.add::<WButton>(WButton::text("Open menu")).observe(
        current_widget,
        move |_trigger: On<Pointer<Click>>, mut query: Query<&mut DrawerDemoState>| {
            if let Ok(mut state) = query.get_mut(state_entity) {
                state.open = true;
            }
        },
    );
    children.add_key("open");

    children.add::<Drawer>((
        Drawer {
            open,
            position: DrawerPosition::Left,
            ..Default::default()
        },
        PassedChildren(
            WidgetChildren::default().with_child::<List>((
                List,
                WoodpeckerStyle {
                    padding: Edge::all(20.0),
                    ..Default::default()
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<ListItem>(drawer_nav_item("Dashboard"))
                        .with_key("dashboard")
                        .with_child::<ListItem>(drawer_nav_item("Settings"))
                        .with_key("settings"),
                ),
            )),
        ),
    ));
    children.add_key("drawer");
    children.observe(
        current_widget,
        move |_trigger: On<Change<DrawerCloseRequested>>,
              mut query: Query<&mut DrawerDemoState>| {
            if let Ok(mut state) = query.get_mut(state_entity) {
                state.open = false;
            }
        },
    );

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_drawer() -> WidgetChildren {
    WidgetChildren::default().with_child::<DrawerDemo>(DrawerDemo)
}

/// Mirrors `examples/modal.rs`'s own state shape, simplified to a single (non-recursive)
/// modal -- `Modal::visible` is caller-owned (no internal fallback), same reasoning as
/// [`DrawerDemo`].
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ModalDemoState {
    visible: bool,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_modal_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct ModalDemo;

fn render_modal_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<ModalDemo>>,
    state_query: Query<&ModalDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity =
        hooks.use_state(&mut commands, *current_widget, ModalDemoState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let visible = state.visible;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<WButton>(WButton::text("Open modal"))
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut query: Query<&mut ModalDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.visible = true;
                }
            },
        );
    children.add_key("open");

    children.add::<Modal>((
        Modal {
            visible,
            title: "I am a modal".into(),
            ..Default::default()
        },
        PassedChildren(
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    flex_direction: WidgetFlexDirection::Column,
                    padding: Edge::all(10.0),
                    width: Units::Percentage(100.0),
                    ..Default::default()
                },
                WidgetChildren::default()
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 16.0,
                            margin: Edge::all(0.0).bottom(10.0),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: "Hello! This is a modal window.".into(),
                        },
                    ))
                    .with_child::<WButton>(WButton::text("Close"))
                    .with_observe(
                        current_widget,
                        move |_trigger: On<Pointer<Click>>,
                              mut query: Query<&mut ModalDemoState>| {
                            if let Ok(mut state) = query.get_mut(state_entity) {
                                state.visible = false;
                            }
                        },
                    ),
            )),
        ),
    ));
    children.add_key("modal");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_modal() -> WidgetChildren {
    WidgetChildren::default().with_child::<ModalDemo>(ModalDemo)
}

