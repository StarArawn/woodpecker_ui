use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<StepperDemo>()
        .add_systems(Startup, startup)
        .run();
}

const STEPS: [&str; 4] = ["Account", "Payment", "Review", "Done"];

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct StepperDemoState {
    active: usize,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct StepperDemo;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<StepperDemo>>,
    state_query: Query<&StepperDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, StepperDemoState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let active = state.active;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<Stepper>((Stepper {
            steps: STEPS.iter().map(|s| s.to_string()).collect(),
            active,
            orientation: StepperOrientation::Horizontal,
            clickable: true,
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<StepperChanged>>, mut query: Query<&mut StepperDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.active = trigger.data.step;
                }
            },
        );
    children.add_key("stepper");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Row,
            justify_content: Some(WidgetAlignContent::SpaceBetween),
            margin: Edge::all(0.0).top(24.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<WButton>(WButton::text("Back"))
            .with_observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>, mut query: Query<&mut StepperDemoState>| {
                    if let Ok(mut state) = query.get_mut(state_entity) {
                        state.active = state.active.saturating_sub(1);
                    }
                },
            )
            .with_key("back")
            .with_child::<WButton>(WButton::text("Next"))
            .with_observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>, mut query: Query<&mut StepperDemoState>| {
                    if let Ok(mut state) = query.get_mut(state_entity) {
                        state.active = (state.active + 1).min(STEPS.len() - 1);
                    }
                },
            )
            .with_key("next"),
    ));
    children.add_key("actions");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            margin: Edge::all(0.0).top(32.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: "Vertical:".into(),
        },
    ));
    children.add_key("vertical-label");

    children.add::<Stepper>((Stepper {
        steps: STEPS.iter().map(|s| s.to_string()).collect(),
        active,
        orientation: StepperOrientation::Vertical,
        clickable: false,
    },));
    children.add_key("vertical-stepper");

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            flex_direction: WidgetFlexDirection::Column,
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<StepperDemo>(StepperDemo)
            .with_key("demo"),
    ));
}
