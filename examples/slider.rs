use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_mod_picking::{
    prelude::{Listener, On},
    DefaultPickingPlugins,
};
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let material_red = materials.add(Color::Srgba(Srgba::RED.with_alpha(0.5)));

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
        material: material_red,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    let root = commands
        .spawn((WoodpeckerAppBundle {
            styles: WoodpeckerStyle {
                padding: Edge::all(10.0),
                ..default()
            },
            children: WidgetChildren::default().with_child::<Slider>(SliderBundle {
                slider: Slider {
                    start: 0.0,
                    end: 1.0,
                    value: 0.5,
                },
                on_changed: On::run(
                    |event: Listener<OnChange<SliderChanged>>,
                     mut material_assets: ResMut<Assets<ColorMaterial>>,
                     query: Query<&Handle<ColorMaterial>>| {
                        for material in query.iter() {
                            material_assets
                                .get_mut(material)
                                .unwrap()
                                .color
                                .set_alpha(event.data.value)
                        }
                    },
                ),
                ..default()
            }),
            ..default()
        },))
        .id();
    ui_context.set_root_widget(root);
}
