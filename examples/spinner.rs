use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, animate_determinate_value)
        .run();
}

fn tile(title: &str, spinner: Spinner) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: 140.0.into(),
            flex_direction: WidgetFlexDirection::Column,
            align_items: Some(WidgetAlignItems::Center),
            gap: (0.0.into(), 12.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Spinner>((spinner,))
            .with_key("spinner")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: title.into(),
                },
            ))
            .with_key("label"),
    )
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let tiles = WidgetChildren::default()
        .with_child::<Element>(tile(
            "Indeterminate",
            Spinner {
                size: 40.0,
                mode: SpinnerMode::Indeterminate,
            },
        ))
        .with_key("indeterminate")
        .with_child::<Element>(tile(
            "Determinate",
            Spinner {
                size: 40.0,
                mode: SpinnerMode::Determinate(0.0),
            },
        ))
        .with_key("determinate");

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Row,
            gap: (32.0.into(), 0.0.into()),
            ..Default::default()
        },
        tiles,
    ));
}

/// Loops the determinate tile's `value` 0..1..0 so the demo shows the arc actually filling,
/// rather than sitting at a single static fraction.
fn animate_determinate_value(time: Res<Time>, mut query: Query<&mut Spinner>) {
    let phase = (time.elapsed_secs() * 0.4).fract();
    let value = if phase < 0.5 {
        phase * 2.0
    } else {
        (1.0 - phase) * 2.0
    };
    for mut spinner in query.iter_mut() {
        if matches!(spinner.mode, SpinnerMode::Determinate(_)) {
            spinner.mode = SpinnerMode::Determinate(value);
        }
    }
}
