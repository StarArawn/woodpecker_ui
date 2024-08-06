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

    let color = Color::Srgba(Srgba::RED);
    let material_red = materials.add(color);

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
        material: material_red,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    let root = commands
        .spawn(WoodpeckerAppBundle {
            styles: WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                padding: Edge::all(0.0).left(50.0),
                ..Default::default()
            },
            children: WidgetChildren::default().with_child::<ColorPicker>((
                ColorPickerBundle {
                    color_picker: ColorPicker {
                        initial_color: color,
                    },
                    ..Default::default()
                },
                On::<Change<ColorPickerChanged>>::run(
                    |event: Listener<Change<ColorPickerChanged>>,
                     mut material_assets: ResMut<Assets<ColorMaterial>>,
                     query: Query<&Handle<ColorMaterial>>| {
                        for material in query.iter() {
                            material_assets.get_mut(material).unwrap().color = event.data.color;
                        }
                    },
                ),
            )),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
