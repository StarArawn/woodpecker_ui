use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<RatingDemo>()
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct RatingDemoState {
    value: f32,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct RatingDemo;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<RatingDemo>>,
    state_query: Query<&RatingDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        RatingDemoState { value: 3.5 },
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let value = state.value;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();

    children
        .add::<Rating>((Rating {
            value,
            max: 5,
            read_only: false,
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<RatingChanged>>, mut query: Query<&mut RatingDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.value = trigger.data.value;
                }
            },
        );
    children.add_key("rating");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            margin: Edge::all(0.0).top(8.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: format!("{value} / 5"),
        },
    ));
    children.add_key("label");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            margin: Edge::all(0.0).top(20.0),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Rating>((Rating {
            value: 4.0,
            max: 5,
            read_only: true,
        },)),
    ));
    children.add_key("read-only");

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
            .with_child::<RatingDemo>(RatingDemo)
            .with_key("demo"),
    ));
}
