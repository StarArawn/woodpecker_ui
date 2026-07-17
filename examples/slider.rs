use bevy::{prelude::*, sprite_render::MeshMaterial2d};
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let material_red = materials.add(Color::Srgba(Srgba::RED.with_alpha(0.5)));

    commands.spawn((
        Mesh2d(meshes.add(Circle { radius: 50.0 })),
        MeshMaterial2d(material_red),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(10.0),
            ..default()
        },
        WidgetChildren::default()
            .with_child::<Slider>(Slider {
                start: 0.0,
                end: 1.0,
                value: 0.5,
            })
            .with_observe(
                root_widget,
                |trigger: On<Change<SliderChanged>>,
                 mut material_assets: ResMut<Assets<ColorMaterial>>,
                 query: Query<&MeshMaterial2d<ColorMaterial>>| {
                    for material in query.iter() {
                        material_assets
                            .get_mut(material)
                            .unwrap()
                            .color
                            .set_alpha(trigger.data.value)
                    }
                },
            ),
    ));
}
