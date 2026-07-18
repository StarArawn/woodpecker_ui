//! Mirrors Bevy's own `bevy_ui` `bsn!` example (`examples/scene/bsn.rs`), but for Woodpecker UI:
//! an entire static tree -- composed from functions returning `impl Scene`, nested `Children
//! [...]`, `on(...)` click handlers, all of it -- authored with `bsn!` and spawned once. Run
//! with `cargo run --example bsn_static_ui --features bevy_bsn`.
//!
//! This works because the tree here never needs to re-render: both buttons just log on click,
//! nothing about their own appearance changes over time (beyond `WButton`'s own built-in hover
//! restyle, which keeps working here exactly as it does anywhere else). That's exactly the
//! condition that makes bsn!'s `Children [...]`/`on(...)` safe to use for a whole tree at once
//! -- they spawn real Bevy `Children`/observers directly, with no notion of Woodpecker's own
//! per-render keyed reconciliation. The runner and layout system both discover widgets by
//! walking the real Bevy hierarchy regardless of how it was built, so a tree spawned this way
//! renders and picks correctly -- but if any part of it needed to change later (a counter, a
//! toggled state), it would need `WidgetChildren`/`add_scene` instead, the same way
//! `bsn_startup.rs` does it.
//!
//! `WButton` *is* used here, describing `ButtonStyles` rather than `WoodpeckerStyle` directly --
//! `WButton::render` unconditionally overwrites `WoodpeckerStyle` from its own `ButtonStyles`
//! component every render (`normal` or `hovered`, depending on live hover state), so a scene
//! describing `WoodpeckerStyle` directly would just get stomped on the very next render.
//! `ButtonStyles` is what actually needs describing, and `WButton`'s own `#[require(...)]`
//! already provides `WidgetRender::Quad` and `Pickable` as defaults, so neither needs restating.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut font_manager: ResMut<FontManager>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let font = asset_server.load("Outfit/static/Outfit-Regular.ttf");
    font_manager.add(&font);

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).apply_scene(ui());
}

fn ui() -> impl Scene {
    bsn! {
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            align_items: WidgetAlignItems::Center,
            justify_content: WidgetAlignContent::Center,
        }
        Children [
            (
                button("Ok", Color::srgb(0.15, 0.15, 0.15), Color::srgb(0.25, 0.25, 0.25))
                on(|_event: On<Pointer<Click>>| info!("Ok pressed!"))
            ),
            (
                button("Cancel", Color::srgb(0.4, 0.15, 0.15), Color::srgb(0.5, 0.2, 0.2))
                on(|_event: On<Pointer<Click>>| info!("Cancel pressed!"))
            ),
        ]
    }
}

fn button(label: &str, normal_color: Color, hovered_color: Color) -> impl Scene {
    let label = label.to_string();
    let base = WoodpeckerStyle {
        width: Units::Pixels(150.0),
        height: Units::Pixels(65.0),
        border: Edge::all(5.0),
        border_radius: Corner::all(32.0),
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        margin: Edge::all(0.0).right(10.0),
        ..Default::default()
    };
    let normal = WoodpeckerStyle {
        background_color: normal_color,
        ..base
    };
    let hovered = WoodpeckerStyle {
        background_color: hovered_color,
        ..base
    };
    bsn! {
        WButton
        ButtonStyles { normal: normal, hovered: hovered }
        Children [(
            Element
            WoodpeckerStyle {
                font_size: 24.0,
                color: Color::srgb(0.9, 0.9, 0.9),
            }
            WidgetRender::Text { content: label }
        )]
    }
}
