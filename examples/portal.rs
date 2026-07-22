//! Demonstrates `WidgetChildren::portal()`, the React-portal equivalent this crate exposes to
//! any widget author (not just the crate's own built-ins): a widget declared deep inside one
//! part of the tree can be *physically* rendered somewhere else entirely, while still being
//! *logically* declared where you wrote it.
//!
//! Click "Toggle menu" on both cards below. Both open the exact same dropdown-style menu, in
//! the exact same spot in the code. The left card's menu is a normal child of its `Clip`'d
//! box, so it gets visually cut off the moment it grows past that box's fixed height -- the
//! same problem a real dropdown/tooltip hits inside any clipped or scrollable container. The
//! right card's menu is `.portal()`-ed, so it's no longer a physical descendant of the clip at
//! all -- it escapes entirely and renders in full, in the corner overlay.
//!
//! (The portaled menu isn't positioned relative to its own trigger -- unlike this crate's own
//! `Tooltip`/`Popover`/`Dropdown`/etc., which compute their floating content's position from
//! the trigger's own `WidgetLayout`, see each widget's own `render`. This example keeps that
//! math out to stay focused on the escape mechanism itself.)
//!
//! Every `.portal()` call with no explicit target resolves to the shared `OverlayRoot`
//! resource, which points at wherever `OverlayRootWidget` is declared -- see its own doc
//! comment for why the app has to declare it explicitly rather than the crate auto-injecting
//! it. This example uses the crate's own `OverlayRootWidget`, the same one `Modal`/`Toast`/
//! `Dropdown`/etc. all portal into -- a real third-party widget has no need to build its own
//! portal target the way this example's own `DemoOverlayRoot` used to before `OverlayRootWidget`
//! existed.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

#[derive(Component, Reflect, Clone, Copy, Default, PartialEq, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
struct MenuCardState {
    open: bool,
}

/// A simple custom (non-built-in) widget -- proves `.portal()` works for any third-party
/// widget, not just the ones this crate ships. A small card with a clipped viewport and a
/// button that opens a dropdown-style menu, optionally `.portal()`-ed.
#[derive(Component, Widget, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(menu_trigger_card_render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct MenuTriggerCard {
    heading: String,
    use_portal: bool,
}

fn menu_item(text: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            font_size: 15.0,
            color: Color::WHITE,
            padding: Edge::all(0.0).top(4.0).bottom(4.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: text.into(),
        },
    )
}

fn menu_trigger_card_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&MenuTriggerCard, &mut WoodpeckerStyle, &mut WidgetChildren)>,
    state_query: Query<&MenuCardState>,
) {
    let Ok((card, mut style, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, MenuCardState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    *style = WoodpeckerStyle {
        width: 260.0.into(),
        flex_direction: WidgetFlexDirection::Column,
        gap: (0.0.into(), 8.0.into()),
        ..Default::default()
    };

    *children = WidgetChildren::default();

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 15.0,
            color: Srgba::new(0.7, 0.7, 0.78, 1.0).into(),
            ..Default::default()
        },
        WidgetRender::Text {
            content: card.heading.clone(),
        },
    ));
    children.add_key("heading");

    // The clipped viewport -- fixed at just tall enough for the trigger row, so an opened
    // menu (which is much taller) has nowhere to go but to get visually cut off, unless it's
    // portaled out of this box's physical `Children` entirely.
    let mut clip_children = WidgetChildren::default();
    clip_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            align_items: Some(WidgetAlignItems::Center),
            width: Units::Percentage(100.0),
            height: 36.0.into(),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 15.0,
                    color: Color::WHITE,
                    flex_grow: 1.0,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Options".into(),
                },
            ))
            .with_child::<WButton>((
                WButton,
                ButtonStyles {
                    normal: WoodpeckerStyle {
                        width: 150.0.into(),
                        flex_shrink: 0.0,
                        ..ButtonStyles::default().normal
                    },
                    hovered: WoodpeckerStyle {
                        width: 150.0.into(),
                        flex_shrink: 0.0,
                        ..ButtonStyles::default().hovered
                    },
                },
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 14.0,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Toggle menu".into(),
                    },
                )),
            ))
            .with_key("toggle_button")
            .with_observe(
                *current_widget,
                move |_: On<Pointer<Click>>, mut state_query: Query<&mut MenuCardState>| {
                    if let Ok(mut state) = state_query.get_mut(state_entity) {
                        state.open = !state.open;
                    }
                },
            ),
    ));
    clip_children.add_key("row");

    if state.open {
        let mut menu = WidgetChildren::default();
        for item in ["Rename", "Duplicate", "Delete"] {
            menu.add::<Element>(menu_item(item));
            menu.add_key(item);
        }
        clip_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(0.0).top(6.0),
                padding: Edge::all(10.0),
                width: 160.0.into(),
                background_color: Srgba::new(0.16, 0.16, 0.21, 1.0).into(),
                border: Edge::all(1.0),
                border_color: Srgba::new(0.45, 0.45, 0.55, 1.0).into(),
                border_radius: Corner::all(6.0),
                flex_direction: WidgetFlexDirection::Column,
                // `OverlayRootWidget` (the shared portal target every `.portal()`-ed widget in
                // the app lands in) deliberately carries no layout opinion of its own -- it's
                // shared by potentially many different widgets' floating content at once, so
                // each one positions itself explicitly, the same way `Tooltip`/`Popover`/
                // `Dropdown`/etc. all do internally. Pinned to the bottom-right corner here
                // just to have a fixed, visually obvious landing spot for the demo.
                position: if card.use_portal {
                    WidgetPosition::Fixed
                } else {
                    WidgetPosition::default()
                },
                right: 24.0.into(),
                bottom: 24.0.into(),
                z_index: card
                    .use_portal
                    .then_some(WidgetZ::Global(StackingTier::Popover as u32)),
                ..Default::default()
            },
            menu,
            WidgetRender::Quad,
        ));
        clip_children.add_key("menu");
        if card.use_portal {
            clip_children.portal();
        }
    }

    children.add::<Clip>((
        Clip,
        WoodpeckerStyle {
            width: 260.0.into(),
            height: 60.0.into(),
            background_color: Srgba::new(0.1, 0.1, 0.14, 1.0).into(),
            border: Edge::all(2.0),
            border_color: Srgba::new(0.4, 0.4, 0.48, 1.0).into(),
            border_radius: Corner::all(8.0),
            padding: Edge::all(8.0),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        clip_children,
    ));
    children.add_key("clip_box");

    children.apply(current_widget.as_parent());
}

#[derive(Component, Widget, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(portal_demo_render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct PortalDemo;

fn portal_demo_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&mut WoodpeckerStyle, &mut WidgetChildren), With<PortalDemo>>,
) {
    let Ok((mut style, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    *style = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Column,
        padding: Edge::all(24.0),
        gap: (0.0.into(), 24.0.into()),
        ..Default::default()
    };

    *children = WidgetChildren::default();

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 18.0,
            color: Color::WHITE,
            width: 700.0.into(),
            ..Default::default()
        },
        WidgetRender::Text {
            content: "Click \"Toggle menu\" on both cards. The left menu gets clipped by its \
                      card the moment it opens. The right one is .portal()-ed and escapes to \
                      the bottom-right corner instead."
                .into(),
        },
    ));
    children.add_key("caption");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            gap: (32.0.into(), 0.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<MenuTriggerCard>(MenuTriggerCard {
                heading: "Without .portal() -- menu gets clipped".into(),
                use_portal: false,
            })
            .with_key("no_portal")
            .with_child::<MenuTriggerCard>(MenuTriggerCard {
                heading: "With .portal() -- menu escapes".into(),
                use_portal: true,
            })
            .with_key("with_portal"),
    ));
    children.add_key("cards");

    children.apply(current_widget.as_parent());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<MenuTriggerCard>()
        .register_widget::<PortalDemo>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<PortalDemo>(PortalDemo)
            // Declared as a normal, non-portaled child of the root -- like `DevtoolsRoot`/
            // `ToastViewport`, it must go through the crate's own reconciliation to ever
            // become a real, tracked `ChildOf(root)` entity (a raw `Commands::spawn` +
            // manual `ChildOf` insert would get silently undone the next time root's own
            // `WidgetChildren` reconciles).
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    );
}
