use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn big_flat_tree() -> Vec<TreeNode> {
    (0..5_000)
        .map(|i| TreeNode {
            key: format!("row{i}"),
            label: format!("Row {i}"),
            children: vec![],
        })
        .collect()
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let current_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*current_widget).insert(
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    color: Srgba::WHITE.into(),
                    margin: Edge::all(16.0).bottom(8.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "5,000 rows, virtualized -- scroll to confirm only the visible \
                              window mounts."
                        .into(),
                },
            ))
            .with_key("hint")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Pixels(320.0),
                    height: Units::Pixels(400.0),
                    margin: Edge::all(0.0).left(16.0),
                    border: Edge::all(1.0),
                    border_color: Srgba::gray(0.4).into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<TreeView>(TreeView {
                    nodes: big_flat_tree(),
                    selected_key: Some("row0".into()),
                    virtualized: true,
                    ..Default::default()
                }),
            ))
            .with_key("tree"),
    );
}
