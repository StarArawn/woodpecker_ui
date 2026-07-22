use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// A 3x3 grid of focusable buttons. Works with mouse clicks always; also responds to a
/// gamepad D-pad (navigate) + south button (confirm) when this example is built with the
/// `gamepad-nav` feature enabled (`cargo run --example gamepad_nav --features gamepad-nav`)
/// and a controller is connected.
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

    let mut grid = WidgetChildren::default();
    for row in 0..3 {
        let mut cell = WidgetChildren::default();
        for col in 0..3 {
            let label = format!("{row},{col}");
            cell.add::<WButton>((WButton::text(label.clone()), Focusable));
            cell.add_key(format!("cell-{row}-{col}"));
            cell.observe(root_widget, move |_: On<WidgetFocus>| {
                info!("Focused {label}");
            });
            let label = format!("{row},{col}");
            cell.on_click_or_activate(root_widget, move |_commands| {
                info!("Activated {label}");
            });
        }
        grid.add::<Element>((
            Element,
            WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                gap: (12.0.into(), 0.0.into()),
                ..Default::default()
            },
            cell,
        ));
        grid.add_key(format!("row-{row}"));
    }

    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Column,
            gap: (0.0.into(), 12.0.into()),
            ..Default::default()
        },
        grid,
    ));
}
