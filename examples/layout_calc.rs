use bevy::prelude::*;
use woodpecker_ui::prelude::*;

const SIDEBAR_WIDTH: f32 = 200.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn panel_label(title: &str, subtitle: &str) -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 22.0,
                color: Color::WHITE,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: title.into(),
            },
        ))
        .with_key("title")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 16.0,
                color: Srgba::new(0.8, 0.8, 0.85, 1.0).into(),
                margin: Edge::all(0.0).top(8.0),
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: subtitle.into(),
            },
        ))
        .with_key("subtitle")
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
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: SIDEBAR_WIDTH.into(),
                    height: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Column,
                    padding: Edge::all(20.0),
                    background_color: Srgba::new(0.15, 0.15, 0.18, 1.0).into(),
                    ..Default::default()
                },
                WidgetRender::Quad,
                panel_label("Sidebar", "width: 200px (fixed)"),
            ))
            .with_key("sidebar")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    // Fills whatever's left of the row after the sidebar's fixed 200px --
                    // equivalent to CSS's `calc(100% - 200px)`. Resize the window: this
                    // panel's width always tracks the sidebar's exactly, which a bare
                    // `Percentage` can't do (it doesn't know about the sidebar's fixed
                    // width at all, so it would either overlap it or leave a gap).
                    width: Units::Calc {
                        percent: 100.0,
                        pixels: -SIDEBAR_WIDTH,
                    },
                    height: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Column,
                    padding: Edge::all(20.0),
                    background_color: Srgba::new(0.2, 0.32, 0.5, 1.0).into(),
                    ..Default::default()
                },
                WidgetRender::Quad,
                panel_label("Content", "width: Units::Calc { percent: 100.0, pixels: -200.0 }"),
            ))
            .with_key("content"),
    ));
}
