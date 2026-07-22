use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// A data-dense card layout using CSS Grid instead of hand-tuned flex-wrap math: 4 equal
/// columns, explicit row heights, and one card that spans 2 columns/2 rows to show
/// `grid_row`/`grid_column` item placement.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

/// `grid_placement` lets a card override where it sits in its parent grid (e.g. spanning
/// multiple tracks) -- merged into the card's own style rather than layered as a second
/// `WoodpeckerStyle` component, since a bundle can't carry the same component type twice.
fn card(
    label: &str,
    color: Color,
    grid_placement: (
        (WidgetGridPlacement, WidgetGridPlacement),
        (WidgetGridPlacement, WidgetGridPlacement),
    ),
) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            background_color: color,
            border_radius: Corner::all(6.0),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            grid_column: grid_placement.0,
            grid_row: grid_placement.1,
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                color: Color::WHITE,
                font_size: 16.0,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: label.into(),
            },
        )),
    )
}

const AUTO_PLACEMENT: (
    (WidgetGridPlacement, WidgetGridPlacement),
    (WidgetGridPlacement, WidgetGridPlacement),
) = (
    (WidgetGridPlacement::Auto, WidgetGridPlacement::Auto),
    (WidgetGridPlacement::Auto, WidgetGridPlacement::Auto),
);

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
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                display: WidgetDisplay::Grid,
                gap: (10.0.into(), 10.0.into()),
                ..Default::default()
            },
            GridTemplate {
                columns: vec![GridTrackSize::Fraction(1.0); 4],
                rows: vec![
                    GridTrackSize::Pixels(120.0),
                    GridTrackSize::Pixels(120.0),
                    GridTrackSize::Pixels(120.0),
                ],
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Element>(card("1", Color::srgb(0.20, 0.45, 0.80), AUTO_PLACEMENT))
                .with_key("1")
                // Spans the next 2 columns and 2 rows -- shows `grid_row`/`grid_column`
                // item placement, not just the container's own track sizing.
                .with_child::<Element>(card(
                    "2 (spans 2x2)",
                    Color::srgb(0.80, 0.35, 0.20),
                    (
                        (WidgetGridPlacement::Auto, WidgetGridPlacement::Span(2)),
                        (WidgetGridPlacement::Auto, WidgetGridPlacement::Span(2)),
                    ),
                ))
                .with_key("2")
                .with_child::<Element>(card("3", Color::srgb(0.30, 0.65, 0.35), AUTO_PLACEMENT))
                .with_key("3")
                .with_child::<Element>(card("4", Color::srgb(0.55, 0.35, 0.75), AUTO_PLACEMENT))
                .with_key("4")
                .with_child::<Element>(card("5", Color::srgb(0.75, 0.60, 0.20), AUTO_PLACEMENT))
                .with_key("5")
                .with_child::<Element>(card("6", Color::srgb(0.35, 0.65, 0.65), AUTO_PLACEMENT))
                .with_key("6"),
        )),
    ));
}
