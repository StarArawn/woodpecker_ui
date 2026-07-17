//! A `VirtualList` of 10,000 custom rows -- each row is real widgets (a colored status badge
//! plus text), not just a plain-text cell like `Table`. Scroll with the mouse wheel; only the
//! rows actually in view (plus a small overscan) are ever spawned.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

const ITEM_COUNT: usize = 10_000;
const ITEM_HEIGHT: f32 = 40.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn item_content(index: usize) -> WidgetChildren {
    let stripe = if index.is_multiple_of(2) {
        colors::DARK_BACKGROUND
    } else {
        colors::BACKGROUND
    };
    let (badge_label, badge_color) = if index.is_multiple_of(7) {
        ("HOT", Srgba::new(0.85, 0.25, 0.25, 1.0).into())
    } else {
        ("NEW", colors::PRIMARY)
    };

    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            align_items: Some(WidgetAlignItems::Center),
            padding: Edge::all(0.0).left(16.0).right(16.0),
            gap: (12.0.into(), 0.0.into()),
            background_color: stripe,
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Pixels(48.0),
                    height: Units::Pixels(20.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    align_items: Some(WidgetAlignItems::Center),
                    background_color: badge_color,
                    border_radius: Corner::all(4.0),
                    ..Default::default()
                },
                WidgetRender::Quad,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 10.0,
                        color: Srgba::WHITE.into(),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: badge_label.into(),
                    },
                )),
            ))
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    color: Srgba::WHITE.into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("Item #{index}"),
                },
            )),
    ))
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands
        .entity(*root_widget)
        .insert(
            WidgetChildren::default().with_child::<VirtualList>(VirtualList {
                item_count: ITEM_COUNT,
                item_extent: ITEM_HEIGHT,
                item_content: VirtualListItemContent::new(item_content),
            }),
        );
}
