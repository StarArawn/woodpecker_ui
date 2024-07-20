use bevy::prelude::*;
use bevy_mod_picking::{debug::DebugPickingMode, DefaultPickingPlugins};
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let root = commands
        .spawn((WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Clip>((ClipBundle {
                styles: WoodpeckerStyle {
                    width: 150.0.into(),
                    height: 100.0.into(),
                    border_radius: Corner::all(50.0.into()),
                    opacity: 0.15,
                    ..Default::default()
                },
                children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: Units::Percentage(100.0),
                        border_radius: Corner::all(50.0.into()),
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
                                    font_size: 20.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                font: asset_server.load("Poppins/Poppins-Regular.ttf"),
                                alignment: VelloTextAlignment::TopLeft,
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
