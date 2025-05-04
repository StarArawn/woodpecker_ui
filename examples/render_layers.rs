use bevy::{prelude::*, render::view::RenderLayers};
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin {
            render_settings: RenderSettings {
                layer: RenderLayers::layer(1),
                ..Default::default()
            },
        })
        .add_systems(Startup, startup)
        .add_systems(Update, flip_render_layers)
        .run();
}

fn flip_render_layers(
    mut query: Query<&mut RenderLayers, With<Camera2d>>,
    input: Res<ButtonInput<KeyCode>>,
    mut local: Local<u8>,
) {
    if input.just_pressed(KeyCode::Space) {
        for mut layer in &mut query {
            *layer = match *local {
                0 => {
                    info!("camera showing layer 0");
                    *local = 1;
                    RenderLayers::layer(0)
                }
                1 => {
                    info!("camera showing layer 1");
                    *local = 0;
                    RenderLayers::layer(1)
                }
                _ => unreachable!(),
            };
        }
    }
}
fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut font_manager: ResMut<FontManager>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView, RenderLayers::layer(1)));

    let font = asset_server.load("Outfit/static/Outfit-Regular.ttf");
    font_manager.add(&font);

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 50.0,
                        color: Srgba::RED.into(),
                        margin: Edge::all(10.0),
                        font: Some(font.id()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Space to change Camera RenderLayer. WoodPecker is on layer 1".into(),
                    word_wrap: true,
                },
            )),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
