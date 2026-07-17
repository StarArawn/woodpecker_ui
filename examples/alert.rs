use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn alert_text(content: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            text_wrap: TextWrap::WordOrGlyph,
            ..Default::default()
        },
        WidgetRender::Text {
            content: content.into(),
        },
    )
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Column,
            width: 420.0.into(),
            gap: (0.0.into(), 12.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Alert>((
                Alert {
                    variant: BadgeVariant::Info,
                    ..Default::default()
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<Element>(alert_text("A new version is available.")),
                ),
            ))
            .with_key("info")
            .with_child::<Alert>((
                Alert {
                    variant: BadgeVariant::Success,
                    ..Default::default()
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<Element>(alert_text("Changes saved successfully.")),
                ),
            ))
            .with_key("success")
            .with_child::<Alert>((
                Alert {
                    variant: BadgeVariant::Warning,
                    dismissible: true,
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<Element>(alert_text("Your storage is almost full.")),
                ),
            ))
            .with_key("warning")
            .with_child::<Alert>((
                Alert {
                    variant: BadgeVariant::Danger,
                    dismissible: true,
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<Element>(alert_text("Failed to connect to the server.")),
                ),
            ))
            .with_key("danger")
            .with_observe(root_widget, |trigger: On<Change<AlertDismissed>>| {
                info!("Alert {:?} dismissed", trigger.event_target());
            }),
    ));
}
