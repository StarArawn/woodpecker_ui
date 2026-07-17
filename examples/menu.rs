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
            .with_child::<Menu>((
                Menu {
                    items: vec!["Cut".into(), "Copy".into(), "Paste".into(), "Delete".into()],
                },
                PassedChildren(WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: 320.0.into(),
                        height: 200.0.into(),
                        background_color: Color::srgb(0.16, 0.18, 0.22),
                        justify_content: Some(WidgetAlignContent::Center),
                        align_items: Some(WidgetAlignItems::Center),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            color: Color::WHITE,
                            text_wrap: TextWrap::None,
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: "Right-click me".into(),
                        },
                    )),
                ))),
            ))
            .with_observe(root_widget, |trigger: On<Change<MenuItemSelected>>| {
                info!("Selected: {}", trigger.data.label);
            })
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    ));
}
