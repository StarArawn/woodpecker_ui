use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
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
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let slice_border = 135.0;

    let cases = [
        // Stretched Scaled sliced sprite
        (
            Vec2::new(100.0, 200.0),
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(slice_border),
                center_scale_mode: SliceScaleMode::Stretch,
                ..default()
            }),
        ),
        // Scaled sliced sprite
        (
            Vec2::new(100.0, 200.0),
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(slice_border),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 0.5 },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 0.2 },
                ..default()
            }),
        ),
        // Scaled sliced sprite horizontally
        (
            Vec2::new(300.0, 200.0),
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(slice_border),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 0.2 },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 0.3 },
                ..default()
            }),
        ),
        // Scaled sliced sprite horizontally with max scale
        (
            Vec2::new(300.0, 200.0),
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(slice_border),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 0.1 },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 0.2 },
                max_corner_scale: 0.2,
            }),
        ),
    ];

    let mut children = WidgetChildren::default();
    let mut position = Vec2::ZERO;
    for (size, scale_mode) in cases {
        children.add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    width: size.x.into(),
                    height: size.y.into(),
                    left: position.x.into(),
                    top: position.y.into(),
                    position: WidgetPosition::Fixed,
                    ..Default::default()
                },
                ..Default::default()
            },
            WidgetRender::NinePatch {
                handle: asset_server.load("slice.png"),
                scale_mode,
            },
        ));
        position.x += size.x;
    }

    let root = commands
        .spawn((WoodpeckerAppBundle {
            children,
            ..Default::default()
        },))
        .id();
    ui_context.set_root_widget(root);
}
