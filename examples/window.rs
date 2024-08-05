use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<WoodpeckerWindow>(
                WoodpeckerWindowBundle {
                    window: WoodpeckerWindow {
                        title: "Image/SVG fits to content.".into(),
                        initial_position: Vec2::new(200.0, 200.0),
                        ..Default::default()
                    },
                    children: PassedChildren(
                        WidgetChildren::default().with_child::<Element>(ElementBundle {
                            styles: WoodpeckerStyle {
                                align_items: Some(WidgetAlignItems::Center),
                                flex_direction: WidgetFlexDirection::Column,
                                padding: Edge::all(10.0),
                                width: Units::Percentage(100.0).into(),
                                ..Default::default()
                            },
                            children: WidgetChildren::default()
                                .with_child::<Element>((
                                    ElementBundle {
                                        styles: WoodpeckerStyle {
                                            width: Units::Auto,
                                            height: Units::Pixels(200.0),
                                            margin: Edge::all(0.0).bottom(10.0),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    WidgetRender::Image {
                                        handle: asset_server.load("woodpecker.jpg"),
                                    },
                                ))
                                .with_child::<Element>((
                                    ElementBundle {
                                        styles: WoodpeckerStyle {
                                            width: Units::Auto,
                                            height: Units::Pixels(200.0),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    WidgetRender::Svg {
                                        handle: asset_server.load("woodpecker_svg/woodpecker.svg"),
                                        path_color: Some(Srgba::RED.into()),
                                    },
                                )),
                            ..Default::default()
                        }),
                    ),
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
