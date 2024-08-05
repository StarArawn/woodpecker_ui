use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut font_manager: ResMut<FontManager>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("Outfit/static/Outfit-Regular.ttf");
    font_manager.add(&font);

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 50.0,
                        color: Srgba::RED.into(),
                        margin: Edge::all(10.0),
                        font: Some(font.id()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Hello World! I am Woodpecker UI!".into(),
                    word_wrap: false,
                },
            )),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
