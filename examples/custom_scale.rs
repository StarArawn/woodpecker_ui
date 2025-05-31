use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::{render_resource::*, view::RenderLayers},
    window::WindowResized,
};
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin {
            render_settings: RenderSettings {
                layer: RenderLayers::layer(2),
                ..Default::default()
            },
        })
        .add_systems(Startup, startup)
        .add_systems(Update, resize_window)
        .run();
}

fn resize_window(
    mut window_resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Transform, With<Mesh2d>>,
) {
    for event in window_resize_events.read() {
        for mut transform in query.iter_mut() {
            let (_offset, size, _scale) = compute_letterboxed_transform(
                Vec2::new(event.width, event.height),
                Vec2::new(640.0, 360.0),
            );
            transform.scale = size.extend(1.0);
            transform.translation = Vec2::new(0.0, 0.0).extend(0.0);
        }
    }
}

/// Computes how to scale and position a virtual resolution (e.g. 320x180)
/// into a real screen (e.g. 1920x1080) with proper letterboxing or pillarboxing.
///
/// Returns:
/// - `offset`: top-left corner of the scaled virtual area in screen space
/// - `size`: size of the scaled virtual area
/// - `scale`: uniform scale factor
pub fn compute_letterboxed_transform(
    screen_resolution: Vec2,
    target_resolution: Vec2,
) -> (Vec2, Vec2, f32) {
    // Compute uniform scale factor to fit whole target into screen
    let scale_x = screen_resolution.x / target_resolution.x;
    let scale_y = screen_resolution.y / target_resolution.y;
    let scale = scale_x.min(scale_y);

    // Scaled size of the virtual content
    let scaled_size = target_resolution * scale;

    // Centered offset (top-left corner)
    let offset = (screen_resolution - scaled_size) / 2.0;

    (offset, scaled_size, scale)
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: 640.0 as u32,
            height: 360.0 as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image.sampler = ImageSampler::nearest();

    let image_handle = images.add(image);

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            target: image_handle.clone().into(),
            ..Default::default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                width: 640.0,
                height: 360.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(2),
        WoodpeckerView,
    ));

    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(ColorMaterial {
            texture: Some(image_handle),
            ..default()
        })),
        Transform::from_xyz(320.0, -140.0, 0.0)
            .with_scale(Vec2::new(640.0 * 2.0, 360.0 * 2.0).extend(1.0)),
    ));

    let lorem_ipsum = r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras sed tellus neque. Proin tempus ligula a mi molestie aliquam. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam venenatis consequat ultricies. Sed ac orci purus. Nullam velit nisl, dapibus vel mauris id, dignissim elementum sapien. Vestibulum faucibus sapien ut erat bibendum, id lobortis nisi luctus. Mauris feugiat at lectus at pretium. Pellentesque vitae finibus ante. Nulla non ex neque. Cras varius, lorem facilisis consequat blandit, lorem mauris mollis massa, eget consectetur magna sem vel enim. Nam aliquam risus pulvinar, volutpat leo eget, eleifend urna. Suspendisse in magna sed ligula vehicula volutpat non vitae augue. Phasellus aliquam viverra consequat. Nam rhoncus molestie purus, sed laoreet neque imperdiet eget. Sed egestas metus eget sodales congue.

 Sed vel ante placerat, posuere lacus sit amet, tempus enim. Cras ullamcorper ex vitae metus consequat, a blandit leo semper. Nunc lacinia porta massa, a tempus leo laoreet nec. Sed vel metus tincidunt, scelerisque ex sit amet, lacinia dui. In sollicitudin pulvinar odio vitae hendrerit. Maecenas mollis tempor egestas. Nulla facilisi. Praesent nisi turpis, accumsan eu lobortis vestibulum, ultrices id nibh. Suspendisse sed dui porta, mollis elit sed, ornare sem. Cras molestie est libero, quis faucibus leo semper at.

 Nulla vel nisl rutrum, fringilla elit non, mollis odio. Donec convallis arcu neque, eget venenatis sem mattis nec. Nulla facilisi. Phasellus risus elit, vehicula sit amet risus et, sodales ultrices est. Quisque vulputate felis orci, non tristique leo faucibus in. Duis quis velit urna. Sed rhoncus dolor vel commodo aliquet. In sed tempor quam. Nunc non tempus ipsum. Praesent mi lacus, vehicula eu dolor eu, condimentum venenatis diam. In tristique ligula a ligula dictum, eu dictum lacus consectetur. Proin elementum egestas pharetra. Nunc suscipit dui ac nisl maximus, id congue velit volutpat. Etiam condimentum, mauris ac sodales tristique, est augue accumsan elit, ut luctus est mi ut urna. Mauris commodo, tortor eget gravida lacinia, leo est imperdiet arcu, a ullamcorper dui sapien eget erat.

 Vivamus pulvinar dui et elit volutpat hendrerit. Praesent luctus dolor ut rutrum finibus. Fusce ut odio ultrices, laoreet est at, condimentum turpis. Morbi at ultricies nibh. Mauris tempus imperdiet porta. Proin sit amet tincidunt eros. Quisque rutrum lacus ac est vehicula dictum. Pellentesque nec augue mi.

 Vestibulum rutrum imperdiet nisl, et consequat massa porttitor vel. Ut velit justo, vehicula a nulla eu, auctor eleifend metus. Ut egestas malesuada metus, sit amet pretium nunc commodo ac. Pellentesque gravida, nisl in faucibus volutpat, libero turpis mattis orci, vitae tincidunt ligula ligula ut tortor. Maecenas vehicula lobortis odio in molestie. Curabitur dictum elit sed arcu dictum, ut semper nunc cursus. Donec semper felis non nisl tincidunt elementum.
    "#.to_string();

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetChildren::default().with_child::<Modal>((
                Modal {
                    visible: true,
                    title: "Scrolling example".into(),
                    ..Default::default()
                },
                PassedChildren(
                    WidgetChildren::default().with_child::<ScrollContextProvider>((
                        ScrollContextProvider::default(),
                        WoodpeckerStyle {
                            margin: Edge::all(0.0).left(10.0).right(10.0).bottom(10.0),
                            width: Units::Percentage(100.0),
                            height: Units::Percentage(100.0),
                            ..Default::default()
                        },
                        WidgetChildren::default().with_child::<ScrollBox>((
                            ScrollBox::default(),
                            PassedChildren(WidgetChildren::default().with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    font_size: 14.0,
                                    color: Srgba::WHITE.into(),
                                    ..Default::default()
                                },
                                WidgetRender::Text {
                                    content: lorem_ipsum,
                                },
                            ))),
                        )),
                    )),
                ),
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}
