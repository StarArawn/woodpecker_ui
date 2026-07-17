use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<PaginationDemo>()
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct PaginationDemoState {
    page: usize,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct PaginationDemo;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<PaginationDemo>>,
    state_query: Query<&PaginationDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        PaginationDemoState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let page = state.page;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            margin: Edge::all(0.0).bottom(12.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: format!("Page {} of 12", page + 1),
        },
    ));
    children.add_key("label");

    children
        .add::<Pagination>((Pagination {
            page,
            page_count: 12,
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<PaginationChanged>>,
                  mut query: Query<&mut PaginationDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.page = trigger.data.page;
                }
            },
        );
    children.add_key("pagination");

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<PaginationDemo>(PaginationDemo)
            .with_key("demo"),
    ));
}
