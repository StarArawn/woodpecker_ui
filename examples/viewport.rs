use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, rotator_system)
        .run();
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.spawn((Camera2d, WoodpeckerView));

    let first_pass_layer = RenderLayers::layer(1);

    let cube_handle = meshes.add(Cuboid::new(4.0, 4.0, 4.0));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // The cube that will be rendered to the texture.
    commands.spawn((
        Mesh3d(cube_handle),
        MeshMaterial3d(cube_material_handle),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        FirstPassCube,
        first_pass_layer.clone(),
    ));

    // Light
    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        first_pass_layer.clone(),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            target: image_handle.clone().into(),
            clear_color: Color::BLACK.into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
        first_pass_layer,
    ));

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetChildren::default().with_child::<WoodpeckerWindow>((
                WoodpeckerWindow {
                    title: "Render Target Viewport".into(),
                    initial_position: Vec2::new(10.0, 10.0),
                    ..Default::default()
                },
                PassedChildren(WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        align_items: Some(WidgetAlignItems::Center),
                        flex_direction: WidgetFlexDirection::Column,
                        padding: Edge::all(10.0),
                        width: Units::Percentage(100.0).into(),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            width: Units::Pixels(512.0),
                            height: Units::Pixels(512.0),
                            margin: Edge::all(0.0).bottom(10.0),
                            ..Default::default()
                        },
                        WidgetRender::RenderTarget {
                            handle: image_handle,
                        },
                    )),
                ))),
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}

/// Rotates the inner cube (first pass)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.5 * time.delta_secs());
        transform.rotate_z(1.3 * time.delta_secs());
    }
}
