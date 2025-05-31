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

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetRender::Image {
                handle: asset_server.load("woodpecker.jpg"),
            },
        ))
        .id();
    ui_context.set_root_widget(root);
}
