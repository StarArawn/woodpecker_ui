//! Demonstrates loading your own icon font instead of the bundled Phosphor font.
//!
//! `WoodpeckerUIPlugin::with_icon_font` takes any asset path your `AssetServer` can resolve
//! (here, `assets/Phosphor.ttf` -- the same font the crate embeds by default, but loaded through
//! the normal asset pipeline to show the mechanism). To use a *different* font, point the path
//! at your own `.ttf`/`.otf` in `assets/` and define your own module of glyph constants (mapping
//! names to that font's codepoints) in place of `woodpecker_ui::icons`.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default().with_icon_font("Phosphor.ttf"))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    icon_font: Res<IconFont>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(10.0),
            ..default()
        },
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 32.0,
                font: Some(icon_font.0.id()),
                ..Default::default()
            },
            WidgetRender::Text {
                content: icons::HEART.into(),
            },
        )),
    ));
}
