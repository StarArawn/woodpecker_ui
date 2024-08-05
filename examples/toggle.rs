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

#[derive(Resource)]
pub struct MaterialList {
    red: Handle<ColorMaterial>,
    blue: Handle<ColorMaterial>,
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let material_red = materials.add(Color::Srgba(Srgba::RED));
    let material_blue = materials.add(Color::Srgba(Srgba::BLUE));

    commands.insert_resource(MaterialList {
        red: material_red.clone(),
        blue: material_blue,
    });

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
            children: WidgetChildren::default().with_child::<Toggle>(ToggleBundle {
                on_changed: On::run(
                    |event: Listener<OnChange<ToggleChanged>>,
                     material_list: Res<MaterialList>,
                     mut query: Query<&mut Handle<ColorMaterial>>| {
                        for mut material in query.iter_mut() {
                            if event.data.checked {
                                info!("Toggle is now checked!");
                                *material = material_list.blue.clone();
                            } else {
                                info!("Toggle is now unchecked!");
                                *material = material_list.red.clone();
                            }
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
