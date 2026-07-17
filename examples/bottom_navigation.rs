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
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    flex_grow: 1.0,
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
            .with_key("content")
            .with_child::<BottomNavigation>(BottomNavigation {
                items: vec![
                    BottomNavItem {
                        icon: icons::HOUSE.into(),
                        label: "Home".into(),
                    },
                    BottomNavItem {
                        icon: icons::MAGNIFYING_GLASS.into(),
                        label: "Search".into(),
                    },
                    BottomNavItem {
                        icon: icons::STAR.into(),
                        label: "Saved".into(),
                    },
                    BottomNavItem {
                        icon: icons::GEAR.into(),
                        label: "Settings".into(),
                    },
                ],
                ..Default::default()
            })
            .with_key("nav")
            .with_observe(
                root_widget,
                |trigger: On<Change<BottomNavigationChanged>>| {
                    info!(
                        "BottomNavigation selected index={} label={:?}",
                        trigger.data.index, trigger.data.label
                    );
                },
            ),
    ));
}
