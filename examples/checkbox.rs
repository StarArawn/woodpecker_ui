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
    let material_blue = materials.add(Color::Srgba(Srgba::BLUE));

    commands.insert_resource(MaterialList {
        red: material_red.clone(),
        blue: material_blue,
    });

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(material_red),
        Transform::default().with_scale(Vec3::splat(128.)),
    ));

    let root = commands.spawn_empty().id();
    commands.entity(root).insert(WoodpeckerAppBundle {
        styles: WoodpeckerStyle {
            padding: Edge::all(10.0),
            ..default()
        },
        children: WidgetChildren::default()
            .with_child::<Checkbox>(CheckboxBundle { ..default() })
            .with_observe(CurrentWidget(root), on_change),
        ..default()
    });
    ui_context.set_root_widget(root);
}

fn on_change(
    trigger: Trigger<Change<CheckboxChanged>>,
    material_list: Res<MaterialList>,
    mut query: Query<&mut MeshMaterial2d<ColorMaterial>>,
) {
    for mut material in query.iter_mut() {
        if trigger.event().data.checked {
            info!("Checkmark is now checked!");
            *material = MeshMaterial2d(material_list.blue.clone());
        } else {
            info!("Checkmark is now unchecked!");
            *material = MeshMaterial2d(material_list.red.clone());
        }
    }
}
