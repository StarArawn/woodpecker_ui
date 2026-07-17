//! A searchable `ComboBox` -- type to filter the list, click the trigger or the arrow to open
//! it, Up/Down to highlight an item, Enter to choose it, Escape to close.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let languages = vec![
        "Rust",
        "Zig",
        "C",
        "C++",
        "Go",
        "Python",
        "TypeScript",
        "JavaScript",
        "Swift",
        "Kotlin",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let current_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*current_widget).insert((
        WoodpeckerStyle {
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
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
                    content: "Type to filter, or click the arrow to browse the full list.".into(),
                },
            ))
            .with_key("hint")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Pixels(280.0),
                    margin: Edge::all(0.0).left(16.0),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<ComboBox>(ComboBox {
                    current_value: "Rust".into(),
                    list: languages,
                    match_mode: ComboBoxMatchMode::Contains,
                }),
            ))
            .with_key("combo_box")
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    ));
}
