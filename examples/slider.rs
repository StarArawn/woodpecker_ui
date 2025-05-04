use bevy::{prelude::*, sprite::MeshMaterial2d};
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, VelloView));

    let material_red = materials.add(Color::Srgba(Srgba::RED.with_alpha(0.5)));

    commands.spawn((
        Mesh2d(meshes.add(Circle { radius: 50.0 })),
        MeshMaterial2d(material_red),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let root = commands.spawn_empty().id();
    commands.entity(root).insert(WoodpeckerAppBundle {
        styles: WoodpeckerStyle {
            padding: Edge::all(10.0),
            ..default()
        },
        children: WidgetChildren::default()
            .with_child::<Slider>(SliderBundle {
                slider: Slider {
                    start: 0.0,
                    end: 1.0,
                    value: 0.5,
                },
                // on_changed: On::run(
                //     |event: Listener<Change<SliderChanged>>,
                //      mut material_assets: ResMut<Assets<ColorMaterial>>,
                //      query: Query<&MeshMaterial2d>| {
                //         for material in query.iter() {
                //             material_assets
                //                 .get_mut(material)
                //                 .unwrap()
                //                 .color
                //                 .set_alpha(event.data.value)
                //         }
                //     },
                // ),
                ..default()
            })
            .with_observe(
                CurrentWidget(root),
                |trigger: Trigger<Change<SliderChanged>>,
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
        ..default()
    });
    ui_context.set_root_widget(root);
}
