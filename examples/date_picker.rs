//! A `DatePicker` with a bounded selectable range. Click the trigger (or the calendar icon)
//! to open it, click a day to choose it, or Tab to focus and use Left/Right/Up/Down/Enter.
//!
//! No wall-clock "today" is read here (see `DatePicker`'s doc comment for why) -- the initial
//! view and the selectable range are just hardcoded dates for the demo.

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

    let current_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*current_widget).insert((
        WoodpeckerStyle {
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    color: Srgba::WHITE.into(),
                    margin: Edge::all(16.0).bottom(8.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Selectable range: 2024-01-01 through 2024-12-31.".into(),
                },
            ))
            .with_key("hint")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Pixels(260.0),
                    margin: Edge::all(0.0).left(16.0),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<DatePicker>(DatePicker {
                    selected_date: None,
                    min_date: Some(CalendarDate::new(2024, 1, 1)),
                    max_date: Some(CalendarDate::new(2024, 12, 31)),
                    initial_view: CalendarDate::new(2024, 6, 1),
                    placeholder: "Pick a date".into(),
                }),
            ))
            .with_key("date_picker")
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    ));
}
