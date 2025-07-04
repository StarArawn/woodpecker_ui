use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

#[derive(Resource)]
pub struct MaterialList {
    red: Handle<ColorMaterial>,
    green: Handle<ColorMaterial>,
    blue: Handle<ColorMaterial>,
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let material_red = materials.add(Color::Srgba(Srgba::RED));
    let material_green = materials.add(Color::Srgba(Srgba::GREEN));
    let material_blue = materials.add(Color::Srgba(Srgba::BLUE));

    commands.insert_resource(MaterialList {
        red: material_red.clone(),
        green: material_green,
        blue: material_blue,
    });

    commands.spawn((
        Mesh2d(meshes.add(Circle { radius: 50.0 })),
        MeshMaterial2d(material_red),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let root = commands.spawn_empty().id();
    commands.entity(root).insert((
        WoodpeckerApp,
        WoodpeckerStyle {
            padding: Edge::all(10.0),
            ..default()
        },
        WidgetChildren::default()
            .with_child::<Dropdown>(Dropdown {
                list: vec!["Red".into(), "Green".into(), "Blue".into()],
                current_value: "Red".into(),
                ..Default::default()
            })
            .with_observe(
                CurrentWidget(root),
                |trigger: Trigger<Change<DropdownChanged>>,
                 material_list: Res<MaterialList>,
                 mut query: Query<&mut MeshMaterial2d<ColorMaterial>>| {
                    for mut material in query.iter_mut() {
                        match trigger.data.value.as_str() {
                            "Red" => *material = MeshMaterial2d(material_list.red.clone()),
                            "Green" => *material = MeshMaterial2d(material_list.green.clone()),
                            "Blue" => *material = MeshMaterial2d(material_list.blue.clone()),
                            _ => {}
                        }
                    }
                },
            ),
    ));
    ui_context.set_root_widget(root);
}
