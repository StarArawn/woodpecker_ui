use bevy::prelude::*;
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

    let color = Color::Srgba(Srgba::RED);
    let material_red = materials.add(color);

    commands.spawn((
        Mesh2d(meshes.add(Circle { radius: 50.0 })),
        MeshMaterial2d(material_red),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let root = commands
        .spawn(WoodpeckerAppBundle {
            styles: WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                padding: Edge::all(0.0).left(50.0),
                ..Default::default()
            },
            children: WidgetChildren::default()
                .with_child::<ColorPicker>(ColorPickerBundle {
                    color_picker: ColorPicker {
                        initial_color: color,
                    },
                    ..Default::default()
                })
                .with_observe(
                    |trigger: Trigger<Change<ColorPickerChanged>>,
                     mut material_assets: ResMut<Assets<ColorMaterial>>,
                     query: Query<&MeshMaterial2d<ColorMaterial>>| {
                        for material in query.iter() {
                            material_assets.get_mut(material).unwrap().color = trigger.data.color;
                        }
                    },
                ),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
