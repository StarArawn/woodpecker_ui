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

    let handle: Handle<Image> = asset_server.load("woodpecker.jpg");

    let items = (1..=8)
        .map(|i| ImageListItem {
            handle: handle.clone(),
            label: Some(format!("Photo {i}")),
        })
        .collect();

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<ImageList>((ImageList {
            items,
            columns: 3,
            tile_size: 100.0,
            gap: 8.0,
        },)),
    ));
}
