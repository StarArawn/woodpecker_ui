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
            flex_direction: WidgetFlexDirection::Column,
            align_items: Some(WidgetAlignItems::Start),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_direction: WidgetFlexDirection::Row,
                    align_items: Some(WidgetAlignItems::Center),
                    gap: (12.0.into(), 0.0.into()),
                    margin: Edge::all(0.0).bottom(20.0),
                    ..Default::default()
                },
                WidgetChildren::default()
                    .with_child::<Skeleton>((Skeleton {
                        variant: SkeletonVariant::Circle,
                        width: 40.0,
                        height: 40.0,
                    },))
                    .with_key("avatar")
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            flex_direction: WidgetFlexDirection::Column,
                            gap: (0.0.into(), 6.0.into()),
                            ..Default::default()
                        },
                        WidgetChildren::default()
                            .with_child::<Skeleton>((Skeleton {
                                variant: SkeletonVariant::Text,
                                width: 140.0,
                                height: 14.0,
                            },))
                            .with_key("name")
                            .with_child::<Skeleton>((Skeleton {
                                variant: SkeletonVariant::Text,
                                width: 90.0,
                                height: 12.0,
                            },))
                            .with_key("subtitle"),
                    ))
                    .with_key("lines"),
            ))
            .with_key("profile-row")
            .with_child::<Skeleton>((Skeleton {
                variant: SkeletonVariant::Rect,
                width: 320.0,
                height: 120.0,
            },))
            .with_key("card"),
    ));
}
