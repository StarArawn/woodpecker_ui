use bevy::prelude::*;
use bevy_mod_picking::{debug::DebugPickingMode, DefaultPickingPlugins};
use style_helpers::FromPercent;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..Default::default()
    });

    let root = commands
        .spawn((WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle::new()
                        .with_size(taffy::Size::from_lengths(150.0, 100.0)),
                    children: WidgetChildren::default().with_child::<Clip>((ClipBundle {
                        children: WidgetChildren::default().with_child::<Element>((
                            ElementBundle {
                                styles: WoodpeckerStyle::new()
                                    .with_size(taffy::Size {
                                        width: Dimension::from_percent(1.0),
                                        height: Dimension::from_percent(1.0),
                                    })
                                    .with_margin(taffy::Rect {
                                        left: LengthPercentageAuto::Length(10.0),
                                        right: LengthPercentageAuto::Length(10.0),
                                        top: LengthPercentageAuto::Length(10.0),
                                        bottom: LengthPercentageAuto::Length(10.0),
                                    }),
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                font: asset_server.load("Poppins/Poppins-Regular.ttf"),
                                size: 20.0,
                                color: Color::WHITE,
                                alignment: VelloTextAlignment::TopLeft,
                                content: "Hello World! I am Woodpecker UI! This text is way too long and thus it clips out of the bottom of our quad.".into(),
                                word_wrap: true,
                            },
                        )),
                        ..Default::default()
                    },)),
                    ..Default::default()
                },
                WidgetRender::Quad {
                    color: Srgba::RED.into(),
                    border_radius: kurbo::RoundedRectRadii::from_single_radius(0.0),
                },
            )),
            ..Default::default()
        },))
        .id();
    ui_context.set_root_widget(root);
}
