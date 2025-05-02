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

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, VelloView));

    let root = commands
        .spawn((WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<IconButton>(IconButtonBundle {
                render: WidgetRender::Svg {
                    handle: asset_server.load("woodpecker_svg/woodpecker.svg"),
                    color: None, // Set by IconButton
                },
                ..Default::default()
            }),
            ..Default::default()
        },))
        .id();
    ui_context.set_root_widget(root);
}
