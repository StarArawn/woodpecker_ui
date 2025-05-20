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

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetChildren::default().with_child::<Clip>((
                Clip,
                WoodpeckerStyle {
                    width: 150.0.into(),
                    height: 100.0.into(),
                    border_radius: Corner::all(50.0),
                    opacity: 0.15,
                    ..Default::default()
                },
                 WidgetChildren::default().with_child::<Element>((
                Element,
                 WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: Units::Percentage(100.0),
                        border_radius: Corner::all(50.0),
                        background_color: Srgba::RED.into(),
                        ..Default::default()
                        },
                        WidgetChildren::default().with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                    width: Units::Percentage(100.0),
                                    height: Units::Percentage(100.0),
                                    margin: Edge {
                                        left: 10.0.into(),
                                        right: 10.0.into(),
                                        top: 10.0.into(),
                                        bottom: 10.0.into(),
                                    },
                                    font_size: 14.0,
                                    ..Default::default()
                                },
                            WidgetRender::Text {
                                content: "Hello World! I am Woodpecker UI! This text is way too long and thus it clips out of the bottom of our quad.".into(),
                            },
                        )),
                    WidgetRender::Quad
                )),
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}
