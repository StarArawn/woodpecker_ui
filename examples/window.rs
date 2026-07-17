use bevy::prelude::*;
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
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<OverlayRootWidget>(OverlayRootWidget)
            .with_child::<WindowingContextProvider>(
                WidgetChildren::default()
                    .with_child::<WoodpeckerWindow>((
                        WoodpeckerWindow {
                            title: "Image/SVG fits to content.".into(),
                            initial_position: Vec2::new(200.0, 200.0),
                            window_styles: WoodpeckerStyle {
                                min_width: 400.0.into(),
                                opacity: 0.75,
                                ..WoodpeckerWindow::default().window_styles
                            },
                            ..Default::default()
                        },
                        PassedChildren(WidgetChildren::default().with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                align_items: Some(WidgetAlignItems::Center),
                                flex_direction: WidgetFlexDirection::Column,
                                padding: Edge::all(10.0),
                                width: Units::Percentage(100.0),
                                ..Default::default()
                            },
                            WidgetChildren::default().with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    width: Units::Auto,
                                    height: Units::Pixels(200.0),
                                    margin: Edge::all(0.0).bottom(10.0),
                                    ..Default::default()
                                },
                                WidgetRender::Image {
                                    handle: asset_server.load("woodpecker.jpg"),
                                },
                            )),
                        ))),
                    ))
                    .with_key("image-window")
                    .with_child::<WoodpeckerWindow>((
                        WoodpeckerWindow {
                            title: "2nd window".into(),
                            initial_position: Vec2::new(200.0, 200.0),
                            window_styles: WoodpeckerStyle {
                                min_width: 400.0.into(),
                                opacity: 0.75,
                                ..WoodpeckerWindow::default().window_styles
                            },
                            ..Default::default()
                        },
                        PassedChildren(WidgetChildren::default().with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                align_items: Some(WidgetAlignItems::Center),
                                flex_direction: WidgetFlexDirection::Column,
                                padding: Edge::all(10.0),
                                width: Units::Percentage(100.0),
                                ..Default::default()
                            },
                            WidgetChildren::default().with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    width: Units::Auto,
                                    height: Units::Pixels(200.0),
                                    ..Default::default()
                                },
                                WidgetRender::Svg {
                                    handle: asset_server.load("woodpecker_svg/woodpecker.svg"),
                                    color: Some(Srgba::RED.into()),
                                },
                            )),
                        ))),
                    ))
                    .with_key("svg-window"),
            ),
    );
}
