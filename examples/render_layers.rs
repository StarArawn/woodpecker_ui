use bevy::{prelude::*, render::view::RenderLayers};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_vello::render::VelloRenderSettings;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .insert_resource(VelloRenderSettings {
            canvas_render_layers: Some(RenderLayers::layer(1)),
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
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
                    *local = 1;
                    RenderLayers::layer(0)
                }
                1 => {
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
    commands.spawn((Camera2dBundle::default(), RenderLayers::layer(1)));

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
