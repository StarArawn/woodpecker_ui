//! A `TreeView` of a small fake project's file structure.
//!
//! Click a row to select it, click the triangle (or Right/Left with the row active) to
//! expand/collapse, or Tab to focus the tree and use Up/Down/Left/Right/Enter to navigate it
//! entirely from the keyboard -- only the tree itself is a single Tab stop, not each row.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// `TreeNode::key` must be unique across the *whole* tree (see its own doc comment) -- `file`/
/// `folder` take a full slash-separated path as the key (so e.g. `src/widgets/tree_view.rs`
/// and `examples/tree_view.rs` don't collide just because they share a bare filename), and
/// derive the displayed label from its last path segment.
fn label_from_path(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn file(path: &str) -> TreeNode {
    TreeNode {
        key: path.into(),
        label: label_from_path(path).into(),
        children: vec![],
    }
}

fn folder(path: &str, children: Vec<TreeNode>) -> TreeNode {
    TreeNode {
        key: path.into(),
        label: label_from_path(path).into(),
        children,
    }
}

fn project_tree() -> Vec<TreeNode> {
    vec![
        folder(
            "src",
            vec![
                folder(
                    "src/widgets",
                    vec![
                        file("src/widgets/button.rs"),
                        file("src/widgets/tree_view.rs"),
                        file("src/widgets/table.rs"),
                    ],
                ),
                file("src/lib.rs"),
                file("src/focus.rs"),
            ],
        ),
        folder(
            "examples",
            vec![file("examples/tree_view.rs"), file("examples/counter.rs")],
        ),
        file("Cargo.toml"),
        file("README.md"),
    ]
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
                    content: "Tab to focus, then Up/Down/Left/Right/Enter to navigate.".into(),
                },
            ))
            .with_key("hint")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Pixels(320.0),
                    margin: Edge::all(0.0).left(16.0),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<TreeView>(TreeView {
                    nodes: project_tree(),
                    selected_key: Some("src/lib.rs".into()),
                    indent: 18.0,
                    ..Default::default()
                }),
            ))
            .with_key("tree"),
    );
}
