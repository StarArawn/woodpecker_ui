use bevy::prelude::*;
use bevy_vello::render::VelloView;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, VelloView));

    let root = commands
        .spawn((WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Clip>((ClipBundle {
                styles: WoodpeckerStyle {
                    width: 150.0.into(),
                    height: 100.0.into(),
                    border_radius: Corner::all(50.0),
                    opacity: 0.15,
                    ..Default::default()
                },
                children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: Units::Percentage(100.0),
                        border_radius: Corner::all(50.0),
                        background_color: Srgba::RED.into(),
                        ..Default::default()
                        },
                        children: WidgetChildren::default().with_child::<Element>((
                            ElementBundle {
                                styles: WoodpeckerStyle {
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
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: "Hello World! I am Woodpecker UI! This text is way too long and thus it clips out of the bottom of our quad.".into(),
                                word_wrap: true,
                            },
                        )),
                        ..Default::default()
                    },
                    WidgetRender::Quad
                )),
                    ..Default::default()
                },
            )),
            ..Default::default()
        },))
        .id();
    ui_context.set_root_widget(root);
}
