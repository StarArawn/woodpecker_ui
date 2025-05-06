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

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 50.0,
                    color: Srgba::RED.into(),
                    margin: Edge::all(10.0),
                    font: Some(font.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Hello World! I am Woodpecker UI!".into(),
                    word_wrap: false,
                },
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}
