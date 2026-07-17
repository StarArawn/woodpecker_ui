use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let mut tiles = WidgetChildren::default();
    for elevation in 0..=4u8 {
        tiles.add::<Paper>((
            Paper { elevation },
            WoodpeckerStyle {
                width: 140.0.into(),
                height: 100.0.into(),
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

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Row,
            gap: (16.0.into(), 0.0.into()),
            ..Default::default()
        },
        tiles,
    ));
}
