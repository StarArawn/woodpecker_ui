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
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(WidgetRender::Image {
        handle: asset_server.load("woodpecker.jpg"),
    });
}
