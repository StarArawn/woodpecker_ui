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
            padding: Edge::all(20.0),
            width: 360.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Timeline>((Timeline {
            items: vec![
                TimelineItem {
                    title: "Order placed".into(),
                    subtitle: Some("Jul 5, 9:12 AM".into()),
                    dot_color: None,
                },
                TimelineItem {
                    title: "Payment confirmed".into(),
                    subtitle: Some("Jul 5, 9:13 AM".into()),
                    dot_color: None,
                },
                TimelineItem {
                    title: "Payment failed, retrying".into(),
                    subtitle: Some("Jul 5, 11:02 AM".into()),
                    dot_color: Some(Srgba::new(0.9, 0.3, 0.3, 1.0).into()),
                },
                TimelineItem {
                    title: "Shipped".into(),
                    subtitle: Some("Jul 6, 2:45 PM".into()),
                    dot_color: None,
                },
            ],
        },)),
    ));
}
