use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// Tracks the left pane's width -- `Splitter` only reports a drag delta, it's up to the
/// caller to decide what that delta resizes. Watched so a resize live-updates the pane.
#[derive(Resource, Reflect, Clone, Copy, PartialEq, Default)]
struct LeftPaneWidth(f32);

/// The left pane's width at the moment the current drag started -- `SplitterChanged::delta`
/// is the *total* distance since drag start (see its own doc comment), not a per-frame
/// increment, so the resize handler below has to add it to this fixed snapshot each time
/// rather than accumulating onto `LeftPaneWidth` directly (which would compound every frame
/// of the drag into a runaway resize).
#[derive(Resource, Default)]
struct LeftPaneWidthAtDragStart(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<SplitterDemo>()
        .register_watched_resource::<LeftPaneWidth>()
        .insert_resource(LeftPaneWidth(240.0))
        .insert_resource(LeftPaneWidthAtDragStart::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands
        .entity(*root_widget)
        .insert(WidgetChildren::default().with_child::<SplitterDemo>(SplitterDemo));
}

/// Watches `LeftPaneWidth` and rebuilds the row whenever it changes -- the standard way a
/// widget here reacts to an external resource, since a widget can't reach into a sibling's
/// components directly from an observer.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_style(), WidgetChildren, WatchedResource<LeftPaneWidth>)]
struct SplitterDemo;

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Row,
        ..Default::default()
    }
}

fn pane(label: &str, width: WoodpeckerStyle, background_color: Color) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            height: Units::Percentage(100.0),
            background_color,
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..width
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
                content: label.into(),
            },
        )),
    )
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<LeftPaneWidth>, &mut WidgetChildren)>,
) {
    let Ok((left_width, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    *children = WidgetChildren::default()
        .with_child::<Element>(pane(
            "Left pane",
            WoodpeckerStyle {
                width: left_width.0 .0.into(),
                ..Default::default()
            },
            Color::srgb(0.16, 0.18, 0.22),
        ))
        .with_key("left")
        .with_child::<Splitter>(Splitter::default())
        .with_key("splitter")
        .with_observe(
            current_widget,
            |_trigger: On<Pointer<DragStart>>,
             left_width: Res<LeftPaneWidth>,
             mut base: ResMut<LeftPaneWidthAtDragStart>| {
                base.0 = left_width.0;
            },
        )
        .with_observe(
            current_widget,
            |trigger: On<Change<SplitterChanged>>,
             mut left_width: ResMut<LeftPaneWidth>,
             base: Res<LeftPaneWidthAtDragStart>| {
                left_width.0 = (base.0 + trigger.data.delta).clamp(100.0, 600.0);
            },
        )
        .with_child::<Element>(pane(
            "Right pane -- drag the divider",
            WoodpeckerStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
            Color::srgb(0.10, 0.11, 0.14),
        ))
        .with_key("right");

    children.apply(current_widget.as_parent());
}
