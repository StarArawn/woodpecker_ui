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

    let card_styles = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        width: 150.0.into(),
        height: 150.0.into(),
        background_color: Srgba::new(0.15, 0.15, 0.18, 1.0).into(),
        border_radius: Corner::all(12.0),
        ..Default::default()
    };

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle::default(),
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WidgetRender::Quad,
                WoodpeckerStyle {
                    left: 50.0.into(),
                    top: 50.0.into(),
                    box_shadow: Some(WidgetBoxShadow::default()),
                    ..card_styles
                },
            ))
            .with_key("soft_shadow")
            .with_child::<Element>((
                Element,
                WidgetRender::Quad,
                WoodpeckerStyle {
                    left: 300.0.into(),
                    top: 50.0.into(),
                    box_shadow: Some(
                        WidgetBoxShadow::new(Srgba::new(0.3, 0.1, 0.8, 0.5).into())
                            .offset(0.0, 12.0)
                            .blur_radius(30.0),
                    ),
                    ..card_styles
                },
            ))
            .with_key("colored_shadow")
            .with_child::<Element>((
                Element,
                WidgetRender::Quad,
                WoodpeckerStyle {
                    left: 550.0.into(),
                    top: 50.0.into(),
                    box_shadow: Some(
                        WidgetBoxShadow::new(Srgba::new(0.1, 0.6, 1.0, 0.8).into())
                            .offset(0.0, 0.0)
                            .spread(4.0)
                            .blur_radius(4.0),
                    ),
                    ..card_styles
                },
            ))
            .with_key("ring_shadow"),
    ));
}
