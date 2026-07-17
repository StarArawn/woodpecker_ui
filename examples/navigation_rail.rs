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
            flex_direction: WidgetFlexDirection::Row,
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<NavigationRail>(NavigationRail {
                items: vec![
                    NavigationRailItem {
                        icon: icons::HOUSE.into(),
                        label: "Home".into(),
                    },
                    NavigationRailItem {
                        icon: icons::MAGNIFYING_GLASS.into(),
                        label: "Search".into(),
                    },
                    NavigationRailItem {
                        icon: icons::STAR.into(),
                        label: "Saved".into(),
                    },
                    NavigationRailItem {
                        icon: icons::GEAR.into(),
                        label: "Settings".into(),
                    },
                ],
                ..Default::default()
            })
            .with_key("rail")
            .with_observe(root_widget, |trigger: On<Change<NavigationRailChanged>>| {
                info!(
                    "NavigationRail selected index={} label={:?}",
                    trigger.data.index, trigger.data.label
                );
            })
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_grow: 1.0,
                    height: Units::Percentage(100.0),
                    padding: Edge::all(20.0),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 14.0,
                        color: Srgba::new(0.7, 0.73, 0.78, 1.0).into(),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Page content goes here.".into(),
                    },
                )),
            ))
            .with_key("content"),
    ));
}
