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
            flex_direction: WidgetFlexDirection::Column,
            gap: (0.0.into(), 12.0.into()),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.6),
                    font_size: 13.0,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Click to type, or hold Alt and drag to scrub".into(),
                },
            ))
            .with_key("hint")
            .with_child::<NumberInput>((NumberInput {
                value: 5.0,
                min: 0.0,
                max: 20.0,
                step: 1.0,
                ..Default::default()
            },))
            .with_key("input")
            .with_observe(root_widget, |trigger: On<Change<NumberInputChanged>>| {
                info!("Value changed to {}", trigger.data.value);
            }),
    ));
}
