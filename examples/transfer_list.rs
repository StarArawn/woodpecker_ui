use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<TransferListDemo>()
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct TransferListDemoState {
    left: Vec<String>,
    right: Vec<String>,
}

impl Default for TransferListDemoState {
    fn default() -> Self {
        Self {
            left: vec![
                "Admin".into(),
                "Editor".into(),
                "Viewer".into(),
                "Billing".into(),
            ],
            right: vec!["Support".into()],
        }
    }
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct TransferListDemo;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<TransferListDemo>>,
    state_query: Query<&TransferListDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        TransferListDemoState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let state = state.clone();
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<TransferList>((TransferList {
            left: state.left,
            right: state.right,
            left_title: "Available roles".into(),
            right_title: "Assigned roles".into(),
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<TransferListChanged>>,
                  mut query: Query<&mut TransferListDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.left = trigger.data.left.clone();
                    state.right = trigger.data.right.clone();
                }
            },
        );
    children.add_key("transfer");

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            width: 480.0.into(),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<TransferListDemo>(TransferListDemo)
            .with_key("demo"),
    ));
}
