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

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).apply_scene(ui());
}

fn ui() -> impl Scene {
    bsn! {
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            align_items: WidgetAlignItems::Center,
            justify_content: WidgetAlignContent::Center,
        }
        Children [
            (
                button("Ok", Color::srgb(0.15, 0.15, 0.15), Color::srgb(0.25, 0.25, 0.25))
                on(|_event: On<Pointer<Click>>| info!("Ok pressed!"))
            ),
            (
                button("Cancel", Color::srgb(0.4, 0.15, 0.15), Color::srgb(0.5, 0.2, 0.2))
                on(|_event: On<Pointer<Click>>| info!("Cancel pressed!"))
            ),
        ]
    }
}

fn button(label: &str, normal_color: Color, hovered_color: Color) -> impl Scene {
    let label = label.to_string();
    let base = WoodpeckerStyle {
        width: Units::Pixels(150.0),
        height: Units::Pixels(65.0),
        border: Edge::all(5.0),
        border_radius: Corner::all(32.0),
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        margin: Edge::all(0.0).right(10.0),
        ..Default::default()
    };
    let normal = WoodpeckerStyle {
        background_color: normal_color,
        ..base
    };
    let hovered = WoodpeckerStyle {
        background_color: hovered_color,
        ..base
    };
    bsn! {
        WButton
        ButtonStyles { normal: normal, hovered: hovered }
        Children [(
            Element
            WoodpeckerStyle {
                font_size: 24.0,
                color: Color::srgb(0.9, 0.9, 0.9),
            }
            WidgetRender::Text { content: label }
        )]
    }
}
