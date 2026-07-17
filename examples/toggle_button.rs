use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<ToggleButtonDemo>()
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ToggleButtonDemoState {
    bold: bool,
    italic: bool,
    view: usize,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct ToggleButtonDemo;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<ToggleButtonDemo>>,
    state_query: Query<&ToggleButtonDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        ToggleButtonDemoState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let state = *state;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            gap: (8.0.into(), 0.0.into()),
            margin: Edge::all(0.0).bottom(20.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<ToggleButton>((
                ToggleButton {
                    selected: state.bold,
                },
                PassedChildren(WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle::default(),
                    WidgetRender::Text {
                        content: "Bold".into(),
                    },
                ))),
            ))
            .with_observe(
                current_widget,
                move |trigger: On<Change<ToggleButtonChanged>>,
                      mut query: Query<&mut ToggleButtonDemoState>| {
                    if let Ok(mut state) = query.get_mut(state_entity) {
                        state.bold = trigger.data.selected;
                    }
                },
            )
            .with_key("bold")
            .with_child::<ToggleButton>((
                ToggleButton {
                    selected: state.italic,
                },
                PassedChildren(WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle::default(),
                    WidgetRender::Text {
                        content: "Italic".into(),
                    },
                ))),
            ))
            .with_observe(
                current_widget,
                move |trigger: On<Change<ToggleButtonChanged>>,
                      mut query: Query<&mut ToggleButtonDemoState>| {
                    if let Ok(mut state) = query.get_mut(state_entity) {
                        state.italic = trigger.data.selected;
                    }
                },
            )
            .with_key("italic"),
    ));
    children.add_key("toggles");

    children
        .add::<ButtonGroup>((ButtonGroup {
            options: vec!["Day".into(), "Week".into(), "Month".into()],
            selected: state.view,
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<ButtonGroupChanged>>,
                  mut query: Query<&mut ToggleButtonDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.view = trigger.data.index;
                }
            },
        );
    children.add_key("view-group");

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<ToggleButtonDemo>(ToggleButtonDemo)
            .with_key("demo"),
    ));
}
