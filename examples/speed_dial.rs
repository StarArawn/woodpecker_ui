use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            padding: Edge::all(20.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    position: WidgetPosition::Absolute,
                    left: 300.0.into(),
                    top: 300.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<SpeedDial>((SpeedDial {
                    icon: "+".into(),
                    actions: vec![
                        SpeedDialAction {
                            icon: "\u{1F4C4}".into(),
                        },
                        SpeedDialAction {
                            icon: "\u{1F4C1}".into(),
                        },
                        SpeedDialAction {
                            icon: "\u{2B06}".into(),
                        },
                    ],
                    placement: PopoverPlacement::Top,
                },)),
            ))
            .with_key("dial")
            .with_observe(
                root_widget,
                |trigger: On<Change<SpeedDialActionClicked>>| {
                    info!("SpeedDial action {} clicked", trigger.data.index);
                },
            )
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    ));
}
