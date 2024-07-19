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
    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..Default::default()
    });

    let root = commands
        .spawn((
            WoodpeckerAppBundle {
                ..Default::default()
            },
            WidgetRender::Text {
                font: asset_server.load("Poppins/Poppins-Regular.ttf"),
                size: 20.0,
                color: Color::WHITE,
                alignment: VelloTextAlignment::TopLeft,
                content: "Hello World! I am Woodpecker UI!".into(),
            },
        ))
        .id();
    ui_context.set_root_widget(root);
}
