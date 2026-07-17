use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn stat_row(label: &str, value: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            justify_content: Some(WidgetAlignContent::SpaceBetween),
            margin: Edge::all(0.0).bottom(8.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::new(0.6, 0.63, 0.7, 1.0).into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.into(),
                },
            ))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::WHITE.into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: value.into(),
                },
            ))
            .with_key("value"),
    )
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Row,
            gap: (20.0.into(), 0.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            // A plain card: title only, body content, no actions.
            .with_child::<Card>((
                Card {
                    title: Some("Server Status".into()),
                    ..Default::default()
                },
                WoodpeckerStyle {
                    width: 260.0.into(),
                    ..Default::default()
                },
                PassedChildren(
                    WidgetChildren::default()
                        .with_child::<Element>(stat_row("Uptime", "14d 6h"))
                        .with_key("uptime")
                        .with_child::<Element>(stat_row("Requests/sec", "1,204"))
                        .with_key("rps")
                        .with_child::<Element>(stat_row("Error rate", "0.02%"))
                        .with_key("errors"),
                ),
            ))
            .with_key("status")
            // A card with a subtitle and an action row.
            .with_child::<Card>((
                Card {
                    title: Some("Delete account".into()),
                    subtitle: Some("This can't be undone.".into()),
                    elevation: 2,
                },
                WoodpeckerStyle {
                    width: 260.0.into(),
                    ..Default::default()
                },
                PassedChildren(WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 13.0,
                        color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
                        text_wrap: TextWrap::WordOrGlyph,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "All of your data will be permanently removed.".into(),
                    },
                ))),
                CardActions(
                    WidgetChildren::default()
                        .with_child::<WButton>(WButton::text("Cancel"))
                        .with_key("cancel")
                        .with_child::<WButton>(WButton::text("Delete"))
                        .with_key("delete"),
                ),
            ))
            .with_key("danger"),
    ));
}
