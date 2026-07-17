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

    // Deliberately uneven heights -- the point of a masonry grid is that a uniform
    // row/column grid (see `examples/image_list.rs`) would waste space here.
    let heights = [140.0, 90.0, 180.0, 110.0, 70.0, 150.0, 100.0, 130.0];
    let items = heights
        .iter()
        .enumerate()
        .map(|(i, &height)| MasonryItem {
            handle: handle.clone(),
            height,
            label: Some(format!("Photo {}", i + 1)),
        })
        .collect();

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Masonry>((Masonry {
            items,
            columns: 3,
            column_width: 120.0,
            gap: 8.0,
        },)),
    ));
}
