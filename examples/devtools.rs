//! The runtime devtools inspector. Press F12 to open the panel: browse the live widget tree,
//! hit "Pick" then click any on-screen widget to select it and see a dump of its style/
//! layout/state, or just watch the render metrics tick.
//!
//! Deliberately exercises the corners the panel's design has to get right: a couple of
//! `WoodpeckerWindow`s and a `Dropdown` it must render/pick above (`StackingTier::Devtools`
//! sits above every other built-in tier, including `Toast`'s), a `Checkbox` (which owns
//! `use_state` with no props at all) to inspect the state-dump path, and resizing the OS
//! window while the panel is open to confirm it survives `WoodpeckerApp`'s reconciliation on
//! every resize.

use bevy::prelude::*;
use woodpecker_ui::devtools::{DevtoolsRoot, WoodpeckerDevtoolsPlugin};
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_plugins(WoodpeckerDevtoolsPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    color: Srgba::WHITE.into(),
                    margin: Edge::all(16.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Press F12 to open the devtools inspector.".into(),
                },
            ))
            .with_key("hint")
            .with_child::<OverlayRootWidget>(OverlayRootWidget)
            .with_child::<WindowingContextProvider>(
                WidgetChildren::default()
                    .with_child::<WoodpeckerWindow>((
                        WoodpeckerWindow {
                            title: "Sample window".into(),
                            initial_position: Vec2::new(120.0, 100.0),
                            window_styles: WoodpeckerStyle {
                                min_width: 280.0.into(),
                                ..WoodpeckerWindow::default().window_styles
                            },
                            ..Default::default()
                        },
                        PassedChildren(
                            WidgetChildren::default().with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    padding: Edge::all(12.0),
                                    flex_direction: WidgetFlexDirection::Column,
                                    width: Units::Percentage(100.0),
                                    ..Default::default()
                                },
                                WidgetChildren::default()
                                    .with_child::<Checkbox>(Checkbox)
                                    .with_key("checkbox")
                                    .with_child::<Dropdown>(Dropdown {
                                        current_value: "Red".into(),
                                        list: vec!["Red".into(), "Green".into(), "Blue".into()],
                                    })
                                    .with_key("dropdown"),
                            )),
                        ),
                    ))
                    .with_key("sample-window")
                    .with_child::<WoodpeckerWindow>((
                        WoodpeckerWindow {
                            title: "Second window".into(),
                            initial_position: Vec2::new(440.0, 220.0),
                            window_styles: WoodpeckerStyle {
                                min_width: 240.0.into(),
                                ..WoodpeckerWindow::default().window_styles
                            },
                            ..Default::default()
                        },
                        PassedChildren(WidgetChildren::default().with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                padding: Edge::all(12.0),
                                width: Units::Percentage(100.0),
                                font_size: 14.0,
                                color: Srgba::WHITE.into(),
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: "Drag me around, then pick me!".into(),
                            },
                        ))),
                    ))
                    .with_key("second-window"),
            )
            .with_key("windowing")
            .with_child::<DevtoolsRoot>(DevtoolsRoot)
            .with_key("devtools"),
    );
}
