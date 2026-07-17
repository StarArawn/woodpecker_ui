use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn paragraph(before: &str, link_label: &str, after: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            align_items: Some(WidgetAlignItems::Baseline),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: before.into(),
                },
            ))
            .with_key("before")
            .with_child::<Link>((Link {
                label: link_label.into(),
            },))
            .with_key("link")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: after.into(),
                },
            ))
            .with_key("after"),
    )
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Column,
            align_items: Some(WidgetAlignItems::Start),
            width: 420.0.into(),
            gap: (0.0.into(), 10.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>(paragraph(
                "Already have an account? ",
                "Sign in",
                " instead.",
            ))
            .with_key("signin")
            .with_child::<Element>(paragraph(
                "Forgot your password? ",
                "Reset it",
                " by email.",
            ))
            .with_key("reset")
            .with_child::<Link>((Link {
                label: "View full changelog".into(),
            },))
            .with_key("changelog")
            .with_observe(root_widget, |trigger: On<Change<LinkClicked>>| {
                info!("Link {:?} clicked", trigger.event_target());
            }),
    ));
}
